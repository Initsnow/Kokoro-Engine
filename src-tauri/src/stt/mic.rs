use crate::stt::stream::{AudioBuffer, SAMPLE_RATE};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig};
use rubato::{FastFixedIn, PolynomialDegree, Resampler};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

const VOLUME_EVENT_INTERVAL: Duration = Duration::from_millis(50);
const RESAMPLER_CHUNK_SIZE: usize = 256;

enum WorkerCommand {
    Start {
        response: SyncSender<Result<(), String>>,
    },
    Stop {
        response: SyncSender<Result<(), String>>,
    },
}

#[derive(Default)]
pub struct NativeMicState {
    control_tx: Mutex<Option<Sender<WorkerCommand>>>,
}

impl NativeMicState {
    pub fn new() -> Self {
        Self {
            control_tx: Mutex::new(None),
        }
    }

    fn ensure_worker(&self, app: &AppHandle) -> Result<Sender<WorkerCommand>, String> {
        let mut guard = self
            .control_tx
            .lock()
            .map_err(|_| "Failed to lock native microphone state".to_string())?;

        if let Some(tx) = guard.as_ref() {
            return Ok(tx.clone());
        }

        let (tx, rx) = mpsc::channel();
        spawn_mic_worker(rx, app.clone());
        *guard = Some(tx.clone());
        Ok(tx)
    }
}

struct NativeInputProcessor {
    channels: usize,
    resampler: Option<FastFixedIn<f32>>,
    pending_mono: Vec<f32>,
    last_volume_emit: Instant,
}

impl NativeInputProcessor {
    fn new(sample_rate: u32, channels: usize) -> Result<Self, String> {
        let resampler = if sample_rate == SAMPLE_RATE {
            None
        } else {
            Some(
                FastFixedIn::<f32>::new(
                    SAMPLE_RATE as f64 / sample_rate as f64,
                    1.0,
                    PolynomialDegree::Cubic,
                    RESAMPLER_CHUNK_SIZE,
                    1,
                )
                .map_err(|err| format!("Failed to create resampler: {err}"))?,
            )
        };

        Ok(Self {
            channels,
            resampler,
            pending_mono: Vec::with_capacity(RESAMPLER_CHUNK_SIZE * 2),
            last_volume_emit: Instant::now() - VOLUME_EVENT_INTERVAL,
        })
    }

    fn process_input<T>(&mut self, input: &[T], app: &AppHandle)
    where
        T: Sample,
        f32: FromSample<T>,
    {
        let mono = interleaved_to_mono(input, self.channels);
        self.emit_volume(&mono, app);

        let output = match self.resampler.as_mut() {
            Some(resampler) => resample_mono_chunk(&mut self.pending_mono, resampler, mono),
            None => mono,
        };

        if output.is_empty() {
            return;
        }

        let audio_buffer = app.state::<AudioBuffer>();
        if let Err(err) = audio_buffer.append_samples(output) {
            eprintln!("[STT] Native mic append failed: {err}");
        }
    }

    fn emit_volume(&mut self, mono: &[f32], app: &AppHandle) {
        if mono.is_empty() || self.last_volume_emit.elapsed() < VOLUME_EVENT_INTERVAL {
            return;
        }

        let rms =
            (mono.iter().map(|sample| sample * sample).sum::<f32>() / mono.len() as f32).sqrt();
        let db = if rms > 0.0 { 20.0 * rms.log10() } else { -60.0 };
        let volume = ((db + 60.0) * 2.0).clamp(0.0, 100.0);
        let _ = app.emit("stt:mic-volume", volume);
        self.last_volume_emit = Instant::now();
    }
}

pub fn start_native_mic(app: &AppHandle, mic_state: &NativeMicState) -> Result<(), String> {
    let tx = mic_state.ensure_worker(app)?;
    let (response_tx, response_rx) = mpsc::sync_channel(1);
    tx.send(WorkerCommand::Start {
        response: response_tx,
    })
    .map_err(|_| "Native microphone worker is unavailable".to_string())?;
    response_rx
        .recv()
        .map_err(|_| "Native microphone worker did not respond".to_string())?
}

pub fn stop_native_mic(app: &AppHandle, mic_state: &NativeMicState) -> Result<(), String> {
    let tx = mic_state.ensure_worker(app)?;
    let (response_tx, response_rx) = mpsc::sync_channel(1);
    tx.send(WorkerCommand::Stop {
        response: response_tx,
    })
    .map_err(|_| "Native microphone worker is unavailable".to_string())?;
    response_rx
        .recv()
        .map_err(|_| "Native microphone worker did not respond".to_string())?
}

fn spawn_mic_worker(rx: Receiver<WorkerCommand>, app: AppHandle) {
    std::thread::spawn(move || {
        let mut stream: Option<Stream> = None;

        while let Ok(command) = rx.recv() {
            match command {
                WorkerCommand::Start { response } => {
                    let result = if stream.is_some() {
                        Ok(())
                    } else {
                        match build_native_input_stream(&app) {
                            Ok(new_stream) => {
                                if let Err(err) = new_stream.play() {
                                    Err(format!("Failed to start microphone stream: {err}"))
                                } else {
                                    stream = Some(new_stream);
                                    Ok(())
                                }
                            }
                            Err(err) => Err(err),
                        }
                    };
                    let _ = response.send(result);
                }
                WorkerCommand::Stop { response } => {
                    stream.take();
                    let _ = app.emit("stt:mic-volume", 0.0f32);
                    let _ = response.send(Ok(()));
                }
            }
        }
    });
}

fn build_native_input_stream(app: &AppHandle) -> Result<Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No input microphone device is available".to_string())?;
    let config = device
        .default_input_config()
        .map_err(|err| format!("Failed to query default microphone config: {err}"))?;

    let stream_config: StreamConfig = config.clone().into();
    let channels = stream_config.channels as usize;
    let sample_rate = stream_config.sample_rate.0;
    let app_handle = app.clone();

    match config.sample_format() {
        SampleFormat::I8 => {
            build_input_stream::<i8>(&device, &stream_config, channels, sample_rate, app_handle)
        }
        SampleFormat::I16 => {
            build_input_stream::<i16>(&device, &stream_config, channels, sample_rate, app_handle)
        }
        SampleFormat::I32 => {
            build_input_stream::<i32>(&device, &stream_config, channels, sample_rate, app_handle)
        }
        SampleFormat::I64 => {
            build_input_stream::<i64>(&device, &stream_config, channels, sample_rate, app_handle)
        }
        SampleFormat::U8 => {
            build_input_stream::<u8>(&device, &stream_config, channels, sample_rate, app_handle)
        }
        SampleFormat::U16 => {
            build_input_stream::<u16>(&device, &stream_config, channels, sample_rate, app_handle)
        }
        SampleFormat::U32 => {
            build_input_stream::<u32>(&device, &stream_config, channels, sample_rate, app_handle)
        }
        SampleFormat::U64 => {
            build_input_stream::<u64>(&device, &stream_config, channels, sample_rate, app_handle)
        }
        SampleFormat::F32 => {
            build_input_stream::<f32>(&device, &stream_config, channels, sample_rate, app_handle)
        }
        SampleFormat::F64 => {
            build_input_stream::<f64>(&device, &stream_config, channels, sample_rate, app_handle)
        }
        sample_format => Err(format!(
            "Unsupported microphone sample format: {sample_format}"
        )),
    }
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    sample_rate: u32,
    app: AppHandle,
) -> Result<Stream, String>
where
    T: SizedSample + Sample + Send + 'static,
    f32: FromSample<T>,
{
    let mut processor = NativeInputProcessor::new(sample_rate, channels)?;
    let err_app = app.clone();

    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                processor.process_input(data, &app);
            },
            move |err| {
                eprintln!("[STT] Native microphone stream error: {err}");
                let _ = err_app.emit("stt:mic-volume", 0.0f32);
            },
            None,
        )
        .map_err(|err| format!("Failed to build microphone input stream: {err}"))
}

fn interleaved_to_mono<T>(input: &[T], channels: usize) -> Vec<f32>
where
    T: Sample,
    f32: FromSample<T>,
{
    if channels <= 1 {
        return input
            .iter()
            .map(|&sample| f32::from_sample(sample))
            .collect();
    }

    input
        .chunks_exact(channels)
        .map(|frame| {
            let sum = frame
                .iter()
                .map(|&sample| f32::from_sample(sample))
                .sum::<f32>();
            sum / channels as f32
        })
        .collect()
}

fn resample_mono_chunk(
    pending_mono: &mut Vec<f32>,
    resampler: &mut FastFixedIn<f32>,
    mono: Vec<f32>,
) -> Vec<f32> {
    pending_mono.extend_from_slice(&mono);

    let mut output = Vec::new();
    loop {
        let needed = resampler.input_frames_next();
        if pending_mono.len() < needed {
            break;
        }

        let input_chunk = vec![pending_mono[..needed].to_vec()];
        match resampler.process(&input_chunk, None) {
            Ok(mut processed) => {
                if let Some(channel) = processed.pop() {
                    output.extend(channel);
                }
            }
            Err(err) => {
                eprintln!("[STT] Native microphone resampling failed: {err}");
                break;
            }
        }
        pending_mono.drain(..needed);
    }

    output
}

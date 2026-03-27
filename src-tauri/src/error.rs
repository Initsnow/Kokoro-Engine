use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 统一的 Kokoro 错误类型，支持结构化序列化
#[derive(Debug, Error, Serialize, Deserialize, Clone)]
#[serde(tag = "code", content = "message")]
pub enum KokoroError {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("数据库错误: {0}")]
    Database(String),

    #[error("LLM 错误: {0}")]
    Llm(String),

    #[error("TTS 错误: {0}")]
    Tts(String),

    #[error("STT 错误: {0}")]
    Stt(String),

    #[error("IO 错误: {0}")]
    Io(String),

    #[error("外部服务错误: {0}")]
    ExternalService(String),

    #[error("MOD 错误: {0}")]
    Mod(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("未授权: {0}")]
    Unauthorized(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

/// 便捷类型别名
pub type KokoroResult<T> = Result<T, KokoroError>;

/// 向后兼容：序列化为 JSON 字符串，未迁移模块的 .map_err(|e| e.to_string()) 仍可用
impl From<KokoroError> for String {
    fn from(e: KokoroError) -> String {
        serde_json::to_string(&e).unwrap_or_else(|_| e.to_string())
    }
}

/// 自动转换 std::io::Error
impl From<std::io::Error> for KokoroError {
    fn from(e: std::io::Error) -> Self {
        KokoroError::Io(e.to_string())
    }
}

/// 自动转换 serde_json::Error
impl From<serde_json::Error> for KokoroError {
    fn from(e: serde_json::Error) -> Self {
        KokoroError::Internal(format!("JSON 序列化错误: {}", e))
    }
}

/// 自动转换 sqlx::Error
impl From<sqlx::Error> for KokoroError {
    fn from(e: sqlx::Error) -> Self {
        KokoroError::Database(e.to_string())
    }
}

/// 自动转换 anyhow::Error
impl From<anyhow::Error> for KokoroError {
    fn from(e: anyhow::Error) -> Self {
        KokoroError::Internal(e.to_string())
    }
}

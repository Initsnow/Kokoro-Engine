# Nix

## Enabling Flakes

On NixOS:

```nix
{
  nix.settings.experimental-features = [ "nix-command" "flakes" ];
}
```

Apply the configuration:

```bash
sudo nixos-rebuild switch
```

On non-NixOS systems, enable temporarily:

```bash
nix --extra-experimental-features "nix-command flakes" develop
```

## Development

```bash
nix develop
npm install
npm run tauri dev
```

## Build

```bash
nix build .#default
./result/bin/tauri-appkokoro-engine
```

Or run directly:

```bash
nix run .#default
```

## NixOS Flake Installation

Add the repository to your system flake inputs:

```nix
{
  inputs.kokoro-engine.url = "github:chyinan/Kokoro-Engine";
}
```

Then install it in your `configuration.nix` module:

```nix
{ pkgs, inputs, ... }:
{
  environment.systemPackages = [
    inputs.kokoro-engine.packages.${pkgs.system}.default
  ];
}
```

## Home Manager Installation

```nix
{ pkgs, inputs, ... }:
{
  home.packages = [
    inputs.kokoro-engine.packages.${pkgs.system}.default
  ];
}
```

## Notes

- The flake currently supports Linux only.
- The package includes runtime configuration for WebKitGTK, glib-networking, GStreamer, and ONNX Runtime.

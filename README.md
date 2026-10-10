# Viboscop
<img width="444" height="517" alt="image" src="https://github.com/user-attachments/assets/6f34a380-6f56-4a2f-b3d9-33a6c486911e" />

Viboscop is a native Rust desktop app for visualizing stereo audio as a live XY oscilloscope.

It supports:
- audio file playback for WAV, FLAC, and MP3 files
- live microphone input
- real-time left/right channel XY plotting as a moving Lissajous trace

The app is designed for a quick, focused workflow: load a track or capture live mic input, then inspect the stereo relationship in real time.

## Features

- Play WAV, FLAC, and MP3 files
- Live microphone input capture
- Real-time stereo XY oscilloscope view
- Source switching between File and Microphone
- Audio output with separate volume control
- Add File dialog for loading tracks from the GUI
- Theme switching
- Fullscreen mode
- Responsive desktop interface

> Desktop audio capture is intentionally not included in the current version. The supported live input sources are file playback and microphone input only.

## Run

```bash
cargo run --release
```

## Build

```bash
cargo build --release
```

Then run:

```bash
./target/release/viboscop
```

## Install

```bash
install -Dm755 target/release/viboscop "$HOME/.local/bin/viboscop"
```

Add `~/.local/bin` to your `PATH` if needed.

## Controls

- Add File: load a WAV, FLAC, or MP3 audio file
- Input Source: switch between File and Microphone
- Play/Pause: start or stop file playback
- Volume: adjust output gain
- Theme: switch the interface theme
- Fullscreen: toggle fullscreen mode
- Clear Trace: clear the oscilloscope history

## Requirements

- Rust and Cargo
- Linux desktop environment
- Audio device available through PipeWire/ALSA
- Microphone access enabled for live input capture

## Development

```bash
cargo fmt
cargo check
cargo test
cargo build --release
```

## License

MIT

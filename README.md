# Viboscop

Viboscop is a native Rust desktop app for playing WAV and FLAC files and visualizing their stereo audio as a live XY oscilloscope.

It loads a file, plays it through your system audio output, and draws the left/right channel pair in real time as a moving Lissajous trace.

This project was vibecoded because there was no practical, easy-to-use Linux tool for this kind of real-time stereo XY oscilloscope workflow.

## Features

- Play WAV and FLAC files
- Real-time stereo XY oscilloscope view
- Audio output with separate volume control
- Add File dialog for loading tracks from the GUI
- Theme switching
- Fullscreen mode
- Responsive desktop interface

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

- Add File: load a WAV or FLAC audio file
- Play/Pause: start or stop playback
- Volume: adjust output gain
- Theme: switch the interface theme
- Fullscreen: toggle fullscreen mode

## Requirements

- Rust and Cargo
- Linux desktop environment
- Audio device available through PipeWire/ALSA

## Development

```bash
cargo fmt
cargo check
cargo test
cargo build --release
```

## License

MIT

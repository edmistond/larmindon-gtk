# Larmindon

Real-time speech-to-text captioning for Linux. Captures system audio via PipeWire and transcribes it using the Nemotron ASR model, displaying live captions in a minimal overlay window.

![Larmindon captioning a weather forecast](larmindon-gtk.jpg)

## Features

- **Real-time captioning** of any system audio source (application streams, microphones, monitor outputs)
- **Minimal overlay UI** — borderless window with just a hamburger menu and close button
- **Always-on-top** via standard compositor controls (right-click the window)
- **Grab-anywhere** window dragging, resizable
- **PipeWire integration** with live device detection and seamless source switching
- **VAD gating** (Silero) — only transcribes when speech is detected
- **Configurable font** family and size for the caption display
- **Diagnostics database** (SQLite) logging inference timing, VAD events, and session data

## Requirements

- Linux with PipeWire (or CPAL as fallback)
- GTK 4.12+
- [Nemotron ASR model](https://huggingface.co/nvidia/parakeet-tdt_ctc-110m) (via parakeet-rs)
- Silero VAD model (included in `models/`)

## Build & Run

```sh
cargo run
```

On first run, configure the model path in Preferences (hamburger menu > Preferences).

### Environment Variables

| Variable | Values | Default | Purpose |
|----------|--------|---------|---------|
| `CHUNK_MS` | 80, 160, 560, 1120 | 560 | ASR chunk size in ms |
| `INTRA_THREADS` | 1+ | 2 | ONNX intra-op parallelism |
| `INTER_THREADS` | 1+ | 1 | ONNX inter-op parallelism |
| `PUNCTUATION_RESET` | 0/1 | 1 | Reset decoder at sentence boundaries |
| `LARMINDON_AUDIO_BACKEND` | cpal, pipewire | auto | Force audio backend |

### Cargo Feature Flags

- `pipewire` (default) — PipeWire audio capture
- `cpal` (default) — CPAL audio capture (fallback)

## Configuration

Settings are stored in `~/.config/larmindon/settings.json` and can be edited via the Preferences dialog. The diagnostics database is at `~/.config/larmindon/larmindon_diag.sqlite`.

## Architecture

```
GTK main thread
  └─ Engine thread (receives Command via mpsc)
       ├─ PipeWire/CPAL audio capture → pushes mono f32 into shared buffer
       └─ Processing thread (per session)
            ├─ Drains shared buffer
            ├─ Resamples to 16kHz (rubato)
            ├─ VAD gating (Silero ONNX, 512-sample frames)
            └─ ASR inference (Nemotron, configurable chunk size)
```

Engine → UI communication uses `std::sync::mpsc`, polled by a glib timeout on the GTK main thread.

## License

MIT — see [LICENSE](LICENSE).

This application dynamically links to GTK4, GLib, and PipeWire which are licensed under LGPL-2.1-or-later. See [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for details.

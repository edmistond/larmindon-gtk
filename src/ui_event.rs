use crate::audio_capture::AudioDevice;
use std::sync::mpsc;

/// Events sent from the audio engine / PipeWire watcher to the GTK UI thread.
pub enum UiEvent {
    Transcription { text: String },
    TranscriptionError { text: String },
    SourceSwitched { device_id: String },
    DevicesChanged { devices: Vec<AudioDevice> },
}

/// Thread-safe sender for UI events.
pub type UiSender = mpsc::Sender<UiEvent>;

/// Receiver for UI events (used on the GTK main thread).
pub type UiReceiver = mpsc::Receiver<UiEvent>;

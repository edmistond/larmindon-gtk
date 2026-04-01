mod audio_capture;
mod audio_config;
mod audio_engine;
mod settings;
mod ui_event;
mod vad;
mod window;

use audio_capture::{ActiveSessionInfo, AudioCapture};
use audio_engine::{AudioEngine, Command};
use settings::Settings;
use ui_event::{UiEvent, UiReceiver};

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::gdk;

use std::cell::RefCell;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("../resources/style.css"));
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not get default display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Create the appropriate audio capture backend based on platform and features
fn create_audio_backend() -> Box<dyn AudioCapture> {
    if let Ok(backend) = std::env::var("LARMINDON_AUDIO_BACKEND") {
        match backend.as_str() {
            "cpal" => {
                println!("Using CPAL backend (via LARMINDON_AUDIO_BACKEND env var)");
                #[cfg(feature = "cpal")]
                return audio_capture::cpal::create_backend();
                #[cfg(not(feature = "cpal"))]
                panic!("CPAL feature not enabled but requested via LARMINDON_AUDIO_BACKEND");
            }
            "pipewire" => {
                #[cfg(all(target_os = "linux", feature = "pipewire"))]
                {
                    println!("Using PipeWire backend (via LARMINDON_AUDIO_BACKEND env var)");
                    return audio_capture::pipewire::create_backend();
                }
                #[cfg(not(all(target_os = "linux", feature = "pipewire")))]
                panic!("PipeWire backend requested but feature not enabled");
            }
            _ => {
                eprintln!("Unknown LARMINDON_AUDIO_BACKEND={backend}, using default");
            }
        }
    }

    #[cfg(all(target_os = "linux", feature = "pipewire"))]
    {
        println!("Attempting PipeWire backend...");
        match test_pipewire_available() {
            Ok(true) => {
                println!("PipeWire available, using PipeWire backend");
                return audio_capture::pipewire::create_backend();
            }
            Ok(false) => println!("PipeWire not available, falling back to CPAL"),
            Err(e) => eprintln!("Error testing PipeWire: {}, falling back to CPAL", e),
        }
    }

    #[cfg(feature = "cpal")]
    {
        println!("Using CPAL backend");
        audio_capture::cpal::create_backend()
    }
    #[cfg(not(feature = "cpal"))]
    {
        panic!("No audio backend available. Enable either 'cpal' or 'pipewire' feature.");
    }
}

#[cfg(all(target_os = "linux", feature = "pipewire"))]
fn test_pipewire_available() -> Result<bool, Box<dyn std::error::Error>> {
    use pipewire::main_loop::MainLoopBox;
    pipewire::init();
    let mainloop = MainLoopBox::new(None)?;
    let _context = pipewire::context::ContextBox::new(&mainloop.loop_(), None)?;
    Ok(true)
}

#[cfg(not(all(target_os = "linux", feature = "pipewire")))]
fn test_pipewire_available() -> Result<bool, Box<dyn std::error::Error>> {
    Ok(false)
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.github.edmistond.larmindon")
        .build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(|app| {
        let settings = Settings::load().with_env_overrides();
        println!(
            "Settings: chunk_ms={}, intra={}, inter={}, punctuation_reset={}, model={}",
            settings.chunk_ms, settings.intra_threads, settings.inter_threads,
            settings.punctuation_reset, settings.model_path,
        );

        // Create channels
        let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();

        let active_session_info = Arc::new(Mutex::new(ActiveSessionInfo::default()));
        let capture_backend = create_audio_backend();

        // Start PipeWire device watcher (Linux only)
        #[cfg(all(target_os = "linux", feature = "pipewire"))]
        let _watcher = {
            let watcher_ui_tx = ui_tx.clone();
            let watcher_cmd_tx = cmd_tx.clone();
            let watcher_session_info = active_session_info.clone();
            let watcher_devices_cache = capture_backend
                .as_any()
                .and_then(|a| a.downcast_ref::<audio_capture::pipewire::PipewireBackend>())
                .map(|pw| pw.last_devices.clone());

            watcher_devices_cache.map(|devices_cache| {
                audio_capture::pipewire::start_watcher(
                    watcher_ui_tx,
                    watcher_cmd_tx,
                    watcher_session_info,
                    devices_cache,
                )
            })
        };

        // Spawn the audio engine thread
        let session_info_for_engine = active_session_info.clone();
        let _engine_thread = thread::spawn(move || {
            let engine = AudioEngine::new(ui_tx, cmd_rx, capture_backend, session_info_for_engine);
            engine.run();
        });

        // Build the window
        let main_window = window::MainWindow::new(app, cmd_tx.clone(), settings.clone());

        // Get initial device list, populate menu, and auto-start
        {
            let (reply_tx, reply_rx) = mpsc::channel();
            let _ = cmd_tx.send(Command::ListDevices { reply: reply_tx });
            if let Ok(devices) = reply_rx.recv() {
                main_window.update_device_menu(&devices);
                let default_dev = audio_capture::select_default_device(&devices);
                if let Some(ref dev_id) = default_dev {
                    main_window.set_active_device(dev_id);
                }
                if default_dev.is_some() {
                    let _ = cmd_tx.send(Command::Start {
                        device_id: default_dev,
                        settings: settings.clone(),
                    });
                }
            }
        }

        // Poll UI events from the engine thread (every 16ms ~ 60fps)
        let ui_rx = RefCell::new(Some(ui_rx));
        let win = main_window.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            let rx_ref = ui_rx.borrow();
            let Some(rx) = rx_ref.as_ref() else {
                return glib::ControlFlow::Break;
            };

            while let Ok(event) = rx.try_recv() {
                match event {
                    UiEvent::Transcription { text } => {
                        win.append_text(&text);
                    }
                    UiEvent::TranscriptionError { text } => {
                        eprintln!("[UI] Error: {}", text);
                    }
                    UiEvent::SourceSwitched { device_id } => {
                        win.set_active_device(&device_id);
                    }
                    UiEvent::DevicesChanged { devices } => {
                        win.update_device_menu(&devices);
                    }
                }
            }

            glib::ControlFlow::Continue
        });

        // Handle window close — send Shutdown command
        let cmd_tx_for_close = cmd_tx.clone();
        main_window.window.connect_close_request(move |_| {
            let _ = cmd_tx_for_close.send(Command::Shutdown);
            glib::Propagation::Proceed
        });

        main_window.window.present();
    });

    app.run();
}

mod audio_capture;
mod audio_config;
mod audio_engine;
mod settings;
mod ui_event;
mod vad;
mod window;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::gdk;

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("../resources/style.css"));
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not get default display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.github.edmistond.larmindon")
        .build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(|app| {
        let main_window = window::MainWindow::new(app);
        main_window.append_text("Larmindon GTK — skeleton window");
        main_window.window.present();
    });

    app.run();
}

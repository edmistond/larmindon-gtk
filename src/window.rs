use crate::audio_capture::AudioDevice;
use crate::audio_engine::Command;
use crate::settings::Settings;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{gio, gdk};

use std::cell::RefCell;
use std::sync::mpsc;

#[derive(Clone)]
pub struct MainWindow {
    pub window: gtk::ApplicationWindow,
    pub text_view: gtk::TextView,
    pub menu_button: gtk::MenuButton,
    cmd_tx: mpsc::Sender<Command>,
    settings: RefCell<Settings>,
    active_device_id: RefCell<Option<String>>,
}

impl MainWindow {
    pub fn new(app: &gtk::Application, cmd_tx: mpsc::Sender<Command>, settings: Settings) -> Self {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Larmindon")
            .default_width(600)
            .default_height(200)
            .build();

        // HeaderBar with hamburger menu
        let headerbar = gtk::HeaderBar::new();
        headerbar.set_show_title_buttons(true);

        let menu_button = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Menu")
            .build();
        headerbar.pack_start(&menu_button);

        window.set_titlebar(Some(&headerbar));

        // Scrolled text view for captions
        let text_view = gtk::TextView::builder()
            .editable(false)
            .cursor_visible(false)
            .wrap_mode(gtk::WrapMode::Word)
            .build();
        text_view.add_css_class("caption-view");

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .child(&text_view)
            .build();

        window.set_child(Some(&scrolled));

        let main_window = Self {
            window,
            text_view,
            menu_button,
            cmd_tx,
            settings: RefCell::new(settings),
            active_device_id: RefCell::new(None),
        };

        main_window.setup_actions();
        main_window.setup_gestures();
        main_window
    }

    fn setup_actions(&self) {
        // switch-source action: stateful string action with checkmark on active device
        let switch_source = gio::SimpleAction::new_stateful(
            "switch-source",
            Some(&String::static_variant_type()),
            &"".to_variant(),
        );

        let cmd_tx = self.cmd_tx.clone();
        let settings = self.settings.clone();
        let active_id = self.active_device_id.clone();
        switch_source.connect_activate(move |action, param| {
            if let Some(device_id) = param.and_then(|p| p.get::<String>()) {
                action.set_state(&device_id.to_variant());
                *active_id.borrow_mut() = Some(device_id.clone());

                // Stop current, start new
                let _ = cmd_tx.send(Command::Stop);
                let _ = cmd_tx.send(Command::Start {
                    device_id: Some(device_id),
                    settings: settings.borrow().clone(),
                });
            }
        });
        self.window.add_action(&switch_source);

        // refresh-devices action
        let cmd_tx = self.cmd_tx.clone();
        let win_clone = self.clone();
        let refresh = gio::SimpleAction::new("refresh-devices", None);
        refresh.connect_activate(move |_, _| {
            let (reply_tx, reply_rx) = mpsc::channel();
            let _ = cmd_tx.send(Command::ListDevices { reply: reply_tx });
            // Use a timeout to avoid blocking the main thread
            let win = win_clone.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(50), move || {
                if let Ok(devices) = reply_rx.try_recv() {
                    win.update_device_menu(&devices);
                }
            });
        });
        self.window.add_action(&refresh);

        // stop action
        let cmd_tx = self.cmd_tx.clone();
        let stop = gio::SimpleAction::new("stop", None);
        stop.connect_activate(move |_, _| {
            let _ = cmd_tx.send(Command::Stop);
        });
        self.window.add_action(&stop);

        // start action (restarts on current or default device)
        let cmd_tx = self.cmd_tx.clone();
        let settings = self.settings.clone();
        let active_id = self.active_device_id.clone();
        let start = gio::SimpleAction::new("start", None);
        start.connect_activate(move |_, _| {
            let _ = cmd_tx.send(Command::Start {
                device_id: active_id.borrow().clone(),
                settings: settings.borrow().clone(),
            });
        });
        self.window.add_action(&start);
    }

    fn setup_gestures(&self) {
        // Grab-anywhere drag to move window via GDK Toplevel::begin_move
        let drag = gtk::GestureDrag::new();
        drag.set_button(gdk::BUTTON_PRIMARY);
        let window = self.window.clone();
        drag.connect_drag_begin(move |gesture, x, y| {
            if let Some(native) = window.native() {
                if let Some(surface) = native.surface() {
                    if let Some(toplevel) = surface.downcast_ref::<gdk::Toplevel>() {
                        if let Some(event) = gesture.last_event(gesture.current_sequence().as_ref()) {
                            if let Some(device) = event.device() {
                                let timestamp = event.time();
                                // Translate widget coords to surface coords
                                let (sx, sy) = native.surface_transform();
                                toplevel.begin_move(&device, gdk::BUTTON_PRIMARY as i32, x + sx, y + sy, timestamp);
                            }
                        }
                    }
                }
            }
            gesture.set_state(gtk::EventSequenceState::Claimed);
        });
        self.text_view.add_controller(drag);

        // Right-click for compositor window menu (always-on-top, workspaces, etc.)
        let right_click = gtk::GestureClick::new();
        right_click.set_button(gdk::BUTTON_SECONDARY);
        let window = self.window.clone();
        right_click.connect_pressed(move |gesture, _n, _x, _y| {
            if let Some(native) = window.native() {
                if let Some(surface) = native.surface() {
                    if let Some(toplevel) = surface.downcast_ref::<gdk::Toplevel>() {
                        if let Some(event) = gesture.last_event(gesture.current_sequence().as_ref()) {
                            toplevel.show_window_menu(&event);
                        }
                    }
                }
            }
            gesture.set_state(gtk::EventSequenceState::Claimed);
        });
        self.text_view.add_controller(right_click);
    }

    pub fn update_device_menu(&self, devices: &[AudioDevice]) {
        let menu = gio::Menu::new();

        // Group devices by type
        let apps_section = gio::Menu::new();
        let inputs_section = gio::Menu::new();
        let monitors_section = gio::Menu::new();

        for dev in devices {
            let item = gio::MenuItem::new(Some(&dev.name), None);
            item.set_action_and_target_value(
                Some("win.switch-source"),
                Some(&dev.id.to_variant()),
            );

            use crate::audio_capture::DeviceType;
            match dev.device_type {
                DeviceType::Application => apps_section.append_item(&item),
                DeviceType::Input => inputs_section.append_item(&item),
                DeviceType::Monitor => monitors_section.append_item(&item),
            }
        }

        if apps_section.n_items() > 0 {
            menu.append_section(Some("Applications"), &apps_section);
        }
        if inputs_section.n_items() > 0 {
            menu.append_section(Some("Inputs"), &inputs_section);
        }
        if monitors_section.n_items() > 0 {
            menu.append_section(Some("Monitors"), &monitors_section);
        }

        if devices.is_empty() {
            menu.append(Some("No devices found"), None);
        }

        // Controls section
        let controls = gio::Menu::new();
        controls.append(Some("Refresh Devices"), Some("win.refresh-devices"));
        controls.append(Some("Stop"), Some("win.stop"));
        controls.append(Some("Start"), Some("win.start"));
        menu.append_section(None, &controls);

        self.menu_button.set_menu_model(Some(&menu));
    }

    pub fn set_active_device(&self, device_id: &str) {
        *self.active_device_id.borrow_mut() = Some(device_id.to_string());
        // Update the action state so the checkmark appears
        if let Some(action) = self.window.lookup_action("switch-source") {
            if let Some(simple) = action.downcast_ref::<gio::SimpleAction>() {
                simple.set_state(&device_id.to_variant());
            }
        }
    }

    pub fn append_text(&self, text: &str) {
        let buffer = self.text_view.buffer();
        let mut end = buffer.end_iter();
        buffer.insert(&mut end, text);

        // Auto-scroll to bottom
        let end_mark = buffer.create_mark(None, &buffer.end_iter(), false);
        self.text_view.scroll_mark_onscreen(&end_mark);
        buffer.delete_mark(&end_mark);
    }
}

use crate::settings::Settings;

use gtk4 as gtk;
use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

/// Show a preferences dialog. Calls `on_save` with the new settings when saved.
pub fn show_preferences(
    current: &Settings,
    on_save: impl Fn(Settings) + 'static,
) {
    let dialog = gtk::Window::builder()
        .title("Preferences")
        .default_width(450)
        .default_height(400)
        .resizable(false)
        .build();

    let grid = gtk::Grid::builder()
        .row_spacing(8)
        .column_spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let mut row = 0;

    // Model path
    let model_label = gtk::Label::new(Some("Model Path"));
    model_label.set_halign(gtk::Align::End);
    let model_entry = gtk::Entry::builder()
        .text(&current.model_path)
        .hexpand(true)
        .build();
    grid.attach(&model_label, 0, row, 1, 1);
    grid.attach(&model_entry, 1, row, 1, 1);
    row += 1;

    // Chunk size dropdown
    let chunk_label = gtk::Label::new(Some("Chunk Size (ms)"));
    chunk_label.set_halign(gtk::Align::End);
    let chunk_options = ["80", "160", "560", "1120"];
    let chunk_dropdown = gtk::DropDown::from_strings(&chunk_options);
    let active_idx = chunk_options
        .iter()
        .position(|&v| v == current.chunk_ms.to_string())
        .unwrap_or(2);
    chunk_dropdown.set_selected(active_idx as u32);
    grid.attach(&chunk_label, 0, row, 1, 1);
    grid.attach(&chunk_dropdown, 1, row, 1, 1);
    row += 1;

    // Intra threads
    let intra_label = gtk::Label::new(Some("Intra Threads"));
    intra_label.set_halign(gtk::Align::End);
    let intra_spin = gtk::SpinButton::with_range(1.0, 16.0, 1.0);
    intra_spin.set_value(current.intra_threads as f64);
    grid.attach(&intra_label, 0, row, 1, 1);
    grid.attach(&intra_spin, 1, row, 1, 1);
    row += 1;

    // Inter threads
    let inter_label = gtk::Label::new(Some("Inter Threads"));
    inter_label.set_halign(gtk::Align::End);
    let inter_spin = gtk::SpinButton::with_range(1.0, 16.0, 1.0);
    inter_spin.set_value(current.inter_threads as f64);
    grid.attach(&inter_label, 0, row, 1, 1);
    grid.attach(&inter_spin, 1, row, 1, 1);
    row += 1;

    // Punctuation reset
    let punct_label = gtk::Label::new(Some("Punctuation Reset"));
    punct_label.set_halign(gtk::Align::End);
    let punct_switch = gtk::Switch::new();
    punct_switch.set_active(current.punctuation_reset);
    punct_switch.set_halign(gtk::Align::Start);
    grid.attach(&punct_label, 0, row, 1, 1);
    grid.attach(&punct_switch, 1, row, 1, 1);
    row += 1;

    // Empty reset threshold
    let empty_label = gtk::Label::new(Some("Empty Reset Threshold"));
    empty_label.set_halign(gtk::Align::End);
    let empty_spin = gtk::SpinButton::with_range(1.0, 20.0, 1.0);
    empty_spin.set_value(current.empty_reset_threshold as f64);
    grid.attach(&empty_label, 0, row, 1, 1);
    grid.attach(&empty_spin, 1, row, 1, 1);
    row += 1;

    // Font picker (family + size in one widget)
    let font_label = gtk::Label::new(Some("Caption Font"));
    font_label.set_halign(gtk::Align::End);
    #[allow(deprecated)] // FontButton deprecated in 4.10 but FontDialogButton needs async
    let font_button = gtk::FontButton::new();
    // Build Pango font string from settings (px → pt for Pango: 1px ≈ 0.75pt)
    if !current.font_family.is_empty() || current.font_size_px > 0 {
        let family = if current.font_family.is_empty() { "Sans" } else { &current.font_family };
        let pt = if current.font_size_px > 0 {
            (current.font_size_px as f64 * 0.75).round() as u32
        } else {
            12
        };
        use gtk::prelude::FontChooserExt;
        font_button.set_font(&format!("{} {}", family, pt));
    }
    font_button.set_hexpand(true);
    grid.attach(&font_label, 0, row, 1, 1);
    grid.attach(&font_button, 1, row, 1, 1);
    row += 1;

    // Error label (hidden by default)
    let error_label = gtk::Label::new(None);
    error_label.set_halign(gtk::Align::Start);
    error_label.add_css_class("error");
    error_label.set_visible(false);
    grid.attach(&error_label, 0, row, 2, 1);
    row += 1;

    // Buttons
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    button_box.set_halign(gtk::Align::End);

    let cancel_btn = gtk::Button::with_label("Cancel");
    let save_btn = gtk::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");

    button_box.append(&cancel_btn);
    button_box.append(&save_btn);
    grid.attach(&button_box, 0, row, 2, 1);

    dialog.set_child(Some(&grid));

    // Cancel closes the dialog
    let dialog_weak = dialog.downgrade();
    cancel_btn.connect_clicked(move |_| {
        if let Some(d) = dialog_weak.upgrade() {
            d.close();
        }
    });

    // Save validates and applies
    let chunk_values: Vec<usize> = chunk_options.iter().map(|s| s.parse().unwrap()).collect();
    let dialog_weak = dialog.downgrade();
    let on_save = Rc::new(RefCell::new(Some(on_save)));
    save_btn.connect_clicked(move |_| {
        // Extract font family and size from FontButton's pango FontDescription
        use gtk::prelude::FontChooserExt;
        let (font_family, font_size_px) = if let Some(desc) = font_button.font_desc() {
            let family = desc.family().map(|f| f.to_string()).unwrap_or_default();
            // Pango size is in Pango units (1/1024 pt); convert to px (approx 1pt = 1.333px)
            let pango_size = desc.size();
            let px = if pango_size > 0 {
                ((pango_size as f64 / pango::SCALE as f64) * 1.333).round() as u32
            } else {
                0
            };
            (family, px)
        } else {
            (String::new(), 0)
        };

        let new_settings = Settings {
            model_path: model_entry.text().to_string(),
            chunk_ms: chunk_values[chunk_dropdown.selected() as usize],
            intra_threads: intra_spin.value() as usize,
            inter_threads: inter_spin.value() as usize,
            punctuation_reset: punct_switch.is_active(),
            empty_reset_threshold: empty_spin.value() as u32,
            font_family,
            font_size_px,
        };

        if let Err(e) = new_settings.validate() {
            error_label.set_text(&e);
            error_label.set_visible(true);
            return;
        }

        if let Err(e) = new_settings.save() {
            error_label.set_text(&e);
            error_label.set_visible(true);
            return;
        }

        if let Some(cb) = on_save.borrow_mut().take() {
            cb(new_settings);
        }

        if let Some(d) = dialog_weak.upgrade() {
            d.close();
        }
    });

    dialog.present();
}

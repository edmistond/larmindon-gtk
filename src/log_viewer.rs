use crate::log_buffer;

use gtk4 as gtk;
use gtk::prelude::*;

pub fn show_log_viewer() {
    let dialog = gtk::Window::builder()
        .title("Logs")
        .default_width(700)
        .default_height(450)
        .build();

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(8);
    vbox.set_margin_start(8);
    vbox.set_margin_end(8);

    let text_view = gtk::TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .wrap_mode(gtk::WrapMode::Word)
        .monospace(true)
        .vexpand(true)
        .build();

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .child(&text_view)
        .build();

    // Populate with current log contents
    let populate = {
        let text_view = text_view.clone();
        move || {
            let lines = log_buffer::snapshot();
            let buffer = text_view.buffer();
            buffer.set_text(&lines.join("\n"));
            // Scroll to bottom
            let end_mark = buffer.create_mark(None, &buffer.end_iter(), false);
            text_view.scroll_mark_onscreen(&end_mark);
            buffer.delete_mark(&end_mark);
        }
    };

    populate();

    // Refresh button
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    button_box.set_halign(gtk::Align::End);

    let refresh_btn = gtk::Button::with_label("Refresh");
    let populate_for_btn = populate.clone();
    refresh_btn.connect_clicked(move |_| {
        populate_for_btn();
    });

    let close_btn = gtk::Button::with_label("Close");
    let dialog_weak = dialog.downgrade();
    close_btn.connect_clicked(move |_| {
        if let Some(d) = dialog_weak.upgrade() {
            d.close();
        }
    });

    button_box.append(&refresh_btn);
    button_box.append(&close_btn);

    vbox.append(&scrolled);
    vbox.append(&button_box);
    dialog.set_child(Some(&vbox));
    dialog.present();
}

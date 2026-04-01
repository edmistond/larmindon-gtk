use gtk4 as gtk;
use gtk::prelude::*;

#[derive(Clone)]
pub struct MainWindow {
    pub window: gtk::ApplicationWindow,
    pub text_view: gtk::TextView,
}

impl MainWindow {
    pub fn new(app: &gtk::Application) -> Self {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Larmindon")
            .default_width(600)
            .default_height(200)
            .build();

        // HeaderBar with hamburger menu placeholder
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

        Self { window, text_view }
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

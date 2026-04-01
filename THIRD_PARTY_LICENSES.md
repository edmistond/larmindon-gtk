# Third-Party Licenses

Larmindon links dynamically to the following LGPL-licensed system libraries:

## GTK 4 / GLib / GDK / Pango / GdkPixbuf

- **License**: LGPL-2.1-or-later
- **Copyright**: The GNOME Project
- **Source**: https://gitlab.gnome.org/GNOME/gtk

## PipeWire / libspa

- **License**: LGPL-2.1-or-later
- **Copyright**: Wim Taymans
- **Source**: https://gitlab.freedesktop.org/pipewire/pipewire

## WebKitGTK (not used)

This application does not use WebKitGTK.

---

All Rust crate dependencies are licensed under MIT, Apache-2.0, or dual MIT/Apache-2.0.
See `Cargo.lock` for the full dependency list.

Per the terms of the LGPL-2.1, this application dynamically links to the above
libraries and does not modify them. Users may substitute compatible versions of
these libraries.

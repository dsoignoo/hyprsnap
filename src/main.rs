mod canvas;
mod clipboard;
mod toolbar;
mod tools;

use std::cell::RefCell;
use std::rc::Rc;

use cairo::{Format, ImageSurface};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, EventControllerKey, Orientation, ScrolledWindow,
};

use tools::{Annotation, ToolKind};

pub struct AppState {
    pub background: ImageSurface,
    pub annotations: Vec<Annotation>,
    pub current: Option<Annotation>,
    pub tool: ToolKind,
    pub color: (f64, f64, f64),
    pub stroke_width: f64,
    pub font_size: f64,
    pub image_path: String,
    pub save_dir: Option<String>,
    pub text_click_pos: Option<(f64, f64)>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: hyprsnap [--save-dir <dir>] <image.png>");
        std::process::exit(1);
    }

    let mut image_path = String::new();
    let mut save_dir: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--save-dir" {
            i += 1;
            if i < args.len() {
                save_dir = Some(args[i].clone());
            } else {
                eprintln!("--save-dir requires an argument");
                std::process::exit(1);
            }
        } else {
            image_path = args[i].clone();
        }
        i += 1;
    }
    if image_path.is_empty() {
        eprintln!("Usage: hyprsnap [--save-dir <dir>] <image.png>");
        std::process::exit(1);
    }

    let app = Application::builder()
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(move |app| {
        let img = image::open(&image_path)
            .expect("Cannot open image file")
            .to_rgba8();
        let img_w = img.width() as i32;
        let img_h = img.height() as i32;

        let mut background = ImageSurface::create(Format::ARgb32, img_w, img_h).unwrap();
        {
            let cairo_stride = background.stride() as usize;
            let mut data = background.data().unwrap();
            for y in 0..img_h as usize {
                for x in 0..img_w as usize {
                    let px = img.get_pixel(x as u32, y as u32).0;
                    let dst_off = y * cairo_stride + x * 4;
                    // Cairo ARGB32: B, G, R, A on little-endian
                    data[dst_off] = px[2];
                    data[dst_off + 1] = px[1];
                    data[dst_off + 2] = px[0];
                    data[dst_off + 3] = px[3];
                }
            }
        }

        let state = Rc::new(RefCell::new(AppState {
            background,
            annotations: Vec::new(),
            current: None,
            tool: ToolKind::Rect,
            color: (1.0, 0.0, 0.0),
            stroke_width: 3.0,
            font_size: 20.0,
            image_path: image_path.clone(),
            save_dir: save_dir.clone(),
            text_click_pos: None,
        }));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Screenshot Editor")
            .default_width(img_w.min(1600))
            .default_height((img_h + 50).min(900))
            .build();

        let da = canvas::build_canvas(Rc::clone(&state), &window);
        let toolbar = toolbar::build_toolbar(Rc::clone(&state), &da, &window);

        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&da));
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        let vbox = GtkBox::new(Orientation::Vertical, 0);
        vbox.append(&toolbar);
        vbox.append(&scrolled);
        window.set_child(Some(&vbox));

        // Keyboard shortcuts
        let key_ctrl = EventControllerKey::new();
        let state_key = Rc::clone(&state);
        let da_key = da.clone();
        let win_key = window.clone();
        key_ctrl.connect_key_pressed(move |_, key, _, modifier| {
            let ctrl = modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK);
            if ctrl {
                match key {
                    gtk4::gdk::Key::z => {
                        state_key.borrow_mut().annotations.pop();
                        da_key.queue_draw();
                        return gtk4::glib::Propagation::Stop;
                    }
                    gtk4::gdk::Key::s => {
                        clipboard::save_to_file(&state_key.borrow(), &win_key);
                        return gtk4::glib::Propagation::Stop;
                    }
                    gtk4::gdk::Key::c => {
                        clipboard::copy_to_clipboard(&state_key.borrow());
                        return gtk4::glib::Propagation::Stop;
                    }
                    _ => {}
                }
            }
            gtk4::glib::Propagation::Proceed
        });
        window.add_controller(key_ctrl);

        window.present();
    });

    app.run_with_args::<String>(&[]);
}

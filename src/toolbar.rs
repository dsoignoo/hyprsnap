use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CssProvider, DrawingArea, Label, Orientation, Separator,
    SpinButton, ToggleButton,
};

use crate::clipboard;
use crate::tools::ToolKind;
use crate::AppState;

pub fn build_toolbar(
    state: Rc<RefCell<AppState>>,
    da: &DrawingArea,
    window: &gtk4::ApplicationWindow,
) -> GtkBox {
    let toolbar = GtkBox::new(Orientation::Horizontal, 6);
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);
    toolbar.set_margin_top(6);
    toolbar.set_margin_bottom(6);
    toolbar.set_valign(Align::Center);

    // Load CSS for color buttons
    let css = CssProvider::new();
    css.load_from_data(
        r#"
        .color-btn { min-width: 28px; min-height: 28px; border-radius: 4px; border: 2px solid #666; padding: 0; }
        .color-btn.active-color { border: 3px solid #fff; }
        .color-red { background: #ff0000; }
        .color-green { background: #00cc00; }
        .color-blue { background: #0066ff; }
        .color-yellow { background: #ffff00; }
        .color-white { background: #ffffff; }
        "#,
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Tool buttons
    let btn_rect = ToggleButton::with_label("Rect");
    let btn_line = ToggleButton::with_label("Line");
    let btn_arrow = ToggleButton::with_label("Arrow");
    let btn_text = ToggleButton::with_label("Text");
    let btn_ocr = ToggleButton::with_label("OCR");
    let btn_crop = ToggleButton::with_label("Crop");
    btn_line.set_group(Some(&btn_rect));
    btn_arrow.set_group(Some(&btn_rect));
    btn_text.set_group(Some(&btn_rect));
    btn_ocr.set_group(Some(&btn_rect));
    btn_crop.set_group(Some(&btn_rect));
    btn_rect.set_active(true);

    let s = Rc::clone(&state);
    btn_rect.connect_toggled(move |btn| {
        if btn.is_active() {
            s.borrow_mut().tool = ToolKind::Rect;
        }
    });
    let s = Rc::clone(&state);
    btn_line.connect_toggled(move |btn| {
        if btn.is_active() {
            s.borrow_mut().tool = ToolKind::Line;
        }
    });
    let s = Rc::clone(&state);
    btn_arrow.connect_toggled(move |btn| {
        if btn.is_active() {
            s.borrow_mut().tool = ToolKind::Arrow;
        }
    });
    let s = Rc::clone(&state);
    btn_text.connect_toggled(move |btn| {
        if btn.is_active() {
            s.borrow_mut().tool = ToolKind::Text;
        }
    });
    let s = Rc::clone(&state);
    btn_ocr.connect_toggled(move |btn| {
        if btn.is_active() {
            s.borrow_mut().tool = ToolKind::Ocr;
        }
    });
    let s = Rc::clone(&state);
    btn_crop.connect_toggled(move |btn| {
        if btn.is_active() {
            s.borrow_mut().tool = ToolKind::Crop;
        }
    });

    toolbar.append(&btn_rect);
    toolbar.append(&btn_line);
    toolbar.append(&btn_arrow);
    toolbar.append(&btn_text);
    toolbar.append(&btn_ocr);
    toolbar.append(&btn_crop);
    toolbar.append(&Separator::new(Orientation::Vertical));

    // Color buttons
    let colors: Vec<(&str, &str, (f64, f64, f64))> = vec![
        ("color-red", "Red", (1.0, 0.0, 0.0)),
        ("color-green", "Green", (0.0, 0.8, 0.0)),
        ("color-blue", "Blue", (0.0, 0.4, 1.0)),
        ("color-yellow", "Yellow", (1.0, 1.0, 0.0)),
        ("color-white", "White", (1.0, 1.0, 1.0)),
    ];

    let color_buttons: Vec<Button> = colors
        .iter()
        .map(|(css_class, tooltip, _color)| {
            let btn = Button::new();
            btn.add_css_class("color-btn");
            btn.add_css_class(css_class);
            btn.set_tooltip_text(Some(tooltip));
            btn
        })
        .collect();

    // Red is default active
    color_buttons[0].add_css_class("active-color");

    for (i, btn) in color_buttons.iter().enumerate() {
        let s = Rc::clone(&state);
        let color = colors[i].2;
        let buttons = color_buttons.clone();
        btn.connect_clicked(move |_btn| {
            s.borrow_mut().color = color;
            for b in &buttons {
                b.remove_css_class("active-color");
            }
            buttons[i].add_css_class("active-color");
        });
        toolbar.append(btn);
    }

    toolbar.append(&Separator::new(Orientation::Vertical));

    // Stroke width
    let lbl = Label::new(Some("Width:"));
    toolbar.append(&lbl);
    let spin = SpinButton::with_range(1.0, 20.0, 1.0);
    spin.set_value(3.0);
    let s = Rc::clone(&state);
    spin.connect_value_changed(move |spin| {
        s.borrow_mut().stroke_width = spin.value();
    });
    toolbar.append(&spin);

    // Font size
    let lbl_font = Label::new(Some("Font:"));
    toolbar.append(&lbl_font);
    let spin_font = SpinButton::with_range(8.0, 72.0, 2.0);
    spin_font.set_value(20.0);
    let s = Rc::clone(&state);
    spin_font.connect_value_changed(move |spin| {
        s.borrow_mut().font_size = spin.value();
    });
    toolbar.append(&spin_font);

    toolbar.append(&Separator::new(Orientation::Vertical));

    // Undo
    let btn_undo = Button::with_label("Undo");
    btn_undo.set_tooltip_text(Some("Ctrl+Z"));
    let s = Rc::clone(&state);
    let da_c = da.clone();
    btn_undo.connect_clicked(move |_| {
        s.borrow_mut().annotations.pop();
        da_c.queue_draw();
    });
    toolbar.append(&btn_undo);

    // Save
    let btn_save = Button::with_label("Save");
    btn_save.set_tooltip_text(Some("Ctrl+S"));
    let s = Rc::clone(&state);
    let win = window.clone();
    btn_save.connect_clicked(move |_| {
        clipboard::save_to_file(&s.borrow(), &win);
    });
    toolbar.append(&btn_save);

    // Copy to clipboard
    let btn_copy = Button::with_label("Copy");
    btn_copy.set_tooltip_text(Some("Ctrl+C"));
    let s = Rc::clone(&state);
    btn_copy.connect_clicked(move |_| {
        clipboard::copy_to_clipboard(&s.borrow());
    });
    toolbar.append(&btn_copy);

    toolbar
}

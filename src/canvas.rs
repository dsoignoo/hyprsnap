use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, DrawingArea, Entry, GestureDrag, Orientation, Window};

use crate::clipboard;
use crate::tools::{draw_annotation, Annotation, ToolKind};
use crate::AppState;

pub fn build_canvas(
    state: Rc<RefCell<AppState>>,
    window: &ApplicationWindow,
) -> DrawingArea {
    let da = DrawingArea::new();
    {
        let st = state.borrow();
        da.set_size_request(st.background.width(), st.background.height());
    }

    // Draw function
    let state_draw = Rc::clone(&state);
    da.set_draw_func(move |_da, cr, _w, _h| {
        let st = state_draw.borrow();
        let _ = cr.set_source_surface(&st.background, 0.0, 0.0);
        let _ = cr.paint();
        for ann in &st.annotations {
            draw_annotation(cr, ann);
        }
        if let Some(ref current) = st.current {
            draw_annotation(cr, current);
        }
    });

    // Drag gesture for drawing
    let gesture = GestureDrag::new();

    let state_begin = Rc::clone(&state);
    gesture.connect_drag_begin(move |_gesture, x, y| {
        let mut st = state_begin.borrow_mut();
        if st.tool == ToolKind::Text {
            st.text_click_pos = Some((x, y));
            return;
        }
        st.current = Some(Annotation {
            kind: st.tool,
            start: (x, y),
            end: (x, y),
            color: st.color,
            stroke_width: st.stroke_width,
            text: None,
            font_size: st.font_size,
        });
    });

    let state_update = Rc::clone(&state);
    let da_update = da.clone();
    gesture.connect_drag_update(move |_gesture, offset_x, offset_y| {
        let mut st = state_update.borrow_mut();
        if st.tool == ToolKind::Text {
            return;
        }
        if let Some(ref mut current) = st.current {
            current.end = (current.start.0 + offset_x, current.start.1 + offset_y);
        }
        drop(st);
        da_update.queue_draw();
    });

    let state_end = Rc::clone(&state);
    let da_end = da.clone();
    let win = window.clone();
    gesture.connect_drag_end(move |_gesture, _offset_x, _offset_y| {
        let mut st = state_end.borrow_mut();
        if st.tool == ToolKind::Text {
            if let Some(pos) = st.text_click_pos.take() {
                let color = st.color;
                let font_size = st.font_size;
                drop(st);
                show_text_dialog(&win, Rc::clone(&state_end), &da_end, pos, color, font_size);
            }
            return;
        }
        if st.tool == ToolKind::Ocr {
            if let Some(ref current) = st.current {
                let start = current.start;
                let end = current.end;
                st.current = None;
                drop(st);
                let st = state_end.borrow();
                clipboard::ocr_region(&st, start, end);
                drop(st);
                da_end.queue_draw();
            }
            return;
        }
        if let Some(ann) = st.current.take() {
            st.annotations.push(ann);
        }
        drop(st);
        da_end.queue_draw();
    });

    da.add_controller(gesture);
    da
}

fn show_text_dialog(
    parent: &ApplicationWindow,
    state: Rc<RefCell<AppState>>,
    da: &DrawingArea,
    pos: (f64, f64),
    color: (f64, f64, f64),
    font_size: f64,
) {
    let dialog = Window::builder()
        .title("Enter Text")
        .modal(true)
        .transient_for(parent)
        .default_width(300)
        .build();

    let vbox = GtkBox::new(Orientation::Vertical, 8);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);

    let entry = Entry::new();
    entry.set_placeholder_text(Some("Type annotation text..."));
    vbox.append(&entry);
    dialog.set_child(Some(&vbox));

    let state_c = Rc::clone(&state);
    let da_c = da.clone();
    let dialog_c = dialog.clone();
    entry.connect_activate(move |entry| {
        let text = entry.text().to_string();
        if !text.is_empty() {
            state_c.borrow_mut().annotations.push(Annotation {
                kind: ToolKind::Text,
                start: pos,
                end: pos,
                color,
                stroke_width: 1.0,
                text: Some(text),
                font_size,
            });
            da_c.queue_draw();
        }
        dialog_c.close();
    });

    // Also close on Escape
    let key_ctrl = gtk4::EventControllerKey::new();
    let dialog_esc = dialog.clone();
    key_ctrl.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            dialog_esc.close();
            return gtk4::glib::Propagation::Stop;
        }
        gtk4::glib::Propagation::Proceed
    });
    dialog.add_controller(key_ctrl);

    dialog.present();
    entry.grab_focus();
}

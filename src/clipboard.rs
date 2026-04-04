use std::io::{Cursor, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use cairo::{Context, Format, ImageSurface};

use crate::tools::draw_annotation;
use crate::AppState;

fn render_to_surface(state: &AppState) -> ImageSurface {
    let w = state.background.width();
    let h = state.background.height();
    let surface = ImageSurface::create(Format::ARgb32, w, h).unwrap();
    let cr = Context::new(&surface).unwrap();

    cr.set_source_surface(&state.background, 0.0, 0.0).unwrap();
    cr.paint().unwrap();

    for ann in &state.annotations {
        draw_annotation(&cr, ann);
    }

    drop(cr);
    surface
}

pub fn save_to_file(state: &AppState, _window: &gtk4::ApplicationWindow) {
    let surface = render_to_surface(state);

    let path = Path::new(&state.image_path);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or(Path::new("/tmp"));
    let out_path = parent.join(format!("{stem}_annotated.png"));

    let mut file = std::fs::File::create(&out_path).unwrap();
    surface.write_to_png(&mut file).unwrap();

    // Notify via notify-send
    let _ = Command::new("notify-send")
        .args(["Screenshot saved", &out_path.to_string_lossy()])
        .spawn();
}

pub fn copy_to_clipboard(state: &AppState) {
    let surface = render_to_surface(state);

    let mut buf = Cursor::new(Vec::new());
    surface.write_to_png(&mut buf).unwrap();
    let png_data = buf.into_inner();

    let mut child = Command::new("wl-copy")
        .args(["--type", "image/png"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("wl-copy not found");

    child.stdin.as_mut().unwrap().write_all(&png_data).unwrap();
    let _ = child.wait();

    let _ = Command::new("notify-send")
        .args(["Screenshot copied to clipboard"])
        .spawn();
}

pub fn ocr_region(state: &AppState, start: (f64, f64), end: (f64, f64)) {
    let x = start.0.min(end.0).max(0.0) as i32;
    let y = start.1.min(end.1).max(0.0) as i32;
    let w = (end.0 - start.0).abs() as i32;
    let h = (end.1 - start.1).abs() as i32;

    if w < 5 || h < 5 {
        return;
    }

    // Crop the background image to the selected region
    let bg_w = state.background.width();
    let bg_h = state.background.height();
    let x = x.min(bg_w - 1);
    let y = y.min(bg_h - 1);
    let w = w.min(bg_w - x);
    let h = h.min(bg_h - y);

    let crop = ImageSurface::create(Format::ARgb32, w, h).unwrap();
    let cr = Context::new(&crop).unwrap();
    cr.set_source_surface(&state.background, -x as f64, -y as f64).unwrap();
    cr.paint().unwrap();
    drop(cr);

    // Write cropped region to temp file
    let tmp_path = "/tmp/hyprsnap_ocr_region.png";
    let mut file = std::fs::File::create(tmp_path).unwrap();
    crop.write_to_png(&mut file).unwrap();
    drop(file);

    // Run tesseract
    let output = Command::new("tesseract")
        .args([tmp_path, "stdout"])
        .output();

    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if text.is_empty() {
                let _ = Command::new("notify-send")
                    .args(["OCR", "No text detected in selection"])
                    .spawn();
                return;
            }
            // Copy text to clipboard
            let mut child = Command::new("wl-copy")
                .stdin(Stdio::piped())
                .spawn()
                .expect("wl-copy not found");
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(text.as_bytes())
                .unwrap();
            let _ = child.wait();

            let _ = Command::new("notify-send")
                .args(["OCR text copied to clipboard", &text])
                .spawn();
        }
        Err(_) => {
            let _ = Command::new("notify-send")
                .args(["OCR Error", "tesseract not found — install with: sudo zypper install tesseract-ocr"])
                .spawn();
        }
    }

    let _ = std::fs::remove_file(tmp_path);
}

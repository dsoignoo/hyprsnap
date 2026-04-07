use cairo::Context;

#[derive(Clone, Copy, PartialEq)]
pub enum ToolKind {
    Rect,
    Line,
    Arrow,
    Text,
    Ocr,
    Crop,
}

#[derive(Clone)]
pub struct Annotation {
    pub kind: ToolKind,
    pub start: (f64, f64),
    pub end: (f64, f64),
    pub color: (f64, f64, f64),
    pub stroke_width: f64,
    pub text: Option<String>,
    pub font_size: f64,
}

pub fn draw_annotation(cr: &Context, ann: &Annotation) {
    cr.set_source_rgb(ann.color.0, ann.color.1, ann.color.2);
    cr.set_line_width(ann.stroke_width);
    cr.set_line_cap(cairo::LineCap::Round);
    cr.set_line_join(cairo::LineJoin::Round);

    match ann.kind {
        ToolKind::Rect => {
            let x = ann.start.0.min(ann.end.0);
            let y = ann.start.1.min(ann.end.1);
            let w = (ann.end.0 - ann.start.0).abs();
            let h = (ann.end.1 - ann.start.1).abs();
            cr.rectangle(x, y, w, h);
            let _ = cr.stroke();
        }
        ToolKind::Line => {
            cr.move_to(ann.start.0, ann.start.1);
            cr.line_to(ann.end.0, ann.end.1);
            let _ = cr.stroke();
        }
        ToolKind::Arrow => {
            cr.move_to(ann.start.0, ann.start.1);
            cr.line_to(ann.end.0, ann.end.1);
            let _ = cr.stroke();

            let angle = (ann.end.1 - ann.start.1).atan2(ann.end.0 - ann.start.0);
            let arm_len = (ann.stroke_width * 4.0).max(12.0);
            let spread = std::f64::consts::PI / 6.0;

            for offset in [std::f64::consts::PI + spread, std::f64::consts::PI - spread] {
                let tip_x = ann.end.0 + arm_len * (angle + offset).cos();
                let tip_y = ann.end.1 + arm_len * (angle + offset).sin();
                cr.move_to(ann.end.0, ann.end.1);
                cr.line_to(tip_x, tip_y);
                let _ = cr.stroke();
            }
        }
        ToolKind::Text => {
            if let Some(ref text) = ann.text {
                cr.set_font_size(ann.font_size);
                cr.move_to(ann.start.0, ann.start.1);
                let _ = cr.show_text(text);
            }
        }
        ToolKind::Ocr | ToolKind::Crop => {
            // Draw a dashed selection rectangle
            let x = ann.start.0.min(ann.end.0);
            let y = ann.start.1.min(ann.end.1);
            let w = (ann.end.0 - ann.start.0).abs();
            let h = (ann.end.1 - ann.start.1).abs();
            cr.set_source_rgba(0.0, 0.6, 1.0, 0.8);
            cr.set_dash(&[6.0, 4.0], 0.0);
            cr.rectangle(x, y, w, h);
            let _ = cr.stroke();
            cr.set_dash(&[], 0.0);
        }
    }
}

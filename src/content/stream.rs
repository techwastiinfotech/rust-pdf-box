use crate::content::color::Color;
use crate::cos::object::{COSDictionary, COSStream};
use crate::error::PdfResult;
use crate::font::truetype::PDTrueTypeFont;
use crate::font::type1::PDType1Font;
use crate::pd::document::PDDocument;
use crate::pd::page::PDPage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendMode {
    Overwrite,
    Append,
    Prepend,
}

/// Content stream builder for rendering text, paths, graphics states and images on a page.
pub struct PDPageContentStream {
    buffer: Vec<u8>,
    pub mode: AppendMode,
}

impl PDPageContentStream {
    pub fn new(mode: AppendMode) -> Self {
        Self {
            buffer: Vec::new(),
            mode,
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.buffer.extend_from_slice(s.as_bytes());
    }

    pub fn write_line(&mut self, s: &str) {
        self.write_str(s);
        self.buffer.push(b'\n');
    }

    // --- Graphics State ---

    pub fn save_graphics_state(&mut self) -> &mut Self {
        self.write_line("q");
        self
    }

    pub fn restore_graphics_state(&mut self) -> &mut Self {
        self.write_line("Q");
        self
    }

    pub fn set_line_width(&mut self, width: f64) -> &mut Self {
        self.write_line(&format!("{:.3} w", width));
        self
    }

    pub fn set_line_cap(&mut self, cap: i32) -> &mut Self {
        self.write_line(&format!("{} J", cap));
        self
    }

    pub fn set_line_join(&mut self, join: i32) -> &mut Self {
        self.write_line(&format!("{} j", join));
        self
    }

    pub fn set_stroke_color(&mut self, color: Color) -> &mut Self {
        match color {
            Color::Rgb(r, g, b) => self.write_line(&format!("{:.3} {:.3} {:.3} RG", r, g, b)),
            Color::Cmyk(c, m, y, k) => {
                self.write_line(&format!("{:.3} {:.3} {:.3} {:.3} K", c, m, y, k))
            }
            Color::Gray(g) => self.write_line(&format!("{:.3} G", g)),
        }
        self
    }

    pub fn set_fill_color(&mut self, color: Color) -> &mut Self {
        match color {
            Color::Rgb(r, g, b) => self.write_line(&format!("{:.3} {:.3} {:.3} rg", r, g, b)),
            Color::Cmyk(c, m, y, k) => {
                self.write_line(&format!("{:.3} {:.3} {:.3} {:.3} k", c, m, y, k))
            }
            Color::Gray(g) => self.write_line(&format!("{:.3} g", g)),
        }
        self
    }

    pub fn set_opacity_on_page(&mut self, page: &mut PDPage, stroke_alpha: f64, fill_alpha: f64) -> &mut Self {
        let gs_name = page.resources.add_ext_gstate(stroke_alpha, fill_alpha);
        self.write_line(&format!("/{} gs", gs_name));
        self
    }

    pub fn set_opacity(&mut self, doc: &mut PDDocument, page_index: usize, stroke_alpha: f64, fill_alpha: f64) -> &mut Self {
        if let Ok(page) = doc.get_page_mut(page_index) {
            let gs_name = page.resources.add_ext_gstate(stroke_alpha, fill_alpha);
            self.write_line(&format!("/{} gs", gs_name));
        }
        self
    }

    pub fn transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> &mut Self {
        self.write_line(&format!(
            "{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} cm",
            a, b, c, d, e, f
        ));
        self
    }

    pub fn translate(&mut self, tx: f64, ty: f64) -> &mut Self {
        self.transform(1.0, 0.0, 0.0, 1.0, tx, ty)
    }

    pub fn scale(&mut self, sx: f64, sy: f64) -> &mut Self {
        self.transform(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }

    pub fn rotate(&mut self, degrees: f64, cx: f64, cy: f64) -> &mut Self {
        let rad = degrees.to_radians();
        let cos_val = rad.cos();
        let sin_val = rad.sin();
        // Translate to origin, rotate, translate back:
        // cm = [cos sin -sin cos (cx - cx*cos + cy*sin) (cy - cx*sin - cy*cos)]
        let e = cx - cx * cos_val + cy * sin_val;
        let f = cy - cx * sin_val - cy * cos_val;
        self.transform(cos_val, sin_val, -sin_val, cos_val, e, f)
    }

    // --- Path Construction & Painting ---

    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.write_line(&format!("{:.3} {:.3} m", x, y));
        self
    }

    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.write_line(&format!("{:.3} {:.3} l", x, y));
        self
    }

    pub fn curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> &mut Self {
        self.write_line(&format!(
            "{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c",
            x1, y1, x2, y2, x3, y3
        ));
        self
    }

    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> &mut Self {
        self.write_line(&format!("{:.3} {:.3} {:.3} {:.3} re", x, y, width, height));
        self
    }

    pub fn circle(&mut self, cx: f64, cy: f64, r: f64) -> &mut Self {
        let k = 0.552284749831 * r;
        self.move_to(cx + r, cy);
        self.curve_to(cx + r, cy + k, cx + k, cy + r, cx, cy + r);
        self.curve_to(cx - k, cy + r, cx - r, cy + k, cx - r, cy);
        self.curve_to(cx - r, cy - k, cx - k, cy - r, cx, cy - r);
        self.curve_to(cx + k, cy - r, cx + r, cy - k, cx + r, cy);
        self.close_path();
        self
    }

    pub fn round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, r: f64) -> &mut Self {
        let r = r.min(w / 2.0).min(h / 2.0);
        let k = 0.552284749831 * r;
        self.move_to(x + r, y);
        self.line_to(x + w - r, y);
        self.curve_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
        self.line_to(x + w, y + h - r);
        self.curve_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
        self.line_to(x + r, y + h);
        self.curve_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
        self.line_to(x, y + r);
        self.curve_to(x, y + r - k, x + r - k, y, x + r, y);
        self.close_path();
        self
    }

    pub fn close_path(&mut self) -> &mut Self {
        self.write_line("h");
        self
    }

    pub fn stroke(&mut self) -> &mut Self {
        self.write_line("S");
        self
    }

    pub fn fill(&mut self) -> &mut Self {
        self.write_line("f");
        self
    }

    pub fn fill_and_stroke(&mut self) -> &mut Self {
        self.write_line("B");
        self
    }

    pub fn clip(&mut self) -> &mut Self {
        self.write_line("W n");
        self
    }

    // --- Text Rendering ---

    pub fn begin_text(&mut self) -> &mut Self {
        self.write_line("BT");
        self
    }

    pub fn end_text(&mut self) -> &mut Self {
        self.write_line("ET");
        self
    }

    pub fn set_font(&mut self, font_name: &str, size: f64) -> &mut Self {
        self.write_line(&format!("/{} {:.3} Tf", font_name, size));
        self
    }

    pub fn set_font_type1(&mut self, doc: &mut PDDocument, page_index: usize, font: PDType1Font, size: f64) -> &mut Self {
        let font_id = doc.cos_doc.add_object(font.to_dict());
        if let Ok(page) = doc.get_page_mut(page_index) {
            let font_name = page.resources.add_font(font_id);
            self.write_line(&format!("/{} {:.3} Tf", font_name, size));
        }
        self
    }

    pub fn set_truetype_font(&mut self, doc: &mut PDDocument, page_index: usize, font: &PDTrueTypeFont, size: f64) -> &mut Self {
        if let Ok(page) = doc.get_page_mut(page_index) {
            let font_name = page.resources.add_font(font.font_id);
            self.write_line(&format!("/{} {:.3} Tf", font_name, size));
        }
        self
    }

    pub fn set_text_matrix(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> &mut Self {
        self.write_line(&format!(
            "{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} Tm",
            a, b, c, d, e, f
        ));
        self
    }

    pub fn set_leading(&mut self, leading: f64) -> &mut Self {
        self.write_line(&format!("{:.3} TL", leading));
        self
    }

    pub fn move_text_position(&mut self, tx: f64, ty: f64) -> &mut Self {
        self.write_line(&format!("{:.3} {:.3} Td", tx, ty));
        self
    }

    pub fn show_text(&mut self, text: &str) -> &mut Self {
        let mut escaped = String::with_capacity(text.len());
        for c in text.chars() {
            match c {
                '(' => escaped.push_str(r"\("),
                ')' => escaped.push_str(r"\)"),
                '\\' => escaped.push_str(r"\\"),
                '\r' => escaped.push_str(r"\r"),
                '\n' => escaped.push_str(r"\n"),
                _ => escaped.push(c),
            }
        }
        self.write_line(&format!("({}) Tj", escaped));
        self
    }

    pub fn show_text_at(&mut self, text: &str, x: f64, y: f64) -> &mut Self {
        self.set_text_matrix(1.0, 0.0, 0.0, 1.0, x, y);
        self.show_text(text);
        self
    }

    pub fn show_text_multiline(&mut self, lines: &[&str], x: f64, y: f64, leading: f64) -> &mut Self {
        self.set_leading(leading);
        self.set_text_matrix(1.0, 0.0, 0.0, 1.0, x, y);
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                self.write_line("T*");
            }
            self.show_text(line);
        }
        self
    }

    // --- Image Drawing ---

    pub fn draw_image(&mut self, image_name: &str, x: f64, y: f64, width: f64, height: f64) -> &mut Self {
        self.save_graphics_state();
        self.transform(width, 0.0, 0.0, height, x, y);
        self.write_line(&format!("/{} Do", image_name));
        self.restore_graphics_state();
        self
    }

    /// Flushes and applies the content stream to the document page.
    pub fn finish(self, doc: &mut PDDocument, page_index: usize) -> PdfResult<()> {
        let page = doc.get_page_mut(page_index)?;
        page.sync_resources();

        let mut stream = COSStream::new(COSDictionary::new(), self.buffer);
        let _ = stream.encode_flate();
        let stream_id = doc.cos_doc.add_object(stream);

        let page = doc.get_page_mut(page_index)?;
        let mut current_ids = page.content_stream_ids();

        match self.mode {
            AppendMode::Overwrite => {
                page.set_contents(stream_id);
            }
            AppendMode::Append => {
                current_ids.push(stream_id);
                page.set_contents_array(current_ids);
            }
            AppendMode::Prepend => {
                current_ids.insert(0, stream_id);
                page.set_contents_array(current_ids);
            }
        }

        page.sync_resources();
        Ok(())
    }

    /// Returns the raw byte buffer of this content stream.
    pub fn finish_to_bytes(self) -> Vec<u8> {
        self.buffer
    }
}

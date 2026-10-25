/// Represents a color in RGB, CMYK, or Grayscale color spaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Rgb(f64, f64, f64),
    Cmyk(f64, f64, f64, f64),
    Gray(f64),
}

impl Color {
    pub const BLACK: Color = Color::Gray(0.0);
    pub const WHITE: Color = Color::Gray(1.0);
    pub const RED: Color = Color::Rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::Rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::Rgb(0.0, 0.0, 1.0);
    pub const GRAY: Color = Color::Gray(0.5);

    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self::Rgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }

    pub fn cmyk(c: f64, m: f64, y: f64, k: f64) -> Self {
        Self::Cmyk(
            c.clamp(0.0, 1.0),
            m.clamp(0.0, 1.0),
            y.clamp(0.0, 1.0),
            k.clamp(0.0, 1.0),
        )
    }

    pub fn gray(g: f64) -> Self {
        Self::Gray(g.clamp(0.0, 1.0))
    }

    /// Parses a hex color string like "#FF0000" or "FF0000".
    pub fn from_hex(hex: &str) -> Option<Self> {
        let s = hex.trim_start_matches('#');
        if s.len() == 6 {
            let r = u8::from_str_radix(&s[0..2], 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&s[2..4], 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&s[4..6], 16).ok()? as f64 / 255.0;
            Some(Self::rgb(r, g, b))
        } else if s.len() == 3 {
            let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()? as f64 / 255.0;
            Some(Self::rgb(r, g, b))
        } else {
            None
        }
    }
}

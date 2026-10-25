use crate::cos::object::{COSDictionary, COSName};

/// Standard 14 built-in Type 1 PDF fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PDType1Font {
    Helvetica,
    HelveticaBold,
    HelveticaOblique,
    HelveticaBoldOblique,
    TimesRoman,
    TimesBold,
    TimesItalic,
    TimesBoldItalic,
    Courier,
    CourierBold,
    CourierOblique,
    CourierBoldOblique,
    Symbol,
    ZapfDingbats,
}

impl PDType1Font {
    pub fn base_font_name(&self) -> &'static str {
        match self {
            Self::Helvetica => "Helvetica",
            Self::HelveticaBold => "Helvetica-Bold",
            Self::HelveticaOblique => "Helvetica-Oblique",
            Self::HelveticaBoldOblique => "Helvetica-BoldOblique",
            Self::TimesRoman => "Times-Roman",
            Self::TimesBold => "Times-Bold",
            Self::TimesItalic => "Times-Italic",
            Self::TimesBoldItalic => "Times-BoldItalic",
            Self::Courier => "Courier",
            Self::CourierBold => "Courier-Bold",
            Self::CourierOblique => "Courier-Oblique",
            Self::CourierBoldOblique => "Courier-BoldOblique",
            Self::Symbol => "Symbol",
            Self::ZapfDingbats => "ZapfDingbats",
        }
    }

    /// Converts this Type 1 font into a PDF font dictionary.
    pub fn to_dict(&self) -> COSDictionary {
        let mut dict = COSDictionary::new();
        dict.set("Type", COSName::new("Font"));
        dict.set("Subtype", COSName::new("Type1"));
        dict.set("BaseFont", COSName::new(self.base_font_name()));
        if !matches!(self, Self::Symbol | Self::ZapfDingbats) {
            dict.set("Encoding", COSName::new("WinAnsiEncoding"));
        }
        dict
    }

    /// Approximate or standard character width in 1/1000 font units.
    pub fn get_char_width(&self, c: char) -> f64 {
        match self {
            Self::Courier | Self::CourierBold | Self::CourierOblique | Self::CourierBoldOblique => {
                // Fixed-width Courier is 600 units for every character
                600.0
            }
            Self::TimesRoman | Self::TimesBold | Self::TimesItalic | Self::TimesBoldItalic => {
                match c {
                    ' ' => 250.0,
                    'i' | 'j' | 'l' | '!' | '.' | ':' | ';' | '\'' => 278.0,
                    'f' | 't' | 'I' => 333.0,
                    'r' => 389.0,
                    's' => 444.0,
                    'a' | 'b' | 'c' | 'd' | 'e' | 'g' | 'h' | 'k' | 'n' | 'o' | 'p' | 'q' | 'u' | 'v' | 'x' | 'y' | 'z' => 500.0,
                    'w' => 722.0,
                    'm' => 778.0,
                    'M' | 'W' => 889.0,
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'J' | 'K' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 667.0,
                    _ => 500.0,
                }
            }
            _ => {
                // Helvetica default widths
                match c {
                    ' ' => 278.0,
                    'i' | 'l' | '!' | '.' | ':' | ';' | '\'' | '|' => 222.0,
                    'j' | 'r' | 't' | 'I' => 278.0,
                    'f' => 278.0,
                    'c' | 's' | 'z' => 500.0,
                    'a' | 'b' | 'd' | 'e' | 'g' | 'h' | 'k' | 'n' | 'o' | 'p' | 'q' | 'u' | 'v' | 'x' | 'y' => 556.0,
                    'w' => 722.0,
                    'm' => 833.0,
                    'W' | 'M' => 833.0,
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'J' | 'K' | 'L' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 667.0,
                    _ => 556.0,
                }
            }
        }
    }

    /// Computes the exact rendered width of a string in user space points at the given font size.
    pub fn get_string_width(&self, text: &str, font_size: f64) -> f64 {
        let total_units: f64 = text.chars().map(|c| self.get_char_width(c)).sum();
        (total_units / 1000.0) * font_size
    }
}

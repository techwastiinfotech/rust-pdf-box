use crate::cos::object::{COSArray, COSObject};

/// A rectangle in default user space units (1/72 inch).
/// Lower-left (x, y) to Upper-right (x, y).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PDRectangle {
    pub lower_left_x: f64,
    pub lower_left_y: f64,
    pub upper_right_x: f64,
    pub upper_right_y: f64,
}

impl PDRectangle {
    pub const A4: PDRectangle = PDRectangle::new(0.0, 0.0, 595.276, 841.890);
    pub const A3: PDRectangle = PDRectangle::new(0.0, 0.0, 841.890, 1190.551);
    pub const A5: PDRectangle = PDRectangle::new(0.0, 0.0, 419.528, 595.276);
    pub const LETTER: PDRectangle = PDRectangle::new(0.0, 0.0, 612.0, 792.0);
    pub const LEGAL: PDRectangle = PDRectangle::new(0.0, 0.0, 612.0, 1008.0);
    pub const TABLOID: PDRectangle = PDRectangle::new(0.0, 0.0, 792.0, 1224.0);

    pub const fn new(llx: f64, lly: f64, urx: f64, ury: f64) -> Self {
        Self {
            lower_left_x: llx,
            lower_left_y: lly,
            upper_right_x: urx,
            upper_right_y: ury,
        }
    }

    pub fn width(&self) -> f64 {
        (self.upper_right_x - self.lower_left_x).abs()
    }

    pub fn height(&self) -> f64 {
        (self.upper_right_y - self.lower_left_y).abs()
    }

    pub fn center(&self) -> (f64, f64) {
        (
            (self.lower_left_x + self.upper_right_x) / 2.0,
            (self.lower_left_y + self.upper_right_y) / 2.0,
        )
    }

    pub fn to_cos_array(&self) -> COSArray {
        let mut arr = COSArray::new();
        arr.push(self.lower_left_x);
        arr.push(self.lower_left_y);
        arr.push(self.upper_right_x);
        arr.push(self.upper_right_y);
        arr
    }

    pub fn from_cos_array(arr: &COSArray) -> Option<Self> {
        if arr.len() < 4 {
            return None;
        }
        let llx = arr.get(0)?.as_float()?;
        let lly = arr.get(1)?.as_float()?;
        let urx = arr.get(2)?.as_float()?;
        let ury = arr.get(3)?.as_float()?;
        Some(Self::new(llx, lly, urx, ury))
    }
}

impl From<PDRectangle> for COSObject {
    fn from(rect: PDRectangle) -> Self {
        COSObject::Array(rect.to_cos_array())
    }
}

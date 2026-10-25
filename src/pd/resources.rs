use crate::cos::object::{COSDictionary, COSName, COSObject, ObjectId};

/// The Resources dictionary of a page or form XObject (/Resources).
#[derive(Debug, Clone, Default)]
pub struct PDResources {
    pub dict: COSDictionary,
}

impl PDResources {
    pub fn new() -> Self {
        Self {
            dict: COSDictionary::new(),
        }
    }

    pub fn from_dict(dict: COSDictionary) -> Self {
        Self { dict }
    }

    /// Adds a font reference and returns the font resource name (e.g., "F1").
    pub fn add_font(&mut self, font_ref: ObjectId) -> String {
        let font_dict = self.dict.entries
            .entry(COSName::new("Font"))
            .or_insert_with(|| COSObject::Dictionary(COSDictionary::new()));

        if let COSObject::Dictionary(d) = font_dict {
            let next_index = d.entries.len() + 1;
            let name = format!("F{}", next_index);
            d.set(COSName::new(&name), font_ref);
            name
        } else {
            "F1".to_string()
        }
    }

    /// Adds an XObject (image or form) reference and returns the resource name (e.g., "Im1").
    pub fn add_xobject(&mut self, prefix: &str, xobj_ref: ObjectId) -> String {
        let xobj_dict = self.dict.entries
            .entry(COSName::new("XObject"))
            .or_insert_with(|| COSObject::Dictionary(COSDictionary::new()));

        if let COSObject::Dictionary(d) = xobj_dict {
            let next_index = d.entries.len() + 1;
            let name = format!("{}{}", prefix, next_index);
            d.set(COSName::new(&name), xobj_ref);
            name
        } else {
            format!("{}1", prefix)
        }
    }

    /// Adds an Extended Graphics State (opacity / alpha) and returns resource name (e.g. "GS1").
    pub fn add_ext_gstate(&mut self, stroke_alpha: f64, fill_alpha: f64) -> String {
        let gs_dict = self.dict.entries
            .entry(COSName::new("ExtGState"))
            .or_insert_with(|| COSObject::Dictionary(COSDictionary::new()));

        if let COSObject::Dictionary(d) = gs_dict {
            let next_index = d.entries.len() + 1;
            let name = format!("GS{}", next_index);
            let mut state_dict = COSDictionary::new();
            state_dict.set("Type", COSName::new("ExtGState"));
            state_dict.set("CA", stroke_alpha);
            state_dict.set("ca", fill_alpha);
            d.set(COSName::new(&name), state_dict);
            name
        } else {
            "GS1".to_string()
        }
    }
}

use crate::content::color::Color;
use crate::cos::document::COSDocument;
use crate::cos::object::{COSArray, COSDictionary, COSName, COSString, ObjectId};
use crate::pd::page::PDPage;
use crate::pd::rectangle::PDRectangle;

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationType {
    Link { uri: String },
    Highlight,
    Underline,
    Text { comment: String },
    Stamp { name: String },
}

/// A PDF Page Annotation (/Subtype /Link, /Highlight, /Text, etc.).
#[derive(Debug, Clone)]
pub struct PDAnnotation {
    pub rect: PDRectangle,
    pub annotation_type: AnnotationType,
    pub color: Option<Color>,
    pub contents: Option<String>,
}

impl PDAnnotation {
    /// Creates a clickable URI web link annotation.
    pub fn link(rect: PDRectangle, uri: impl Into<String>) -> Self {
        Self {
            rect,
            annotation_type: AnnotationType::Link { uri: uri.into() },
            color: Some(Color::rgb(0.0, 0.4, 0.8)),
            contents: None,
        }
    }

    /// Creates a text highlight annotation.
    pub fn highlight(rect: PDRectangle, comment: Option<String>) -> Self {
        Self {
            rect,
            annotation_type: AnnotationType::Highlight,
            color: Some(Color::rgb(1.0, 1.0, 0.0)), // Yellow highlight
            contents: comment,
        }
    }

    /// Attaches this annotation to a page dictionary and stores it in the COSDocument.
    pub fn attach_to_page(&self, page: &mut PDPage, cos_doc: &mut COSDocument) -> ObjectId {
        let mut dict = COSDictionary::new();
        dict.set("Type", COSName::new("Annot"));
        dict.set("Rect", self.rect);

        if let Some(c) = self.color {
            if let Color::Rgb(r, g, b) = c {
                let mut c_arr = COSArray::new();
                c_arr.push(r);
                c_arr.push(g);
                c_arr.push(b);
                dict.set("C", c_arr);
            }
        }

        if let Some(ref text) = self.contents {
            dict.set("Contents", COSString::literal(text.clone().into_bytes()));
        }

        match &self.annotation_type {
            AnnotationType::Link { uri } => {
                dict.set("Subtype", COSName::new("Link"));
                let mut action = COSDictionary::new();
                action.set("Type", COSName::new("Action"));
                action.set("S", COSName::new("URI"));
                action.set("URI", COSString::literal(uri.clone().into_bytes()));
                dict.set("A", action);
                dict.set("F", 4); // Print flag
            }
            AnnotationType::Highlight => {
                dict.set("Subtype", COSName::new("Highlight"));
            }
            AnnotationType::Underline => {
                dict.set("Subtype", COSName::new("Underline"));
            }
            AnnotationType::Text { comment } => {
                dict.set("Subtype", COSName::new("Text"));
                dict.set("Contents", COSString::literal(comment.clone().into_bytes()));
            }
            AnnotationType::Stamp { name } => {
                dict.set("Subtype", COSName::new("Stamp"));
                dict.set("Name", COSName::new(name));
            }
        }

        let annot_id = cos_doc.add_object(dict);

        let mut annots = page.dict.get_array("Annots").cloned().unwrap_or_default();
        annots.push(annot_id);
        page.dict.set("Annots", annots);

        annot_id
    }
}

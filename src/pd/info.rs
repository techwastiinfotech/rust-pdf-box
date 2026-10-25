use crate::cos::object::{COSDictionary, COSString};
use chrono::{DateTime, Utc};

/// Document metadata (/Info dictionary).
#[derive(Debug, Clone, Default)]
pub struct PDDocumentInformation {
    pub dict: COSDictionary,
}

impl PDDocumentInformation {
    pub fn new() -> Self {
        let mut info = Self {
            dict: COSDictionary::new(),
        };
        info.set_producer("rust-pdf-box (Techwasti Infotech)");
        let now = Utc::now();
        info.set_creation_date(now);
        info.set_modification_date(now);
        info
    }

    pub fn from_dict(dict: COSDictionary) -> Self {
        Self { dict }
    }

    pub fn title(&self) -> Option<String> {
        self.dict.get_string("Title").map(|s| s.to_string_lossy())
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.dict
            .set("Title", COSString::literal(title.into().into_bytes()));
    }

    pub fn author(&self) -> Option<String> {
        self.dict.get_string("Author").map(|s| s.to_string_lossy())
    }

    pub fn set_author(&mut self, author: impl Into<String>) {
        self.dict
            .set("Author", COSString::literal(author.into().into_bytes()));
    }

    pub fn subject(&self) -> Option<String> {
        self.dict.get_string("Subject").map(|s| s.to_string_lossy())
    }

    pub fn set_subject(&mut self, subject: impl Into<String>) {
        self.dict
            .set("Subject", COSString::literal(subject.into().into_bytes()));
    }

    pub fn keywords(&self) -> Option<String> {
        self.dict
            .get_string("Keywords")
            .map(|s| s.to_string_lossy())
    }

    pub fn set_keywords(&mut self, keywords: impl Into<String>) {
        self.dict
            .set("Keywords", COSString::literal(keywords.into().into_bytes()));
    }

    pub fn creator(&self) -> Option<String> {
        self.dict.get_string("Creator").map(|s| s.to_string_lossy())
    }

    pub fn set_creator(&mut self, creator: impl Into<String>) {
        self.dict
            .set("Creator", COSString::literal(creator.into().into_bytes()));
    }

    pub fn producer(&self) -> Option<String> {
        self.dict
            .get_string("Producer")
            .map(|s| s.to_string_lossy())
    }

    pub fn set_producer(&mut self, producer: impl Into<String>) {
        self.dict
            .set("Producer", COSString::literal(producer.into().into_bytes()));
    }

    pub fn set_creation_date(&mut self, dt: DateTime<Utc>) {
        let pdf_date = format_pdf_date(dt);
        self.dict
            .set("CreationDate", COSString::literal(pdf_date.into_bytes()));
    }

    pub fn set_modification_date(&mut self, dt: DateTime<Utc>) {
        let pdf_date = format_pdf_date(dt);
        self.dict
            .set("ModDate", COSString::literal(pdf_date.into_bytes()));
    }
}

pub fn format_pdf_date(dt: DateTime<Utc>) -> String {
    // D:YYYYMMDDHHmmSSZ
    format!("D:{}Z", dt.format("%Y%m%d%H%M%S"))
}

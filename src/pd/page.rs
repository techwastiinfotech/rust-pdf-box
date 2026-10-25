use crate::cos::object::{COSArray, COSDictionary, COSName, COSObject, ObjectId};
use crate::pd::rectangle::PDRectangle;
use crate::pd::resources::PDResources;

/// Represents a single page in a PDF document.
#[derive(Debug, Clone)]
pub struct PDPage {
    pub dict: COSDictionary,
    pub page_id: Option<ObjectId>,
    pub resources: PDResources,
}

impl PDPage {
    /// Creates a new page with the given media box.
    pub fn new(media_box: PDRectangle) -> Self {
        let mut dict = COSDictionary::new();
        dict.set("Type", COSName::new("Page"));
        dict.set("MediaBox", media_box);

        let resources = PDResources::new();
        dict.set("Resources", resources.dict.clone());

        Self {
            dict,
            page_id: None,
            resources,
        }
    }

    /// Loads a page from an existing page dictionary and its object ID.
    pub fn from_dict(dict: COSDictionary, page_id: ObjectId) -> Self {
        let resources_dict = dict.get_dict("Resources").cloned().unwrap_or_default();
        let resources = PDResources::from_dict(resources_dict);

        Self {
            dict,
            page_id: Some(page_id),
            resources,
        }
    }

    pub fn media_box(&self) -> PDRectangle {
        self.dict
            .get_array("MediaBox")
            .and_then(PDRectangle::from_cos_array)
            .unwrap_or(PDRectangle::A4)
    }

    pub fn set_media_box(&mut self, rect: PDRectangle) {
        self.dict.set("MediaBox", rect);
    }

    pub fn crop_box(&self) -> PDRectangle {
        self.dict
            .get_array("CropBox")
            .and_then(PDRectangle::from_cos_array)
            .unwrap_or_else(|| self.media_box())
    }

    pub fn set_crop_box(&mut self, rect: PDRectangle) {
        self.dict.set("CropBox", rect);
    }

    pub fn rotation(&self) -> i32 {
        self.dict.get_integer("Rotate").unwrap_or(0) as i32
    }

    pub fn set_rotation(&mut self, rotation: i32) {
        // Standard PDF rotation is in multiples of 90 degrees
        let normalized = ((rotation % 360) + 360) % 360;
        self.dict.set("Rotate", normalized as i64);
    }

    /// Synchronizes the resources dictionary back into the page dictionary.
    pub fn sync_resources(&mut self) {
        self.dict.set("Resources", self.resources.dict.clone());
    }

    /// Retrieves all content stream ObjectIds referenced in `/Contents`.
    pub fn content_stream_ids(&self) -> Vec<ObjectId> {
        match self.dict.get("Contents") {
            Some(COSObject::Reference(id)) => vec![*id],
            Some(COSObject::Array(arr)) => {
                arr.iter().filter_map(|item| item.as_reference()).collect()
            }
            _ => Vec::new(),
        }
    }

    /// Sets a single content stream reference for `/Contents`.
    pub fn set_contents(&mut self, content_ref: ObjectId) {
        self.dict.set("Contents", content_ref);
    }

    /// Sets multiple content streams for `/Contents`.
    pub fn set_contents_array(&mut self, content_refs: Vec<ObjectId>) {
        let mut arr = COSArray::new();
        for id in content_refs {
            arr.push(id);
        }
        self.dict.set("Contents", arr);
    }
}

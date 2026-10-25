use crate::cos::document::COSDocument;
use crate::cos::object::{COSArray, COSDictionary, COSName, ObjectId};
use crate::error::{PdfError, PdfResult};

/// The Document Catalog dictionary (/Root).
#[derive(Debug, Clone)]
pub struct PDDocumentCatalog {
    pub catalog_id: ObjectId,
    pub pages_id: ObjectId,
}

impl PDDocumentCatalog {
    /// Creates a new catalog and page tree root in the given COSDocument.
    pub fn new(cos_doc: &mut COSDocument) -> Self {
        let pages_id = cos_doc.allocate_id();
        let mut pages_dict = COSDictionary::new();
        pages_dict.set("Type", COSName::new("Pages"));
        pages_dict.set("Kids", COSArray::new());
        pages_dict.set("Count", 0);
        cos_doc.set_object(pages_id, pages_dict);

        let catalog_id = cos_doc.allocate_id();
        let mut cat_dict = COSDictionary::new();
        cat_dict.set("Type", COSName::new("Catalog"));
        cat_dict.set("Pages", pages_id);
        cos_doc.set_object(catalog_id, cat_dict);

        cos_doc.trailer.set("Root", catalog_id);

        Self {
            catalog_id,
            pages_id,
        }
    }

    /// Loads the catalog and resolves the pages root from an existing document.
    pub fn from_doc(cos_doc: &COSDocument) -> PdfResult<Self> {
        let catalog_id = cos_doc
            .trailer
            .get_reference("Root")
            .ok_or_else(|| PdfError::MissingEntry("Trailer missing /Root".to_string()))?;

        let cat_dict = cos_doc
            .get_catalog()
            .ok_or_else(|| PdfError::MissingEntry("Root is not a dictionary".to_string()))?;

        let pages_id = cat_dict.get_reference("Pages").ok_or_else(|| {
            PdfError::MissingEntry("Catalog missing /Pages reference".to_string())
        })?;

        Ok(Self {
            catalog_id,
            pages_id,
        })
    }

    /// Traverses the page tree and collects all page object IDs in sequence.
    pub fn collect_page_ids(&self, cos_doc: &COSDocument) -> Vec<ObjectId> {
        let mut page_ids = Vec::new();
        self.collect_pages_recursive(cos_doc, self.pages_id, &mut page_ids);
        page_ids
    }

    fn collect_pages_recursive(
        &self,
        cos_doc: &COSDocument,
        node_id: ObjectId,
        page_ids: &mut Vec<ObjectId>,
    ) {
        if let Some(obj) = cos_doc.get_object(node_id) {
            if let Some(dict) = obj.as_dict() {
                match dict.get_name("Type") {
                    Some("Page") => {
                        page_ids.push(node_id);
                    }
                    Some("Pages") => {
                        if let Some(kids) = dict.get_array("Kids") {
                            for kid in kids.iter() {
                                if let Some(kid_id) = kid.as_reference() {
                                    self.collect_pages_recursive(cos_doc, kid_id, page_ids);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

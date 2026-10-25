use crate::cos::object::{COSArray, COSDictionary, COSObject, COSStream, ObjectId};
use crate::error::PdfResult;
use crate::pd::document::PDDocument;
use crate::pd::page::PDPage;
use std::collections::BTreeMap;

/// Merges multiple PDF documents into a single document with safe object ID remapping.
pub struct PDFMerger;

impl PDFMerger {
    /// Merges multiple documents in order and returns the unified PDDocument.
    pub fn merge(docs: &[&PDDocument]) -> PdfResult<PDDocument> {
        let mut target_doc = PDDocument::new();
        for &source_doc in docs {
            Self::append_document(&mut target_doc, source_doc)?;
        }
        Ok(target_doc)
    }

    /// Appends all pages and referenced objects from source_doc into target_doc.
    pub fn append_document(target: &mut PDDocument, source: &PDDocument) -> PdfResult<()> {
        let mut id_map: BTreeMap<ObjectId, ObjectId> = BTreeMap::new();

        // 1. Allocate target IDs for all objects in source
        for &src_id in source.cos_doc.objects.keys() {
            let new_id = target.cos_doc.allocate_id();
            id_map.insert(src_id, new_id);
        }

        // 2. Clone and remap objects into target
        for (&src_id, src_obj) in &source.cos_doc.objects {
            let target_id = id_map[&src_id];
            let remapped_obj = Self::remap_object(src_obj, &id_map);
            target.cos_doc.set_object(target_id, remapped_obj);
        }

        // 3. Add remapped pages to target
        for src_page in source.pages() {
            let mut new_page_dict = if let Some(dict_obj) = src_page
                .page_id
                .and_then(|id| source.cos_doc.get_object(id))
            {
                if let Some(d) = dict_obj.as_dict() {
                    Self::remap_dictionary(d, &id_map)
                } else {
                    Self::remap_dictionary(&src_page.dict, &id_map)
                }
            } else {
                Self::remap_dictionary(&src_page.dict, &id_map)
            };

            // Set new parent to target pages root
            new_page_dict.set("Parent", target.catalog.pages_id);

            let page_obj_id = target.cos_doc.allocate_id();
            let new_page = PDPage::from_dict(new_page_dict.clone(), page_obj_id);
            target.cos_doc.set_object(page_obj_id, new_page_dict);
            target.pages.push(new_page);
        }

        target.sync();
        Ok(())
    }

    fn remap_object(obj: &COSObject, id_map: &BTreeMap<ObjectId, ObjectId>) -> COSObject {
        match obj {
            COSObject::Reference(id) => {
                if let Some(&new_id) = id_map.get(id) {
                    COSObject::Reference(new_id)
                } else {
                    COSObject::Reference(*id)
                }
            }
            COSObject::Array(arr) => {
                let mut new_arr = COSArray::new();
                for item in arr.iter() {
                    new_arr.push(Self::remap_object(item, id_map));
                }
                COSObject::Array(new_arr)
            }
            COSObject::Dictionary(d) => COSObject::Dictionary(Self::remap_dictionary(d, id_map)),
            COSObject::Stream(s) => {
                let new_dict = Self::remap_dictionary(&s.dict, id_map);
                COSObject::Stream(COSStream::new(new_dict, s.data.clone()))
            }
            other => other.clone(),
        }
    }

    fn remap_dictionary(
        dict: &COSDictionary,
        id_map: &BTreeMap<ObjectId, ObjectId>,
    ) -> COSDictionary {
        let mut new_dict = COSDictionary::new();
        for (k, v) in &dict.entries {
            new_dict.set(k.clone(), Self::remap_object(v, id_map));
        }
        new_dict
    }
}

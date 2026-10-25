use crate::cos::document::COSDocument;
use crate::cos::object::{COSArray, ObjectId};
use crate::cos::parser::Parser;
use crate::cos::writer::Writer;
use crate::error::{PdfError, PdfResult};
use crate::pd::catalog::PDDocumentCatalog;
use crate::pd::info::PDDocumentInformation;
use crate::pd::page::PDPage;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// High-level PDF document representing pages, metadata, catalog and content streams.
#[derive(Debug, Clone)]
pub struct PDDocument {
    pub cos_doc: COSDocument,
    pub catalog: PDDocumentCatalog,
    pub pages: Vec<PDPage>,
    pub info: PDDocumentInformation,
}

impl Default for PDDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl PDDocument {
    /// Creates a new, empty PDF document with catalog and info dictionaries.
    pub fn new() -> Self {
        let mut cos_doc = COSDocument::new();
        let catalog = PDDocumentCatalog::new(&mut cos_doc);

        let info = PDDocumentInformation::new();
        let info_id = cos_doc.add_object(info.dict.clone());
        cos_doc.trailer.set("Info", info_id);

        Self {
            cos_doc,
            catalog,
            pages: Vec::new(),
            info,
        }
    }

    /// Loads a PDF document from raw byte slice.
    pub fn load_from_bytes(bytes: &[u8]) -> PdfResult<Self> {
        let parser = Parser::new(bytes);
        let cos_doc = parser.parse()?;
        let catalog = PDDocumentCatalog::from_doc(&cos_doc)?;

        // Collect and load all pages
        let page_ids = catalog.collect_page_ids(&cos_doc);
        let mut pages = Vec::with_capacity(page_ids.len());

        for id in page_ids {
            if let Some(obj) = cos_doc.get_object(id) {
                if let Some(dict) = obj.as_dict() {
                    pages.push(PDPage::from_dict(dict.clone(), id));
                }
            }
        }

        let info = if let Some(info_dict) = cos_doc.get_info() {
            PDDocumentInformation::from_dict(info_dict.clone())
        } else {
            PDDocumentInformation::new()
        };

        Ok(Self {
            cos_doc,
            catalog,
            pages,
            info,
        })
    }

    /// Loads a PDF document from a file path.
    pub fn load_from_file(path: impl AsRef<Path>) -> PdfResult<Self> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Self::load_from_bytes(&bytes)
    }

    /// Number of pages in the document.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Returns a slice of all pages.
    pub fn pages(&self) -> &[PDPage] {
        &self.pages
    }

    /// Returns a mutable slice of all pages.
    pub fn pages_mut(&mut self) -> &mut [PDPage] {
        &mut self.pages
    }

    /// Gets a page reference by 0-based index.
    pub fn get_page(&self, index: usize) -> PdfResult<&PDPage> {
        self.pages
            .get(index)
            .ok_or_else(|| PdfError::PageOperation(format!("Page index {index} out of bounds (count {})", self.pages.len())))
    }

    /// Gets a mutable page reference by 0-based index.
    pub fn get_page_mut(&mut self, index: usize) -> PdfResult<&mut PDPage> {
        let count = self.pages.len();
        self.pages
            .get_mut(index)
            .ok_or_else(|| PdfError::PageOperation(format!("Page index {index} out of bounds (count {count})")))
    }

    /// Appends a new page to the document.
    pub fn add_page(&mut self, mut page: PDPage) -> ObjectId {
        page.sync_resources();
        let page_id = if let Some(id) = page.page_id {
            self.cos_doc.set_object(id, page.dict.clone());
            id
        } else {
            let id = self.cos_doc.allocate_id();
            page.page_id = Some(id);
            page.dict.set("Parent", self.catalog.pages_id);
            self.cos_doc.set_object(id, page.dict.clone());
            id
        };

        self.pages.push(page);
        self.update_page_tree();
        page_id
    }

    /// Inserts a page at a given 0-based index.
    pub fn insert_page(&mut self, index: usize, mut page: PDPage) -> ObjectId {
        page.sync_resources();
        let page_id = if let Some(id) = page.page_id {
            self.cos_doc.set_object(id, page.dict.clone());
            id
        } else {
            let id = self.cos_doc.allocate_id();
            page.page_id = Some(id);
            page.dict.set("Parent", self.catalog.pages_id);
            self.cos_doc.set_object(id, page.dict.clone());
            id
        };

        if index >= self.pages.len() {
            self.pages.push(page);
        } else {
            self.pages.insert(index, page);
        }

        self.update_page_tree();
        page_id
    }

    /// Removes a page by 0-based index.
    pub fn remove_page(&mut self, index: usize) -> PdfResult<()> {
        if index >= self.pages.len() {
            return Err(PdfError::PageOperation(format!(
                "Cannot remove page at index {index}, total pages {}",
                self.pages.len()
            )));
        }
        let removed = self.pages.remove(index);
        if let Some(id) = removed.page_id {
            self.cos_doc.objects.remove(&id);
            self.cos_doc.dirty_objects.remove(&id);
        }
        self.update_page_tree();
        Ok(())
    }

    /// Updates the `/Pages` dictionary with the current list of page IDs and count.
    fn update_page_tree(&mut self) {
        let mut kids = COSArray::new();
        for p in &self.pages {
            if let Some(id) = p.page_id {
                kids.push(id);
            }
        }

        if let Some(pages_obj) = self.cos_doc.get_object_mut(self.catalog.pages_id) {
            if let Some(dict) = pages_obj.as_dict_mut() {
                dict.set("Kids", kids);
                dict.set("Count", self.pages.len() as i64);
            }
        }
    }

    /// Synchronizes all modified pages, resources, and info metadata back to the COSDocument.
    pub fn sync(&mut self) {
        for page in &mut self.pages {
            page.sync_resources();
            if let Some(id) = page.page_id {
                self.cos_doc.set_object(id, page.dict.clone());
            }
        }

        let info_id = if let Some(id) = self.cos_doc.trailer.get_reference("Info") {
            id
        } else {
            let id = self.cos_doc.allocate_id();
            self.cos_doc.trailer.set("Info", id);
            id
        };
        self.cos_doc.set_object(info_id, self.info.dict.clone());
        self.update_page_tree();
    }

    /// Saves the document to any writer implementing `Write`.
    pub fn save_to_writer<W: Write>(&mut self, writer: W) -> PdfResult<()> {
        self.sync();
        let mut cos_writer = Writer::new(writer);
        cos_writer.write_document(&self.cos_doc)
    }

    /// Saves the document to a file.
    pub fn save_to_file(&mut self, path: impl AsRef<Path>) -> PdfResult<()> {
        let file = File::create(path)?;
        self.save_to_writer(file)
    }

    /// Appends an incremental update to original bytes and writes the result.
    pub fn save_incremental<W: Write>(&mut self, prev_bytes: &[u8], mut writer: W) -> PdfResult<()> {
        self.sync();
        writer.write_all(prev_bytes)?;

        // Find prev startxref offset in original bytes
        let parser = Parser::new(prev_bytes);
        let prev_startxref = parser.find_startxref().unwrap_or(0);

        let mut cos_writer = Writer::with_offset(writer, prev_bytes.len());
        cos_writer.write_incremental_update(&self.cos_doc, prev_startxref)
    }
}

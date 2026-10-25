use crate::content::stream::{AppendMode, PDPageContentStream};
use crate::error::PdfResult;
use crate::interactive::form::PDAcroForm;
use crate::manipulation::merger::PDFMerger;
use crate::manipulation::splitter::PDFSplitter;
use crate::pd::document::PDDocument;
use crate::pd::page::PDPage;
use crate::pd::rectangle::PDRectangle;
use crate::sign::signer::Signer;
use crate::stamp::stamper::Stamper;
use crate::text::stripper::PDFTextStripper;
use std::fs::File;
use std::path::Path;

/// Context-oriented facade for intuitive, modern PDF processing.
pub struct PdfDocument {
    pub inner: PDDocument,
}

impl Default for PdfDocument {
    fn default() -> Self {
        Self::create()
    }
}

impl PdfDocument {
    /// Creates a new, blank PDF document.
    pub fn create() -> Self {
        Self {
            inner: PDDocument::new(),
        }
    }

    /// Opens a PDF document from a file path.
    pub fn open(path: impl AsRef<Path>) -> PdfResult<Self> {
        let inner = PDDocument::load_from_file(path)?;
        Ok(Self { inner })
    }

    /// Opens a PDF document from in-memory bytes.
    pub fn open_from_bytes(bytes: &[u8]) -> PdfResult<Self> {
        let inner = PDDocument::load_from_bytes(bytes)?;
        Ok(Self { inner })
    }

    /// Adds a blank page with the specified dimensions (e.g. `PDRectangle::A4`).
    pub fn add_blank_page(&mut self, size: PDRectangle) -> usize {
        let page = PDPage::new(size);
        self.inner.add_page(page);
        self.inner.page_count() - 1
    }

    /// Returns the total number of pages.
    pub fn page_count(&self) -> usize {
        self.inner.page_count()
    }

    /// Sets the document title in metadata.
    pub fn set_title(&mut self, title: impl Into<String>) -> &mut Self {
        self.inner.info.set_title(title);
        self
    }

    /// Sets the document author in metadata.
    pub fn set_author(&mut self, author: impl Into<String>) -> &mut Self {
        self.inner.info.set_author(author);
        self
    }

    /// Draws onto a page using a closure that receives a `PDPageContentStream`.
    pub fn draw_on_page<F>(&mut self, page_index: usize, mode: AppendMode, f: F) -> PdfResult<()>
    where
        F: FnOnce(&mut PDPageContentStream, &mut PDDocument) -> PdfResult<()>,
    {
        let mut stream = PDPageContentStream::new(mode);
        stream.save_graphics_state();
        f(&mut stream, &mut self.inner)?;
        stream.restore_graphics_state();
        stream.finish(&mut self.inner, page_index)
    }

    /// Returns a fluent Stamper for watermarking, headers/footers, and rubber stamps.
    pub fn stamp(&self) -> Stamper {
        Stamper::new()
    }

    /// Applies a simple diagonal text watermark across all pages.
    pub fn watermark(&mut self, text: impl Into<String>) -> PdfResult<()> {
        self.stamp()
            .simple_watermark(text)
            .apply(&mut self.inner)
    }

    /// Digitally signs the document using the given Signer and returns signed PDF bytes.
    pub fn sign(&mut self, signer: Signer) -> PdfResult<Vec<u8>> {
        signer.sign_document(&mut self.inner)
    }

    /// Extracts all text from the document.
    pub fn extract_text(&self) -> PdfResult<String> {
        PDFTextStripper::extract_text(&self.inner)
    }

    /// Merges another document into this document.
    pub fn merge(&mut self, other: &PdfDocument) -> PdfResult<()> {
        PDFMerger::append_document(&mut self.inner, &other.inner)
    }

    /// Splits this document into individual 1-page documents.
    pub fn split_single_pages(&self) -> PdfResult<Vec<PdfDocument>> {
        let docs = PDFSplitter::split_into_single_pages(&self.inner)?;
        Ok(docs.into_iter().map(|inner| PdfDocument { inner }).collect())
    }

    /// Flattens all interactive form fields into static page contents.
    pub fn flatten_forms(&mut self) -> PdfResult<()> {
        if let Some(mut acro_form) = PDAcroForm::from_doc(&self.inner) {
            acro_form.flatten(&mut self.inner)?;
        }
        Ok(())
    }

    /// Saves the document to a file path.
    pub fn save(&mut self, path: impl AsRef<Path>) -> PdfResult<()> {
        let file = File::create(path)?;
        self.inner.save_to_writer(file)
    }

    /// Saves the document to an in-memory byte vector.
    pub fn to_bytes(&mut self) -> PdfResult<Vec<u8>> {
        let mut buf = Vec::new();
        self.inner.save_to_writer(&mut buf)?;
        Ok(buf)
    }
}

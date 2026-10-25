use crate::error::PdfResult;
use crate::manipulation::merger::PDFMerger;
use crate::pd::document::PDDocument;

/// Splits PDF documents into single pages or page chunks.
pub struct PDFSplitter;

impl PDFSplitter {
    /// Splits a document into multiple 1-page documents.
    pub fn split_into_single_pages(doc: &PDDocument) -> PdfResult<Vec<PDDocument>> {
        Self::split_by_chunk_size(doc, 1)
    }

    /// Splits a document into chunks of `chunk_size` pages each.
    pub fn split_by_chunk_size(doc: &PDDocument, chunk_size: usize) -> PdfResult<Vec<PDDocument>> {
        let chunk_size = chunk_size.max(1);
        let mut results = Vec::new();
        let total_pages = doc.page_count();

        for chunk_start in (0..total_pages).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(total_pages);
            let mut sub_doc = PDDocument::new();

            for p_idx in chunk_start..chunk_end {
                let page = doc.get_page(p_idx)?;
                let mut temp_single = PDDocument::new();
                temp_single.add_page(page.clone());
                PDFMerger::append_document(&mut sub_doc, &temp_single)?;
            }

            results.push(sub_doc);
        }

        Ok(results)
    }
}

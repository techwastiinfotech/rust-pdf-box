use crate::error::{PdfError, PdfResult};
use crate::pd::document::PDDocument;

/// Common page operations (rotation, duplicate, reorder, delete).
pub struct PageOps;

impl PageOps {
    /// Rotates a page by 90, 180, or 270 degrees clockwise.
    pub fn rotate_page(doc: &mut PDDocument, page_index: usize, degrees: i32) -> PdfResult<()> {
        let page = doc.get_page_mut(page_index)?;
        let current_rotation = page.rotation();
        page.set_rotation(current_rotation + degrees);
        Ok(())
    }

    /// Duplicates a page at `page_index` and appends it to the end of the document.
    pub fn duplicate_page(doc: &mut PDDocument, page_index: usize) -> PdfResult<()> {
        let page = doc.get_page(page_index)?.clone();
        doc.add_page(page);
        Ok(())
    }

    /// Moves a page from `from_index` to `to_index`.
    pub fn move_page(doc: &mut PDDocument, from_index: usize, to_index: usize) -> PdfResult<()> {
        let total = doc.page_count();
        if from_index >= total || to_index >= total {
            return Err(PdfError::PageOperation(format!(
                "Invalid move page bounds: from {from_index} to {to_index}, total {total}"
            )));
        }

        let page = doc.pages.remove(from_index);
        doc.pages.insert(to_index, page);
        doc.sync();
        Ok(())
    }
}

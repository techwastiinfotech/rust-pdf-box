# Text Extraction (`PDFTextStripper`)

`PDFTextStripper` extracts plain text from PDF content streams while respecting text positioning, word spacing, and line breaks.

---

## 1. Overview

PDF content streams represent text as instructions such as:
- `BT` (Begin Text) and `ET` (End Text)
- `Tf` (Set Font and Size)
- `Td` / `TD` / `Tm` (Position and Text Matrix)
- `Tj` / `TJ` / `'` / `"` (Show Text String and Kerning Arrays)

`PDFTextStripper` parses these operators, decodes string literals and hex sequences according to font encodings, and groups characters into words, sentences, and paragraphs.

---

## 2. Basic Text Extraction

Extract the entire text content of a PDF file:

```rust
use rust_pdf_box::prelude::*;
use rust_pdf_box::text::PDFTextStripper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = PdfDocument::open("document.pdf")?;

    let stripper = PDFTextStripper::new();
    let extracted_text = stripper.get_text(&doc.inner)?;

    println!("--- Extracted Content ---");
    println!("{}", extracted_text);

    Ok(())
}
```

---

## 3. Page-by-Page Extraction

To extract text from specific pages:

```rust
use rust_pdf_box::prelude::*;
use rust_pdf_box::text::PDFTextStripper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = PdfDocument::open("document.pdf")?;
    let stripper = PDFTextStripper::new();

    let page_count = doc.page_count();
    for page_num in 1..=page_count {
        let page_text = stripper.get_page_text(&doc.inner, page_num)?;
        println!("=== Page {} ===", page_num);
        println!("{}", page_text);
    }

    Ok(())
}
```

---

## 4. CLI Extraction

You can also use the standalone `pdfbox` command-line tool:

```bash
# Print to stdout
pdfbox text document.pdf

# Save to output file
pdfbox text document.pdf extracted_output.txt
```

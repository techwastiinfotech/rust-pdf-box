# Apache PDFBox to rust-pdf-box Migration Guide

This guide helps Java developers familiar with **Apache PDFBox** transition seamlessly to **rust-pdf-box**. It highlights conceptual equivalents, architectural differences, memory management benefits, and side-by-side code comparisons.

---

## 1. Architectural Philosophy

| Dimension | Apache PDFBox (Java) | rust-pdf-box (Rust) |
| :--- | :--- | :--- |
| **Runtime & Dependencies** | Requires JVM (JDK 8+), off-heap memory buffers, BouncyCastle for CMS | Pure Rust, zero C-libraries, single lightweight static binary |
| **Concurrency & Safety** | Manual synchronization, mutable shared state | Safe multithreading (`Send + Sync`), compiler-enforced borrow checker |
| **Resource Management** | `AutoCloseable` / `close()`, garbage collection pauses | RAII (deterministic destruction), minimal allocation, zero memory leaks |
| **API Paradigm** | Imperative object graphs with mutable setters | Context-oriented fluent builders with strict type guarantees |
| **Cryptographic Signatures**| Heavy external provider configuration (BouncyCastle, PKCS#11) | Pure Rust self-signed key generation, ISO 32000 detached PKCS#7 signing |

---

## 2. Class & Struct Mapping Cheat Sheet

| Apache PDFBox (Java) | rust-pdf-box (Rust) | Notes |
| :--- | :--- | :--- |
| `org.apache.pdfbox.pdmodel.PDDocument` | `rust_pdf_box::facade::PdfDocument` or `rust_pdf_box::pd::PDDocument` | High-level document facade or granular PD model |
| `org.apache.pdfbox.pdmodel.PDPage` | `rust_pdf_box::pd::PDPage` | Page object holding media box and resources |
| `org.apache.pdfbox.pdmodel.common.PDRectangle` | `rust_pdf_box::pd::PDRectangle` | Standard paper sizes (`LETTER`, `A4`, `LEGAL`, custom) |
| `org.apache.pdfbox.pdmodel.PDPageContentStream` | `rust_pdf_box::content::PDPageContentStream` | Vector drawing, color models, text matrix |
| `org.apache.pdfbox.pdmodel.font.PDType1Font` | `rust_pdf_box::font::PDType1Font` | Built-in Standard 14 PDF fonts with glyph width tables |
| `org.apache.pdfbox.pdmodel.font.PDTrueTypeFont` | `rust_pdf_box::font::PDTrueTypeFont` | Embedded TrueType fonts (`.ttf` parsing) |
| `org.apache.pdfbox.pdmodel.graphics.image.PDImageXObject` | `rust_pdf_box::image::PDImageXObject` | Native JPEG passthrough (`/DCTDecode`) & PNG with `/SMask` |
| `org.apache.pdfbox.text.PDFTextStripper` | `rust_pdf_box::text::PDFTextStripper` | Content stream text extraction & layout reconstruction |
| `org.apache.pdfbox.multipdf.PDFMergerUtility` | `rust_pdf_box::manipulation::PDFMerger` | Merging PDFs with conflict-free object ID remapping |
| `org.apache.pdfbox.multipdf.Splitter` | `rust_pdf_box::manipulation::PDFSplitter` | Splitting documents into page ranges or single pages |
| `org.apache.pdfbox.pdmodel.interactive.form.PDAcroForm` | `rust_pdf_box::interactive::PDAcroForm` | Form inspection, setting values, flattening |
| `org.apache.pdfbox.pdmodel.interactive.digitalsignature.PDSignature` | `rust_pdf_box::sign::Signer` | Cryptographic signature creation & appearance generation |
| `org.apache.pdfbox.cos.*` (`COSDictionary`, `COSArray`, etc.) | `rust_pdf_box::cos::*` | Direct low-level access to the PDF object tree |

---

## 3. Side-by-Side Code Comparisons

### 3.1 Creating a Blank Document with Text

#### Apache PDFBox (Java):
```java
try (PDDocument doc = new PDDocument()) {
    PDPage page = new PDPage(PDRectangle.A4);
    doc.addPage(page);

    try (PDPageContentStream stream = new PDPageContentStream(doc, page)) {
        stream.beginText();
        stream.setFont(PDType1Font.HELVETICA_BOLD, 16);
        stream.newLineAtOffset(50, 750);
        stream.showText("Hello from Apache PDFBox!");
        stream.endText();
    }

    doc.save("output.pdf");
}
```

#### rust-pdf-box (Rust):
```rust
use rust_pdf_box::prelude::*;

fn main() -> PdfResult<()> {
    let mut doc = PdfDocument::create();
    let page_idx = doc.add_page(PDRectangle::A4);

    let mut stream = PDPageContentStream::new(&mut doc.inner, page_idx, AppendMode::Overwrite)?;
    stream.begin_text()?;
    let font_ref = stream.set_font_type1(&mut doc.inner, page_idx, PDType1Font::HelveticaBold, 16.0)?;
    stream.move_text_position(50.0, 750.0)?;
    stream.show_text("Hello from rust-pdf-box!")?;
    stream.end_text()?;

    doc.save("output.pdf")?;
    Ok(())
}
```

---

### 3.2 Stamping & Watermarking

In Apache PDFBox, adding a background watermark across multiple pages typically requires building a secondary overlay PDF with `Overlay` or manually opening a content stream in `PREPEND` mode for every page.

#### rust-pdf-box Fluent Context API:
```rust
use rust_pdf_box::prelude::*;

let mut doc = PdfDocument::open("document.pdf")?;

// 1. Semi-transparent diagonal watermark across all pages
doc.stamp()
    .pages(PageRange::All)
    .watermark("INTERNAL ONLY", 48.0, Color::rgb(0.9, 0.1, 0.1), 45.0, 0.25)
    .apply(&mut doc.inner)?;

// 2. Official rubber stamp on the first page
doc.stamp()
    .pages(PageRange::Single(1))
    .position(StampPosition::TopRight { margin_x: 36.0, margin_y: 36.0 })
    .rubber_stamp("APPROVED", Some("Audit Team".into()), Color::rgb(0.0, 0.5, 0.2), -10.0)
    .apply(&mut doc.inner)?;

// 3. Running page numbers in the footer
doc.stamp()
    .pages(PageRange::All)
    .footer("Page {page} of {total}", StampPosition::BottomCenter { margin_y: 25.0 }, 10.0, Color::GRAY)
    .apply(&mut doc.inner)?;

doc.save("stamped.pdf")?;
```

---

### 3.3 Digital Signatures (PKCS#7 / CMS)

In Apache PDFBox, signing requires implementing `SignatureInterface`, providing a Java Keystore (`KeyStore.getInstance("PKCS12")`), wiring BouncyCastle CMS generators, calculating byte ranges manually, and managing temporary files.

#### rust-pdf-box Pure Rust Digital Signing:
```rust
use rust_pdf_box::prelude::*;

let mut doc = PdfDocument::open("contract.pdf")?;

// Generate self-signed RSA key in memory (or load standard PEM / PKCS#8 key)
let key = SigningKey::generate_self_signed("CN=Techwasti Signer, O=Techwasti Infotech")?;

let signer = Signer::new(key)
    .with_name("Techwasti Infotech")
    .with_reason("Contract Authorization")
    .with_location("Mumbai, India")
    .with_visual_stamp(
        VisualSignature::at_page(1)
            .rect(50.0, 50.0, 260.0, 80.0)
            .title("Digitally Signed Document")
            .detail("Authorized by Techwasti Security\nIntegrity: ISO 32000 Detached PKCS#7")
    );

// Produces signed PDF bytes with cryptographically valid /ByteRange and /Contents
let signed_bytes = doc.sign(signer)?;
std::fs::write("contract_signed.pdf", &signed_bytes)?;

// Cryptographic verification
let report = SignatureVerifier::verify(&signed_bytes)?;
assert!(report.is_valid);
```

---

### 3.4 AcroForms & Field Flattening

#### Apache PDFBox (Java):
```java
PDAcroForm acroForm = doc.getDocumentCatalog().getAcroForm();
if (acroForm != null) {
    PDField field = acroForm.getField("EmployeeName");
    if (field instanceof PDTextField) {
        field.setValue("Alex Johnson");
    }
    acroForm.flatten();
}
```

#### rust-pdf-box (Rust):
```rust
use rust_pdf_box::prelude::*;

let mut doc = PdfDocument::open("application_form.pdf")?;

if let Some(mut form) = doc.inner.catalog_mut()?.acro_form()? {
    // Inspect or update fields
    form.set_field_value(&mut doc.inner, "EmployeeName", "Alex Johnson")?;
    form.set_field_value(&mut doc.inner, "Approved", "Yes")?;

    // Burn interactive fields into static visible page graphics
    form.flatten(&mut doc.inner)?;
}

doc.save("application_flattened.pdf")?;
```

---

### 3.5 PDF Merging and Splitting

#### Apache PDFBox (Java):
```java
PDFMergerUtility merger = new PDFMergerUtility();
merger.addSource("doc1.pdf");
merger.addSource("doc2.pdf");
merger.setDestinationFileName("merged.pdf");
merger.mergeDocuments(MemoryUsageSetting.setupMainMemoryOnly());
```

#### rust-pdf-box (Rust):
```rust
use rust_pdf_box::prelude::*;

let doc1 = PdfDocument::open("doc1.pdf")?;
let doc2 = PdfDocument::open("doc2.pdf")?;

let mut merged = PdfDocument::create();
merged.merge(&doc1)?;
merged.merge(&doc2)?;
merged.save("merged.pdf")?;

// Or split:
let splitter = PDFSplitter::new(&doc1);
let page_docs = splitter.split_into_pages()?;
for (idx, page_doc) in page_docs.into_iter().enumerate() {
    page_doc.save(&format!("page_{}.pdf", idx + 1))?;
}
```

---

## 4. Key Advantages of rust-pdf-box

1. **Performance**: No garbage collection spikes, lower memory usage, and near-instant cold starts (ideal for AWS Lambda, Cloud Run, Docker containers, and edge microservices).
2. **Safe Concurrency**: Immutable PDF documents can be read simultaneously across threads without locks.
3. **Ergonomic Facade**: Common workflows take 3 lines of code instead of 30 lines of boilerplate.
4. **Self-Contained**: No external native binaries (`libpng`, `libjpeg`, `libssl`, `poppler`) required.

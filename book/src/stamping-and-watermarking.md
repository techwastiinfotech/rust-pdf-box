# Context-Oriented Stamping & Watermarking Guide

`rust-pdf-box` provides a fluent, context-oriented stamping engine designed to make watermarking, rubber stamping, headers/footers, and bates numbering effortless in Rust.

---

## 1. Quick Example

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::open("invoice.pdf")?;

    // Fluent watermark on all pages
    doc.stamp()
        .pages(PageRange::All)
        .watermark("CONFIDENTIAL", 54.0, Color::rgb(0.85, 0.1, 0.1), 45.0, 0.25)
        .apply(&mut doc.inner)?;

    doc.save("invoice_watermarked.pdf")?;
    Ok(())
}
```

---

## 2. Targeting Page Ranges

You can selectively target pages using `PageRange`:

| PageRange Variant | Description | Example |
| :---------------- | :---------- | :------ |
| `PageRange::All` | Applies to every page in the document | `doc.stamp().pages(PageRange::All)` |
| `PageRange::Single(n)` | Applies to 1-based page `n` | `doc.stamp().pages(PageRange::Single(1))` |
| `PageRange::Range(a..=b)` | Applies to range `a` through `b` (inclusive) | `doc.stamp().pages(PageRange::Range(2..=5))` |
| `PageRange::Odd` | Applies to odd pages (1, 3, 5, ...) | `doc.stamp().pages(PageRange::Odd)` |
| `PageRange::Even` | Applies to even pages (2, 4, 6, ...) | `doc.stamp().pages(PageRange::Even)` |
| `PageRange::Pages(vec)` | Applies to specific page numbers | `doc.stamp().pages(PageRange::Pages(vec![1, 4, 7]))` |

---

## 3. Stamp Placement and Positions

Use `StampPosition` to anchor your stamp:

- `StampPosition::Center`: Centered horizontally and vertically on the page.
- `StampPosition::TopLeft { margin_x, margin_y }`: Upper-left corner with offsets.
- `StampPosition::TopCenter { margin_y }`: Top-centered (ideal for headers).
- `StampPosition::TopRight { margin_x, margin_y }`: Upper-right corner (ideal for stamps/seals).
- `StampPosition::BottomLeft { margin_x, margin_y }`: Lower-left corner.
- `StampPosition::BottomCenter { margin_y }`: Bottom-centered (ideal for footers & page numbers).
- `StampPosition::BottomRight { margin_x, margin_y }`: Lower-right corner.
- `StampPosition::Custom { x, y }`: Exact user space coordinates (1/72 inch).

---

## 4. Layering: UnderContent vs OverContent

- **`StampLayer::UnderContent`**: Prepend graphics before page contents. Perfect for semi-transparent diagonal watermarks ("CONFIDENTIAL", "DRAFT") so the stamp sits behind page text and never obscures critical data.
- **`StampLayer::OverContent`**: Append graphics after page contents. Ideal for rubber stamps ("APPROVED", "PAID"), visual signatures, headers, and footers.

---

## 5. Rubber Stamps

Create authentic boxed rubber stamps with rounded borders and optional subtitles:

```rust
doc.stamp()
    .pages(PageRange::Single(1))
    .position(StampPosition::TopRight { margin_x: 36.0, margin_y: 36.0 })
    .rubber_stamp(
        "APPROVED",
        Some("Techwasti Infotech".to_string()),
        Color::rgb(0.0, 0.6, 0.2), // Green border & text
        -15.0,                      // Slight rotation angle
    )
    .apply(&mut doc.inner)?;
```

---

## 6. Running Headers & Footers with Dynamic Placeholders

Headers and footers support automatic `{page}` and `{total}` substitutions:

```rust
// Running Header
doc.stamp()
    .pages(PageRange::All)
    .header(
        "Statement of Work - Page {page} of {total}",
        StampPosition::TopCenter { margin_y: 20.0 },
        10.0,
        Color::GRAY,
    )
    .apply(&mut doc.inner)?;

// Running Footer
doc.stamp()
    .pages(PageRange::All)
    .footer(
        "CONFIDENTIAL - Techwasti Infotech (C) 2026",
        StampPosition::BottomCenter { margin_y: 20.0 },
        9.0,
        Color::GRAY,
    )
    .apply(&mut doc.inner)?;
```

---

## 7. Image Stamping

You can stamp PNG (with transparency) or JPEG logos onto any page:

```rust
let img_bytes = std::fs::read("logo.png")?;
let img = PDImageXObject::create_from_image_bytes(&mut doc.inner.cos_doc, &img_bytes)?;

doc.stamp()
    .pages(PageRange::Single(1))
    .position(StampPosition::TopRight { margin_x: 40.0, margin_y: 40.0 })
    .image(img, 120.0, 50.0, 0.85) // width, height, opacity
    .apply(&mut doc.inner)?;
```

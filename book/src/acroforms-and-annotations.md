# Interactive Forms (AcroForms) and Annotations

`rust-pdf-box` provides a robust engine for creating, inspecting, filling, and flattening interactive PDF forms (**AcroForms**) and annotations (**Links, Highlights, and Notes**).

---

## 1. Overview & Architecture

Interactive PDF forms are rooted in the document catalog dictionary (`/Root`) under the `/AcroForm` entry. Each AcroForm contains:
- `/Fields`: An array of field dictionaries (`PDFormField`).
- `/NeedAppearances`: A boolean indicating whether viewers should regenerate appearance streams.
- `/DA`: Default appearance string containing font and color commands (e.g., `/Helv 10 Tf 0 g`).

```text
Document Catalog (/Root)
 └── /AcroForm
      ├── /Fields [ Ref(10 0 R), Ref(11 0 R) ]
      ├── /DA "(/Helv 10 Tf 0 g)"
      └── /NeedAppearances true
```

Annotations reside directly on individual pages under `/Annots` and link visual geometry (`/Rect`) with actions (such as URI navigation) or form field widgets (`/Subtype /Widget`).

---

## 2. Reading and Inspecting Form Fields

You can query all form fields present in a document:

```rust
use rust_pdf_box::prelude::*;

let mut doc = PdfDocument::open("sample_form.pdf")?;

if let Some(form) = doc.inner.catalog_mut()?.acro_form()? {
    let fields = form.fields(&doc.inner)?;
    for field in &fields {
        println!("Field Name: {}", field.name);
        println!("  Type: {:?}", field.field_type);
        println!("  Value: {:?}", field.value);
        println!("  Read-only: {}", field.is_read_only());
    }
}
```

### Supported Field Types

| Field Type | PDF Type Code | Description |
| :--- | :--- | :--- |
| `FormFieldType::Text` | `/Tx` | Single-line or multi-line text input fields |
| `FormFieldType::Button` | `/Btn` | Pushbuttons, checkboxes (`/Yes` or `/Off`), and radio buttons |
| `FormFieldType::Choice` | `/Ch` | Dropdown combo boxes and multi-selection list boxes |
| `FormFieldType::Signature`| `/Sig` | Cryptographic signature placeholders |
| `FormFieldType::Unknown` | Custom | Vendor-specific or specialized field types |

---

## 3. Modifying Field Values

To set or update form field values programmatically:

```rust
use rust_pdf_box::prelude::*;

let mut doc = PdfDocument::open("sample_form.pdf")?;

if let Some(mut form) = doc.inner.catalog_mut()?.acro_form()? {
    // Fill text fields
    form.set_field_value(&mut doc.inner, "FullName", "Jane Doe")?;
    form.set_field_value(&mut doc.inner, "Email", "jane.doe@techwasti.com")?;
    form.set_field_value(&mut doc.inner, "Comments", "Expedited delivery requested.")?;

    // Check a checkbox (typically "Yes" or "Off")
    form.set_field_value(&mut doc.inner, "SubscribeNewsletter", "Yes")?;
}

doc.save("filled_form.pdf")?;
```

---

## 4. Form Flattening

Form flattening converts interactive widgets into static, non-editable vector graphics directly rendered onto the page content streams, and removes the `/AcroForm` catalog entry.

### Why Flatten?
- **Security & Immutability**: Prevents recipients from tampering with filled data in legal contracts and invoices.
- **Universal Rendering**: Ensures consistent appearance across third-party mobile viewers and print drivers that do not evaluate AcroForm appearances.
- **Archival Compliance**: Essential for PDF/A long-term archiving standards.

### How to Flatten:

```rust
use rust_pdf_box::prelude::*;

let mut doc = PdfDocument::open("filled_form.pdf")?;

if let Some(mut form) = doc.inner.catalog_mut()?.acro_form()? {
    // Burns all field values into page content streams and clears widget annotations
    form.flatten(&mut doc.inner)?;
}

doc.save("contract_permanent.pdf")?;
```

---

## 5. Adding Annotations to Pages

### Clickable Hyperlink Annotations (`/Link`)

You can attach clickable URI links to arbitrary page rectangles:

```rust
use rust_pdf_box::prelude::*;
use rust_pdf_box::interactive::PDAnnotation;

let mut doc = PdfDocument::create();
let page_idx = doc.add_page(PDRectangle::A4);

// Create a clickable link rectangle: (x, y, width, height)
let link = PDAnnotation::link(
    PDRectangle::new(50.0, 700.0, 200.0, 20.0),
    "https://techwasti.com"
);

// Register annotation on the page
doc.inner.page_mut(page_idx)?.add_annotation(&mut doc.inner, link)?;
doc.save("document_with_links.pdf")?;
```

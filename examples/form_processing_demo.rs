use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rust-pdf-box: AcroForm Inspection, Filling & Flattening Demo ===");

    // 1. Load the sample AcroForm PDF
    let mut doc = PdfDocument::open("tests/fixtures/sample_form.pdf")?;

    // 2. Inspect all form fields
    let acro_form =
        PDAcroForm::from_doc(&doc.inner).expect("Document does not contain an AcroForm");

    println!("Detected Form Fields:");
    for field in acro_form.fields(&doc.inner) {
        println!(
            "  -> Name: {:<12} | Type: {:<10?} | Current Value: {:?}",
            field.name, field.field_type, field.value
        );
    }

    // 3. Programmatically populate form field values
    println!("\nUpdating form field values...");
    let mut acro_form_mut = acro_form;
    acro_form_mut.set_field_value(&mut doc.inner, "full_name", "Robert Oppenheimer")?;
    acro_form_mut.set_field_value(&mut doc.inner, "email", "oppenheimer@techwasti.com")?;
    acro_form_mut.set_field_value(&mut doc.inner, "company", "Techwasti Quantum Lab")?;

    println!("Form fields updated successfully!");

    // 4. Flatten the form: converts interactive widgets to static PDF graphics
    println!("Flattening form (burning interactive fields into static page content)...");
    doc.flatten_forms()?;

    // 5. Save the finalized PDF
    doc.save("flattened_form_demo.pdf")?;
    println!("Saved flattened form to 'flattened_form_demo.pdf'!");

    Ok(())
}

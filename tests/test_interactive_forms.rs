use rust_pdf_box::prelude::*;

#[test]
fn test_acroform_reading_writing_and_flattening() {
    let mut doc = PdfDocument::open("tests/fixtures/sample_form.pdf")
        .expect("failed loading sample_form.pdf");

    // 1. Inspect form fields
    let acro = PDAcroForm::from_doc(&doc.inner).expect("has AcroForm");
    let fields = acro.fields(&doc.inner);
    assert_eq!(fields.len(), 3);

    let names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
    assert!(names.contains(&"full_name".to_string()));
    assert!(names.contains(&"email".to_string()));
    assert!(names.contains(&"company".to_string()));

    // 2. Modify field values
    let mut acro_mut = acro;
    acro_mut.set_field_value(&mut doc.inner, "full_name", "Nikola Tesla").expect("set full_name");
    acro_mut.set_field_value(&mut doc.inner, "email", "tesla@techwasti.com").expect("set email");

    // Verify modified values in memory
    let updated_fields = acro_mut.fields(&doc.inner);
    let name_field = updated_fields.iter().find(|f| f.name == "full_name").unwrap();
    assert_eq!(name_field.value, Some("Nikola Tesla".to_string()));

    // 3. Flatten form
    doc.flatten_forms().expect("flatten failed");

    // After flattening, /AcroForm should be removed
    assert!(PDAcroForm::from_doc(&doc.inner).is_none());

    // Save and reload to verify format integrity
    let bytes = doc.to_bytes().expect("to_bytes failed");
    let reloaded = PdfDocument::open_from_bytes(&bytes).expect("open failed");
    assert_eq!(reloaded.page_count(), 1);
    assert!(PDAcroForm::from_doc(&reloaded.inner).is_none());
}

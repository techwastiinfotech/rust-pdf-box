use rust_pdf_box::prelude::*;

#[test]
fn test_digital_signature_integrity_and_tamper_detection() {
    let mut doc = PdfDocument::create();
    doc.add_blank_page(PDRectangle::A4);

    doc.draw_on_page(0, AppendMode::Overwrite, |stream, inner_doc| {
        stream.begin_text();
        stream.set_font_type1(inner_doc, 0, PDType1Font::HelveticaBold, 16.0);
        stream.show_text_at("Confidential Master Services Agreement", 50.0, 750.0);
        stream.end_text();
        Ok(())
    }).expect("draw failed");

    // 1. Generate key and sign
    let key = SigningKey::generate_self_signed("CN=Techwasti Signer, O=Techwasti Infotech")
        .expect("key generation failed");

    let signer = Signer::new(key)
        .with_name("Chief Security Officer")
        .with_reason("Legal Approval")
        .with_location("India")
        .with_visual_stamp(
            VisualSignature::at_page(1)
                .rect(50.0, 50.0, 250.0, 120.0)
                .title("Digitally Signed")
                .detail("Signer: Techwasti CSO\nStatus: Verified"),
        );

    let signed_bytes = doc.sign(signer).expect("signing failed");

    // 2. Verify untouched document
    let valid_report = SignatureVerifier::verify(&signed_bytes).expect("verification failed");
    assert!(valid_report.is_valid, "Untouched PDF signature must be valid");
    assert_eq!(valid_report.signer_name, Some("Chief Security Officer".to_string()));
    assert_eq!(valid_report.reason, Some("Legal Approval".to_string()));

    // 3. Tamper detection: flip a byte in the document body before the signature
    let mut tampered_bytes = signed_bytes.clone();
    // Byte 50 is in the PDF header / initial objects (Range 1)
    tampered_bytes[50] ^= 0xFF;

    let tampered_report = SignatureVerifier::verify(&tampered_bytes).expect("verification ran");
    assert!(
        !tampered_report.is_valid,
        "Tampered document must fail cryptographic digest verification!"
    );
}

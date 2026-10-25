use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rust-pdf-box: Digital Signing Demo ===");

    // 1. Create a document
    let mut doc = PdfDocument::create();
    doc.set_title("Executive Employment Agreement")
        .set_author("Techwasti Infotech");

    doc.add_blank_page(PDRectangle::A4);

    doc.draw_on_page(0, AppendMode::Overwrite, |stream, inner_doc| {
        stream.begin_text();
        stream.set_font_type1(inner_doc, 0, PDType1Font::HelveticaBold, 20.0);
        stream.set_fill_color(Color::rgb(0.05, 0.1, 0.25));
        stream.show_text_at("EXECUTIVE SERVICE AGREEMENT", 50.0, 750.0);
        stream.end_text();

        stream.begin_text();
        stream.set_font_type1(inner_doc, 0, PDType1Font::Helvetica, 11.0);
        stream.set_fill_color(Color::rgb(0.2, 0.2, 0.2));
        stream.show_text_multiline(
            &[
                "This document confirms acceptance of mutual obligations.",
                "Authorized signing officer has validated terms electronically.",
                "The digital signature below is legally binding under international e-sign laws.",
            ],
            50.0,
            710.0,
            16.0,
        );
        stream.end_text();
        Ok(())
    })?;

    // 2. Generate a self-signed X.509 certificate and RSA key in pure Rust (Zero OpenSSL dependency required!)
    println!("Generating RSA key pair and self-signed X.509 certificate...");
    let signing_key =
        SigningKey::generate_self_signed("CN=Techwasti Signer, O=Techwasti Infotech, C=IN")?;

    // 3. Configure the Signer with a visual appearance stamp
    let signer = Signer::new(signing_key)
        .with_name("Techwasti Infotech Representative")
        .with_reason("Approval and Execution of Terms")
        .with_location("Bangalore, India")
        .with_contact_info("legal@techwasti.com")
        .with_visual_stamp(
            VisualSignature::at_page(1)
                .rect(50.0, 80.0, 320.0, 160.0)
                .title("Digitally Signed by Techwasti Infotech")
                .detail("Signer: Techwasti Authorized Signatory\nDate: 2026-10-25 13:30 UTC\nReason: Service Contract Approval\nStatus: Cryptographically Verified")
                .border_color(Color::rgb(0.0, 0.45, 0.85))
                .background_color(Color::rgb(0.95, 0.98, 1.0)),
        );

    // 4. Sign the PDF
    println!("Computing detached PKCS#7 / CMS SHA-256 signature and updating ByteRange...");
    let signed_bytes = doc.sign(signer)?;

    // 5. Save the digitally signed PDF
    std::fs::write("signed_contract_demo.pdf", &signed_bytes)?;
    println!("Saved digitally signed document to 'signed_contract_demo.pdf'!");

    // 6. Verify signature integrity in the saved PDF
    println!("Verifying digital signature in saved file...");
    let report = SignatureVerifier::verify(&signed_bytes)?;
    println!("Verification Result:");
    println!("  -> Valid: {}", report.is_valid);
    println!("  -> Signer: {:?}", report.signer_name);
    println!("  -> Reason: {:?}", report.reason);
    println!("  -> Location: {:?}", report.location);
    println!("  -> Signing Time: {:?}", report.signing_time);
    println!(
        "  -> Document SHA-256 Digest: {}",
        report.computed_digest_hex
    );

    assert!(report.is_valid, "Signature verification failed!");
    println!("Digital signature verified successfully!");

    Ok(())
}

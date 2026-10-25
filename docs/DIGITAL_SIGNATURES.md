# Digital Signatures & Security Guide

`rust-pdf-box` implements full **ISO 32000-1 / ISO 32000-2** and Adobe Acrobat compliant detached PKCS#7 / CMS digital signatures in pure Rust.

---

## 1. How PDF Digital Signing Works

A PDF signature does NOT hash the entire file as a single blob. Instead, it leaves an exact byte placeholder for the signature dictionary (`/Contents <00...00>`), and hashes all other bytes in the document using `/ByteRange`:

```text
[ Document Bytes Part 1 ]  <-- Range 1: 0 to offset1
/Contents < 3082... >      <-- Excluded signature bytes gap (offset1 to offset2)
[ Document Bytes Part 2 ]  <-- Range 2: offset2 to offset2 + len2
```

1. **Placeholder Allocation**: The document is prepared with `/ByteRange [ 0 offset1 offset2 len2 ]` and `/Contents <000...000>`.
2. **Digest Calculation**: SHA-256 is computed over Part 1 and Part 2.
3. **PKCS#7 / CMS Generation**:
   - `ContentType`: `id-data` (1.2.840.113549.1.7.1)
   - `SignedAttributes`:
     - `contentType`: `id-data`
     - `messageDigest`: SHA-256 hash of Part 1 + Part 2
     - `signingTime`: timestamp
   - `SignerInfo`: RSA-SHA256 signature calculated over DER encoded `SignedAttributes`.
4. **Hex Insertion**: The DER bytes of the PKCS#7 structure are converted to uppercase hexadecimal and written directly into the placeholder inside `<` and `>`.

---

## 2. In-Memory Self-Signed Certificate Generation

To eliminate the need for complicated OpenSSL command-line invocations, `rust-pdf-box` provides a built-in X.509 v3 self-signed certificate generator:

```rust
use rust_pdf_box::prelude::*;

let key = SigningKey::generate_self_signed("CN=Techwasti Signer, O=Techwasti Infotech, C=IN")?;
```

This generates:
- An RSA 2048-bit (or 3072/4096-bit) private key.
- An X.509 v3 certificate valid for 10 years with Subject and Issuer RDNs, SubjectPublicKeyInfo, and RSA-SHA256 signature.

---

## 3. Signing a PDF with a Visual Appearance Stamp

A visual signature creates both the cryptographic `/Type /Sig` dictionary and a visual `/Type /Annot /Subtype /Widget` on the selected page:

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::open("contract.pdf")?;

    let key = SigningKey::generate_self_signed("CN=Techwasti Official")?;

    let signer = Signer::new(key)
        .with_name("Techwasti Infotech Authorized Signatory")
        .with_reason("Service Contract Approval")
        .with_location("Bangalore, India")
        .with_contact_info("legal@techwasti.com")
        .with_visual_stamp(
            VisualSignature::at_page(1)
                .rect(50.0, 50.0, 290.0, 130.0)
                .title("Digitally Signed by Techwasti")
                .detail("Signer: Techwasti Authorized\nDate: 2026-10-25\nStatus: Verified")
                .border_color(Color::rgb(0.0, 0.45, 0.85))
                .background_color(Color::rgb(0.96, 0.98, 1.0)),
        );

    let signed_pdf_bytes = doc.sign(signer)?;
    std::fs::write("contract_signed.pdf", &signed_pdf_bytes)?;

    println!("Signed PDF generated successfully!");
    Ok(())
}
```

---

## 4. Digital Signature Verification & Tamper Detection

`SignatureVerifier` extracts the `/ByteRange`, recalculates SHA-256 over the designated file slices, and verifies that the embedded PKCS#7 signed digest matches:

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pdf_bytes = std::fs::read("contract_signed.pdf")?;

    let report = SignatureVerifier::verify(&pdf_bytes)?;

    println!("Signature Verification:");
    println!("  -> Valid: {}", report.is_valid);
    println!("  -> Signer Name: {:?}", report.signer_name);
    println!("  -> Reason: {:?}", report.reason);
    println!("  -> Location: {:?}", report.location);
    println!("  -> Signing Time: {:?}", report.signing_time);
    println!("  -> SHA-256 Digest: {}", report.computed_digest_hex);

    if !report.is_valid {
        eprintln!("WARNING: Document has been tampered with!");
    }

    Ok(())
}
```

use rust_pdf_box::prelude::*;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn print_usage() {
    println!(
        r#"rust-pdf-box CLI (v{})
A high-performance command-line utility for PDF manipulation, signing, and stamping.
Maintained by Techwasti Infotech <techwastiinfotech@gmail.com>

USAGE:
    pdfbox <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    info <file.pdf>
        Display PDF metadata, page count, and catalog details.

    text <file.pdf> [output.txt]
        Extract all text content from the PDF.

    watermark <input.pdf> <output.pdf> <text> [opacity: 0.0-1.0] [angle_deg]
        Stamp a semi-transparent diagonal watermark across all pages.

    stamp <input.pdf> <output.pdf> <title> [subtitle]
        Apply an official rubber stamp in the top-right corner.

    sign <input.pdf> <output.pdf> <signer_name> [reason] [location]
        Digitally sign the PDF with an ISO 32000 CMS/PKCS#7 signature and visual stamp.

    verify <signed.pdf>
        Cryptographically verify detached signatures and detect document tampering.

    merge <output.pdf> <input1.pdf> <input2.pdf> [input3.pdf...]
        Merge two or more PDF files into a single document.

    split <input.pdf> [output_directory]
        Split a multi-page PDF into individual single-page PDF files.

    flatten <input.pdf> <output.pdf>
        Burn all interactive AcroForm fields into permanent page graphics.

    help, --help, -h
        Show this help message.

EXAMPLES:
    pdfbox info sample.pdf
    pdfbox text sample.pdf extracted.txt
    pdfbox watermark contract.pdf watermarked.pdf "CONFIDENTIAL" 0.25 45
    pdfbox stamp document.pdf stamped.pdf "APPROVED" "Security Team"
    pdfbox sign contract.pdf signed.pdf "Techwasti Authority" "Contract Execution" "India"
    pdfbox verify signed.pdf
    pdfbox merge combined.pdf part1.pdf part2.pdf
    pdfbox split presentation.pdf ./pages/
    pdfbox flatten application.pdf flattened.pdf
"#,
        env!("CARGO_PKG_VERSION")
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = args[1].as_str();
    let result = match command {
        "info" => cmd_info(&args[2..]),
        "text" => cmd_text(&args[2..]),
        "watermark" => cmd_watermark(&args[2..]),
        "stamp" => cmd_stamp(&args[2..]),
        "sign" => cmd_sign(&args[2..]),
        "verify" => cmd_verify(&args[2..]),
        "merge" => cmd_merge(&args[2..]),
        "split" => cmd_split(&args[2..]),
        "flatten" => cmd_flatten(&args[2..]),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        unknown => {
            eprintln!("Error: Unknown subcommand '{}'\n", unknown);
            print_usage();
            process::exit(1);
        }
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

fn cmd_info(args: &[String]) -> PdfResult<()> {
    if args.is_empty() {
        eprintln!("Usage: pdfbox info <file.pdf>");
        process::exit(1);
    }

    let path = &args[0];
    let doc = PdfDocument::open(path)?;
    let count = doc.page_count();

    println!("=== PDF Information: {} ===", path);
    println!("Pages:       {}", count);

    if let Some(title) = doc.inner.info.title() {
        println!("Title:       {}", title);
    }
    if let Some(author) = doc.inner.info.author() {
        println!("Author:      {}", author);
    }
    if let Some(subject) = doc.inner.info.subject() {
        println!("Subject:     {}", subject);
    }
    if let Some(creator) = doc.inner.info.creator() {
        println!("Creator:     {}", creator);
    }
    if let Some(producer) = doc.inner.info.producer() {
        println!("Producer:    {}", producer);
    }

    if let Some(acro) = PDAcroForm::from_doc(&doc.inner) {
        let fields = acro.fields(&doc.inner);
        println!("AcroForm:    {} interactive fields found", fields.len());
    }

    Ok(())
}

fn cmd_text(args: &[String]) -> PdfResult<()> {
    if args.is_empty() {
        eprintln!("Usage: pdfbox text <file.pdf> [output.txt]");
        process::exit(1);
    }

    let path = &args[0];
    let doc = PdfDocument::open(path)?;
    let extracted = doc.extract_text()?;

    if args.len() >= 2 {
        let out_path = &args[1];
        fs::write(out_path, &extracted).map_err(PdfError::Io)?;
        println!("Extracted text written to {}", out_path);
    } else {
        println!("{}", extracted);
    }

    Ok(())
}

fn cmd_watermark(args: &[String]) -> PdfResult<()> {
    if args.len() < 3 {
        eprintln!(
            "Usage: pdfbox watermark <input.pdf> <output.pdf> <text> [opacity: 0.0-1.0] [angle_deg]"
        );
        process::exit(1);
    }

    let input_path = &args[0];
    let output_path = &args[1];
    let watermark_text = &args[2];
    let opacity: f64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.25);
    let angle: f64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(45.0);

    let mut doc = PdfDocument::open(input_path)?;
    doc.stamp()
        .pages(PageRange::All)
        .watermark(
            watermark_text,
            54.0,
            Color::rgb(0.85, 0.15, 0.15),
            angle,
            opacity,
        )
        .apply(&mut doc.inner)?;

    doc.save(output_path)?;
    println!(
        "Watermark '{}' applied successfully to {}",
        watermark_text, output_path
    );
    Ok(())
}

fn cmd_stamp(args: &[String]) -> PdfResult<()> {
    if args.len() < 3 {
        eprintln!("Usage: pdfbox stamp <input.pdf> <output.pdf> <title> [subtitle]");
        process::exit(1);
    }

    let input_path = &args[0];
    let output_path = &args[1];
    let title = &args[2];
    let subtitle = args.get(3).cloned();

    let mut doc = PdfDocument::open(input_path)?;
    doc.stamp()
        .pages(PageRange::Single(1))
        .position(StampPosition::TopRight {
            margin_x: 36.0,
            margin_y: 36.0,
        })
        .rubber_stamp(title, subtitle, Color::rgb(0.0, 0.55, 0.2), -12.0)
        .apply(&mut doc.inner)?;

    doc.save(output_path)?;
    println!("Rubber stamp '{}' applied to {}", title, output_path);
    Ok(())
}

fn cmd_sign(args: &[String]) -> PdfResult<()> {
    if args.len() < 3 {
        eprintln!("Usage: pdfbox sign <input.pdf> <output.pdf> <signer_name> [reason] [location]");
        process::exit(1);
    }

    let input_path = &args[0];
    let output_path = &args[1];
    let signer_name = &args[2];
    let reason = args
        .get(3)
        .cloned()
        .unwrap_or_else(|| "Document Authorization".to_string());
    let location = args
        .get(4)
        .cloned()
        .unwrap_or_else(|| "Techwasti Cryptographic Module".to_string());

    println!("Generating in-memory self-signed X.509 RSA signing key...");
    let dn = format!("CN={}, O=Techwasti Infotech", signer_name);
    let key = SigningKey::generate_self_signed(&dn)?;

    let mut doc = PdfDocument::open(input_path)?;
    let signer = Signer::new(key)
        .with_name(signer_name)
        .with_reason(&reason)
        .with_location(&location)
        .with_visual_stamp(
            VisualSignature::at_page(1)
                .rect(50.0, 50.0, 260.0, 90.0)
                .title(format!("Digitally Signed: {}", signer_name))
                .detail(format!(
                    "Reason: {}\nLocation: {}\nSecurity: ISO 32000 CMS",
                    reason, location
                )),
        );

    let signed_bytes = doc.sign(signer)?;
    fs::write(output_path, &signed_bytes).map_err(PdfError::Io)?;

    println!("Document signed and written to {}", output_path);
    Ok(())
}

fn cmd_verify(args: &[String]) -> PdfResult<()> {
    if args.is_empty() {
        eprintln!("Usage: pdfbox verify <signed.pdf>");
        process::exit(1);
    }

    let path = &args[0];
    let data = fs::read(path).map_err(PdfError::Io)?;
    let report = SignatureVerifier::verify(&data)?;

    println!("=== Signature Verification: {} ===", path);
    println!("Cryptographically Valid: {}", report.is_valid);
    if let Some(name) = report.signer_name {
        println!("Signer Name:             {}", name);
    }
    if let Some(reason) = report.reason {
        println!("Signing Reason:          {}", reason);
    }
    if let Some(location) = report.location {
        println!("Signing Location:        {}", location);
    }
    if let Some(time) = report.signing_time {
        println!("Signing Timestamp:       {}", time);
    }

    if !report.is_valid {
        eprintln!("WARNING: Document signature is INVALID or file has been tampered with!");
        process::exit(2);
    }

    Ok(())
}

fn cmd_merge(args: &[String]) -> PdfResult<()> {
    if args.len() < 3 {
        eprintln!("Usage: pdfbox merge <output.pdf> <input1.pdf> <input2.pdf> [input3.pdf...]");
        process::exit(1);
    }

    let output_path = &args[0];
    let input_paths = &args[1..];

    let mut combined = PdfDocument::create();
    for input_path in input_paths {
        let doc = PdfDocument::open(input_path)?;
        combined.merge(&doc)?;
        println!("Merged: {} ({} pages)", input_path, doc.page_count());
    }

    combined.save(output_path)?;
    println!(
        "Successfully merged into {} (total {} pages)",
        output_path,
        combined.page_count()
    );
    Ok(())
}

fn cmd_split(args: &[String]) -> PdfResult<()> {
    if args.is_empty() {
        eprintln!("Usage: pdfbox split <input.pdf> [output_directory]");
        process::exit(1);
    }

    let input_path = &args[0];
    let out_dir = args.get(1).map(|s| s.as_str()).unwrap_or(".");
    fs::create_dir_all(out_dir).map_err(PdfError::Io)?;

    let doc = PdfDocument::open(input_path)?;
    let pages = PDFSplitter::split_into_single_pages(&doc.inner)?;

    let stem = Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");

    for (idx, page_doc) in pages.into_iter().enumerate() {
        let page_filename = format!("{}_page_{}.pdf", stem, idx + 1);
        let dest = Path::new(out_dir).join(page_filename);
        let mut facade = PdfDocument { inner: page_doc };
        facade.save(dest.to_str().unwrap())?;
        println!("Wrote: {}", dest.display());
    }

    println!("Document split into {} pages.", doc.page_count());
    Ok(())
}

fn cmd_flatten(args: &[String]) -> PdfResult<()> {
    if args.len() < 2 {
        eprintln!("Usage: pdfbox flatten <input.pdf> <output.pdf>");
        process::exit(1);
    }

    let input_path = &args[0];
    let output_path = &args[1];

    let mut doc = PdfDocument::open(input_path)?;
    doc.flatten_forms()?;
    doc.save(output_path)?;

    println!("Forms flattened and saved to {}", output_path);
    Ok(())
}

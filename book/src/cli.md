# Command Line Utility (CLI)

`rust-pdf-box` includes a high-performance standalone command-line executable named `pdfbox`. It allows developers, sysadmins, and automation pipelines to manipulate, sign, stamp, and inspect PDF files without writing code.

---

## Installation

### Cargo Install

If you have Rust and Cargo installed:

```bash
cargo install --path . --bin pdfbox
```

Or install directly from git:

```bash
cargo install --git https://github.com/techwastiinfotech/rust-pdf-box.git --bin pdfbox
```

### Pre-Compiled Standalone Binaries

Pre-compiled single-binary executables with zero external dependencies are published on every GitHub Release for:
- Linux (`x86_64`)
- Windows (`x86_64`)
- macOS (`Apple Silicon arm64` and `Intel x86_64`)

---

## Subcommand Reference

### 1. Document Information (`info`)

Inspect document metadata, PDF version, catalog references, and page count:

```bash
pdfbox info sample.pdf
```

Example output:
```text
=== PDF Document Info ===
File: sample.pdf
Version: 1.7
Page Count: 12
Title: Annual Financial Report 2026
Author: Techwasti Infotech
Producer: rust-pdf-box v0.1.0
Encrypted: false
```

---

### 2. Text Extraction (`text`)

Extract all readable text streams from a PDF file:

```bash
# Print to standard output
pdfbox text invoice.pdf

# Save directly to a text file
pdfbox text invoice.pdf extracted.txt
```

---

### 3. Watermarking (`watermark`)

Stamp a diagonal, semi-transparent watermark across all pages:

```bash
pdfbox watermark input.pdf watermarked.pdf "CONFIDENTIAL" [opacity] [angle_deg]
```

Example:
```bash
pdfbox watermark contract.pdf contract_marked.pdf "DRAFT VERSION" 0.2 45
```

---

### 4. Rubber Stamping (`stamp`)

Apply an official boxed rubber stamp in the upper right corner:

```bash
pdfbox stamp input.pdf stamped.pdf "<TITLE>" ["<SUBTITLE>"]
```

Example:
```bash
pdfbox stamp proposal.pdf approved.pdf "APPROVED" "Security Compliance Team"
```

---

### 5. Digital Signatures (`sign`)

Digitally sign a PDF document with an ISO 32000 CMS/PKCS#7 detached signature and an official visual appearance stamp:

```bash
pdfbox sign input.pdf signed.pdf "<SIGNER_NAME>" ["<REASON>"] ["<LOCATION>"]
```

Example:
```bash
pdfbox sign contract.pdf contract_signed.pdf "Techwasti Authority" "Enterprise Contract Agreement" "Bangalore, India"
```

---

### 6. Signature Verification (`verify`)

Verify detached digital signatures and detect if the document was altered or tampered with after signing:

```bash
pdfbox verify contract_signed.pdf
```

Example output:
```text
=== Signature Verification ===
Found 1 signature(s):
  [1] Signer: CN=Techwasti Authority
      Filter: /Adobe.PPKLite
      SubFilter: /adbe.pkcs7.detached
      Status: VALID (Document integrity verified)
```

---

### 7. Merging PDFs (`merge`)

Merge multiple PDF documents into a single document:

```bash
pdfbox merge output.pdf file1.pdf file2.pdf file3.pdf
```

---

### 8. Splitting PDFs (`split`)

Split a multi-page PDF into single-page PDF files:

```bash
pdfbox split multi_page.pdf ./output_directory/
```

---

### 9. Flattening Forms (`flatten`)

Burn all interactive AcroForm fields into permanent page graphics so form inputs can no longer be edited:

```bash
pdfbox flatten input_form.pdf flattened.pdf
```

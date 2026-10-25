# rust-pdf-box Documentation

> **A comprehensive, pure Rust implementation of Apache PDFBox featuring modern, context-oriented, and ergonomic PDF processing.**

---

<div class="hero-banner">

### 🚀 Get Started Immediately

| Resource | Description | Link |
| :--- | :--- | :--- |
| **Quick Start** | 5-minute tutorial to create, stamp, and sign PDFs | [Read Guide →](quickstart.md) |
| **CLI Utility** | Standalone `pdfbox` command-line utility reference | [CLI Reference →](cli.md) |
| **System Architecture** | Multi-layer COS, PD, content stream, and crypto diagrams | [View Architecture →](architecture.md) |
| **API Documentation** | Full Rustdoc crate reference with all types and traits | [Browse Rustdoc →](api/rust_pdf_box/index.html) |

</div>

---

## What is rust-pdf-box?

`rust-pdf-box` is an open-source, pure Rust library bringing the full feature set of **Apache PDFBox** into modern Rust with **zero external C/C++ or OpenSSL dependencies**.

Designed and maintained by **Techwasti Infotech**, `rust-pdf-box` eliminates the boilerplate typical of low-level PDF manipulation. It provides a simple, context-oriented facade that makes common, high-demand operations—such as **digital signing with ISO 32000 compliance**, **watermarking**, **rubber stamping**, **interactive AcroForm processing**, and **document merging/splitting**—straightforward, fast, and reliable.

---

## Core Feature Guides

- 📑 **[Stamping & Watermarking](stamping-and-watermarking.md)**  
  Apply semi-transparent diagonal watermarks, official rubber stamps, headers, and footers across page ranges with dynamic `{page}` and `{total}` substitutions.

- 🔒 **[Digital Signatures (ISO 32000 PKCS#7 / CMS)](digital-signatures.md)**  
  Pure-Rust detached PKCS#7 signing, built-in X.509 v3 self-signed certificate generation, visual signature appearances, and cryptographic tamper verification.

- 📝 **[Interactive Forms (AcroForms) & Annotations](acroforms-and-annotations.md)**  
  Inspect, update, and flatten interactive form fields (`/Tx`, `/Btn`, `/Ch`, `/Sig`), and attach clickable hyperlink annotations (`/Link`).

- 🗂️ **[Document Assembly](assembly-and-manipulation.md)**  
  Merge multiple PDF documents with automatic conflict-free object ID remapping, split into single pages, or rotate pages.

- 🔍 **[Text Extraction](text-extraction.md)**  
  Extract UTF-8 text from PDF content streams with accurate layout reconstruction using `PDFTextStripper`.

- 🔄 **[Apache PDFBox Migration Guide](migration-guide.md)**  
  Class-by-class comparison, memory safety advantages, and side-by-side Java vs. Rust code samples.

- 🏢 **[Enterprise Integration & Artifacts](enterprise-integration.md)**  
  Guidance for Python, Node.js, Docker, pre-compiled platform binaries, and CI/CD pipelines.

- 📚 **[Full Rustdoc API Reference](api/rust_pdf_box/index.html)**  
  Complete auto-generated Rust documentation covering every module, function, and struct.

---

## Project Links & Community

- **GitHub Repository**: [techwastiinfotech/rust-pdf-box](https://github.com/techwastiinfotech/rust-pdf-box)
- **Crates.io Package**: [crates.io/crates/rust-pdf-box](https://crates.io/crates/rust-pdf-box)
- **Docs.rs Reference**: [docs.rs/rust-pdf-box](https://docs.rs/rust-pdf-box)
- **License**: MIT or Apache-2.0
- **Maintained by**: [Techwasti Infotech](https://techwasti.com)

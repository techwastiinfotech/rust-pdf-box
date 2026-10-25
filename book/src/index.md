# Welcome to rust-pdf-box

<div align="center">

[![CI](https://github.com/techwastiinfotech/rust-pdf-box/actions/workflows/ci.yml/badge.svg)](https://github.com/techwastiinfotech/rust-pdf-box/actions)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-teal.svg)](https://techwastiinfotech.github.io/rust-pdf-box)
[![Crates.io](https://img.shields.io/crates/v/rust-pdf-box.svg)](https://crates.io/crates/rust-pdf-box)
[![Docs.rs](https://docs.rs/rust-pdf-box/badge.svg)](https://docs.rs/rust-pdf-box)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](#)
[![Maintained by Techwasti](https://img.shields.io/badge/maintained%20by-techwasti%20infotech-teal.svg)](https://techwasti.com)

**A comprehensive, pure Rust implementation of Apache PDFBox featuring modern, context-oriented, and ergonomic PDF processing.**

</div>

---

## What is rust-pdf-box?

`rust-pdf-box` is an open-source, pure Rust library bringing the full feature set of **Apache PDFBox** into modern Rust with zero external C/C++ or OpenSSL dependencies.

Designed and maintained by **Techwasti Infotech**, `rust-pdf-box` eliminates the boilerplate typical of low-level PDF manipulation. It provides a simple, context-oriented facade that makes common, high-demand operations—such as **digital signing with ISO 32000 compliance**, **watermarking**, **rubber stamping**, **interactive AcroForm processing**, and **document merging/splitting**—straightforward and reliable.

---

## Highlights & Capabilities

- **Pure Rust Cryptography & Digital Signatures**:
  Full ISO 32000 detached PKCS#7 / CMS digital signatures (`/adbe.pkcs7.detached`, `/ETSI.CAdES.detached`) with built-in self-signed certificate generation (`SigningKey::generate_self_signed(...)`) or external PKCS#8/PEM key loading.
- **Context-Oriented Stamping & Watermarking**:
  Fluent builder API for watermarks, rubber stamps, running headers/footers, and bates numbering. Supports multi-page range targeting (`PageRange::All`, `PageRange::Odd`, `PageRange::Even`, `PageRange::Range(1..=5)`).
- **Interactive Forms (AcroForms) & Annotations**:
  Form field inspection, value editing, checkbox/radio toggles, and form flattening. Clickable hyperlink annotations (`/Link`) and visual markups.
- **Content Stream Engine**:
  Vector path construction & painting, transparency/alpha, RGB/CMYK colors, standard Type 1 fonts (Helvetica, Times, Courier, etc.), TrueType font embedding, and image embedding (JPEG `/DCTDecode` passthrough and PNG with `/SMask` soft mask).
- **Document Assembly & Text Extraction**:
  Merge multiple PDF files with collision-free object ID remapping, split PDFs into ranges or single pages, and extract clean UTF-8 text with position metrics via `PDFTextStripper`.
- **Standalone CLI Utility (`pdfbox`)**:
  Pre-compiled standalone binaries for Linux, macOS, and Windows to inspect, sign, watermark, stamp, merge, split, and extract text without needing Rust installed.

---

## Navigating this Documentation

- **[Quick Start](quickstart.md)**: Add the crate to your project and build your first PDF in minutes.
- **[Command Line Utility (CLI)](cli.md)**: Explore the standalone `pdfbox` command-line utility.
- **[System Architecture](architecture.md)**: Understand the COS object layer, PD model, content stream engine, and crypto subsystem.
- **[Feature Guides](stamping-and-watermarking.md)**: Deep dives into stamping, digital signatures, AcroForms, merging/splitting, and text extraction.
- **[Migration Guide](migration-guide.md)**: Detailed cheat sheet and side-by-side code comparisons for Java Apache PDFBox developers.
- **[Rustdoc API Reference](api-reference.md)**: Access the complete generated API reference with every struct, method, and trait.

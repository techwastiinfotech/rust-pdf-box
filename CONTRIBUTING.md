# Contributing to rust-pdf-box

Thank you for your interest in contributing to **`rust-pdf-box`**! We welcome community contributions, bug reports, feature requests, and documentation improvements.

`rust-pdf-box` is maintained by **Techwasti Infotech** and the open source community to provide a high-performance, ergonomic, and fully featured Rust implementation of Apache PDFBox.

---

## Code of Conduct

All contributors and participants are expected to adhere to the [Code of Conduct](CODE_OF_CONDUCT.md).

---

## How Can I Contribute?

### 1. Reporting Bugs
- Check the [issue tracker](https://github.com/techwastiinfotech/rust-pdf-box/issues) first to ensure the bug hasn't already been reported.
- If you find a new bug, please file an issue using our [Bug Report Template](.github/ISSUE_TEMPLATE/bug_report.md).
- Whenever possible, include a minimal reproducible Rust code snippet and a non-confidential sample PDF.

### 2. Suggesting Features
- Feature ideas are welcome! Open an issue using our [Feature Request Template](.github/ISSUE_TEMPLATE/feature_request.md).
- Detail the desired API and reference relevant ISO 32000 / Apache PDFBox specifications where appropriate.

### 3. Submitting Pull Requests
1. **Fork the repository** on GitHub.
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/<your-username>/rust-pdf-box.git
   cd rust-pdf-box
   ```
3. **Create a topic branch**:
   ```bash
   git checkout -b feature/my-new-feature
   ```
4. **Make your changes** following our design principles:
   - Keep APIs context-oriented and intuitive.
   - Maintain high performance and zero-cost abstractions.
   - Add unit tests for all new functions, operators, and bug fixes.
5. **Run tests and formatting checks**:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets
   cargo test
   ```
6. **Commit your changes**:
   - Write clear, concise commit messages (preferably conventional commits: `feat: ...`, `fix: ...`, `docs: ...`, `test: ...`).
7. **Push to your fork** and open a Pull Request against `main`.

---

## Development Environment Setup

Ensure you have Rust stable installed (`rustup update stable`).

```bash
# Verify environment
cargo check
cargo test

# Run examples
cargo run --example stamping_demo
cargo run --example digital_signature_demo
```

---

## Architecture Overview

- **`src/cos/`**: Low-level Certificate/Content Object Stream model (objects, dictionaries, arrays, streams, tokenizer lexer, parser, writer).
- **`src/pd/`**: High-level Page Description document object model (`PDDocument`, `PDPage`, `PDRectangle`, `PDResources`).
- **`src/content/`**: Content stream generator (`PDPageContentStream`), paths, graphics state, colors.
- **`src/font/`**: Standard 14 Type 1 fonts + TrueType/OpenType embedding engine.
- **`src/image/`**: JPEG DCTDecode passthrough and PNG with alpha SoftMasking (`/SMask`).
- **`src/stamp/`**: Fluent, context-oriented stamping, watermarking, headers/footers, rubber stamps.
- **`src/crypto/` & `src/sign/`**: Pure-Rust X.509 key generation, detached PKCS#7 / CMS signatures, ByteRange calculation, and signature verification.
- **`src/interactive/`**: Interactive AcroForms, form fields, and form flattening.
- **`src/manipulation/`**: PDF merging, splitting, and page reordering.
- **`src/text/`**: `PDFTextStripper` text extraction engine.
- **`src/facade.rs`**: Ergonomic top-level `PdfDocument` API.

---

## Questions?

Feel free to open a Discussion or reach out to the maintainers at `techwastiinfotech@gmail.com`.

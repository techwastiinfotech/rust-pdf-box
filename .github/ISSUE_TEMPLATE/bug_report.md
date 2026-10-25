---
name: Bug Report
about: Create a report to help us improve rust-pdf-box
title: "[BUG] "
labels: ["bug"]
assignees: ""
---

### Describe the Bug
A clear and concise description of what the bug is.

### To Reproduce
Steps or Rust code snippet to reproduce the behavior:
```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Paste minimal reproduction code here
    Ok(())
}
```

### Expected Behavior
A clear and concise description of what you expected to happen.

### PDF Document Details (if applicable)
- PDF Version (e.g., 1.4, 1.7, 2.0):
- Origin (e.g., Adobe Acrobat, Chrome, ReportLab, PDFBox):
- Encrypted/Signed: [Yes / No]
- Attach sample PDF if non-confidential.

### Environment
- OS: [e.g. Windows 11, Ubuntu 24.04, macOS Sonoma]
- Rust version: (`rustc --version`)
- `rust-pdf-box` version: [e.g. 0.1.0]

### Additional Context
Add any other context about the problem here (e.g. panic stack trace, console logs).

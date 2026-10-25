# Enterprise Integration & Artifacts

`rust-pdf-box` can be integrated into your projects in multiple ways: as a native Rust library, as a pre-compiled standalone CLI binary, as a step in your CI/CD workflows, or from languages like Python, Node.js, and Go.

---

## 1. Using as a Rust Dependency

### Option A: From Crates.io (Recommended)

Add `rust-pdf-box` directly to your `Cargo.toml`:

```toml
[dependencies]
rust-pdf-box = "0.1.0"
```

Or run:
```bash
cargo add rust-pdf-box
```

### Option B: Directly from GitHub

To track the latest updates on the `main` branch:

```toml
[dependencies]
rust-pdf-box = { git = "https://github.com/techwastiinfotech/rust-pdf-box.git", branch = "main" }
```

Or pin to a specific release tag:

```toml
[dependencies]
rust-pdf-box = { git = "https://github.com/techwastiinfotech/rust-pdf-box.git", tag = "v0.1.0" }
```

---

## 2. Using Pre-Compiled Binary Artifacts

For projects written in **Python, Node.js, Go, PHP, or Shell Scripts**, you can use the standalone `pdfbox` binary without needing Rust installed.

### Pre-Compiled Platform Downloads

Every GitHub release publishes pre-compiled binaries with SHA-256 checksums:

| Platform | Target Architecture | Archive |
| :--- | :--- | :--- |
| **Linux** | `x86_64-unknown-linux-gnu` | `pdfbox-linux-x86_64.tar.gz` |
| **Windows** | `x86_64-pc-windows-msvc` | `pdfbox-windows-x86_64.zip` |
| **macOS (Apple Silicon)** | `aarch64-apple-darwin` | `pdfbox-macos-aarch64.tar.gz` |
| **macOS (Intel)** | `x86_64-apple-darwin` | `pdfbox-macos-x86_64.tar.gz` |

### Quick Install (Linux / macOS)

```bash
curl -sSL https://github.com/techwastiinfotech/rust-pdf-box/releases/latest/download/pdfbox-linux-x86_64.tar.gz | tar -xz
sudo mv pdfbox /usr/local/bin/
pdfbox --help
```

---

## 3. Using in Other GitHub Actions Workflows

You can integrate `rust-pdf-box` into any GitHub Actions workflow to watermark, sign, merge, or inspect PDFs during your build pipeline:

```yaml
name: PDF Processing Pipeline

on: [push]

jobs:
  stamp-and-sign:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Download rust-pdf-box CLI
        run: |
          curl -sSL https://github.com/techwastiinfotech/rust-pdf-box/releases/latest/download/pdfbox-linux-x86_64.tar.gz | tar -xz
          chmod +x pdfbox

      - name: Watermark Document
        run: |
          ./pdfbox watermark document.pdf watermarked.pdf "CONFIDENTIAL" 0.3 45

      - name: Digitally Sign Document
        run: |
          ./pdfbox sign watermarked.pdf signed.pdf "Techwasti Automated CI" "Automated Verification" "Global"

      - name: Verify Cryptographic Signature
        run: |
          ./pdfbox verify signed.pdf

      - name: Upload Final PDF
        uses: actions/upload-artifact@v4
        with:
          name: processed-document
          path: signed.pdf
```

---

## 4. Using from Python or Node.js

Because the `pdfbox` binary is self-contained with zero runtime shared-library dependencies, you can invoke it from any language:

### Python Example

```python
import subprocess

# Stamp a watermark
subprocess.run([
    "pdfbox", "watermark",
    "invoice.pdf", "invoice_stamped.pdf",
    "PAID", "0.2", "45"
], check=True)

# Digitally sign
subprocess.run([
    "pdfbox", "sign",
    "invoice_stamped.pdf", "invoice_final.pdf",
    "Billing Department", "Invoice Clearance", "India"
], check=True)

# Verify signature
result = subprocess.run(["pdfbox", "verify", "invoice_final.pdf"], capture_output=True, text=True)
print(result.stdout)
```

### Node.js Example

```javascript
const { execFileSync } = require('child_process');

// Extract text
const text = execFileSync('pdfbox', ['text', 'report.pdf'], { encoding: 'utf-8' });
console.log('Extracted Text:', text);

// Merge documents
execFileSync('pdfbox', ['merge', 'combined.pdf', 'part1.pdf', 'part2.pdf']);
```

---

## 5. Docker Integration

Add `pdfbox` to your Docker containers with a single curl step:

```dockerfile
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y curl ca-certificates && rm -rf /var/lib/apt/lists/*

RUN curl -sSL https://github.com/techwastiinfotech/rust-pdf-box/releases/latest/download/pdfbox-linux-x86_64.tar.gz | tar -xz -C /usr/local/bin

ENTRYPOINT ["pdfbox"]
```

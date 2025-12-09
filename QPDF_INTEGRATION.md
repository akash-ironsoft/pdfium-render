# QPDF Integration for pdfium-render

This document describes the QPDF integration added to pdfium-render, enabling PDF to JSON conversion alongside existing PDFium functionality.

## What Was Added

### 1. QPDF FFI Bindings (`src/bindings/qpdf.rs`)
- `IPDF_QPDF_PDFToJSON()` - Convert PDF to QPDF JSON
- `IPDF_QPDF_FreeString()` - Free allocated strings

### 2. High-Level Rust API (`src/qpdf.rs`)
- `QpdfJson` struct - Safe wrapper for QPDF JSON output
- `QpdfJsonVersion` enum - V1 (basic) or V2 (extended)
- Automatic memory management via `Drop` trait
- Thread-safe (`Send` + `Sync`)

### 3. Error Handling (`src/error.rs`)
Added new error variants:
- `PdfiumError::QpdfConversionFailed`
- `PdfiumError::NullPointer`
- `PdfiumError::InvalidUtf8`

### 4. WASM Example (`examples/wasm_qpdf.rs`)
Browser-ready functions:
- `extract_text()` - Extract text from PDF
- `pdf_to_json()` - Convert PDF to QPDF JSON
- `get_page_count()` - Get number of pages
- `get_version()` - Get library version

## Project Status

✅ **Completed:**
- QPDF FFI bindings added
- High-level safe API implemented
- Error handling integrated
- WASM example created
- Module exports configured

⏳ **Next Steps:**
1. Configure for custom PDFium build
2. Build the project
3. Test with your PDFium+QPDF binary

## Building

### Prerequisites

Your custom PDFium build must include:
- QPDF integration
- The `IPDF_QPDF_*` functions exposed in headers
- WASM build output

**Your PDFium location:**
```
/home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor
```

### Configuration Options

#### Option A: Environment Variable (Recommended for Testing)

```bash
export PDFIUM_CUSTOM_BUILD_PATH=/home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor/build/emscripten/wasm/release
```

#### Option B: Build Script

Create/modify `.cargo/config.toml`:
```toml
[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "link-arg=--allow-undefined",
]
```

The extern functions will be resolved at runtime when loaded with PDFium WASM.

### Build Commands

```bash
cd /home/akash/Dev/ironsoft/pdfium-render

# For WASM (browser)
wasm-pack build --target web --release

# For WASM example
cd examples
wasm-pack build --target web --out-dir ../wasm_example_pkg --release
```

## Usage Example

### Rust Code

```rust
use pdfium_render::prelude::*;

// Extract text
let pdfium = Pdfium::default();
let document = pdfium.load_pdf_from_file("document.pdf", None)?;

for page in document.pages().iter() {
    let text = page.text()?;
    println!("{}", text.all());
}

// Convert to QPDF JSON
let pdf_bytes = std::fs::read("document.pdf")?;
let json = QpdfJson::from_bytes(&pdf_bytes, QpdfJsonVersion::V2)?;
println!("{}", json.as_str()?);
```

### WASM (JavaScript)

```javascript
import init, { extract_text, pdf_to_json, get_version } from './pkg/pdfium_render.js';

// Initialize
await init();

// Extract text
const pdfBytes = new Uint8Array(await file.arrayBuffer());
const text = extract_text(pdfBytes);

// Convert to JSON
const json = pdf_to_json(pdfBytes, 2); // version 2
const parsed = JSON.parse(json);
```

## Architecture

```
Browser/Node.js
     │
     ├── Your Rust WASM (pdfium-render)
     │   ├── High-level safe API
     │   ├── QpdfJson wrapper
     │   └── FFI declarations
     │
     └── PDFium WASM Module
         ├── PDFium C++ library
         └── QPDF C++ library (integrated)
```

The Rust code declares `extern "C"` functions that are resolved at runtime when both WASM modules are loaded together.

## Testing

### 1. Build the Example

```bash
cd /home/akash/Dev/ironsoft/pdfium-render/examples

# Build to WASM
wasm-pack build --target web --out-dir ../wasm_test --release
```

### 2. Create Test HTML

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>QPDF Test</title>
</head>
<body>
    <input type="file" id="file" accept=".pdf">
    <button id="extract">Extract Text</button>
    <button id="json">To JSON</button>
    <pre id="output"></pre>

    <script type="module">
        import init, { extract_text, pdf_to_json } from './wasm_test/pdfium_render.js';

        await init();

        document.getElementById('extract').onclick = async () => {
            const file = document.getElementById('file').files[0];
            const bytes = new Uint8Array(await file.arrayBuffer());
            const text = extract_text(bytes);
            document.getElementById('output').textContent = text;
        };

        document.getElementById('json').onclick = async () => {
            const file = document.getElementById('file').files[0];
            const bytes = new Uint8Array(await file.arrayBuffer());
            const json = pdf_to_json(bytes, 2);
            document.getElementById('output').textContent = json;
        };
    </script>
</body>
</html>
```

### 3. Serve and Test

```bash
python3 -m http.server 8000
# Open http://localhost:8000
```

## API Reference

### `QpdfJson`

Safe wrapper for QPDF JSON output.

**Methods:**
- `from_bytes(pdf_data: &[u8], version: QpdfJsonVersion) -> Result<Self, PdfiumError>`
- `as_str(&self) -> Result<&str, PdfiumError>`
- `to_string(&self) -> Result<String, PdfiumError>`

**Example:**
```rust
let json = QpdfJson::from_bytes(&pdf_data, QpdfJsonVersion::V2)?;
let json_str = json.as_str()?;
```

### `QpdfJsonVersion`

JSON format version enum.

**Variants:**
- `V1` - Basic JSON structure with objects and streams
- `V2` - Extended JSON with encryption info, object streams, etc.

### WASM Functions

**`extract_text(pdf_bytes: &[u8]) -> Result<String, JsValue>`**
Extract text from all pages.

**`pdf_to_json(pdf_bytes: &[u8], version: u8) -> Result<String, JsValue>`**
Convert PDF to QPDF JSON (version 1 or 2).

**`get_page_count(pdf_bytes: &[u8]) -> Result<u16, JsValue>`**
Get the number of pages.

**`get_version() -> String`**
Get library version information.

## Integration with Existing Code

The QPDF functionality integrates seamlessly with existing pdfium-render code:

```rust
use pdfium_render::prelude::*;

let pdfium = Pdfium::default();
let document = pdfium.load_pdf_from_file("doc.pdf", None)?;

// Use existing pdfium-render features
for page in document.pages().iter() {
    let text = page.text()?;
    let bitmap = page.render()?.as_image();
    // ... etc
}

// Use new QPDF features
let pdf_bytes = std::fs::read("doc.pdf")?;
let json = QpdfJson::from_bytes(&pdf_bytes, QpdfJsonVersion::V2)?;
```

## Troubleshooting

### "undefined symbol: IPDF_QPDF_PDFToJSON"

**Cause:** PDFium WASM module doesn't include QPDF functions.

**Solution:** Ensure you're using your custom PDFium build from:
```
/home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor
```

### "QpdfConversionFailed" error

**Cause:** PDF structure incompatible with QPDF or corrupted PDF.

**Solution:** Verify PDF is valid and try with a simpler PDF first.

### Build errors with bindgen

**Cause:** Missing QPDF headers in bindgen search path.

**Solution:** For WASM, the functions are declared as `extern "C"` and resolved at runtime, so bindgen errors can be ignored if building for WASM target.

## Files Modified

```
pdfium-render/
├── src/
│   ├── bindings/
│   │   └── qpdf.rs          # NEW - QPDF FFI bindings
│   ├── qpdf.rs               # NEW - High-level API
│   ├── error.rs              # MODIFIED - Added error variants
│   └── lib.rs                # MODIFIED - Exported qpdf module
└── examples/
    └── wasm_qpdf.rs          # NEW - WASM example
```

## Next Steps

1. **Test basic build:**
   ```bash
   cd /home/akash/Dev/ironsoft/pdfium-render
   cargo check
   ```

2. **Build for WASM:**
   ```bash
   wasm-pack build --target web --release
   ```

3. **Create integration test:**
   - Use your custom PDFium WASM
   - Load both modules in browser
   - Test text extraction and JSON conversion

4. **Consider publishing:**
   - Fork maintained separately, or
   - Contribute back to ajrcarey/pdfium-render, or
   - Publish as `pdfium-render-qpdf` crate

## Benefits of This Approach

✅ **Leverages pdfium-render:**
- Mature, well-tested codebase
- Active maintenance
- Comprehensive PDFium bindings
- Excellent documentation

✅ **Clean integration:**
- Follows pdfium-render patterns
- Idiomatic Rust API
- Proper error handling
- Memory safe

✅ **Future-proof:**
- Easy to sync with upstream pdfium-render
- Can add more QPDF functions easily
- Maintainable architecture

## Support

For issues with:
- **pdfium-render**: https://github.com/ajrcarey/pdfium-render/issues
- **This QPDF integration**: File in your fork/repo
- **PDFium/QPDF build**: Check Universal.PdfEditor documentation

---

Created: 2025-12-08
Branch: `feature/qpdf-integration`

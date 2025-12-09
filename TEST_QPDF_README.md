# QPDF Test Page - Quick Start Guide

This simple HTML page demonstrates using your custom PDFium build with QPDF integration directly in the browser.

## What This Does

- ✅ Loads your custom PDFium WASM (includes QPDF functions)
- ✅ Uses direct Emscripten `cwrap()` calls (simple approach)
- ✅ Extracts text from PDFs using PDFium
- ✅ Converts PDFs to QPDF JSON (v1 and v2)
- ✅ Shows PDF info (page count, dimensions)

## How to Run

### 1. Start a Local Web Server

From the pdfium-render directory:

```bash
cd /home/akash/Dev/ironsoft/pdfium-render
python3 -m http.server 8000
```

### 2. Open in Browser

Navigate to:
```
http://localhost:8000/test_qpdf.html
```

### 3. Use the Interface

1. Wait for "✅ PDFium loaded successfully!" message
2. Click "Choose File" and select a PDF
3. Click any operation button:
   - **Extract Text** - Extracts text from all pages
   - **Convert to JSON V1** - Basic QPDF JSON structure
   - **Convert to JSON V2** - Extended QPDF JSON with encryption info
   - **Get PDF Info** - Shows page count and dimensions

## How It Works

### Architecture

```
Browser
  │
  ├── test_qpdf.html (JavaScript)
  │     ├── Loads PDFium WASM module
  │     ├── Uses Module.cwrap() to wrap C functions
  │     └── Calls functions directly
  │
  └── PDFium WASM Module (pdfium.wasm + pdfium.js)
        ├── PDFium C++ library
        └── QPDF C++ library
```

### Key Functions Used

**PDFium Functions (Text Extraction):**
- `FPDF_LoadMemDocument()` - Load PDF from memory
- `FPDF_GetPageCount()` - Get number of pages
- `FPDFText_LoadPage()` - Load text layer for a page
- `FPDFText_GetText()` - Extract text content

**QPDF Functions (JSON Conversion):**
- `IPDF_QPDF_PDFToJSON()` - Convert PDF to JSON
- `IPDF_QPDF_FreeString()` - Free allocated JSON string

### Code Pattern

```javascript
// 1. Load PDFium module
Module = await PDFiumModule();

// 2. Wrap functions
FPDF.IPDF_QPDF_PDFToJSON = Module.cwrap('IPDF_QPDF_PDFToJSON', 'number', ['number', 'number', 'number']);

// 3. Allocate memory
const wasmBuffer = Module._malloc(pdfBuffer.byteLength);
Module.HEAPU8.set(new Uint8Array(pdfBuffer), wasmBuffer);

// 4. Call function
const jsonPtr = FPDF.IPDF_QPDF_PDFToJSON(wasmBuffer, pdfBuffer.byteLength, 2);

// 5. Get result
const jsonString = Module.UTF8ToString(jsonPtr);

// 6. Free memory
FPDF.IPDF_QPDF_FreeString(jsonPtr);
Module._free(wasmBuffer);
```

## Troubleshooting

### "Failed to load PDFium"

**Check pdfium.js path:**
The HTML references: `../iron-universal/Universal.PdfEditor/pdf-editor-app/pdfium.js`

Make sure this path is correct relative to test_qpdf.html.

**Solution:**
```bash
# Verify the file exists
ls /home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor/pdf-editor-app/pdfium.js
ls /home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor/pdf-editor-app/pdfium.wasm
```

If the path is different, edit line 143 in test_qpdf.html:
```html
<script src="CORRECT_PATH_TO/pdfium.js"></script>
```

### CORS Errors

If you see CORS errors in the browser console:

**Problem:** File:// protocol doesn't work due to CORS restrictions.

**Solution:** Always use a web server (python http.server, nginx, apache, etc.)

### "IPDF_QPDF_PDFToJSON is not a function"

**Problem:** Your PDFium build doesn't include QPDF functions.

**Solution:** Make sure you're using the custom PDFium build from:
```
/home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor/pdf-editor-app/
```

### Memory Errors

If you see memory allocation errors with large PDFs:

**Problem:** WASM heap is too small.

**Solution:** Edit pdfium.js to increase initial memory:
```javascript
// Find this line in pdfium.js and increase the value
INITIAL_MEMORY: 256*1024*1024  // 256 MB
```

## What About the Rust Wrapper?

The pdfium-render fork with QPDF integration (Rust code) is still valuable:

**This HTML page:**
- ✅ Simple, direct testing
- ✅ Quick prototype
- ✅ Low-level control
- ❌ Manual memory management
- ❌ No type safety

**Rust wrapper (pdfium-render fork):**
- ✅ Safe, high-level API
- ✅ Automatic memory management
- ✅ Type safety
- ✅ Better for production
- ❌ More complex setup

To use the Rust wrapper, see: `QPDF_INTEGRATION.md`

## Next Steps

### For Quick Testing
Continue using this HTML page - it's perfect for testing your PDFium+QPDF build.

### For Production
Build the Rust wrapper:
```bash
cd /home/akash/Dev/ironsoft/pdfium-render
wasm-pack build --target web --release
```

Then create a similar HTML that loads the Rust WASM instead of calling PDFium directly.

## File Locations

- **Test page:** `/home/akash/Dev/ironsoft/pdfium-render/test_qpdf.html`
- **PDFium WASM:** `/home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor/pdf-editor-app/pdfium.wasm`
- **PDFium JS:** `/home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor/pdf-editor-app/pdfium.js`
- **Rust fork:** `/home/akash/Dev/ironsoft/pdfium-render/` (branch: feature/qpdf-integration)

## Testing Checklist

- [ ] Start web server
- [ ] Open http://localhost:8000/test_qpdf.html
- [ ] See "✅ PDFium loaded successfully!"
- [ ] Select a PDF file
- [ ] Click "Extract Text" - see text output
- [ ] Click "Convert to JSON V2" - see JSON structure
- [ ] Click "Get PDF Info" - see page information

## Support

If something doesn't work:
1. Check browser console (F12) for errors
2. Verify pdfium.js path is correct
3. Ensure you're using a web server (not file://)
4. Check that pdfium.wasm is in the same directory as pdfium.js

---

**Status:** Ready for testing
**Created:** 2025-12-08
**Location:** `/home/akash/Dev/ironsoft/pdfium-render/`

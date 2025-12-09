//! WASM example demonstrating PDFium text extraction and QPDF JSON conversion
//!
//! This example shows how to use pdfium-render with QPDF integration in a browser environment.
//!
//! # Building
//!
//! ```bash
//! wasm-pack build examples --target web --out-dir wasm_qpdf_pkg --release
//! ```
//!
//! # Usage
//!
//! See the accompanying HTML file for how to load and use this WASM module in a browser.

use pdfium_render::prelude::*;
use wasm_bindgen::prelude::*;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Extract text from all pages of a PDF document
///
/// # Arguments
/// * `pdf_bytes` - The PDF file data as a byte array
///
/// # Returns
/// The extracted text from all pages, or an error message
#[wasm_bindgen]
pub fn extract_text(pdf_bytes: &[u8]) -> Result<String, JsValue> {
    // Get the Pdfium instance
    let pdfium = Pdfium::default();

    // Load the PDF from bytes
    let document = pdfium
        .load_pdf_from_byte_slice(pdf_bytes, None)
        .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {:?}", e)))?;

    let mut all_text = String::new();

    // Extract text from each page
    for (index, page) in document.pages().iter().enumerate() {
        let text = page
            .text()
            .map_err(|e| JsValue::from_str(&format!("Failed to get page text: {:?}", e)))?;

        all_text.push_str(&format!("--- Page {} ---\n", index + 1));
        all_text.push_str(text.all().as_str());
        all_text.push_str("\n\n");
    }

    if all_text.is_empty() {
        all_text = "No text found in PDF".to_string();
    }

    Ok(all_text)
}

/// Convert a PDF document to QPDF JSON format
///
/// # Arguments
/// * `pdf_bytes` - The PDF file data as a byte array
/// * `version` - QPDF JSON version (1 or 2)
///
/// # Returns
/// JSON string representing the PDF structure, or an error message
#[wasm_bindgen]
pub fn pdf_to_json(pdf_bytes: &[u8], version: u8) -> Result<String, JsValue> {
    // Validate version
    let qpdf_version = match version {
        1 => QpdfJsonVersion::V1,
        2 => QpdfJsonVersion::V2,
        _ => return Err(JsValue::from_str("Invalid version: must be 1 or 2")),
    };

    // Convert to JSON
    let json = QpdfJson::from_bytes(pdf_bytes, qpdf_version)
        .map_err(|e| JsValue::from_str(&format!("QPDF conversion failed: {:?}", e)))?;

    json.to_string()
        .map_err(|e| JsValue::from_str(&format!("Failed to convert JSON to string: {:?}", e)))
}

/// Get version information
#[wasm_bindgen]
pub fn get_version() -> String {
    format!("pdfium-render with QPDF v{}", env!("CARGO_PKG_VERSION"))
}

/// Get the number of pages in a PDF
#[wasm_bindgen]
pub fn get_page_count(pdf_bytes: &[u8]) -> Result<u16, JsValue> {
    let pdfium = Pdfium::default();

    let document = pdfium
        .load_pdf_from_byte_slice(pdf_bytes, None)
        .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {:?}", e)))?;

    Ok(document.pages().len())
}

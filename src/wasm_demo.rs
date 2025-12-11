use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use std::ffi::{CStr, c_char, c_int, c_void};

// FFI declarations for QPDF functions - only available on non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
extern "C" {
    fn IPDF_QPDF_PDFToJSON(
        pdf_data: *const c_void,
        pdf_size: usize,
        version: c_int,
    ) -> *mut c_char;

    fn IPDF_QPDF_FreeString(str: *mut c_char);
}

#[wasm_bindgen]
pub struct PdfProcessor {
    pdfium: crate::prelude::Pdfium,
}

#[wasm_bindgen]
impl PdfProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<PdfProcessor, JsValue> {
        // Set panic hook for better error messages
        console_error_panic_hook::set_once();

        let pdfium = crate::prelude::Pdfium::default();

        Ok(PdfProcessor { pdfium })
    }

    /// Extract all text from a PDF
    #[wasm_bindgen(js_name = extractText)]
    pub fn extract_text(&self, pdf_bytes: &[u8]) -> Result<String, JsValue> {
        let document = self.pdfium
            .load_pdf_from_byte_vec(pdf_bytes.to_vec(), None)
            .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {:?}", e)))?;

        let mut all_text = String::new();

        for (index, page) in document.pages().iter().enumerate() {
            all_text.push_str(&format!("\n=== Page {} ===\n", index + 1));

            let text = page.text()
                .map_err(|e| JsValue::from_str(&format!("Failed to get text: {:?}", e)))?
                .all();

            all_text.push_str(&text);
            all_text.push('\n');
        }

        Ok(all_text)
    }

    /// Get PDF metadata
    #[wasm_bindgen(js_name = getMetadata)]
    pub fn get_metadata(&self, pdf_bytes: &[u8]) -> Result<JsValue, JsValue> {
        use crate::prelude::PdfDocumentMetadataTagType;

        let document = self.pdfium
            .load_pdf_from_byte_vec(pdf_bytes.to_vec(), None)
            .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {:?}", e)))?;

        let metadata = serde_json::json!({
            "page_count": document.pages().len() as usize,
            "version": format!("{:?}", document.version()),
            "title": document.metadata().get(PdfDocumentMetadataTagType::Title).map(|t| t.value().to_string()).unwrap_or_default(),
            "author": document.metadata().get(PdfDocumentMetadataTagType::Author).map(|t| t.value().to_string()).unwrap_or_default(),
            "subject": document.metadata().get(PdfDocumentMetadataTagType::Subject).map(|t| t.value().to_string()).unwrap_or_default(),
            "creator": document.metadata().get(PdfDocumentMetadataTagType::Creator).map(|t| t.value().to_string()).unwrap_or_default(),
            "producer": document.metadata().get(PdfDocumentMetadataTagType::Producer).map(|t| t.value().to_string()).unwrap_or_default(),
        });

        Ok(serde_wasm_bindgen::to_value(&metadata)?)
    }

    /// Convert PDF to QPDF JSON format
    /// Note: QPDF is not currently available in WASM builds
    #[wasm_bindgen(js_name = pdfToJson)]
    pub fn pdf_to_json(&self, pdf_bytes: &[u8], version: i32) -> Result<String, JsValue> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::ffi::{CStr, c_int, c_void};
            unsafe {
                let json_ptr = IPDF_QPDF_PDFToJSON(
                    pdf_bytes.as_ptr() as *const c_void,
                    pdf_bytes.len(),
                    version as c_int,
                );

                if json_ptr.is_null() {
                    return Err(JsValue::from_str("QPDF conversion failed"));
                }

                let json_cstr = CStr::from_ptr(json_ptr);
                let json_str = json_cstr.to_str()
                    .map_err(|e| JsValue::from_str(&format!("Invalid UTF-8: {}", e)))?
                    .to_string();

                IPDF_QPDF_FreeString(json_ptr);

                Ok(json_str)
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (pdf_bytes, version); // Suppress unused variable warnings
            Err(JsValue::from_str("QPDF is not available in WASM builds yet. Please use native builds for PDF-to-JSON conversion."))
        }
    }

    /// Get page count quickly
    #[wasm_bindgen(js_name = getPageCount)]
    pub fn get_page_count(&self, pdf_bytes: &[u8]) -> Result<usize, JsValue> {
        let document = self.pdfium
            .load_pdf_from_byte_vec(pdf_bytes.to_vec(), None)
            .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {:?}", e)))?;

        Ok(document.pages().len() as usize)
    }

    /// Extract text from a specific page
    #[wasm_bindgen(js_name = extractPageText)]
    pub fn extract_page_text(&self, pdf_bytes: &[u8], page_index: usize) -> Result<String, JsValue> {
        let document = self.pdfium
            .load_pdf_from_byte_vec(pdf_bytes.to_vec(), None)
            .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {:?}", e)))?;

        let page = document.pages().get(page_index as u16)
            .map_err(|e| JsValue::from_str(&format!("Invalid page index: {:?}", e)))?;

        let text = page.text()
            .map_err(|e| JsValue::from_str(&format!("Failed to get text: {:?}", e)))?
            .all();

        Ok(text)
    }
}

/// Initialize WASM module (called from JavaScript)
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

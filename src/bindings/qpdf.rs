//! FFI bindings for QPDF functions integrated into PDFium
//!
//! These bindings provide access to QPDF functionality that has been
//! integrated into the custom PDFium build.

use std::os::raw::{c_char, c_int, c_void};

extern "C" {
    /// Convert a PDF document to QPDF JSON format.
    ///
    /// # Arguments
    /// * `pdf_data` - Pointer to the PDF file data in memory
    /// * `pdf_size` - Size of the PDF file data in bytes
    /// * `version` - QPDF JSON version (1 or 2)
    ///   - Version 1: Basic JSON structure with objects and streams
    ///   - Version 2: Extended JSON with encryption info, object streams, etc.
    ///
    /// # Returns
    /// A pointer to a null-terminated C string containing the JSON data.
    /// Returns NULL if the conversion fails.
    ///
    /// # Safety
    /// The returned string is allocated with malloc() and must be freed
    /// by calling IPDF_QPDF_FreeString() when done.
    ///
    /// # Examples
    /// ```c
    /// char* json = IPDF_QPDF_PDFToJSON(pdf_data, size, 2);
    /// if (json) {
    ///     printf("%s\n", json);
    ///     IPDF_QPDF_FreeString(json);
    /// }
    /// ```
    pub fn IPDF_QPDF_PDFToJSON(
        pdf_data: *const c_void,
        pdf_size: usize,
        version: c_int,
    ) -> *mut c_char;

    /// Free a string allocated by IPDF_QPDF_PDFToJSON.
    ///
    /// # Arguments
    /// * `str` - Pointer to the string to free
    ///
    /// # Safety
    /// This function must only be called with pointers returned by
    /// IPDF_QPDF_PDFToJSON(). Calling with NULL is safe (no-op).
    /// Do not call with already freed pointers (undefined behavior).
    ///
    /// # Examples
    /// ```c
    /// char* json = IPDF_QPDF_PDFToJSON(pdf_data, size, 2);
    /// if (json) {
    ///     // Use json...
    ///     IPDF_QPDF_FreeString(json);
    /// }
    /// ```
    pub fn IPDF_QPDF_FreeString(str: *mut c_char);
}

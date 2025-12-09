//! High-level Rust API for QPDF functionality
//!
//! This module provides safe, idiomatic Rust wrappers around the QPDF
//! functions that have been integrated into PDFium.

use crate::bindings::qpdf::{IPDF_QPDF_FreeString, IPDF_QPDF_PDFToJSON};
use crate::error::{PdfiumError, PdfiumInternalError};
use std::ffi::CStr;
use std::os::raw::c_int;

/// QPDF JSON format version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QpdfJsonVersion {
    /// Version 1: Basic JSON structure with objects and streams
    V1 = 1,
    /// Version 2: Extended JSON with encryption info, object streams, etc.
    V2 = 2,
}

/// QPDF JSON representation of a PDF document
///
/// This struct owns the JSON string returned by QPDF and ensures
/// it is properly freed when dropped.
///
/// # Examples
///
/// ```no_run
/// use pdfium_render::prelude::*;
///
/// let pdfium = Pdfium::default();
/// let pdf_bytes = std::fs::read("document.pdf")?;
///
/// let json = QpdfJson::from_bytes(&pdf_bytes, QpdfJsonVersion::V2)?;
/// println!("{}", json.as_str()?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct QpdfJson {
    ptr: *mut std::os::raw::c_char,
}

impl QpdfJson {
    /// Convert PDF document bytes to QPDF JSON format
    ///
    /// # Arguments
    /// * `pdf_data` - Raw PDF file data
    /// * `version` - QPDF JSON version to generate
    ///
    /// # Returns
    /// `Ok(QpdfJson)` containing the JSON representation, or
    /// `Err(PdfiumError)` if conversion fails
    ///
    /// # Errors
    /// - `PdfiumError::PdfiumLibraryInternalError` if PDF data is invalid
    /// - `PdfiumError::QpdfConversionFailed` if QPDF processing fails
    ///
    /// # Examples
    /// ```no_run
    /// use pdfium_render::qpdf::{QpdfJson, QpdfJsonVersion};
    ///
    /// let pdf_bytes = std::fs::read("document.pdf")?;
    /// let json = QpdfJson::from_bytes(&pdf_bytes, QpdfJsonVersion::V2)?;
    /// let json_str = json.to_string()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_bytes(pdf_data: &[u8], version: QpdfJsonVersion) -> Result<Self, PdfiumError> {
        if pdf_data.is_empty() {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        let ptr = unsafe {
            IPDF_QPDF_PDFToJSON(
                pdf_data.as_ptr() as *const std::os::raw::c_void,
                pdf_data.len(),
                version as c_int,
            )
        };

        if ptr.is_null() {
            Err(PdfiumError::QpdfConversionFailed)
        } else {
            Ok(Self { ptr })
        }
    }

    /// Get JSON as a string slice
    ///
    /// # Returns
    /// `Ok(&str)` containing the JSON, or `Err(PdfiumError)` if
    /// the internal string is invalid UTF-8
    ///
    /// # Examples
    /// ```no_run
    /// # use pdfium_render::qpdf::{QpdfJson, QpdfJsonVersion};
    /// # let pdf_bytes = vec![0u8; 100];
    /// let json = QpdfJson::from_bytes(&pdf_bytes, QpdfJsonVersion::V2)?;
    /// let json_str = json.as_str()?;
    /// println!("JSON length: {}", json_str.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn as_str(&self) -> Result<&str, PdfiumError> {
        if self.ptr.is_null() {
            return Err(PdfiumError::NullPointer);
        }

        unsafe {
            CStr::from_ptr(self.ptr)
                .to_str()
                .map_err(|_| PdfiumError::InvalidUtf8)
        }
    }

    /// Get JSON as an owned String
    ///
    /// # Returns
    /// `Ok(String)` containing the JSON, or `Err(PdfiumError)` if
    /// the internal string is invalid UTF-8
    ///
    /// # Examples
    /// ```no_run
    /// # use pdfium_render::qpdf::{QpdfJson, QpdfJsonVersion};
    /// # let pdf_bytes = vec![0u8; 100];
    /// let json = QpdfJson::from_bytes(&pdf_bytes, QpdfJsonVersion::V2)?;
    /// let json_string = json.to_string()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn to_string(&self) -> Result<String, PdfiumError> {
        self.as_str().map(|s| s.to_owned())
    }

    /// Get the raw pointer to the C string
    ///
    /// # Safety
    /// The returned pointer is only valid as long as this QpdfJson instance exists.
    /// Do not free the pointer yourself - it will be freed when this instance is dropped.
    pub unsafe fn as_ptr(&self) -> *const std::os::raw::c_char {
        self.ptr
    }
}

impl Drop for QpdfJson {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                IPDF_QPDF_FreeString(self.ptr);
            }
            self.ptr = std::ptr::null_mut();
        }
    }
}

// Ensure QpdfJson is safe to send between threads
unsafe impl Send for QpdfJson {}
unsafe impl Sync for QpdfJson {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qpdf_version_values() {
        assert_eq!(QpdfJsonVersion::V1 as c_int, 1);
        assert_eq!(QpdfJsonVersion::V2 as c_int, 2);
    }

    #[test]
    fn test_empty_pdf_data() {
        let result = QpdfJson::from_bytes(&[], QpdfJsonVersion::V1);
        assert!(result.is_err());
    }
}

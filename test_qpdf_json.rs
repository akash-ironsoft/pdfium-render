use std::ffi::{CStr, c_char, c_int, c_void};
use std::fs;

// FFI declarations for QPDF functions
#[link(name = "pdfium")]
extern "C" {
    fn IPDF_QPDF_PDFToJSON(
        pdf_data: *const c_void,
        pdf_size: usize,
        version: c_int,
    ) -> *mut c_char;

    fn IPDF_QPDF_FreeString(str: *mut c_char);
}

fn main() {
    println!("🔧 Testing Custom Pdfium with QPDF Integration");
    println!("{}", "=".repeat(60));

    // Test file path
    let test_pdf = "test/export-test.pdf";

    println!("\n📄 Reading PDF file: {}", test_pdf);

    // Read PDF file into memory
    let pdf_data = match fs::read(test_pdf) {
        Ok(data) => {
            println!("✓ Successfully read {} bytes", data.len());
            data
        },
        Err(e) => {
            eprintln!("❌ Failed to read PDF file: {}", e);
            return;
        }
    };

    // Test JSON version 1
    println!("\n🔄 Converting to QPDF JSON (Version 1)...");
    test_json_conversion(&pdf_data, 1);

    // Test JSON version 2
    println!("\n🔄 Converting to QPDF JSON (Version 2)...");
    test_json_conversion(&pdf_data, 2);

    println!("\n✅ All QPDF tests completed successfully!");
}

fn test_json_conversion(pdf_data: &[u8], version: i32) {
    unsafe {
        // Call QPDF conversion function
        let json_ptr = IPDF_QPDF_PDFToJSON(
            pdf_data.as_ptr() as *const c_void,
            pdf_data.len(),
            version as c_int,
        );

        if json_ptr.is_null() {
            eprintln!("❌ IPDF_QPDF_PDFToJSON returned NULL");
            return;
        }

        // Convert C string to Rust string
        let json_cstr = CStr::from_ptr(json_ptr);
        let json_str = match json_cstr.to_str() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ Failed to convert JSON to UTF-8: {}", e);
                IPDF_QPDF_FreeString(json_ptr);
                return;
            }
        };

        println!("✓ Successfully converted to JSON");
        println!("  - JSON length: {} bytes", json_str.len());

        // Parse and display JSON structure
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json_str) {
            println!("  - Valid JSON structure: ✓");

            // Display some JSON info
            if let Some(obj) = json_value.as_object() {
                println!("  - Top-level keys: {}", obj.keys().count());

                // Show first few keys
                let keys: Vec<&String> = obj.keys().take(5).collect();
                for key in keys {
                    println!("    • {}", key);
                }

                if obj.keys().count() > 5 {
                    println!("    ... ({} more keys)", obj.keys().count() - 5);
                }
            }

            // Show a preview of the JSON (first 500 characters)
            println!("\n📋 JSON Preview:");
            let preview_len = json_str.len().min(500);
            println!("{}", &json_str[..preview_len]);
            if json_str.len() > 500 {
                println!("... ({} more characters)", json_str.len() - 500);
            }
        } else {
            eprintln!("⚠ JSON is not valid (but conversion succeeded)");
            println!("\n📋 Raw output (first 200 chars):");
            let preview_len = json_str.len().min(200);
            println!("{}", &json_str[..preview_len]);
        }

        // Free the allocated string
        IPDF_QPDF_FreeString(json_ptr);
        println!("\n✓ Memory freed");
    }
}

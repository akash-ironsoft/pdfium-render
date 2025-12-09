use pdfium_render::prelude::*;

fn main() -> Result<(), PdfiumError> {
    println!("Testing custom Pdfium with QPDF integration...");

    // With static linking, just use default
    let pdfium = Pdfium::default();

    println!("✓ Successfully initialized Pdfium");

    // Try to open a test PDF
    let document = pdfium.load_pdf_from_file("test/export-test.pdf", None)?;

    println!("✓ Successfully loaded PDF document");
    println!("  - Pages: {}", document.pages().len());
    println!("  - Version: {:?}", document.version());

    // Try to render first page
    let page = document.pages().first()?;
    println!("✓ Got first page");
    println!("  - Width: {:.2} points", page.width().value);
    println!("  - Height: {:.2} points", page.height().value);

    let render_config = PdfRenderConfig::new()
        .set_target_width(800)
        .set_maximum_height(1000);

    let bitmap = page.render_with_config(&render_config)?;
    println!("✓ Successfully rendered page to bitmap");
    println!("  - Bitmap size: {}x{}", bitmap.width(), bitmap.height());

    println!("\n✅ All tests passed! Your custom Pdfium library is working correctly.");

    Ok(())
}

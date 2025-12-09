#!/bin/bash
# Run QPDF JSON conversion test

PDFIUM_PATH="/home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor/pdfium-workspace/Universal.Pdfium/out/linux-x64-shared"

echo "Running QPDF test with custom Pdfium library..."
LD_LIBRARY_PATH=.:$PDFIUM_PATH ./test_qpdf_save_json

#!/bin/bash
# Quick test server for QPDF integration

echo "========================================"
echo "  PDFium + QPDF Test Server"
echo "========================================"
echo ""
echo "Starting web server..."
echo ""
echo "Once started, open your browser to:"
echo "  http://localhost:8000/test_qpdf.html"
echo ""
echo "Press Ctrl+C to stop the server"
echo "========================================"
echo ""

cd /home/akash/Dev/ironsoft/pdfium-render
python3 -m http.server 8000

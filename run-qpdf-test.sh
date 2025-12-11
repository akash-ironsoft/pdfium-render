#!/bin/bash
set -e

PDFIUM_PATH="/home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor/pdfium-workspace/Universal.Pdfium/out/linux-x64-shared"

if [ ! -f test_qpdf_save_json ]; then
  SERDE_JSON=$(ls target/debug/deps/libserde_json-*.rlib 2>/dev/null | head -1)
  rustc test_qpdf_save_json.rs --edition 2021 -L dependency=target/debug/deps -L "$PDFIUM_PATH" -l pdfium --extern serde_json="$SERDE_JSON" -C link-arg=-Wl,-rpath,"$PDFIUM_PATH" -o test_qpdf_save_json
fi

LD_LIBRARY_PATH="$PDFIUM_PATH" ./test_qpdf_save_json

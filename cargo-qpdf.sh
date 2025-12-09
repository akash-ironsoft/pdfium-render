#!/bin/bash
# Wrapper script to run cargo with custom Pdfium library

export RUSTFLAGS="-L ."
export LD_LIBRARY_PATH=".:/home/akash/Dev/ironsoft/iron-universal/Universal.PdfEditor/pdfium-workspace/Universal.Pdfium/out/linux-x64-shared"

# Run cargo with all arguments passed to this script
cargo "$@"

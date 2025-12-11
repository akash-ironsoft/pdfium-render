#!/bin/bash
set -e

wasm-pack build examples/ --target no-modules --release
mkdir -p release
cp examples/pkg/pdfium_render_wasm_example.js release/
cp examples/pkg/pdfium_render_wasm_example_bg.wasm release/

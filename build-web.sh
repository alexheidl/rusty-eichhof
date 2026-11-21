#!/bin/sh
# Builds the browser version into web/ (serve that folder with any static web server).
set -e
cd "$(dirname "$0")"
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/last-eichhof.wasm web/
# Optional: shrink it with binaryen's wasm-opt when it is installed (the release workflow does).
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    --enable-mutable-globals --enable-multivalue --enable-reference-types \
    web/last-eichhof.wasm -o web/last-eichhof.wasm
fi
echo "Done: open web/index.html through a web server (for example: python3 -m http.server -d web)"

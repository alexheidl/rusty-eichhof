#!/bin/sh
# Builds the browser version into web/ (serve that folder with any static web server).
set -e
cd "$(dirname "$0")"
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/last-eichhof.wasm web/
echo "Done: open web/index.html through a web server (for example: python3 -m http.server -d web)"

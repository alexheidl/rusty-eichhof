#!/bin/sh
# Makes a public zip for this computer in dist/ (without the original game's files).
set -e
cd "$(dirname "$0")"
cargo build --release --no-default-features --target-dir target/public
name="last-eichhof-$(uname -s | tr 'A-Z' 'a-z')-$(uname -m)"
mkdir -p "dist/$name"
cp target/public/release/last-eichhof "dist/$name/"
cp PLAYERS.md "dist/$name/README.txt"
(cd dist && zip -qr "$name.zip" "$name")
echo "Done: dist/$name.zip"

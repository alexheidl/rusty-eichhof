#!/bin/sh
# Makes a ready-to-play zip for this computer in dist/ (original game data built in).
set -e
cd "$(dirname "$0")"
cargo build --release
name="last-eichhof-$(uname -s | tr 'A-Z' 'a-z')-$(uname -m)"
mkdir -p "dist/$name"
cp target/release/last-eichhof "dist/$name/"
cp PLAYERS.md "dist/$name/README.txt"
cp data/READ.ME "dist/$name/ORIGINAL-READ.ME"
cp LICENSE "dist/$name/LICENSE.txt"
cp NOTICE "dist/$name/NOTICE.txt"
(cd dist && zip -qr "$name.zip" "$name")
echo "Done: dist/$name.zip"

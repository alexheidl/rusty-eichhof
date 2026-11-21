#!/bin/sh
# Builds the desktop version for this computer (Windows, macOS or Linux).
set -e
cd "$(dirname "$0")"
cargo build --release
echo "Done: target/release/last-eichhof"

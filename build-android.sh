#!/bin/sh
# Builds an Android APK with macroquad's official build container (needs Docker).
set -e
cd "$(dirname "$0")"
docker run --rm -v "$(pwd)":/root/src -w /root/src notfl3/cargo-apk cargo quad-apk build --release
echo "Done: target/android-artifacts/release/apk/last-eichhof.apk"

# Packaging and releasing on GitHub

The original game is freeware (its READ.ME: "PLEASE copy it to all your friends"), so its data is
part of this repository: `data/BEER.DAT` is built into the program by the default `embedded`
feature. Release zips are ready to play.

## Checks

`.github/workflows/ci.yml` runs on every push and as the first step of each release: clippy, the
unit tests (decoded game data against golden fingerprints, the CONFIG.HIG format, key names) and
the self-test, which plays level 1 with scripted keys and compares the final screen and score with
a recorded fingerprint (`REPLAY` in `src/selftest.rs`). A change that alters gameplay or drawing
on purpose must update that fingerprint (the self-test JSON prints the new one). The release's web
build is shrunk with `wasm-opt` and opened in headless Chrome to make sure it starts.

`cargo build --release --no-default-features` still makes a build without the data, which reads
`BEER.DAT` from beside the program instead.

## Making a release

1. Raise `version` in `Cargo.toml` and commit.
2. Create a tag such as `v1.0.0` and upload it to GitHub.
3. GitHub Actions (`.github/workflows/release.yml`) builds Windows, macOS (Apple silicon and
   Intel), Linux and web zips and puts them in a **draft release**. Look it over on GitHub and
   press *Publish*. (It can also be started by hand from the Actions tab.)

To try packaging locally first: `./package.sh` makes `dist/last-eichhof-<system>.zip`.

## Notes

- Android: `./build-android.sh` (needs Docker) makes an APK.
- macOS warns about an unsigned program the first time (right-click → Open).

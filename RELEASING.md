# Packaging and releasing on GitHub

There are two kinds of builds:

- **Private build** (`cargo build --release`, the default): the original `BEER.DAT` and
  `CONFIG.HIG` are built into the program. For your own computer only — never publish it.
- **Public build** (`cargo build --release --no-default-features`): only the new code. Players
  copy `BEER.DAT` from their own copy of the game next to the program; without it the program
  says so and closes.

## One-time setup

1. Create an empty repository on GitHub.
2. Make this `app` folder a git repository, check with `git status` that `BEER.DAT` and
   `CONFIG.HIG` are not listed (`.gitignore` keeps them out), commit, and upload it to the new
   GitHub repository.

## Making a release

1. Raise `version` in `Cargo.toml` and commit.
2. Create a tag such as `v1.0.0` and upload it to GitHub.
3. GitHub Actions (`.github/workflows/release.yml`) builds Windows, macOS (Apple silicon and
   Intel), Linux and web zips — all without the original data — and puts them in a **draft
   release**. Look it over on GitHub and press *Publish*. (It can also be started by hand from
   the Actions tab.)

To try packaging locally first: `./package.sh` makes `dist/last-eichhof-<system>.zip`.

## Notes

- Android: `./build-android.sh` (needs Docker) makes an APK; a public APK should be built with
  `--no-default-features`.
- macOS warns about an unsigned program the first time (right-click → Open).
- The original READ.ME calls the game freeware ("please copy it to all your friends"), but the
  names and artwork in BEER.DAT belong to their owners, so the public release ships only the new
  program.

This build contains the original game's own files, for the private use of the person who owns the game, on their own computer. It must not be shared or published.

> In this source repository the original game's files are **not** included (`BEER.DAT` and `CONFIG.HIG` are ignored); a private build embeds them from `data/`, the public release reads `BEER.DAT` from the player's own copy. See `RELEASING.md`.

# The Last Eichhof — for today's computers, phones and browsers

A port of the 1993 freeware DOS shoot-'em-up *The Last Eichhof* by ALPHA-HELIX. It runs the original
pictures, sprites, fonts, sounds, music and level data from the game's `BEER.DAT`, with the game rules
re-implemented from the original program (every number is listed in `MECHANICS.md`).

## Playing

| Action | Keyboard | Touch screen |
|---|---|---|
| Move the bottle | arrow keys | drag on the right half of the screen (the bottle follows your finger's movement) |
| Fire | hold Space | touch and hold the left half |
| Pause | P | tap the top-right corner |
| Back / give up | Esc | tap the top-left corner |
| Menus | 1–6, or arrows + Space/Enter | swipe up/down, tap to choose |

Keys can be changed in *Options → Choose keys*, like in the original. Settings and high scores are
saved in the original `CONFIG.HIG` format (`~/.config/last-eichhof/` or `%APPDATA%\last-eichhof\`).
The original cheat still works: start the program with the argument `007.7`.

## Building

- Desktop (Windows, macOS, Linux): `./build-desktop.sh` (or `cargo build --release`).
- Web: `./build-web.sh`, then serve the `web/` folder with any web server.
- Android: `./build-android.sh` (uses macroquad's build container through Docker).

Checks: `--screenshot <file.png>` saves a picture of the window after about 120 frames;
`--self-test <file.json>` plays the first level with scripted keys and reports whether moving,
shooting and the status bar work. `--scene <name>` (menu, store, menucard, sell, place, hs, title,
story, gameover, victory, intro:N, play:N:ticks) jumps to one screen.

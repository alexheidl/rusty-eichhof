# The Last Eichhof — for today's computers, phones and browsers

> The original freeware game's data (`data/BEER.DAT`, `data/CONFIG.HIG`, released by ALPHA-HELIX with "PLEASE copy it to all your friends") is included and built into the program, so the downloads on the [Releases](../../releases) page are ready to play. See `RELEASING.md`.

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

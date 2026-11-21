# The Last Eichhof: asset formats

All multi-byte values are little-endian. "FUN_1000_xxxx" names refer to the
Ghidra decompilation of BEER.EXE (code segment 1000, data segment DS = 0x1A97;
DS offset N is file offset `0x200 + 0xA970 + N` in BEER.EXE).
The prototype decoders are in `tools/eich.py`; the Rust ones in `src/dat.rs`.

## Display facts that matter for all graphics

* The game runs a Mode X variant of **320x220** (CRTC table at DS:0x125C, set by
  FUN_1000_6e6d: vertical display end 0x1B7 = 440 scan lines, double-scanned).
  Page size 0x44C0 = 220 x 80 bytes; two pages (double buffering), sprite
  cache from VRAM offset 0x8980.
* Play field clip rectangle: x 0..319, y 0..219 (DS:0x364C/0x364A/0x3648/0x3646,
  FUN_1000_6b21). The status bar BAR.SPR (320x9) is drawn at y = 211
  (FUN_1000_1f6e).
* Full-screen 320x200 pictures are drawn 10 lines down (TITLE.PCX, MENU.PCX:
  `FUN_1000_6d74(pcx, 10)`), LANDSCAP.PCX 5 lines down, the others at line 0.

## BEER.DAT (archive)

| offset | size | field |
|---|---|---|
| 0x00 | 32 | magic `"ALPHA-HELIX COMBINER VER 3.3"` + `0x1A` + padding |
| 0x20 | 2 | entry count (78) |
| 0x22 | 24 x count | directory |

Directory entry (24 bytes; in memory at archive+0x54, FUN_1000_533a):

| offset | size | field |
|---|---|---|
| 0 | 14 | name, NUL padded, upper case |
| 14 | 4 | size |
| 18 | 2 | cache flags: bit 0 = keep a copy in the EMS/XMS file cache, bit 1 = only when there is free room (FUN_1000_50ed). Values 0, 1, 3. Data is never compressed. |
| 20 | 4 | absolute file offset |

Names are matched case-insensitively (FUN_1000_a49c). Files are loaded whole
into memory, except TITLE.SND and HS.SND which are streamed (see Sounds).

## Pointer tables (SLI, level SND, FOE, EXP, WEAPONS.SHT)

Many files begin with `u16 count` followed by `count` entries whose first 4
bytes are a real-mode far pointer `offset:u16, segment:u16`. The loader adds
the address of the **first byte after the count** to it (FUN_1000_1975,
FUN_1000_14d8), so

    position_in_file = 2 + segment * 16 + offset

(The pointers are normalised: offset < 16.) Entry sizes: `.SLI` 6 bytes
(pointer + u16 param), `.SND` (level), `.FOE`, `.EXP`, `WEAPONS.SHT` 4 bytes.
FOE/EXP/SHT pointers go into the object table at DS:0x1996/0x1998 and address
behaviour records (enemy, explosion and shot definitions: flags, sprite index,
movement scripts); those are not decoded by `dat.rs`.

## Palettes

* PCX palettes: the last 768 bytes (after a `0x0C` marker); the game stores
  each byte `>> 2` as a 6-bit DAC value into its working palette DS:0x25A4
  (FUN_1000_6d74). The DAC is not written then: screens fade in from black
  with FUN_1000_77f1 (subtracts 60, 54, ... 0 from every 6-bit component per
  vertical retrace) and fade out with FUN_1000_77af (subtract 0, 6, ... 60).
  FUN_1000_6be4(0) blanks all 256 colours; FUN_1000_6c6a(r,g,b) fades every
  colour up to one RGB value (used to fade to white before the BLICK intro).
* **Gameplay palette**: the EXE's built-in palette, 768 bytes of 6-bit values
  at DS:0x0F5C (file offset 0xBACC). FUN_1000_6bc5 copies it to DS:0x25A4 and
  programs the DAC (INT 10h AX=1012h); it runs at graphics start-up
  (FUN_1000_6b21) and after every picture screen, so it is active during play
  and for every sprite, font and SLI graphic. MENU, TITLE, SKY, LANDSCAP, STORE,
  MENUCARD, SELL and LONGTIME.PCX carry exactly the same palette; only
  BACK.PCX, CREDIT.PCX and SCORE.PCX have their own. `dat::game_palette()`
  takes it from MENU.PCX; `dat::exe_palette()` reads it from the EXE.
* Main menu colour cycling: FUN_1000_6c03(0xE8, 0xFE, phase) each frame rotates
  DAC entries 0xE8..0xFE by one step (DAC[i] = pal[i - phase'] wrapping inside
  the range), FUN_1000_122c.
* 6-bit to 8-bit: `v8 = (v6 << 2) | (v6 >> 4)`.

## PCX (`*.PCX`)

Standard ZSoft PCX version 5, 8 bits/pixel, 1 plane, 320 bytes per line,
RLE (`b >= 0xC0`: run of `b & 0x3F` copies of the next byte), 256-colour
palette at the end. Heights: 200, except LONGTIME.PCX (320x400) and
CREDIT.PCX (320x440), which are taller than the screen and scrolled.
FUN_1000_6d74(pcx, line) decodes `ymax + 1` rows directly into one Mode X
page starting at scan line `line` (it ignores bytes-per-line, which is 320 in
every file anyway).

## Sprites (`*.SPR`, and the blocks addressed by `*.SLI`)

One format for both:

| offset | size | field |
|---|---|---|
| 0 | 2 | w (pixels) |
| 2 | 2 | h |
| 4 | 2 | n = number of frames |
| 6 | n x w x h | frames one after another, each row-major, 1 byte = palette index |

Plain linear bytes: no RLE, no planar layout, no hotspot (sprites are placed
by their top-left corner; collisions use w x h boxes, FUN_1000_71a1).
MENCUR.SPR has one stray trailing byte.

Two ways the game draws them:

* **Opaque** (FUN_1000_72c5(spr, x, y, frame)): every byte is written, index 0
  shows as palette colour 0 (black). Used for fonts and UI pictures (BAR,
  GO, CLOSE, MENTXT, ARROW...). Fonts: frame = character code - 0x20, advance
  = w (first u16), drawn by FUN_1000_208c. FONT1 8x8 (59 glyphs, 0x20..0x5A),
  FONT2 8x11 (62), FONT3 12x14 (59; codes A-Z hold lower-case looking
  glyphs), MFONT 6x7, OFONT 6x8, CFONT/BARFONT 4x5/4x7 digits.
* **Masked** sprites (FUN_1000_6794 builds them from SLI entries and from
  MENCUR/MENCUR2/MENBACK/EXIT2/BARFONT.SPR; drawn by FUN_1000_7346):
  **index 0 is transparent** (the builder sets a plane-mask bit only for
  non-zero bytes). FUN_1000_7489 draws the same mask filled with one colour
  (DS:0x0F58 = 15, white) for the hit flash.

### Sprite libraries (`LEVELn.SLI`, `WEAPONS.SLI`)

`u16 count`, then `count` x `{offset:u16, segment:u16, param:u16}`; the far
pointer (relative to file offset 2) addresses a sprite block as above.
Sprites get consecutive slot numbers in load order (weapons first, then the
level's), which FOE/EXP/SHT records refer to.

`param` (FUN_1000_6794), values seen: 2, 4, 0xA, 0xC:

* bits 0-2 = `4 - max_shift`: the builder makes pre-shifted planar copies for
  sub-4-pixel positions, shifts 0, 2, ... up to `4 - (param & 7)`.
  `param & 7 == 2`: copies at shift 0 and 2, so x is effectively rounded down
  to even; `== 4`: one copy, x rounded down to a multiple of 4 (the drawer
  uses `x & ~3` plus, when `x & 2`, the shift-2 copy). Odd x never occurs.
* bit 3 = animation speed. Each sprite object keeps a counter that is
  increased per game tick by a step (slot field +6) and wraps at `2 * n`
  (slot field +8); the shown frame is `counter >> 1` (FUN_1000_7036,
  FUN_1000_76b1). Bit 3 set: step 1 (each frame lasts 2 ticks); clear: step 2
  (a new frame every tick). Objects created with the "once" flag disappear
  after one cycle.

## Sounds (`*.SND`)

Header (FUN_1000_4903 for sounds in memory; FUN_1000_49c1 reads the same
fields from disk for streamed ones):

| offset | size | field |
|---|---|---|
| 0 | 2 | priority: a sound starts only if its priority >= that of the sound playing (one voice, no mixing); the playing sound's priority is kept in DS:0x0D79 |
| 2 | 2 | rate in kHz: 15, 12, 10 or 9 |
| 4 | 2 | flags: bit 0 = Creative 4-bit ADPCM (DSP command 0x75 for the first DMA block, 0x74 after), else 8-bit unsigned PCM (DSP command 0x14) |
| 6 | 4 | data length in bytes |
| 10 | len | data |

**Rate**: FUN_1000_48ad sends DSP command 0x40 with time constant
`256 - 1000 / khz` (integer division), so the real rate is
`1_000_000 / (1000 / khz)`: 15 -> **15151 Hz**, 12 -> 12048 Hz,
10 -> 10000 Hz, 9 -> 9009 Hz. Mono, single-cycle DMA in blocks of at most
32 KiB (FUN_1000_4636).

* Stand-alone files: all 15 kHz PCM except LONGTIME.SND (12 kHz). Their file
  size is `11 + len` (one spare byte at the end; the streamer reads an 11-byte
  header, so for TITLE/HS the first data byte is skipped — inaudible).
* TITLE.SND and HS.SND are too big to load (1.2 MB / 2.4 MB): they are
  streamed from the archive through a double buffer of `0x78 * 0x200` bytes
  (FUN_1000_56de opens the entry, FUN_1000_49c1 plays) — at the same rate
  their header says, 15151 Hz, played once. No different rate is used.
* FUN_1000_49a3 / FUN_1000_4ad5 set the loop flag (DS:0x0D73 bit 2): BLICK.SND,
  SELL.SND, BUY.SND, LONGTIME.SND loop until stopped (FUN_1000_4b20).

ADPCM (Creative 4-bit, as SoundBlaster DSP decodes it): first data byte =
initial sample; then each byte yields two samples, **high nibble first**.
`i = clamp(nibble + scale, 0, 63)`; `sample = clamp(sample + STEP[i], 0, 255)`;
`scale = (scale + ADJ[i]) & 0xFF`, with scale starting at 0 and the tables
from `decode_adpcm4` in `src/dat.rs`. Decoded output stays centred on 128
without clipping (low-nibble-first clips and drifts, so the order is
confirmed).

### `LEVELn.SND` (level sound bank)

It *is* audio: a pointer table (`u16 count` + 4-byte far pointers, relative
to file offset 2) of embedded sounds, each in the SND format above (mostly
15 kHz ADPCM, some 9/12 kHz PCM). FUN_1000_1975 registers the pointers in the
object table DS:0x1996 at base index DS:0x1D92; behaviour scripts play sound
`k` with opcodes 0x8002/0x8003/0x8006 via
`FUN_1000_4903(table[k + DS:0x1D92])` (FUN_1000_5fc2, FUN_1000_613e,
FUN_1000_653e). Counts: LEVEL0 2, LEVEL1 23, LEVEL2 24, LEVEL3 12, LEVEL4 15.

## Other level files

* `LEVELn.DSC` (50 bytes): u16 level number, NUL-terminated level name at 2
  (shown on the GO screen, FUN_1000_1755), two u16 bonuses at 0x2C and 0x2E
  added to the score/money counters.
* `LEVELn.STA` (star field, FUN_1000_672a/6fb5): `u16 count`, then
  `count` x `{x, y, colour, speed}` (i16). Stars are single pixels in plane 0
  (visible x = `x & ~3`), moving down `speed` lines per frame and wrapping at
  the bottom of the play field.
* `LEVELn.TBL`: `u16 count` + data (the level's spawn timetable; not decoded).
* `LEVELn.FOE`, `LEVELn.EXP`, `WEAPONS.SHT`: pointer tables of behaviour
  records (see above).
* `WEAPONS.WPN`: `u16 count` (11) + 30-byte records: name[20] then five u16
  (stats); the first record is copied to DS:0x1DCC as the starting weapon.

## Intro ("BLICK" newspaper, FUN_1000_5a6f)

* `BLICK.PAK` (FUN_1000_5937): a **640x350, 16-colour** picture for EGA mode
  0x10, stored as 4 bit planes (map mask 1, 2, 4, 8 in that order) of
  28000 bytes (= 640 x 350 / 8) each, MSB = leftmost pixel. RLE stream that
  runs continuously across the planes: control byte `c < 0x80`: repeat the
  next byte `c + 1` times; `c >= 0x80`: copy the next `c - 0x7F` bytes. The
  file decodes exactly to its end. Colours: palette indices 0..15 of the game
  palette (which are the standard EGA colours); the game also fades it in from
  white.
* `PAPER.FNT`: 59 glyphs for characters 0x20..0x5A, 200 bytes each:
  5 byte-columns x 40 rows, 1 bit per pixel, MSB left, column-major
  (`glyph + col * 40 + row`). Glyph = 40x40 pixels, set bits are drawn in
  colour 15, clear bits in 0. The scroller (FUN_1000_5a09/5a59) writes one
  8-pixel column per frame at the right edge of the 40 scan lines from 311 down and
  scrolls those lines left by 8 pixels, text from DS:0x0E50.
* `BLICK.SND`: 15 kHz PCM, looped.

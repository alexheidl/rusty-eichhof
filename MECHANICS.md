# The Last Eichhof — mechanics sheet

Every number below was read from the original program (BEER.EXE, Borland C++ 1991, 16-bit DOS) with
Ghidra, or from the data files inside BEER.DAT. Function addresses are Ghidra's `1000:xxxx` (linear
`0x1xxxx`). Data addresses are offsets in the data segment `1a97`. Values marked **estimated** were
looked for in the code and not found.

## Timing and screen
| What | Value | Source |
|---|---|---|
| Logic tick | 20.0 Hz (PIT divisor 0xE8F6 = 59638 → 1193182/59638 Hz); every game step waits for one tick | `FUN_1000_2039(0xe8f6)` in startup, timer ISR at `1000:6716` sets `DS:24f6`, `FUN_1000_6503` waits for it |
| Screen | 320 × 220, 256-colour Mode X, two pages 0x44C0 bytes apart (80 bytes × 220 lines) | `FUN_1000_72c5`, `FUN_1000_714e` (page xor 0x44c0) |
| Playfield | x 0..319, y 0..211 (`DS:3646` = 0xD3 during play); status bar `BAR.SPR` drawn at (0, 211) | `FUN_1000_6b21`, main `FUN_1000_075c`, `FUN_1000_1f6e` |
| Full-screen pictures | 320×200 PCX drawn at a y offset (title 10, menu 10, landscape 5, others 0) | `FUN_1000_6d74(pcx, yoff)` calls |
| Sprite x resolution | drawn at x rounded down to even pixels (Mode X pre-shifted copies) | `FUN_1000_7346` (`x & 0xfffc` + alt image when `x & 2`) |
| Hit flash | sprite drawn once as a solid silhouette in colour 15 (`DS:0f58`) | `FUN_1000_70f8`, `FUN_1000_7489` |
| Background colour | 0 (black) | `DS:0f5a` |

## Controls (defaults, remappable in Options → Choose keys)
| Action | Key | Source |
|---|---|---|
| Up / Down / Left / Right | arrow keys (scan 0x48/0x50/0x4B/0x4D) | `FUN_1000_0825` defaults, stored in `CONFIG.HIG` |
| Fire | Space (0x39), hold for auto-fire | `FUN_1000_6393`, `FUN_1000_6342` |
| Pause | P (0x19) | `FUN_1000_6393` |
| Abort game | Esc (counts as game over) | `FUN_1000_653e` (`DS:1ec6 = -1`) |
| Menus | 1–6 or Up/Down + Fire | `FUN_1000_122c` |

## Player ("the Eichhof")
| What | Value | Source |
|---|---|---|
| Movement speed | 4 px per tick (80 px/s), each "I WANT TO BE FAST" +2, refused once speed > 7 (so max 8) | `DS:1ec8` set in `FUN_1000_14d8`, `FUN_1000_2c54`, `FUN_1000_2bad` |
| Movement | no inertia: moves exactly speed px per held direction per tick; clamped so the main bottle stays inside x 0..319, y 0..211 | `FUN_1000_5d6a` |
| Start position | bottle centred on (150, 190) (top-left rounded to multiple of 4) | `FUN_1000_2224(0x96,0xbe)` |
| Lives | 4 at new game; one is used per attempt, the bar shows up to 6 spare bottles | `FUN_1000_14d8`, `FUN_1000_1fa3`, `FUN_1000_64ab` |
| Start invulnerability | 70 ticks (3.5 s), blinking (flash every other 2 ticks) | `DS:0f44 = 0x46` in `FUN_1000_653e`, `FUN_1000_5d6a` |
| Death | touching any enemy object; explosion 0 at the bottle, all mounted parts removed; after 60 ticks restart from the last checkpoint | `FUN_1000_62e3`, `FUN_1000_653e` |
| Mounted parts | the bottle plus up to 6 bought weapons (7 parts) placed anywhere in the shop's placement box | `FUN_1000_22cb`, `FUN_1000_2bad` |

## Weapons (WEAPONS.WPN: name, sprite, shot, price, reload, type)
Firing: while Fire is held each part counts down; when its counter was 0 it fires its shot and the
counter is set to `reload` (so one shot every reload+1 ticks, the first one immediately). Without
Fire the counters keep counting down to 0. (`FUN_1000_6342`)

| # | Name | Shot | Price (Fr.) | Reload (ticks) | Where sold |
|---|---|---|---|---|---|
| 0 | EICHHOF LAGER 58CL (the ship) | 0 | 0.00 | 3 | built in |
| 1 | STANGE | 12 | 3.00 | 6 | store |
| 2 | PONY | 14 | 3.35 | 4 | store |
| 3 | BARBARA BRAEU | 10 | 3.20 | 4 | store |
| 4 | DUNKEL | 13 | 3.00 | 5 | store |
| 5 | CAN 33CL | 6 | 2.40 | 6 | menu card |
| 6 | CHUEBELI | 9 | 2.60 | 5 | menu card |
| 7 | POKAL | 1 | 4.00 | 20 | menu card |
| 8 | XENON 2 CANNON | 8 | 5.40 | 8 | menu card |
| 9 | BONUS GUTTERE (+1 life) | – | 0.30 | – | menu card, schnaps bonus |
| 10 | I WANT TO BE FAST (+2 speed) | – | 2.00 | – | menu card, schnaps bonus |

Shots (WEAPONS.SHT, each `[dx, dy, damage, homing speed, sprite, script…]`; script = one (vx,vy)
step per tick, 0x8001 n = also fire shot n, 0x8002 = home on nearest shootable enemy, 0x8003 =
keep velocity and bounce off the playfield edges, 0x8000 = end). Shots leaving the playfield vanish.
(`FUN_1000_5bd3`, `FUN_1000_5dee`). Damages: shot0 2, shot1 6 (splits into shots 2–5 with 3/3/3/2),
6/7: 2, 8: 6, 9: 3, 10/11: 2, 12: 3, 13: 2 (homing, speed 12), 14/15: 2 (bouncing).

Damage rule (`FUN_1000_622e`): on overlap, a shootable enemy flashes; shot damage −= enemy hp and
enemy hp −= the shot's previous damage; the enemy dies when its hp reaches 0 or below (score +=
points, explosion spawned) and the shot vanishes when its remaining damage is < 1 (so strong shots
pierce weak enemies). Invulnerable enemies (flag 2) absorb shots completely.

## Enemies (LEVELn.FOE: `[flags, hp, points, explosion, sprite, aim speed, script…]`)
| Flag | Meaning | Source |
|---|---|---|
| 0x01 | counts toward the level goal (`DSC +0x2a`); killing all of them ends the level | `FUN_1000_622e` |
| 0x02 | cannot be destroyed, swallows shots | `FUN_1000_622e` |
| 0x04 | shots pass through (decoration / unshootable bullet) | `FUN_1000_622e` |
| 0x08 | stops the level timeline while alive (boss) | `FUN_1000_5c3f`, `FUN_1000_622e` |
| 0x10 | follows its script | `FUN_1000_5fc2` |
| 0x20 | flies straight (Bresenham) at `aim speed` px/tick toward the player's position at spawn (or a given target, then switches to its script) and is removed off screen | `FUN_1000_5c3f`, `FUN_1000_5fc2` |

Script ops: (dx,dy) move per tick, 0x8000 remove, 0x8001 s change sprite, 0x8002 n x y spawn
enemy n at offset (x,y) (enemy bullets are enemies), 0x8005 remember loop start and position,
0x8004 jump back to the loop start and restore that position, 0x8006 s play level sound s.
Any contact with any enemy object kills the player (`FUN_1000_62e3`).

Explosions (LEVELn.EXP, `FUN_1000_5d1b`, `FUN_1000_613e`): start at the dead object's centre;
ops 0x8001 s x y = one-shot animation, 0x8002 wait a tick, 0x8003 s sound, 0x8004 remove the dead
object's sprite, 0x8005 n x y spawn enemy, 0x8006 n x y tx ty spawn aimed enemy, 0x8000 end.

## Levels (LEVELn.DSC / .TBL / .STA)
| Level | Name | Goal kills | Score bonus at start | Money at start |
|---|---|---|---|---|
| 0 | EASY START | 13 | 0 | +4.00 |
| 1 | FELDSCHLOESSCHEN | 1 | +20000 | +0.20 |
| 2 | OH WEISSBIER | 1 | +20000 | +5.60 |
| 3 | NO MORE COCKTAILS | 3 | +50000 | +4.40 |
| 4 | THE DAY AFTER | 1 | +20000 | 0 |

Timeline (`FUN_1000_653e`): entries `[tick, x, y, what]`; what ≥ 0 spawns enemy `what` at (x,y);
0x8000 stars start scrolling, 0x8001 stars stop, 0x8002 play level sound x, 0x8003 checkpoint
(restart point after a death: timeline position, score and remaining goal are restored). The
timeline clock stops while a flag-0x08 enemy lives. When the goal count reaches 0 the level ends
100 ticks (5 s) later. Stars (.STA `[x, y, colour, speed]`) fall `speed` px per tick, wrapping at y 211.

## Score, money, shop
| What | Value | Source |
|---|---|---|
| Score | sum of enemy points (unsigned 16-bit each), shown as 8 digits with a gap after the last two | `FUN_1000_622e`, `FUN_1000_6404` |
| Money after a level | money += ((score − score at last shop) + 1500) / 2500 × 5 (in Rappen; shown as Fr. x.yy) | `FUN_1000_3ddc` |
| Selling | 3/4 of the price (integer) | `FUN_1000_272d`, `FUN_1000_2621` |
| Shop order | sell screen (if any weapon mounted), then the store (Stange, Pony, Barbara Bräu, Dunkel; left = menu card, further left = exit) | `FUN_1000_3ddc`, `FUN_1000_376b` |
| Placement | weapon moved in 4 px steps inside x 156..268, y 20..95 around the bottle shown at (212, 59); Fire places it (refused on OVERLAP), Esc cancels (money already paid) | `FUN_1000_22cb` |
| High scores | 8 entries, name 15 characters, winner text 33 characters for first place | `FUN_1000_4039` |
| Cheat (command line `007.n`) | bit 1 unlimited lives, bit 2 money 9999.99, bit 4 invulnerable | `FUN_1000_05d9` |

## Sound
| What | Value | Source |
|---|---|---|
| Voices | one effect at a time; a new effect only interrupts one of lower or equal priority (SND header word 0) | `FUN_1000_4903`, SND header |
| Sample rates | SND header kHz 15 → 15151 Hz, 12 → 12048 Hz (DSP time constant 256 − 1000/kHz); most sounds 15151 Hz, LONGTIME 12048 Hz | `FUN_1000_48ad` |
| Formats | 8-bit unsigned PCM or Creative 4-bit ADPCM (header flag bit 0) | SND header, DSP commands 0x74/0x75 |
| Music | TITLE.SND and HS.SND play once; BLICK, SELL, BUY and LONGTIME.SND loop | `FUN_1000_49c1`, `FUN_1000_49a3` |
| Fades | 11 steps subtracting 6 (of 63) from every palette component, 2 screen refreshes (70 Hz) per step | `FUN_1000_77af`, `FUN_1000_77f1` |

## Touch controls (new; the original has a keyboard only)
Right half: drag to steer (finger movement becomes arrow steps at the bottle's speed, so it follows
1:1). Left half: hold to fire. Tap: select / any key. Top-left corner: Esc. Top-right corner: P.
Chosen with the person (they suggested split-screen steering and firing).

## Not found in the code (estimated)
- What happens after the last timeline entry: the original keeps reading past the table; the port
  simply stops spawning (estimated).

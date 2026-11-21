# Credits

## The original game

**The Last Eichhof** — "An Alpha Helix Production, (c) copyright 1993", Switzerland. Released as
freeware.

| Who | What |
|---|---|
| **Tritone** | programming, sound effects and level design |
| **Tweety** | graphics and fonts |
| **Zynax** of Dark System | additional programming and music |

(Names and roles as given in the game's own scrolling credits in `BEER.EXE`.)

From the original release notes (`data/READ.ME`): *"Thank you for playing this game. And by the
way: PLEASE copy it to all your friends."* And from the game: *"This game is FREEWARE. Please
copy like crazy. If you like it, just send a postcard to ALPHA-HELIX."*

All pictures, sprites, fonts, sound effects, music and level data in this port are theirs.

## This port

This port re-implements the program in Rust and plays from the original game's own data. Every original file it uses is listed below with where it came from. They are used unchanged.

## Original files used

| File | Source path | Used as |
|---|---|---|
| BEER.DAT | `data/BEER.DAT`, unchanged from the original game, built into the program | the game's whole data archive (contents below) |
| CONFIG.HIG | `data/CONFIG.HIG`, unchanged from the original game, built into the program | default keys, sound setting and the original high-score table |
| BEER.EXE | original game: BEER.EXE | not copied; studied for the rules and numbers (see MECHANICS.md). Its built-in palette equals MENU.PCX's, so the palette is read from there. |
| READ.ME | `data/READ.ME`, unchanged from the original game | shipped in the release zips as ORIGINAL-READ.ME |

## Inside BEER.DAT

| Member | Bytes | Used as |
|---|---|---|
| FONT1.SPR | 3782 | sprite / font |
| FONT2.SPR | 5462 | sprite / font |
| FONT3.SPR | 9918 | sprite / font |
| BAR.SPR | 2886 | sprite / font |
| GO.SPR | 5102 | sprite / font |
| CLOSE.SPR | 7846 | sprite / font |
| ARROW.SPR | 2046 | sprite / font |
| MENCUR.SPR | 1159 | sprite / font |
| MENU.PCX | 6924 | full-screen picture |
| BARFONT.SPR | 342 | sprite / font |
| EXIT.SPR | 1366 | sprite / font |
| STORE.PCX | 54748 | full-screen picture |
| EXIT2.SPR | 18054 | sprite / font |
| MENUCARD.PCX | 53344 | full-screen picture |
| MFONT.SPR | 2484 | sprite / font |
| CFONT.SPR | 246 | sprite / font |
| OFONT.SPR | 2982 | sprite / font |
| LANDSCAP.PCX | 47021 | full-screen picture |
| MENTXT.SPR | 13146 | sprite / font |
| MENBACK.SPR | 2566 | sprite / font |
| MENCUR2.SPR | 1094 | sprite / font |
| PAPER.FNT | 11800 | intro ticker font |
| BLICK.PAK | 64873 | intro newspaper picture |
| SELL.PCX | 11093 | full-screen picture |
| LONGTIME.PCX | 46735 | full-screen picture |
| CREDIT.PCX | 58504 | full-screen picture |
| BACK.PCX | 8834 | full-screen picture |
| TITLE.PCX | 46708 | full-screen picture |
| SCORE.PCX | 5203 | full-screen picture |
| SKY.PCX | 46871 | full-screen picture |
| CLOSE.SND | 59471 | sound or music |
| BLICK.SND | 234345 | sound or music |
| GO.SND | 119453 | sound or music |
| MENU.SND | 60217 | sound or music |
| SELL.SND | 52735 | sound or music |
| BUY.SND | 234999 | sound or music |
| TOD.SND | 106169 | sound or music |
| LONGTIME.SND | 217189 | sound or music |
| WEAPONS.WPN | 332 | weapon list and prices |
| WEAPONS.SHT | 1842 | shot definitions |
| WEAPONS.SLI | 12606 | sprite library |
| LEVEL0.DSC | 50 | level name, goal and bonuses |
| LEVEL0.SLI | 15430 | sprite library |
| LEVEL0.SND | 115028 | sound or music |
| LEVEL0.FOE | 836 | enemy definitions and movement scripts |
| LEVEL0.EXP | 50 | explosion scripts |
| LEVEL0.TBL | 138 | level timeline |
| LEVEL0.STA | 258 | star field |
| LEVEL1.DSC | 50 | level name, goal and bonuses |
| LEVEL1.SLI | 46774 | sprite library |
| LEVEL1.SND | 279976 | sound or music |
| LEVEL1.FOE | 17954 | enemy definitions and movement scripts |
| LEVEL1.EXP | 1042 | explosion scripts |
| LEVEL1.TBL | 698 | level timeline |
| LEVEL1.STA | 162 | star field |
| LEVEL2.DSC | 50 | level name, goal and bonuses |
| LEVEL2.SLI | 43154 | sprite library |
| LEVEL2.SND | 344196 | sound or music |
| LEVEL2.FOE | 61816 | enemy definitions and movement scripts |
| LEVEL2.EXP | 1036 | explosion scripts |
| LEVEL2.TBL | 930 | level timeline |
| LEVEL2.STA | 162 | star field |
| LEVEL3.DSC | 50 | level name, goal and bonuses |
| LEVEL3.SLI | 35574 | sprite library |
| LEVEL3.SND | 171716 | sound or music |
| LEVEL3.FOE | 9530 | enemy definitions and movement scripts |
| LEVEL3.EXP | 790 | explosion scripts |
| LEVEL3.TBL | 562 | level timeline |
| LEVEL3.STA | 514 | star field |
| LEVEL4.DSC | 50 | level name, goal and bonuses |
| LEVEL4.SLI | 34772 | sprite library |
| LEVEL4.SND | 228710 | sound or music |
| LEVEL4.FOE | 36320 | enemy definitions and movement scripts |
| LEVEL4.EXP | 942 | explosion scripts |
| LEVEL4.TBL | 1234 | level timeline |
| LEVEL4.STA | 258 | star field |
| HS.SND | 2420501 | sound or music |
| TITLE.SND | 1231213 | sound or music |

## Libraries

- macroquad, miniquad and quad-snd (MIT/Apache-2.0) for the window, graphics, input and sound.

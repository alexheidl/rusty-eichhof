//! Every screen outside the playing field: newspaper intro, title, credits, main menu, options,
//! key setup, story, quit, level intro, game over, victory, high scores and the shop
//! (sell counter, store, menu card, weapon placement). Positions come from the original
//! functions named on each screen.

use crate::ctx::{Ctx, Mode, G};
use crate::game::{place_parts, Field, Hero, Mount, Outcome, Sprites};
use crate::gfx::Spr;
use crate::res::Pal;

/// A few sprites over a picture, erased with colour 0 like the original (FUN_1000_76b1 + 75c9).
pub struct Ui {
    pub sp: Sprites,
    pub lib: Vec<Spr>,
    prev: Vec<(i32, i32, i32, i32)>,
    pub flash_col: u8,
}

impl Ui {
    pub fn new(lib: Vec<Spr>) -> Ui {
        Ui { sp: Sprites::new(), lib, prev: vec![], flash_col: 15 }
    }
    /// Draw the sprites for this step and wait for the next tick (FUN_1000_6503 + 76b1).
    pub async fn step(&mut self, ctx: &mut Ctx) {
        self.draw(ctx);
        ctx.tick().await;
    }
    pub fn draw(&mut self, ctx: &mut Ctx) {
        for (x, y, w, h) in self.prev.drain(..) {
            ctx.scr.fill_rect(x, y, w, h, 0);
        }
        self.sp.draw(&mut ctx.scr, &self.lib, self.flash_col);
        for h in 0..60 {
            let (x, y, w, hh) = self.sp.cur(h);
            if w > 0 {
                let x0 = x & !3;
                self.prev.push((x0, y, (x - x0 + w + 3) & !3, hh));
            }
        }
    }
}

fn money_digits(ctx: &mut Ctx, font: &Spr, right_x: i32, y: i32, value: u32) {
    money_places(ctx, font, right_x, y, value, 10);
}

/// FUN_1000_20df: two decimals, '.', then the francs without leading zeros. The original leaves
/// leading-zero places untouched; here they are blanked (so a shrinking amount leaves no stale
/// digits), but only within `places` places so neighbouring text stays intact.
fn money_places(ctx: &mut Ctx, font: &Spr, right_x: i32, y: i32, value: u32, places: i32) {
    let w = font.w as i32;
    let mut x = right_x;
    let mut v = value;
    for _ in 0..2 {
        ctx.scr.blit_opaque(font, (v % 10) as usize + 0x10, x, y);
        v /= 10;
        x -= w;
    }
    ctx.scr.blit_opaque(font, 0x0e, x, y);
    let mut zeros = 0;
    for k in 2..10 {
        x -= w;
        let d = v % 10;
        v /= 10;
        if d == 0 && k != 2 {
            zeros += 1;
            if k < places {
                ctx.scr.blit_opaque(font, 0, x, y);
            }
        } else {
            ctx.scr.blit_opaque(font, d as usize + 0x10, x, y);
            for z in 1..=zeros {
                ctx.scr.blit_opaque(font, 0x10, x + z * w, y);
            }
            zeros = 0;
        }
    }
}

fn picture(ctx: &mut Ctx, name: &str, yoff: i32) -> Pal {
    let (pix, w, h, pal) = ctx.res.pcx(name);
    ctx.scr.picture(&pix, w, h, yoff, 0);
    pal
}

fn black(ctx: &mut Ctx) {
    ctx.scr.fade = 63;
}

fn font(ctx: &Ctx, k: &str) -> Spr {
    ctx.res.font(k).clone()
}

// ------------------------------------------------------------------ intro and title

/// Web only: browsers keep a page silent until the player clicks or presses a key. Ask for
/// that first, so the newspaper intro (which ends at the first key) has its tune.
pub async fn press_to_start(ctx: &mut Ctx) {
    let f = font(ctx, "font1");
    let pal = ctx.res.game_pal;
    ctx.scr.set_pal6(&pal);
    ctx.scr.fade = 0;
    ctx.scr.clear(0);
    for (y, line) in [(90, "THE LAST EICHHOF"), (120, "CLICK OR PRESS A KEY")] {
        let x = (320 - line.len() as i32 * f.w as i32) / 2;
        ctx.scr.text(&f, x, y, line);
    }
    loop {
        ctx.frame().await;
        if macroquad::input::get_last_key_pressed().is_some()
            || macroquad::input::is_mouse_button_pressed(macroquad::input::MouseButton::Left)
            || !macroquad::input::touches().is_empty()
        {
            break;
        }
    }
    // Let go first, or the same press would end the intro at once.
    while !macroquad::input::get_keys_down().is_empty()
        || macroquad::input::is_mouse_button_down(macroquad::input::MouseButton::Left)
        || !macroquad::input::touches().is_empty()
    {
        ctx.frame().await;
    }
    ctx.scr.clear(0);
}

/// The newspaper intro with the scrolling ticker (FUN_1000_5a6f), until a key is pressed.
pub async fn newspaper(ctx: &mut Ctx) {
    let pak = crate::dat::decode_pak(ctx.res.dat.file("BLICK.PAK").unwrap_or(&[]));
    let glyphs = crate::dat::decode_paper_font(ctx.res.dat.file("PAPER.FNT").unwrap_or(&[]));
    ctx.scr.set_size(640, 350);
    let mut pal = ctx.res.game_pal;
    pal[15] = [63, 63, 63];
    ctx.scr.set_pal6(&pal);
    ctx.scr.pix.copy_from_slice(&pak.pixels);
    black(ctx);
    let music = ctx.res.sound("BLICK.SND").await;
    ctx.play_music(&music, true);
    // INTRO.C: glowto(63,63,63) then glowin(1): up to white, then the paper fades in.
    ctx.glow_to_white().await;
    ctx.fade_in().await;
    let text: Vec<u8> = b" . . . ALPHA HELIX PRODUCTION. (C) COPYRIGHT 1993.   PROGRAMMING, SOUND FX AND LEVEL DESIGN BY TRITONE.   GRAPHICS AND FONTS BY TWEETY.   ADDITIONAL PROGRAMMING AND MUSIC BY ZYNAX OF DARK SYSTEM.   ".to_vec();
    let (mut ci, mut col) = (0usize, 0usize);
    let mut acc = 0.0f64;
    ctx.wait_release().await;
    loop {
        // One 8-pixel column per vertical retrace (70 Hz): write it at the right edge, scroll left.
        let g = text[ci].saturating_sub(0x20) as usize;
        let w = ctx.scr.w;
        for row in 0..39usize {
            let y = 311 + row;
            if y >= 350 {
                break;
            }
            let line = &mut ctx.scr.pix[y * w..y * w + w];
            line.copy_within(8.., 0);
            for b in 0..8 {
                let on = glyphs.get(g).map(|f| f.pixels[row * 40 + col * 8 + b] != 0).unwrap_or(false);
                line[w - 8 + b] = if on { 15 } else { 0 };
            }
        }
        col += 1;
        if col > 4 {
            col = 0;
            ci = (ci + 1) % text.len();
        }
        acc += 1.0 / 70.0;
        while acc > 0.0 {
            ctx.frame().await;
            acc -= (ctx.dt()).min(0.1);
            if ctx.mode == Mode::SelfTest {
                acc = 0.0;
            }
        }
        if macroquad::input::get_last_key_pressed().is_some()
            || macroquad::input::is_mouse_button_pressed(macroquad::input::MouseButton::Left)
            || ctx.touch.seen && !macroquad::input::touches().is_empty()
            || ctx.quit
        {
            break;
        }
    }
    ctx.stop_music();
    ctx.fade_out().await;
    ctx.scr.set_size(320, 220);
    ctx.scr.clear(0);
    ctx.wait_release().await;
}

/// FUN_1000_0b07: "BUT ALPHA HELIX STRIKES BACK WITH" then the title, with the title tune;
/// the credits roll when nobody presses a key.
pub async fn title(ctx: &mut Ctx) {
    black(ctx);
    ctx.scr.clear(0);
    let (title_pix, tw, th, title_pal) = ctx.res.pcx("TITLE.PCX");
    picture(ctx, "BACK.PCX", 0);
    // Both pictures are loaded before the fade, so BACK.PCX is shown with the title palette.
    ctx.scr.set_pal6(&title_pal);
    let music = ctx.res.sound("TITLE.SND").await;
    ctx.play_music(&music, false);
    ctx.fade_in().await;
    ctx.wait_or_key(0x1f6).await;
    ctx.fade_out().await;
    ctx.scr.clear(0);
    ctx.scr.picture(&title_pix, tw, th, 10, 0);
    ctx.fade_in().await;
    if !ctx.wait_or_key(0x1cc).await {
        credits(ctx).await;
    }
    ctx.fade_out().await;
    ctx.stop_music();
}

/// FUN_1000_0a35: the 440-line credits picture rolls down and up twice.
async fn credits(ctx: &mut Ctx) {
    ctx.fade_out().await;
    let (pix, w, h, pal) = ctx.res.pcx("CREDIT.PCX");
    ctx.scr.set_pal6(&pal);
    ctx.scr.picture(&pix, w, h, 0, 0);
    ctx.fade_in().await;
    let mut dir = 2;
    let mut off = 0;
    let mut bounces = 0;
    if ctx.wait_or_key(100).await {
        return;
    }
    loop {
        ctx.scr.picture(&pix, w, h, 0, off);
        retrace(ctx).await;
        off += dir;
        if off < 0 {
            bounces += 1;
            dir = -dir;
            off += 2 * dir;
            if ctx.wait_or_key(100).await {
                return;
            }
        }
        if off > 0xcc {
            dir = -dir;
            off += 2 * dir;
            if ctx.wait_or_key(100).await {
                return;
            }
        }
        if bounces == 2 || ctx.any() {
            return;
        }
    }
}

async fn retrace(ctx: &mut Ctx) {
    if ctx.mode == Mode::SelfTest {
        ctx.frame().await;
        return;
    }
    ctx.retrace_wait(1).await;
}

// ------------------------------------------------------------------ main menu

/// The main menu (FUN_1000_122c / 0c27). Returns when the player quits.
pub async fn main_menu(ctx: &mut Ctx) {
    loop {
        let item = menu_choice(ctx).await;
        match item {
            1 => new_game(ctx).await,
            2 => high_scores(ctx, None).await,
            3 => title(ctx).await,
            4 => story(ctx).await,
            6 => {
                quit(ctx).await;
                if ctx.quit {
                    return;
                }
            }
            _ => {}
        }
    }
}

async fn menu_choice(ctx: &mut Ctx) -> i32 {
    black(ctx);
    ctx.scr.clear(0);
    let pal = picture(ctx, "MENU.PCX", 10);
    ctx.scr.set_pal6(&pal);
    let mut ui = Ui::new(vec![font(ctx, "mencur")]);
    let cur = ui.sp.create(&ui.lib, 0, 0x8e, 0x3a, 0);
    let mentxt = font(ctx, "mentxt");
    ctx.scr.blit_opaque(&mentxt, 0, 0xa0, 0x2e);
    let snd = ctx.res.sound("MENU.SND").await;
    ctx.play(&snd);
    ui.draw(ctx);
    ctx.fade_in().await;
    let mut item = 1;
    let mut rot = 0i32;
    let ofont = font(ctx, "ofont");
    loop {
        while ctx.any() {
            cycle(ctx, &pal, &mut rot);
            ui.step(ctx).await;
        }
        while !ctx.any() {
            cycle(ctx, &pal, &mut rot);
            ui.step(ctx).await;
        }
        let fire = ctx.down(G::Fire) || ctx.down(G::Enter);
        let pick = [G::N1, G::N2, G::N3, G::N4, G::N5, G::N6]
            .iter()
            .position(|g| ctx.down(*g))
            .map(|i| i as i32 + 1)
            .or(if fire { Some(item) } else { None });
        if let Some(p) = pick {
            if p == 5 {
                // Options stay on this screen (FUN_1000_0ef6).
                options(ctx, &mut ui, cur, &mentxt, &ofont).await;
                // MENU.C: back in the main menu the tune starts again and item 1 is selected.
                ctx.play(&snd);
                item = 1;
                continue;
            }
            ctx.fade_out().await;
            ctx.stop_fx();
            return p;
        }
        if ctx.down(G::Up) && item > 1 {
            for _ in 0..3 {
                ui.sp.move_by(cur, 0, -3);
                cycle(ctx, &pal, &mut rot);
                ui.step(ctx).await;
            }
            item -= 1;
        }
        if ctx.down(G::Down) && item < 6 {
            for _ in 0..3 {
                ui.sp.move_by(cur, 0, 3);
                cycle(ctx, &pal, &mut rot);
                ui.step(ctx).await;
            }
            item += 1;
        }
    }
}

/// Colours 0xE8..0xFE rotate in the menu (FUN_1000_6c03).
fn cycle(ctx: &mut Ctx, base: &Pal, rot: &mut i32) {
    let (lo, hi) = (0xe8i32, 0xfei32);
    let n = hi - lo + 1;
    *rot -= 1;
    if *rot < 0 {
        *rot = hi - lo;
    }
    for c in lo..=hi {
        let idx = lo + (*rot + (c - hi)).rem_euclid(n);
        ctx.scr.pal[c as usize] = base[idx as usize];
    }
}

/// MENU.C options(): the sound stops and the colours stand still while it is open.
async fn options(ctx: &mut Ctx, ui: &mut Ui, cur: i32, mentxt: &Spr, of: &Spr) {
    ctx.stop_fx();
    let draw = |ctx: &mut Ctx| {
        ctx.scr.blit_opaque(mentxt, 1, 0xa0, 0x2e);
        ctx.scr.text(of, 0x9f, 0x34, "OPTIONS");
        let s = if ctx.save.sound { "1 SOUND ON" } else { "1 SOUND OFF" };
        ctx.scr.text(of, 0x9f, 0x3f, s);
        ctx.scr.text(of, 0x9f, 0x48, "2 CHOOSE KEYS");
        ctx.scr.text(of, 0x9f, 0x51, "3 MAINMENU");
    };
    draw(ctx);
    ui.sp.set_pos(cur, 0x8e, 0x41);
    let mut item = 1;
    loop {
        while ctx.any() {
            ui.step(ctx).await;
        }
        while !ctx.any() {
            ui.step(ctx).await;
        }
        let fire = ctx.down(G::Fire) || ctx.down(G::Enter);
        if ctx.down(G::N1) || (item == 1 && fire) {
            ctx.save.sound = !ctx.save.sound;
            if !ctx.save.sound {
                ctx.stop_fx();
                ctx.stop_music();
            }
            ctx.scr.text(of, 0x9f, 0x3f, if ctx.save.sound { "1 SOUND ON " } else { "1 SOUND OFF" });
            crate::save::store(&ctx.save);
        }
        if ctx.down(G::N2) || (item == 2 && fire) {
            ctx.scr.blit_opaque(mentxt, 1, 0xa0, 0x2e);
            ui.sp.remove(cur);
            choose_keys(ctx, of).await;
            draw(ctx);
            // Afterwards item 1 is selected again (MENU.C: selected = showoptions()).
            let c = ui.sp.create(&ui.lib, 0, 0x8e, 0x41, 0);
            debug_assert_eq!(c, cur);
            item = 1;
            continue;
        }
        if ctx.down(G::N3) || (item == 3 && fire) {
            ctx.scr.blit_opaque(mentxt, 0, 0xa0, 0x2e);
            ui.sp.set_pos(cur, 0x8e, 0x3a);
            return;
        }
        if ctx.down(G::Up) && item > 1 {
            for _ in 0..3 {
                ui.sp.move_by(cur, 0, -3);
                ui.step(ctx).await;
            }
            item -= 1;
        }
        if ctx.down(G::Down) && item < 3 {
            for _ in 0..3 {
                ui.sp.move_by(cur, 0, 3);
                ui.step(ctx).await;
            }
            item += 1;
        }
    }
}

/// FUN_1000_0ddb: press a key for each action.
async fn choose_keys(ctx: &mut Ctx, of: &Spr) {
    let labels = [("LEFT", 0x35, 2usize), ("RIGHT", 0x3f, 3), ("UP", 0x49, 0), ("DOWN", 0x53, 1), ("FIRE", 0x5d, 4), ("PAUSE", 0x67, 5)];
    for (label, y, slot) in labels {
        while !macroquad::input::get_keys_down().is_empty() {
            ctx.tick().await;
        }
        ctx.scr.text(of, 0xa0, y, label);
        let key = loop {
            ctx.tick().await;
            if ctx.mode == Mode::SelfTest {
                break None;
            }
            if let Some(k) = macroquad::input::get_keys_down().into_iter().next() {
                break Some(k);
            }
        };
        if let Some(k) = key {
            ctx.save.keys[slot] = k;
            ctx.scr.text(of, 200, y, &crate::save::key_name(k));
        }
    }
    // MENU.C defkeys: waitforkey() (all keys up, then one more press) before saving.
    ctx.wait_release().await;
    if ctx.mode != Mode::SelfTest {
        ctx.wait_press().await;
    }
    crate::save::store(&ctx.save);
}

/// FUN_1000_1171: the two-page story picture.
async fn story(ctx: &mut Ctx) {
    black(ctx);
    let (pix, w, h, pal) = ctx.res.pcx("LONGTIME.PCX");
    ctx.scr.set_pal6(&pal);
    ctx.scr.picture(&pix, w, h, 0, 0);
    let music = ctx.res.sound("LONGTIME.SND").await;
    ctx.play_music(&music, true);
    ctx.fade_in().await;
    ctx.wait_press().await;
    let mut off = 0;
    while off < 0xc6 {
        ctx.scr.picture(&pix, w, h, 0, off);
        retrace(ctx).await;
        off += 2;
    }
    ctx.poll_now();
    ctx.wait_press().await;
    ctx.fade_out().await;
    ctx.stop_music();
    ctx.wait_release().await;
}

/// FUN_1000_099f: the closing picture and sound.
async fn quit(ctx: &mut Ctx) {
    ctx.wait_release().await;
    ctx.scr.clear(0);
    let close = font(ctx, "close");
    ctx.scr.blit_opaque(&close, 0, 0x14, 0x6e);
    let snd = ctx.res.sound("CLOSE.SND").await;
    ctx.play(&snd);
    ctx.fade_in().await;
    loop {
        if ctx.any() || !ctx.fx_playing() {
            break;
        }
        ctx.tick().await;
    }
    ctx.fade_out().await;
    ctx.stop_fx();
    // A web page or a phone app can't close itself: go back to the title instead.
    if cfg!(any(target_arch = "wasm32", target_os = "android")) {
        title(ctx).await;
    } else {
        ctx.quit = true;
    }
}

// ------------------------------------------------------------------ a game

/// FUN_1000_1fa3: levels, lives, shop, then game over or victory and the high scores.
pub async fn new_game(ctx: &mut Ctx) {
    let cheat = ctx.cheat;
    let mut hero = Hero::new(&ctx.res.weapons, cheat);
    ctx.scr.set_pal6(&ctx.res.game_pal.clone());
    let mut result;
    loop {
        let level = ctx.res.level(hero.level).await;
        level_intro(ctx, &level.name).await;
        let mut field = Field::new(ctx, level, &mut hero);
        loop {
            if hero.cheat & 1 == 0 {
                hero.lives -= 1;
            }
            result = field.play(ctx, &mut hero).await;
            if result != Outcome::Died || hero.lives <= 0 {
                break;
            }
        }
        if hero.cheat & 1 == 0 {
            hero.lives += 1;
        }
        hero.level += 1;
        if result != Outcome::Won || hero.level == 5 {
            break;
        }
        shop(ctx, &mut hero).await;
    }
    if result != Outcome::Won || hero.level != 5 {
        game_over(ctx).await;
    } else {
        victory(ctx).await;
    }
    high_scores(ctx, Some(hero.score)).await;
}

/// FUN_1000_1755: "GO" and the level name.
async fn level_intro(ctx: &mut Ctx, name: &str) {
    ctx.scr.set_pal6(&ctx.res.game_pal.clone());
    ctx.scr.fade = 0;
    ctx.scr.clear(0);
    ctx.wait_release().await;
    let go = font(ctx, "go");
    let f3 = font(ctx, "font3");
    ctx.scr.blit_opaque(&go, 0, 0x44, 0x6e);
    let x = 0x9f - ((name.len() as i32 * f3.w as i32) >> 1);
    ctx.scr.text(&f3, x, 0x28, name);
    let snd = ctx.res.sound("GO.SND").await;
    ctx.play(&snd);
    if ctx.mode == Mode::SelfTest {
        ctx.tick().await;
    } else {
        ctx.wait_press().await;
    }
    ctx.stop_fx();
}

/// FUN_1000_18cf: the landscape after losing the last bottle.
async fn game_over(ctx: &mut Ctx) {
    ctx.wait_release().await;
    black(ctx);
    ctx.scr.clear(0);
    let pal = picture(ctx, "LANDSCAP.PCX", 5);
    ctx.scr.set_pal6(&pal);
    let snd = ctx.res.sound("TOD.SND").await;
    ctx.play(&snd);
    ctx.fade_in().await;
    loop {
        if ctx.any() || !ctx.fx_playing() {
            break;
        }
        ctx.tick().await;
    }
    ctx.fade_out().await;
    ctx.stop_fx();
    ctx.scr.clear(0);
}

/// FUN_1000_182a: all five levels done.
async fn victory(ctx: &mut Ctx) {
    ctx.wait_release().await;
    black(ctx);
    ctx.scr.clear(0);
    let pal = picture(ctx, "SKY.PCX", 0);
    ctx.scr.set_pal6(&pal);
    let f1 = font(ctx, "font1");
    ctx.scr.text(&f1, 0x28, 0xcc, "SORRY. NO HERO TUNE THIS TIME.");
    ctx.fade_in().await;
    ctx.wait_press().await;
    ctx.fade_out().await;
    ctx.scr.clear(0);
}

/// FUN_1000_4039: the high-score table; with a score, enter a name (and the winner text).
pub async fn high_scores(ctx: &mut Ctx, score: Option<u32>) {
    black(ctx);
    ctx.scr.clear(0);
    let pal = picture(ctx, "SCORE.PCX", 0);
    ctx.scr.set_pal6(&pal);
    let f1 = font(ctx, "font1");
    let f2 = font(ctx, "font2");
    let mut rank = None;
    if let Some(s) = score {
        if let Some(i) = ctx.save.table.iter().position(|h| (h.score as i32) < s as i32) {
            ctx.save.table.insert(i, crate::ctx::HighScore { score: s, name: String::new() });
            ctx.save.table.truncate(8);
            rank = Some(i);
            if i == 0 {
                // HISCORE.C: a new winner's old text is gone before the table is shown.
                ctx.save.winner.clear();
            }
        }
    }
    for i in 0..8 {
        let y = 0x3c + 14 * i as i32;
        let (name, sc) = ctx.save.table.get(i).map(|h| (h.name.clone(), h.score)).unwrap_or_default();
        ctx.scr.text(&f1, 0x37, y, &name);
        money_digits(ctx, &f1, 0xff, y, sc);
    }
    let wt = ctx.save.winner.clone();
    ctx.scr.text(&f2, (0x140 - wt.len() as i32 * f2.w as i32) >> 1, 0xd0, &wt);
    let music = ctx.res.sound("HS.SND").await;
    ctx.play_music(&music, false);
    ctx.fade_in().await;
    if let Some(r) = rank {
        let y = 0x3c + 14 * r as i32;
        let name = type_line(ctx, &f1, 0x37, y, 15, false).await;
        let name = if name.is_empty() { "TOO DRUNK".to_string() } else { name };
        ctx.scr.text(&f1, 0x37, y, &name);
        ctx.save.table[r].name = name;
        if r == 0 {
            ctx.scr.text(&f1, 0x40, 0xc2, "PLEASE ENTER WINNER TEXT");
            ctx.scr.fill_rect(0, 0xd1, 320, 10, 0);
            let t = type_line(ctx, &f2, 0, 0xd0, 33, true).await;
            let t = if t.is_empty() { "TOO DRUNK TO WRITE DOWN A TEXT!!!".to_string() } else { t };
            ctx.scr.fill_rect(0, 0xd0, 320, 10, 0);
            ctx.scr.text(&f2, (0x140 - t.len() as i32 * f2.w as i32) >> 1, 0xd0, &t);
            ctx.save.winner = t;
            ctx.scr.text(&f1, 0x40, 0xc2, "                        ");
        }
        crate::save::store(&ctx.save);
    } else if let Some(s) = score {
        ctx.scr.text(&f1, 0x37, 0xb7, "YOUR SCORE");
        money_digits(ctx, &f1, 0xff, 0xb7, s);
    }
    if ctx.mode != Mode::SelfTest {
        while ctx.any() && ctx.music_playing() {
            ctx.tick().await;
        }
        while !ctx.any() && ctx.music_playing() {
            ctx.tick().await;
        }
    }
    ctx.fade_out().await;
    ctx.stop_music();
    ctx.wait_release().await;
}

/// Name / winner-text entry with the blinking '#' cursor (FUN_1000_3eb7).
async fn type_line(ctx: &mut Ctx, f: &Spr, x0: i32, y: i32, max: usize, centred: bool) -> String {
    let mut s = String::new();
    ctx.typed.clear();
    ctx.wait_release().await;
    let mut blink = 0;
    let w = f.w as i32;
    loop {
        if ctx.mode == Mode::SelfTest {
            return s;
        }
        let cx = if centred { (0x140 - (s.len() as i32 + 1) * w) / 2 + s.len() as i32 * w } else { x0 + s.len() as i32 * w };
        if blink == 0 {
            blink = 8;
            ctx.scr.blit_opaque(f, 3, cx, y);
        }
        if blink == 4 {
            ctx.scr.blit_opaque(f, 0, cx, y);
        }
        blink -= 1;
        ctx.tick().await;
        let mut done = ctx.down(G::Enter) && !ctx.typed.iter().any(|c| *c != '\r' && *c != '\n');
        let typed: Vec<char> = std::mem::take(&mut ctx.typed);
        let mut erased = false;
        for c in typed {
            let c = c.to_ascii_uppercase();
            let ok = if centred {
                ('A'..=']').contains(&c) || ('\''..=';').contains(&c) || " !\"=?".contains(c)
            } else {
                c.is_ascii_uppercase() || c.is_ascii_digit() || " ?.'!\"".contains(c)
            };
            if c == '\r' || c == '\n' {
                done = true;
            } else if c == '\u{8}' {
                s.pop();
                erased = true;
            } else if ok {
                // HISCORE.C: when the line is full, a new character replaces the last one.
                if s.len() >= max {
                    s.pop();
                }
                s.push(c);
            }
        }
        // Some systems send Backspace only as a key, others also as a character: erase once.
        if !erased && macroquad::input::is_key_pressed(macroquad::input::KeyCode::Backspace) {
            s.pop();
        }
        ctx.scr.blit_opaque(f, 0, cx, y);
        if centred {
            ctx.scr.fill_rect(0, y, 320, f.h as i32, 0);
            ctx.scr.text(f, (0x140 - (s.len() as i32 + 1) * w) / 2, y, &s);
        } else {
            ctx.scr.fill_rect(x0, y, w * (max as i32 + 1), f.h as i32, 0);
            ctx.scr.text(f, x0, y, &s);
        }
        if done {
            ctx.scr.blit_opaque(f, 0, if centred { 0 } else { x0 + s.len() as i32 * w }, y);
            ctx.wait_release().await;
            return s;
        }
    }
}

// ------------------------------------------------------------------ the shop

/// FUN_1000_3ddc: money for the level, then the sell counter and the store.
pub async fn shop(ctx: &mut Ctx, hero: &mut Hero) {
    let gain = hero.score.wrapping_sub(hero.last_shop_score).wrapping_add(1500);
    hero.money = hero.money.wrapping_add(gain / 2500 * 5);
    hero.last_shop_score = hero.score;
    if hero.cheat & 2 != 0 {
        hero.money = 999_995; // SHOP.C: Fr. 9999.95
    }
    let sell_music = ctx.res.sound("SELL.SND").await;
    ctx.play_music(&sell_music, true);
    if hero.mounts.len() > 1 {
        sell(ctx, hero).await;
    }
    ctx.stop_music();
    let buy_music = ctx.res.sound("BUY.SND").await;
    ctx.play_music(&buy_music, true);
    if ctx.mode == Mode::SelfTest {
        ctx.stop_music();
        return;
    }
    while !store(ctx, hero).await {}
    ctx.stop_music();
}

fn weapon_lib(ctx: &Ctx) -> Vec<Spr> {
    ctx.res.wsprites.clone()
}

/// FUN_1000_272d: sell mounted weapons for 3/4 of their price.
async fn sell(ctx: &mut Ctx, hero: &mut Hero) {
    black(ctx);
    ctx.scr.clear(0);
    let pal = picture(ctx, "SELL.PCX", 0);
    ctx.scr.set_pal6(&pal);
    let exit2 = font(ctx, "exit2");
    let f1 = font(ctx, "font1");
    let w = f1.w as i32;
    ctx.scr.text(&f1, 0xb0, 0xd2, "YOUR MONEY");
    money_places(ctx, &f1, 0x13f - w, 0xd2, hero.money, 7);
    ctx.scr.text(&f1, 0xa0 - 15 * w, 0, "CHOOSE WEAPON YOU WANT TO SELL");
    let mut ui = Ui::new(weapon_lib(ctx));
    let (mut hs, _, _) = place_parts(&mut ui.sp, &ui.lib, hero, 0xd4, 0x3b);
    ctx.scr.text(&f1, 0xa0, 0x96, "EXIT");
    let mut sel = 0usize;
    let mut door = -1i32;
    let mut pulse = 0x60i32;
    let mut dp = 2;
    ui.draw(ctx);
    ctx.fade_in().await;
    let exit_anim = |ctx: &mut Ctx, door: &mut i32| {
        // FUN_1000_2697: the exit sign opens while EXIT is chosen and closes otherwise.
        if *door < 4 && -2 < *door {
            ctx.scr.fill_rect(0x14, 0x19, exit2.w as i32, exit2.h as i32, 0);
            if *door < 3 {
                *door += 1;
                ctx.scr.blit(&exit2, *door as usize, 0x14, 0x19);
            } else {
                ctx.scr.blit(&exit2, *door as usize, 0x14, 0x19);
                *door += 1;
            }
        }
        if *door < -2 {
            *door += 1;
            ctx.scr.fill_rect(0x14, 0x19, exit2.w as i32, exit2.h as i32, 0);
            if *door < -3 {
                ctx.scr.blit(&exit2, (-4 - *door) as usize, 0x14, 0x19);
            }
        }
    };
    let show = |ctx: &mut Ctx, hero: &Hero, sel: usize| {
        if sel == 0 {
            ctx.scr.text(&f1, 0xa0, 0x96, "EXIT                ");
            ctx.scr.text(&f1, 0xa0, 0xaa, "                      ");
        } else {
            ctx.scr.text(&f1, 0xa0, 0x96, "                    ");
            ctx.scr.text(&f1, 0xa0, 0x96, &hero.mounts[sel].w.name);
            ctx.scr.text(&f1, 0xa0, 0xaa, "VALUE:");
            let v = hero.mounts[sel].w.price as u32 * 3 / 4;
            money_digits(ctx, &f1, 0x117, 0xaa, v);
        }
    };
    while hero.mounts.len() > 1 {
        while ctx.any() {
            exit_anim(ctx, &mut door);
            ui.step(ctx).await;
        }
        let mut leave = false;
        loop {
            exit_anim(ctx, &mut door);
            if sel != 0 {
                pulse += dp;
                if !(0x60..=0x6f).contains(&pulse) {
                    dp = -dp;
                    pulse += dp;
                }
                ui.flash_col = pulse as u8;
                ui.sp.flash(hs[sel]);
            }
            let n = hero.mounts.len();
            if ctx.down(G::Right) || ctx.down(G::Left) {
                let right = ctx.down(G::Right);
                let was_exit = sel == 0;
                sel = if right { (sel + 1) % n } else { (sel + n - 1) % n };
                if was_exit {
                    door = -4 - door;
                }
                if sel == 0 {
                    door = if door == -2 { -1 } else { -4 - door };
                }
                show(ctx, hero, sel);
                while ctx.down(G::Right) || ctx.down(G::Left) {
                    exit_anim(ctx, &mut door);
                    ui.step(ctx).await;
                }
            }
            ui.step(ctx).await;
            if ctx.down(G::Fire) || ctx.down(G::Enter) {
                break;
            }
            if ctx.mode == Mode::SelfTest {
                leave = true;
                break;
            }
        }
        if sel == 0 || leave {
            break;
        }
        // Sell: the money counts up (SHOP.C givemoneyback), one Rappen per pass without waiting
        // for the timer, so it is there at once.
        hero.money += hero.mounts[sel].w.price as u32 * 3 / 4;
        money_places(ctx, &f1, 0x13f - w, 0xd2, hero.money, 7);
        ui.step(ctx).await;
        ui.sp.remove(hs[sel]);
        hero.mounts.remove(sel);
        hs.remove(sel);
        sel = 0;
        door = if door == -2 { -1 } else { -4 - door };
        show(ctx, hero, 0);
    }
    ctx.fade_out().await;
}

/// FUN_1000_376b: the store counter. Returns true when the player leaves the shop.
async fn store(ctx: &mut Ctx, hero: &mut Hero) -> bool {
    black(ctx);
    ctx.scr.clear(0);
    let pal = picture(ctx, "STORE.PCX", 0);
    ctx.scr.set_pal6(&pal);
    let wl = weapon_lib(ctx);
    let weapons = ctx.res.weapons.clone();
    let last = weapons.len().min(5) - 1;
    // The goods on the counter.
    let mut item_x = vec![0i32; last + 1];
    let mut x = 0xd0;
    for i in 1..=last {
        if weapons[i].kind & 0x8000 == 0 {
            continue;
        }
        let s = &wl[weapons[i].sprite];
        ctx.scr.blit(s, 0, x, 0x89 - s.h as i32);
        item_x[i] = x;
        x += s.w as i32 + 4;
    }
    let arrow = font(ctx, "arrow");
    let exit = font(ctx, "exit");
    let f1 = font(ctx, "font1");
    let cf = font(ctx, "cfont");
    let w = f1.w as i32;
    let mut ui = Ui::new(vec![arrow.clone(), exit.clone()]);
    let ah = arrow.h as i32;
    let arrow_x = |i: usize| item_x[i] + wl[weapons[i].sprite].w as i32 / 2 - arrow.w as i32 / 2;
    let mut sel: i32 = 1;
    let mut cur = ui.sp.create(&ui.lib, 0, arrow_x(1), 0x6b - ah, 0);
    let price_sign = |ctx: &mut Ctx, v: u32| {
        // FUN_1000_363c: the price on the little sign.
        let cw = cf.w as i32;
        let mut x = 0x3a;
        let mut v = v;
        for _ in 0..2 {
            ctx.scr.blit_opaque(&cf, (v % 10) as usize, x, 0x69);
            v /= 10;
            x -= cw;
        }
        ctx.scr.blit_opaque(&cf, 10, x, 0x69);
        let mut zeros = 0;
        for k in 2..5 {
            x -= cw;
            let d = v % 10;
            v /= 10;
            if d == 0 && k != 2 {
                ctx.scr.blit_opaque(&cf, 11, x, 0x69);
                zeros += 1;
            } else {
                ctx.scr.blit_opaque(&cf, d as usize, x, 0x69);
                for z in 1..=zeros {
                    ctx.scr.blit_opaque(&cf, 0, x + z * cw, 0x69);
                }
                zeros = 0;
            }
        }
    };
    ctx.scr.text(&f1, 0, 0xd2, &weapons[1].name);
    price_sign(ctx, weapons[1].price as u32);
    ctx.scr.text(&f1, 0xb0, 0xd2, "YOUR MONEY");
    money_places(ctx, &f1, 0x13f - w, 0xd2, hero.money, 7);
    ui.draw(ctx);
    ctx.fade_in().await;
    while ctx.any() {
        ui.step(ctx).await;
    }
    loop {
        let mut ok = true;
        if ctx.down(G::Right) && (sel as usize) < last {
            ctx.scr.text(&f1, 0, 0xd2, "                 ");
            sel += 1;
            if sel == 0 {
                ui.sp.remove(cur);
                ctx.scr.text(&f1, 0, 0xd2, "MENU CARD");
                ctx.scr.blit(&exit, 0, 4, 0x3b);
                cur = ui.sp.create(&ui.lib, 0, 0xe, 0x6a - ah, 0);
                while ctx.any() {
                    ui.step(ctx).await;
                }
            } else if sel == 1 {
                ui.sp.set_pos(cur, arrow_x(1), 0x6b - ah);
                ctx.scr.text(&f1, 0, 0xd2, &weapons[1].name);
                price_sign(ctx, weapons[1].price as u32);
                while ctx.any() {
                    ui.step(ctx).await;
                }
            } else {
                let s = sel as usize;
                let dist = wl[weapons[s - 1].sprite].w as i32 / 2 + wl[weapons[s].sprite].w as i32 / 2 + 4;
                let mut d = 0;
                while d < dist {
                    ui.sp.move_by(cur, 2, 0);
                    ui.step(ctx).await;
                    d += 2;
                }
                ctx.scr.text(&f1, 0, 0xd2, &weapons[s].name);
                price_sign(ctx, weapons[s].price as u32);
            }
        }
        if ctx.down(G::Left) && sel > -1 {
            ctx.scr.text(&f1, 0, 0xd2, "                 ");
            sel -= 1;
            if sel == -1 {
                ui.sp.remove(cur);
                cur = ui.sp.create(&ui.lib, 1, 4, 0x3b, 0);
                ctx.scr.text(&f1, 0, 0xd2, "EXIT SHOP");
                while ctx.any() {
                    ui.step(ctx).await;
                }
            } else if sel == 0 {
                ui.sp.set_pos(cur, 0xe, 0x6a - ah);
                ctx.scr.text(&f1, 0, 0xd2, "MENU CARD");
                price_sign(ctx, 0);
                while ctx.any() {
                    ui.step(ctx).await;
                }
            } else {
                let s = sel as usize;
                let dist = wl[weapons[s + 1].sprite].w as i32 / 2 + wl[weapons[s].sprite].w as i32 / 2 + 4;
                let mut d = 0;
                while d < dist {
                    ui.sp.move_by(cur, -2, 0);
                    ui.step(ctx).await;
                    d += 2;
                }
                ctx.scr.text(&f1, 0, 0xd2, &weapons[s].name);
                price_sign(ctx, weapons[s].price as u32);
            }
        }
        ui.step(ctx).await;
        if ctx.down(G::Fire) || ctx.down(G::Enter) {
            if sel < 1 {
                break;
            }
            let wp = &weapons[sel as usize];
            if wp.price as u32 > hero.money {
                ok = false;
                ctx.scr.text(&f1, 0, 0xd2, "NOT ENOUGH MONEY");
            }
            if hero.mounts.len() > 6 {
                ok = false;
                ctx.scr.text(&f1, 0, 0xd2, "NO PLACE TO PUT");
            }
            if ok {
                break;
            }
        }
    }
    if sel == -1 {
        ctx.fade_out().await;
        return true;
    }
    if sel == 0 {
        ctx.fade_out().await;
        menu_card(ctx, hero).await;
        return false;
    }
    let wp = weapons[sel as usize].clone();
    pay(ctx, &mut ui, hero, &f1, wp.price as u32).await;
    ctx.fade_out().await;
    place(ctx, hero, &wp).await;
    false
}

/// Paying (SHOP.C payforweapon): the money counts down one Rappen per pass without waiting for
/// the timer, so the new amount shows at once.
async fn pay(ctx: &mut Ctx, ui: &mut Ui, hero: &mut Hero, f1: &Spr, price: u32) {
    if hero.cheat & 2 != 0 {
        return;
    }
    hero.money = hero.money.saturating_sub(price);
    money_places(ctx, f1, 0x13f - f1.w as i32, 0xd2, hero.money, 7);
    ui.step(ctx).await;
}

/// FUN_1000_2c54: drinks (weapons 5..) and schnaps bonus (extra life, speed).
async fn menu_card(ctx: &mut Ctx, hero: &mut Hero) {
    let weapons = ctx.res.weapons.clone();
    let mut header = "COOL AND FRESH DRINKS";
    'rebuild: loop {
        black(ctx);
        ctx.scr.clear(0);
        let pal = picture(ctx, "MENUCARD.PCX", 0);
        ctx.scr.set_pal6(&pal);
        let mf = font(ctx, "mfont");
        let f1 = font(ctx, "font1");
        let fw = f1.w as i32;
        let mh = mf.h as i32;
        let mut ui = Ui::new(vec![font(ctx, "mencur2"), font(ctx, "menback")]);
        ui.sp.create(&ui.lib, 1, 0x30, 0x1a, 0);
        let cur = ui.sp.create(&ui.lib, 0, 0x30, 0x1b, 2);
        ctx.scr.text(&mf, 0x45, 0xb4, "BACK TO SHOP");
        ctx.scr.text(&mf, 0x45, 0xc, header);
        // Items and where the cursor sits for each.
        let mut items: Vec<(usize, i32)> = vec![];
        let mut y = 0xc + mh + 6;
        for (i, wp) in weapons.iter().enumerate().skip(5) {
            if wp.kind & 0x8000 != 0 {
                ctx.scr.text(&mf, 0x45, y, &dotted(&wp.name));
                money_digits(ctx, &mf, 0xfe, y, wp.price as u32);
                items.push((i, y + 2));
                y += mh + 3;
            }
        }
        y += 8;
        ctx.scr.text(&mf, 0x45, y, "SCHNAPS BONUS");
        y += mh + 6;
        for (i, wp) in weapons.iter().enumerate().skip(5) {
            if wp.kind & 0x8000 == 0 {
                ctx.scr.text(&mf, 0x45, y, &dotted(&wp.name));
                money_digits(ctx, &mf, 0xfe, y, wp.price as u32);
                items.push((i, y + 2));
                y += mh + 3;
            }
        }
        let back = items.len();
        let ypos = |c: usize| if c == back { 0xb4 } else { items[c].1 };
        ctx.scr.text(&f1, 0xb0, 0xd2, "YOUR MONEY");
        money_places(ctx, &f1, 0x13f - fw, 0xd2, hero.money, 7);
        ui.draw(ctx);
        ctx.fade_in().await;
        let mut c = 0usize;
        loop {
            while ctx.any() {
                ui.step(ctx).await;
            }
            loop {
                if ctx.down(G::Up) {
                    ctx.scr.text(&f1, 0, 0xd2, "                 ");
                    if c == 0 {
                        c = back;
                        ui.sp.set_pos(cur, 0x30, ypos(c));
                        while ctx.any() {
                            ui.step(ctx).await;
                        }
                    } else {
                        let (from, to) = (ypos(c), ypos(c - 1));
                        c -= 1;
                        let mut yy = from;
                        while yy > to {
                            yy = (yy - 2).max(to);
                            ui.sp.set_pos(cur, 0x30, yy);
                            ui.step(ctx).await;
                        }
                    }
                }
                if ctx.down(G::Down) {
                    ctx.scr.text(&f1, 0, 0xd2, "                 ");
                    if c == back {
                        c = 0;
                        ui.sp.set_pos(cur, 0x30, ypos(0));
                        while ctx.any() {
                            ui.step(ctx).await;
                        }
                    } else {
                        let (from, to) = (ypos(c), ypos(c + 1));
                        c += 1;
                        let mut yy = from;
                        while yy < to {
                            yy = (yy + 2).min(to);
                            ui.sp.set_pos(cur, 0x30, yy);
                            ui.step(ctx).await;
                        }
                    }
                }
                ui.step(ctx).await;
                if ctx.down(G::Fire) || ctx.down(G::Enter) {
                    break;
                }
            }
            if c == back {
                ctx.fade_out().await;
                return;
            }
            let wp = weapons[items[c].0].clone();
            // FUN_1000_2bad
            if wp.price as u32 > hero.money && hero.cheat & 2 == 0 {
                ctx.scr.text(&f1, 0, 0xd2, "NOT ENOUGH MONEY");
                continue;
            }
            if wp.kind & 1 != 0 && hero.speed > 7 {
                ctx.scr.text(&f1, 0, 0xd2, "ONLY 2 SPEEDUPS");
                continue;
            }
            if wp.kind & 0x8000 != 0 && hero.mounts.len() > 6 {
                ctx.scr.text(&f1, 0, 0xd2, "NO PLACE TO PUT");
                continue;
            }
            pay(ctx, &mut ui, hero, &f1, wp.price as u32).await;
            if wp.kind & 0x8000 == 0 {
                if wp.kind == 1 {
                    hero.speed += 2;
                } else if wp.kind == 2 {
                    hero.lives += 1;
                }
                continue;
            }
            ctx.fade_out().await;
            place(ctx, hero, &wp).await;
            header = "GETRAENKE UND WAFFEN";
            continue 'rebuild;
        }
    }
}

/// A menu-card line: the name padded with dots to 28 characters (SHOP.C), the price is
/// written over the end.
fn dotted(name: &str) -> String {
    let mut s: String = name.chars().take(28).collect();
    while s.len() < 28 {
        s.push('.');
    }
    s
}

/// FUN_1000_22cb: move the new weapon around the bottle and fix it with Fire (Esc gives up).
async fn place(ctx: &mut Ctx, hero: &mut Hero, wp: &crate::res::Wpn) {
    black(ctx);
    ctx.scr.clear(0);
    let pal = picture(ctx, "SELL.PCX", 0);
    ctx.scr.set_pal6(&pal);
    let f1 = font(ctx, "font1");
    let fw = f1.w as i32;
    let mut ui = Ui::new(weapon_lib(ctx));
    let (hs, bx, by) = place_parts(&mut ui.sp, &ui.lib, hero, 0xd4, 0x3b);
    ctx.scr.text(&f1, 0xa0 - 13 * fw, 0, "PLACE WEAPON OR PRESS ESC!");
    let (mut x, mut y) = (0x9c, 0x14);
    let item = ui.sp.create(&ui.lib, wp.sprite, x, y, 2);
    let (_, _, iw, ih) = ui.sp.cur(item);
    let mut pulse = 0x60i32;
    let mut dp = 2;
    ui.draw(ctx);
    ctx.fade_in().await;
    while ctx.any() {
        ui.step(ctx).await;
    }
    let placed;
    loop {
        pulse += dp;
        if !(0x60..=0x6f).contains(&pulse) {
            dp = -dp;
            pulse += dp;
        }
        ui.flash_col = pulse as u8;
        ui.sp.flash(item);
        let clear = |ctx: &mut Ctx| ctx.scr.text(&f1, 0xa0, 0x96, "       ");
        if ctx.down(G::Up) {
            if y - 4 >= 0x14 {
                y -= 4;
            }
            clear(ctx);
        }
        if ctx.down(G::Down) {
            if y + 4 <= ((0x5f - ih) & !3) {
                y += 4;
            }
            clear(ctx);
        }
        if ctx.down(G::Left) {
            if x - 4 >= 0x9c {
                x -= 4;
            }
            clear(ctx);
        }
        if ctx.down(G::Right) {
            if x + 4 <= ((0x10c - iw) & !3) {
                x += 4;
            }
            clear(ctx);
        }
        ui.sp.set_pos(item, x, y);
        let mut done = None;
        if ctx.down(G::Fire) || ctx.down(G::Enter) {
            done = Some(true);
            for h in &hs {
                if ui.sp.overlap(item, *h) {
                    ctx.scr.text(&f1, 0xa0, 0x96, "OVERLAP");
                    done = None;
                }
            }
        }
        if ctx.down(G::Esc) || ctx.mode == Mode::SelfTest {
            done = Some(false);
        }
        ui.step(ctx).await;
        if let Some(d) = done {
            placed = d;
            break;
        }
    }
    if placed {
        hero.mounts.push(Mount { rx: x - bx, ry: y - by, w: wp.clone() });
    }
    ctx.fade_out().await;
}

// ------------------------------------------------------------------ developer shortcuts

fn test_keys(t: u64) -> Vec<G> {
    let mut v = vec![G::Fire];
    if (t / 25).is_multiple_of(2) {
        v.push(G::Left);
    } else {
        v.push(G::Right);
    }
    v
}

/// `--scene name`: jump straight to one screen (for checking it against the original).
pub async fn scene(ctx: &mut Ctx, name: &str) {
    let parts: Vec<&str> = name.split(':').collect();
    let mut hero = Hero::new(&ctx.res.weapons, 0);
    hero.money = 1234;
    let w = ctx.res.weapons.clone();
    hero.mounts.push(Mount { rx: -16, ry: 4, w: w[3].clone() });
    hero.mounts.push(Mount { rx: 20, ry: 8, w: w[7].clone() });
    match parts[0] {
        "title" => title(ctx).await,
        "menu" => {
            menu_choice(ctx).await;
        }
        "story" => story(ctx).await,
        "hs" => high_scores(ctx, None).await,
        "gameover" => game_over(ctx).await,
        "victory" => victory(ctx).await,
        "sell" => sell(ctx, &mut hero).await,
        "store" => {
            store(ctx, &mut hero).await;
        }
        "menucard" => menu_card(ctx, &mut hero).await,
        "place" => place(ctx, &mut hero, &w[2]).await,
        "intro" => {
            let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let level = ctx.res.level(n).await;
            level_intro(ctx, &level.name).await;
        }
        "play" => {
            let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let ticks: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(100);
            let level = ctx.res.level(n).await;
            hero.cheat = 4;
            let mut field = Field::new(ctx, level, &mut hero);
            ctx.script = Some(test_keys);
            ctx.fast_until = ctx.ticks + ticks as u64;
            ctx.shot_frame = ctx.frames + ticks as u64 / 4 + 4;
            field.play_for(ctx, &mut hero, ticks + 1000).await;
        }
        _ => {}
    }
}

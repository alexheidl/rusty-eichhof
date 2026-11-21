//! The playing field: a re-implementation of the original's object system (FUN_1000_5b82 ..
//! FUN_1000_6511), its sprite engine (FUN_1000_7036 .. FUN_1000_76b1) and the per-tick loop
//! (FUN_1000_653e). Numbers and rules are documented in MECHANICS.md.

use crate::ctx::{Ctx, G};
use crate::gfx::{Screen, Spr};
use crate::res::{Level, Table, Wpn};

pub const PLAY_BOTTOM: i32 = 0xd3; // DS:3646 during play
pub const RIGHT: i32 = 0x13f;

// ------------------------------------------------------------------ sprite engine

#[derive(Clone, Default)]
struct SObj {
    active: bool,
    spr: usize,
    w: i32,
    h: i32,
    x: i32,
    y: i32,
    /// Position at the last draw (what FUN_1000_71f2 reports).
    px: i32,
    py: i32,
    anim: u16,
    oneshot: bool,
    kill: u8,
    flash: bool,
}

/// 60 sprite slots drawn in slot order: normal sprites take slots 30, 29 .. 1 and "top" sprites
/// (flag 2) take 30, 31 .. 59, exactly like FUN_1000_7036.
pub struct Sprites {
    s: Vec<SObj>,
}

impl Sprites {
    pub fn new() -> Self {
        Sprites { s: vec![SObj::default(); 60] }
    }
    pub fn clear(&mut self) {
        for o in &mut self.s {
            o.active = false;
        }
    }
    pub fn create(&mut self, lib: &[Spr], spr: usize, x: i32, y: i32, flags: u16) -> i32 {
        let Some(def) = lib.get(spr) else { return -1 };
        let mut top = flags & 2 != 0;
        for _ in 0..2 {
            for k in 0..30 {
                let i = if top { 30 + k } else { 30 - k };
                if !self.s[i].active {
                    self.s[i] = SObj {
                        active: true,
                        spr,
                        w: def.w as i32,
                        h: def.h as i32,
                        x,
                        y,
                        px: x,
                        py: y,
                        anim: 0,
                        oneshot: flags & 1 != 0,
                        kill: 0,
                        flash: false,
                    };
                    return i as i32;
                }
            }
            top = !top;
        }
        -1
    }
    fn get(&self, h: i32) -> Option<&SObj> {
        if h < 0 {
            return None;
        }
        self.s.get(h as usize).filter(|o| o.active)
    }
    fn get_mut(&mut self, h: i32) -> Option<&mut SObj> {
        if h < 0 {
            return None;
        }
        self.s.get_mut(h as usize).filter(|o| o.active)
    }
    pub fn set_pos(&mut self, h: i32, x: i32, y: i32) {
        if let Some(o) = self.get_mut(h) {
            o.x = x;
            o.y = y;
        }
    }
    pub fn move_by(&mut self, h: i32, dx: i32, dy: i32) {
        if let Some(o) = self.get_mut(h) {
            o.x += dx;
            o.y += dy;
        }
    }
    pub fn flash(&mut self, h: i32) {
        if let Some(o) = self.get_mut(h) {
            o.flash = true;
        }
    }
    pub fn delete(&mut self, h: i32) {
        if let Some(o) = self.get_mut(h) {
            o.kill = 2;
        }
    }
    /// Removes a sprite at once (FUN_1000_714e).
    pub fn remove(&mut self, h: i32) {
        if let Some(o) = self.get_mut(h) {
            o.active = false;
        }
    }
    pub fn change(&mut self, lib: &[Spr], h: i32, spr: usize) {
        let Some(def) = lib.get(spr) else { return };
        if let Some(o) = self.get_mut(h) {
            o.spr = spr;
            o.w = def.w as i32;
            o.h = def.h as i32;
            o.anim = 0;
        }
    }
    /// Position at the last draw plus the size (FUN_1000_71f2 / 73f2).
    pub fn pos(&self, h: i32) -> (i32, i32, i32, i32) {
        self.get(h).map(|o| (o.px, o.py, o.w, o.h)).unwrap_or((0, 0, 0, 0))
    }
    pub fn cur(&self, h: i32) -> (i32, i32, i32, i32) {
        self.get(h).map(|o| (o.x, o.y, o.w, o.h)).unwrap_or((0, 0, 0, 0))
    }
    /// Bounding-box overlap test (FUN_1000_71a1).
    pub fn overlap(&self, a: i32, b: i32) -> bool {
        let (Some(a), Some(b)) = (self.get(a), self.get(b)) else { return false };
        a.x - b.w <= b.x && b.x <= a.x + a.w && a.y - b.h <= b.y && b.y <= a.y + a.h
    }
    /// Outside the play field (FUN_1000_7240).
    pub fn offscreen(&self, h: i32, bottom: i32) -> bool {
        match self.get(h) {
            Some(o) => o.x < 0 || o.x > RIGHT || o.y < 0 || o.y > bottom,
            None => true,
        }
    }
    /// One display step (FUN_1000_76b1): advance animations, retire finished sprites, draw.
    pub fn draw(&mut self, scr: &mut Screen, lib: &[Spr], flash_col: u8) {
        for o in self.s.iter_mut().filter(|o| o.active) {
            let Some(def) = lib.get(o.spr) else { continue };
            let frames2 = (def.count().max(1) * 2) as u16;
            let mut a = o.anim + def.speed;
            if a >= frames2 {
                a = 0;
                if o.oneshot {
                    o.kill = 2;
                    o.oneshot = false;
                }
            }
            o.anim = a;
            if o.kill != 0 {
                o.kill -= 1;
                if o.kill == 0 {
                    o.active = false;
                }
            }
        }
        for o in self.s.iter_mut().filter(|o| o.active) {
            o.px = o.x;
            o.py = o.y;
            if o.kill != 0 {
                continue;
            }
            let Some(def) = lib.get(o.spr) else { continue };
            let x = o.x - o.x.rem_euclid(def.xgran.max(1));
            let fr = (o.anim >> 1) as usize;
            if o.flash {
                scr.blit_flat(def, fr, x, o.y, flash_col);
                o.flash = false;
            } else {
                scr.blit(def, fr, x, o.y);
            }
        }
    }
}

// ------------------------------------------------------------------ the player between levels

#[derive(Clone)]
pub struct Mount {
    pub rx: i32,
    pub ry: i32,
    pub w: Wpn,
}

pub struct Hero {
    pub mounts: Vec<Mount>,
    pub lives: i32,
    pub speed: i32,
    pub score: u32,
    pub money: u32,
    pub last_shop_score: u32,
    pub level: usize,
    pub cheat: u16,
}

impl Hero {
    /// New game (FUN_1000_14d8).
    pub fn new(weapons: &[Wpn], cheat: u16) -> Hero {
        Hero {
            mounts: vec![Mount { rx: 0, ry: 0, w: weapons[0].clone() }],
            lives: 4,
            speed: 4,
            score: 0,
            money: 0,
            last_shop_score: 0,
            level: 0,
            cheat,
        }
    }
}

#[derive(Clone, Copy)]
struct Part {
    h: i32,
    shot: usize,
    reload: i16,
    counter: i16,
}

/// Places the ship and its mounted weapons around a centre (FUN_1000_2224); returns the
/// rounded top-left the mounts are relative to.
pub fn place_parts(sp: &mut Sprites, lib: &[Spr], hero: &Hero, cx: i32, cy: i32) -> (Vec<i32>, i32, i32) {
    let (mw, mh) = lib.get(hero.mounts[0].w.sprite).map(|s| (s.w as i32, s.h as i32)).unwrap_or((0, 0));
    let bx = cx - mw / 2;
    let by = (cy - mh / 2) & !3;
    let bx4 = bx & !3;
    let mut hs = vec![];
    for m in &hero.mounts {
        hs.push(sp.create(lib, m.w.sprite, m.rx + bx4, m.ry + by, 2));
    }
    (hs, bx4, by)
}

// ------------------------------------------------------------------ the level

#[derive(Clone, Copy, Default)]
struct Shot {
    h: i32,
    ready: bool,
    dmg: i16,
    speed: i16,
    vx: i16,
    vy: i16,
    pc: usize,
}

#[derive(Clone, Copy, Default)]
struct Foe {
    h: i32,
    flags: u16,
    points: u16,
    exp: i16,
    hp: i16,
    loop_pc: usize,
    loop_x: i32,
    loop_y: i32,
    pc: usize,
    aim: i16,
    adx: i32,
    ady: i32,
    err: i32,
    sx: i32,
    sy: i32,
    cnt: i32,
}

#[derive(Clone, Copy, Default)]
struct Exp {
    src: i32,
    x: i32,
    y: i32,
    pc: usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    Won,
    Died,
    Quit,
}

/// Restart point of a level (DS:0f36..0f42): set when the level starts and at 0x8003 entries.
#[derive(Clone, Copy)]
pub struct Checkpoint {
    tbl_pos: usize,
    time: u16,
    score: u32,
    goal: i16,
}

pub struct Field {
    shots_t: &'static Table,
    pub lib: Vec<Spr>,
    nweap: usize,
    pub sp: Sprites,
    parts: Vec<Option<Part>>,
    shots: Vec<Option<Shot>>,
    foes: Vec<Option<Foe>>,
    exps: Vec<Option<Exp>>,
    stars: Vec<(i32, i32, u8, i32)>,
    stars_moving: bool,
    pub level: Level,
    pub check: Checkpoint,
    // score display state (FUN_1000_6404)
    sd_count: u16,
    sd_x: i32,
    sd_val: u32,
    /// Statistics for the self-test.
    pub stats: Stats,
}

#[derive(Default, Clone, Debug)]
pub struct Stats {
    pub shots_fired: u32,
    pub hits: u32,
    pub kills: u32,
    pub moved: i32,
}

impl Field {
    pub fn new(ctx: &Ctx, level: Level, hero: &mut Hero) -> Field {
        let mut lib = ctx.res.wsprites.clone();
        let nweap = lib.len();
        lib.extend(level.sprites.iter().cloned());
        // FUN_1000_1975: the level's score bonus and starting money.
        hero.score = hero.score.wrapping_add(level.score_bonus as u32);
        hero.money = hero.money.wrapping_add(level.money as u32);
        let stars = level.stars.iter().map(|s| ((s.x as i32) & !3, s.y as i32, s.colour, s.speed as i32)).collect();
        let check = Checkpoint { tbl_pos: 0, time: 0, score: hero.score, goal: level.goal };
        Field {
            shots_t: ctx.res.shots,
            lib,
            nweap,
            sp: Sprites::new(),
            parts: vec![None; 7],
            shots: vec![None; 30],
            foes: vec![None; 26],
            exps: vec![None; 8],
            stars,
            // FUN_1000_1975 ends with FUN_1000_6f25: the stars start moving.
            stars_moving: true,
            level,
            check,
            sd_count: 0,
            sd_x: 0x72,
            sd_val: 0,
            stats: Stats::default(),
        }
    }

    fn lvl_spr(&self, i: i16) -> usize {
        self.nweap + i.max(0) as usize
    }

    // ----- spawning

    /// FUN_1000_5bd3: a shot of type `t` at `pos + (dx,dy)`.
    fn spawn_shot(&mut self, t: usize, x: i32, y: i32, spawner: Option<usize>) {
        let Some(slot) = self.shots.iter().position(|s| s.is_none()) else { return };
        let sh = self.shots_t;
        let Some(&st) = sh.starts.get(t) else { return };
        let ready = match spawner {
            None => true,
            Some(i) => slot <= i,
        };
        let h = self.sp.create(&self.lib, sh.w(st + 4) as usize, x + sh.w(st) as i32, y + sh.w(st + 1) as i32, 0);
        self.shots[slot] = Some(Shot { h, ready, dmg: sh.w(st + 2), speed: sh.w(st + 3), vx: 0, vy: 0, pc: st + 5 });
    }

    /// FUN_1000_5c3f: enemy `t` at (x,y); `target` for aimed enemies that then follow a script.
    fn spawn_foe(&mut self, t: i16, x: i32, y: i32, target: Option<(i32, i32)>, step: &mut u16, player: i32) {
        let Some(slot) = self.foes.iter().position(|f| f.is_none()) else { return };
        let Some(&st) = self.level.foes.starts.get(t.max(0) as usize) else { return };
        let ft = &self.level.foes;
        let flags = ft.w(st) as u16;
        let mut f = Foe { flags, ..Default::default() };
        if flags & 8 != 0 {
            *step = 0;
        }
        if flags & 6 == 0 {
            f.hp = ft.w(st + 1);
            f.points = ft.w(st + 2) as u16;
            f.exp = ft.w(st + 3);
        }
        if flags & 0x10 != 0 {
            f.pc = st + 6;
        }
        if flags & 0x20 != 0 {
            f.aim = ft.w(st + 5);
            let (tx, ty) = match target {
                Some(t) if flags & 0x10 != 0 => t,
                _ => {
                    let (px, py, _, _) = self.sp.pos(player);
                    (px, py)
                }
            };
            let mut dx = tx - x;
            f.sx = 1;
            if dx < 0 {
                dx = -dx;
                f.sx = -1;
            }
            let mut dy = ty - y;
            f.sy = 1;
            if dy < 0 {
                dy = -dy;
                f.sy = -1;
            }
            f.adx = dx;
            f.ady = dy;
            let m = dx.max(dy);
            f.cnt = m;
            f.err = m >> 1;
        }
        let spr = self.lvl_spr(ft.w(st + 4));
        f.h = self.sp.create(&self.lib, spr, x, y, 0);
        self.foes[slot] = Some(f);
    }

    /// FUN_1000_5d1b: explosion `t` centred on sprite `src`.
    fn spawn_exp(&mut self, src: i32, t: i16) {
        let Some(slot) = self.exps.iter().position(|e| e.is_none()) else { return };
        let Some(&st) = self.level.exps.starts.get(t.max(0) as usize) else { return };
        let (x, y, w, h) = self.sp.pos(src);
        self.exps[slot] = Some(Exp { src, x: x + w / 2, y: y + h / 2, pc: st + 1 });
    }

    // ----- per-tick updates

    /// FUN_1000_6342: fire / count down every mounted part.
    fn fire(&mut self, firing: bool) {
        for i in 0..self.parts.len() {
            let Some(mut p) = self.parts[i] else { continue };
            if firing {
                let was = p.counter;
                p.counter -= 1;
                if was == 0 {
                    p.counter = p.reload;
                    let (x, y, _, _) = self.sp.pos(p.h);
                    self.spawn_shot(p.shot, x, y, None);
                    self.stats.shots_fired += 1;
                }
            } else if p.counter != 0 {
                p.counter -= 1;
            }
            self.parts[i] = Some(p);
        }
    }

    /// FUN_1000_5d6a: move every part by (dx,dy), clamped by the main bottle.
    fn steer(&mut self, mut dx: i32, mut dy: i32, blink: bool) {
        let Some(main) = self.parts[0] else { return };
        if blink {
            self.sp.flash(main.h);
        }
        let (x, y, w, h) = self.sp.pos(main.h);
        let nx = x + dx;
        if nx < 0 {
            dx -= nx;
        } else if nx + w > RIGHT {
            dx -= nx + w - RIGHT;
        }
        let ny = y + dy;
        if ny < 0 {
            dy -= ny;
        } else if ny + h > PLAY_BOTTOM {
            dy -= ny + h - PLAY_BOTTOM;
        }
        self.stats.moved += dx.abs() + dy.abs();
        for p in self.parts.iter().flatten() {
            self.sp.move_by(p.h, dx, dy);
        }
    }

    /// FUN_1000_622e: shots against shootable enemies. Returns sounds to play (none here).
    fn collide_shots(&mut self, hero: &mut Hero, goal: &mut i16, step: &mut u16) {
        for fi in 0..self.foes.len() {
            for si in 0..self.shots.len() {
                let Some(mut f) = self.foes[fi] else { break };
                if f.flags & 4 != 0 {
                    break;
                }
                let Some(mut s) = self.shots[si] else { continue };
                if !self.sp.overlap(f.h, s.h) {
                    continue;
                }
                if f.flags & 2 == 0 {
                    self.sp.flash(f.h);
                    self.stats.hits += 1;
                    let old = s.dmg;
                    s.dmg -= f.hp;
                    let before = f.hp as u16;
                    f.hp = f.hp.wrapping_sub(old);
                    if before < old as u16 || f.hp == 0 {
                        hero.score = hero.score.wrapping_add(f.points as u32);
                        self.foes[fi] = None;
                        self.spawn_exp(f.h, f.exp);
                        self.stats.kills += 1;
                        if f.flags & 1 != 0 {
                            *goal -= 1;
                        }
                        if f.flags & 8 != 0 {
                            *step = 1;
                        }
                    } else {
                        self.foes[fi] = Some(f);
                    }
                } else {
                    s.dmg = 0;
                }
                if s.dmg < 1 {
                    self.sp.delete(s.h);
                    self.shots[si] = None;
                } else {
                    self.shots[si] = Some(s);
                }
            }
        }
    }

    /// FUN_1000_62e3: any enemy touching the main bottle kills it.
    fn collide_player(&mut self) -> bool {
        let Some(main) = self.parts[0] else { return false };
        for f in self.foes.iter().flatten() {
            if self.sp.overlap(f.h, main.h) {
                self.spawn_exp(main.h, 0);
                for p in self.parts.iter_mut() {
                    if let Some(pp) = p.take() {
                        self.sp.delete(pp.h);
                    }
                }
                return true;
            }
        }
        false
    }

    /// FUN_1000_5dee: shot scripts.
    fn update_shots(&mut self) {
        let sh = self.shots_t;
        for i in 0..self.shots.len() {
            let Some(mut s) = self.shots[i] else { continue };
            if !s.ready {
                s.ready = true;
                self.shots[i] = Some(s);
                continue;
            }
            loop {
                let w = sh.w(s.pc) as u16;
                if w & 0xfff0 == 0x8000 && w <= 0x8003 {
                    match w {
                        0x8000 => {
                            self.sp.delete(s.h);
                            self.shots[i] = None;
                            break;
                        }
                        0x8001 => {
                            let (x, y, _, _) = self.sp.pos(s.h);
                            let t = sh.w(s.pc + 1) as usize;
                            s.pc += 2;
                            self.shots[i] = Some(s);
                            self.spawn_shot(t, x, y, Some(i));
                            continue;
                        }
                        0x8002 => {
                            s.pc += 1;
                            self.home(&mut s);
                            self.shots[i] = Some(s);
                            self.retire_offscreen_shot(i);
                            break;
                        }
                        _ => {
                            // 0x8003: keep the velocity, bounce off the edges.
                            s.pc += 1;
                            let (x, y, w, h) = self.sp.pos(s.h);
                            let mut nx = x + s.vx as i32;
                            if nx < 0 || nx + w > RIGHT {
                                s.vx = -s.vx;
                                nx += 2 * s.vx as i32;
                            }
                            let mut ny = y + s.vy as i32;
                            if ny < 0 || ny + h > PLAY_BOTTOM {
                                s.vy = -s.vy;
                                ny += 2 * s.vy as i32;
                            }
                            self.sp.set_pos(s.h, nx, ny);
                            self.shots[i] = Some(s);
                            break;
                        }
                    }
                } else {
                    s.vx = w as i16;
                    s.vy = sh.w(s.pc + 1);
                    s.pc += 2;
                    self.sp.move_by(s.h, s.vx as i32, s.vy as i32);
                    self.shots[i] = Some(s);
                    self.retire_offscreen_shot(i);
                    break;
                }
            }
        }
    }

    fn retire_offscreen_shot(&mut self, i: usize) {
        if let Some(s) = self.shots[i] {
            if self.sp.offscreen(s.h, PLAY_BOTTOM) {
                self.sp.delete(s.h);
                self.shots[i] = None;
            }
        }
    }

    /// Homing step (op 0x8002): steer toward the first shootable enemy, 1 px/tick per axis.
    fn home(&mut self, s: &mut Shot) {
        let (sx, sy, _, _) = self.sp.pos(s.h);
        let target = self.foes.iter().flatten().find(|f| f.flags & 6 == 0).map(|f| self.sp.pos(f.h));
        if let Some((fx, fy, fw, fh)) = target {
            let ddx = fx + fw / 2 - sx;
            let ddy = fy + fh / 2 - sy;
            let ax = ddx.abs();
            let ay = ddy.abs();
            let sp = (s.speed as i32).max(1);
            let (tvx, tvy) = if ay < ax {
                (if ddx < 0 { -sp } else { sp }, ddy / (ax / sp + 1))
            } else {
                (ddx / (ay / sp + 1), if ddy < 0 { -sp } else { sp })
            };
            s.vx = if tvx <= s.vx as i32 { s.vx - 1 } else { s.vx + 1 };
            s.vy = if tvy <= s.vy as i32 { s.vy - 1 } else { s.vy + 1 };
        }
        self.sp.move_by(s.h, s.vx as i32, s.vy as i32);
    }

    /// FUN_1000_5fc2: enemy scripts and aimed flight.
    fn update_foes(&mut self, ctx: &mut Ctx, step: &mut u16, player: i32) {
        for i in 0..self.foes.len() {
            let Some(mut f) = self.foes[i] else { continue };
            if f.flags & 0x20 == 0 {
                let mut stuck = false;
                loop {
                    let ft = &self.level.foes;
                    let w = ft.w(f.pc) as u16;
                    if w & 0xfff0 != 0x8000 {
                        let dy = ft.w(f.pc + 1);
                        f.pc += 2;
                        self.sp.move_by(f.h, w as i16 as i32, dy as i32);
                        break;
                    }
                    match w {
                        0x8000 => {
                            self.sp.delete(f.h);
                            self.foes[i] = None;
                            stuck = true;
                            break;
                        }
                        0x8001 => {
                            let s = self.lvl_spr(ft.w(f.pc + 1));
                            f.pc += 2;
                            self.sp.change(&self.lib, f.h, s);
                        }
                        0x8002 => {
                            let (t, ox, oy) = (ft.w(f.pc + 1), ft.w(f.pc + 2) as i32, ft.w(f.pc + 3) as i32);
                            f.pc += 4;
                            let (x, y, _, _) = self.sp.pos(f.h);
                            self.foes[i] = Some(f);
                            self.spawn_foe(t, x + ox, y + oy, None, step, player);
                        }
                        0x8004 => {
                            f.pc = f.loop_pc;
                            self.sp.set_pos(f.h, f.loop_x, f.loop_y);
                        }
                        0x8005 => {
                            f.loop_pc = f.pc + 1;
                            f.pc += 1;
                            let (x, y, _, _) = self.sp.pos(f.h);
                            f.loop_x = x;
                            f.loop_y = y;
                        }
                        0x8006 => {
                            let s = ft.w(f.pc + 1);
                            f.pc += 2;
                            let snd = self.level.sounds.get(s.max(0) as usize).cloned().flatten();
                            ctx.play(&snd);
                        }
                        _ => {
                            stuck = true;
                            break;
                        }
                    }
                }
                if stuck {
                    if self.foes[i].is_some() {
                        self.foes[i] = Some(f);
                    }
                    continue;
                }
                self.foes[i] = Some(f);
            } else {
                let (mut x, mut y, _, _) = self.sp.pos(f.h);
                let mut n = f.aim;
                let mut err = f.err;
                let mut arrived = false;
                loop {
                    if err < f.adx {
                        err += f.ady;
                        x += f.sx;
                    }
                    if f.adx <= err {
                        err -= f.adx;
                        y += f.sy;
                    }
                    if f.flags & 0x10 != 0 {
                        f.cnt -= 1;
                        if f.cnt == 0 {
                            f.flags &= !0x20;
                            arrived = true;
                            break;
                        }
                    }
                    n -= 1;
                    if n <= 0 {
                        break;
                    }
                }
                self.sp.set_pos(f.h, x, y);
                if arrived {
                    self.foes[i] = Some(f);
                    continue;
                }
                f.err = err;
                if self.sp.offscreen(f.h, PLAY_BOTTOM) {
                    self.sp.delete(f.h);
                    self.foes[i] = None;
                } else {
                    self.foes[i] = Some(f);
                }
            }
        }
    }

    /// FUN_1000_613e: explosion scripts.
    fn update_exps(&mut self, ctx: &mut Ctx, step: &mut u16, player: i32) {
        for i in 0..self.exps.len() {
            let Some(mut e) = self.exps[i] else { continue };
            loop {
                let et = &self.level.exps;
                let w = et.w(e.pc) as u16;
                e.pc += 1;
                match w {
                    0x8000 => {
                        self.exps[i] = None;
                        break;
                    }
                    0x8001 => {
                        let (s, ox, oy) = (et.w(e.pc), et.w(e.pc + 1) as i32, et.w(e.pc + 2) as i32);
                        e.pc += 3;
                        let spr = self.lvl_spr(s);
                        self.sp.create(&self.lib, spr, e.x + ox, e.y + oy, 3);
                    }
                    0x8003 => {
                        let s = et.w(e.pc);
                        e.pc += 1;
                        let snd = self.level.sounds.get(s.max(0) as usize).cloned().flatten();
                        ctx.play(&snd);
                    }
                    0x8004 => {
                        self.sp.delete(e.src);
                    }
                    0x8005 => {
                        let (t, ox, oy) = (et.w(e.pc), et.w(e.pc + 1) as i32, et.w(e.pc + 2) as i32);
                        e.pc += 3;
                        self.spawn_foe(t, e.x + ox, e.y + oy, None, step, player);
                    }
                    0x8006 => {
                        let (t, ox, oy) = (et.w(e.pc), et.w(e.pc + 1) as i32, et.w(e.pc + 2) as i32);
                        let (tx, ty) = (et.w(e.pc + 3) as i32, et.w(e.pc + 4) as i32);
                        e.pc += 5;
                        self.spawn_foe(t, e.x + ox, e.y + oy, Some((tx, ty)), step, player);
                    }
                    _ => {
                        // 0x8002 (and anything unknown): wait for the next tick.
                        self.exps[i] = Some(e);
                        break;
                    }
                }
            }
        }
    }

    // ----- drawing

    fn draw_stars(&mut self, scr: &mut Screen) {
        for s in self.stars.iter_mut() {
            if self.stars_moving {
                s.1 += s.3;
                if s.1 >= PLAY_BOTTOM {
                    s.1 -= PLAY_BOTTOM;
                }
            }
            // The original always plots a star one step ahead of its position, moving or not
            // (XMODEASM starfield).
            let y = (s.1 + s.3).rem_euclid(PLAY_BOTTOM);
            if s.0 >= 0 && s.0 < 320 && (0..PLAY_BOTTOM).contains(&y) {
                scr.pix[y as usize * scr.w + s.0 as usize] = s.2;
            }
        }
    }

    fn draw_field(&mut self, ctx: &mut Ctx, flash: u8) {
        let scr = &mut ctx.scr;
        let w = scr.w;
        scr.pix[..PLAY_BOTTOM as usize * w].iter_mut().for_each(|p| *p = 0);
        self.draw_stars(scr);
        scr.clip = (0, 0, 320, PLAY_BOTTOM);
        self.sp.draw(scr, &self.lib, flash);
        scr.reset_clip();
    }

    /// The status bar and the spare bottles (FUN_1000_1f6e, FUN_1000_64ab).
    fn draw_bar(&mut self, ctx: &mut Ctx, hero: &Hero) {
        let bar = ctx.res.font("bar").clone();
        ctx.scr.blit_opaque(&bar, 0, 0, PLAY_BOTTOM);
        let bf = ctx.res.font("barfont").clone();
        let mut x = 0x28;
        for _ in 0..6 {
            ctx.scr.blit(&bf, 10, x, 0xd4);
            x -= 4;
        }
        let mut x = 0x28;
        for _ in 0..hero.lives.clamp(0, 6) {
            ctx.scr.blit(&bf, 11, x, 0xd4);
            x -= 4;
        }
    }

    /// FUN_1000_6404: one digit of the score per tick, right to left, gap after two digits.
    fn score_digit(&mut self, ctx: &mut Ctx, score: u32) {
        let bf = ctx.res.font("barfont").clone();
        if self.sd_count == 16 {
            self.sd_count = 0;
        }
        if self.sd_count == 0 {
            self.sd_val = score;
            self.sd_x = 0x72;
        } else if self.sd_count == 4 {
            self.sd_x -= 4;
        }
        let x = self.sd_x - self.sd_x.rem_euclid(bf.xgran.max(1));
        ctx.scr.blit(&bf, 10, x, 0xd4);
        let d = (self.sd_val % 10) as usize;
        if self.sd_count & 1 == 0 {
            ctx.scr.blit(&bf, d, x, 0xd4);
        } else {
            self.sd_val /= 10;
            ctx.scr.blit(&bf, d, x, 0xd4);
            self.sd_x -= 6;
        }
        self.sd_count += 1;
    }

    // ----- one attempt at the level (FUN_1000_1f6e + FUN_1000_2224 + FUN_1000_653e)

    pub async fn play(&mut self, ctx: &mut Ctx, hero: &mut Hero) -> Outcome {
        self.play_for(ctx, hero, u32::MAX).await
    }

    /// Plays at most `limit` ticks (the self-test uses a limit; normal play has none).
    pub async fn play_for(&mut self, ctx: &mut Ctx, hero: &mut Hero, limit: u32) -> Outcome {
        let mut ticks_left = limit;
        // FUN_1000_1f6e: fresh screen, bar, empty tables.
        ctx.scr.set_pal6(&ctx.res.game_pal.clone());
        ctx.scr.fade = 0;
        ctx.scr.clear(0);
        self.sp.clear();
        for v in self.parts.iter_mut() {
            *v = None;
        }
        self.shots.iter_mut().for_each(|s| *s = None);
        self.foes.iter_mut().for_each(|s| *s = None);
        self.exps.iter_mut().for_each(|s| *s = None);
        self.draw_bar(ctx, hero);
        self.sd_count = 0;
        let (hs, _, _) = place_parts(&mut self.sp, &self.lib, hero, 0x96, 0xbe);
        for (i, h) in hs.iter().enumerate() {
            if i < 7 {
                let w = &hero.mounts[i].w;
                self.parts[i] = Some(Part { h: *h, shot: w.shot, reload: w.reload, counter: 0 });
            }
        }
        let player = self.parts[0].map(|p| p.h).unwrap_or(-1);
        ctx.touch.step = hero.speed as f32;

        // FUN_1000_653e
        hero.score = self.check.score;
        let mut goal = self.check.goal;
        let mut step: u16 = 1;
        let mut timer: i32 = 0x46;
        let mut result = 0;
        let mut pause_latch = false;
        let mut pos = self.check.tbl_pos;
        let mut time = self.check.time;
        loop {
            // Timeline entries due now.
            while pos < self.level.tbl.len() && self.level.tbl[pos].t == time {
                let e = self.level.tbl[pos];
                pos += 1;
                match e.what {
                    0x8000 => self.stars_moving = true,
                    0x8001 => self.stars_moving = false,
                    0x8002 => {
                        let snd = self.level.sounds.get(e.x.max(0) as usize).cloned().flatten();
                        ctx.play(&snd);
                    }
                    0x8003 => {
                        self.check = Checkpoint { tbl_pos: pos, time, score: hero.score, goal };
                    }
                    w if w & 0x8000 != 0 => {}
                    w => self.spawn_foe(w as i16, e.x as i32, e.y as i32, None, &mut step, player),
                }
            }
            if ctx.down(G::Esc) {
                hero.lives = -1;
                ctx.stop_fx();
                return Outcome::Quit;
            }
            // FUN_1000_6393: fire, steer, pause.
            let firing = ctx.down(G::Fire);
            self.fire(firing);
            let mut dx = 0;
            let mut dy = 0;
            if ctx.down(G::Left) {
                dx -= hero.speed;
            }
            if ctx.down(G::Right) {
                dx += hero.speed;
            }
            if ctx.down(G::Up) {
                dy -= hero.speed;
            }
            if ctx.down(G::Down) {
                dy += hero.speed;
            }
            // Pause (GAMEASM keyboard): wait until all keys are up, then for any key. The key
            // that ends the pause is not swallowed, the movement read before the pause still
            // applies, and a held P doesn't pause again until it is pressed anew.
            if !ctx.down(G::Pause) {
                pause_latch = false;
            } else if !pause_latch {
                ctx.wait_release().await;
                ctx.wait_press().await;
                pause_latch = true;
            }
            self.steer(dx, dy, timer & 2 != 0);
            ctx.tick().await;
            self.draw_field(ctx, 15);
            self.collide_shots(hero, &mut goal, &mut step);
            if goal == 0 {
                goal = 1;
                timer = 100;
                result = 1;
            }
            if hero.cheat & 4 == 0 && timer == 0 && self.collide_player() {
                timer = 0x3c;
                result = 2;
            }
            self.score_digit(ctx, hero.score);
            self.update_shots();
            self.update_foes(ctx, &mut step, player);
            self.update_exps(ctx, &mut step, player);
            time = time.wrapping_add(step);
            ticks_left -= 1;
            if ticks_left == 0 {
                return Outcome::Quit;
            }
            if timer != 0 {
                timer -= 1;
                if timer == 0 && result != 0 {
                    ctx.stop_fx();
                    return if result == 1 { Outcome::Won } else { Outcome::Died };
                }
            }
        }
    }
}


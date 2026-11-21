//! Shared state for every screen: the indexed screen, input, sound, the 20 Hz clock and the saved
//! settings / high scores. Screens are written as async functions that call `tick().await` once
//! per original game step, which mirrors how the original program was structured.

use crate::gfx::Screen;
use crate::res::{Res, Snd};
use macroquad::audio::{play_sound, stop_sound, PlaySoundParams, Sound};

use macroquad::prelude::*;
use std::collections::HashSet;

/// Seconds per original game tick (PIT divisor 0xE8F6 → 20.007 Hz).
pub const TICK: f64 = 59638.0 / 1193182.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum G {
    Up,
    Down,
    Left,
    Right,
    Fire,
    Pause,
    Esc,
    Enter,
    Back,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Normal,
    Screenshot,
    SelfTest,
}

pub struct HighScore {
    pub score: u32,
    pub name: String,
}

pub struct Save {
    /// Up, Down, Left, Right, Fire, Pause.
    pub keys: [KeyCode; 6],
    pub sound: bool,
    pub table: Vec<HighScore>,
    pub winner: String,
}

pub type FrameHook = Box<dyn FnMut(&mut Ctx)>;

pub struct Ctx {
    pub scr: Screen,
    pub res: Res,
    pub save: Save,
    pub mode: Mode,
    held: HashSet<G>,
    /// Characters typed since the last tick (high-score names).
    pub typed: Vec<char>,
    /// Scripted input for the self-test.
    pub script_keys: HashSet<G>,
    acc: f64,
    pub frames: u64,
    pub ticks: u64,
    pub quit: bool,
    /// Run ticks without waiting until this tick (developer scenes).
    pub fast_until: u64,
    /// Frame at which `--screenshot` saves the window.
    pub shot_frame: u64,
    /// The original's `007.n` command-line cheat bits (1 lives, 2 money, 4 invulnerable).
    pub cheat: u16,
    /// Self-test input schedule: keys held at a given tick.
    pub script: Option<fn(u64) -> Vec<G>>,
    fx: Option<Sound>,
    music: Option<Sound>,
    fx_until: f64,
    fx_prio: u16,
    music_until: f64,
    time: f64,
    pub touch: crate::touch::Touch,
    /// A hook run before each presented frame (screenshot / self-test bookkeeping).
    pub on_frame: Option<FrameHook>,
}

impl Ctx {
    pub fn new(res: Res, mode: Mode) -> Ctx {
        let save = crate::save::load();
        Ctx {
            scr: Screen::new(),
            res,
            save,
            mode,
            held: HashSet::new(),
            typed: vec![],
            script_keys: HashSet::new(),
            acc: 0.0,
            frames: 0,
            ticks: 0,
            quit: false,
            fast_until: 0,
            shot_frame: 120,
            cheat: 0,
            script: None,
            fx: None,
            music: None,
            fx_until: 0.0,
            fx_prio: 0,
            music_until: 0.0,
            time: 0.0,
            touch: crate::touch::Touch::default(),
            on_frame: None,
        }
    }

    pub fn down(&self, g: G) -> bool {
        self.held.contains(&g)
    }

    /// The original's "a key is down" flag (DS:24f8).
    pub fn any(&self) -> bool {
        !self.held.is_empty()
    }

    /// Re-reads the keyboard now (for loops that wait on display retraces, not ticks).
    pub fn poll_now(&mut self) {
        self.poll();
    }

    fn poll(&mut self) {
        self.held.clear();
        if self.mode == Mode::SelfTest || self.script.is_some() {
            if let Some(f) = self.script {
                self.script_keys = f(self.ticks).into_iter().collect();
            }
            self.held.extend(self.script_keys.iter().copied());
            return;
        }
        let k = self.save.keys;
        let map = [
            (k[0], G::Up),
            (k[1], G::Down),
            (k[2], G::Left),
            (k[3], G::Right),
            (k[4], G::Fire),
            (k[5], G::Pause),
            (KeyCode::Escape, G::Esc),
            (KeyCode::Enter, G::Enter),
            (KeyCode::KpEnter, G::Enter),
            (KeyCode::Backspace, G::Back),
            (KeyCode::Key1, G::N1),
            (KeyCode::Key2, G::N2),
            (KeyCode::Key3, G::N3),
            (KeyCode::Key4, G::N4),
            (KeyCode::Key5, G::N5),
            (KeyCode::Key6, G::N6),
            (KeyCode::Kp1, G::N1),
            (KeyCode::Kp2, G::N2),
            (KeyCode::Kp3, G::N3),
            (KeyCode::Kp4, G::N4),
            (KeyCode::Kp5, G::N5),
            (KeyCode::Kp6, G::N6),
        ];
        for (code, g) in map {
            if is_key_down(code) {
                self.held.insert(g);
            }
        }
        // Any other key also counts for "press any key" screens.
        if !get_keys_down().is_empty() && self.held.is_empty() {
            self.held.insert(G::Enter);
        }
        for g in self.touch.held() {
            self.held.insert(g);
        }
    }

    /// Seconds the last frame took; in screenshot mode a fixed 1/60 s, so the 120th frame always
    /// shows the same moment whatever the display speed.
    pub fn dt(&self) -> f64 {
        if self.mode == Mode::Screenshot {
            1.0 / 60.0
        } else {
            get_frame_time() as f64
        }
    }

    /// One presented frame (no game step).
    pub async fn frame(&mut self) {
        while let Some(c) = get_char_pressed() {
            self.typed.push(c);
        }
        self.touch.update();
        if let Some(mut hook) = self.on_frame.take() {
            hook(self);
            if self.on_frame.is_none() {
                self.on_frame = Some(hook);
            }
        }
        self.scr.present();
        self.touch.draw();
        next_frame().await;
        self.frames += 1;
        self.time += self.dt();
        if self.fx.is_some() && self.time > self.fx_until {
            self.fx = None;
        }
        if self.music.is_some() && self.time > self.music_until {
            self.music = None;
        }
    }

    /// Waits for the next original game step (20 Hz) while keeping the window drawn.
    pub async fn tick(&mut self) {
        let fast = self.mode == Mode::SelfTest || self.ticks < self.fast_until;
        match fast {
            true => {
                // Run as fast as possible: show only every 16th step.
                if self.ticks.is_multiple_of(16) {
                    self.frame().await;
                }
            }
            false => {
                loop {
                    if self.acc >= TICK {
                        self.acc -= TICK;
                        break;
                    }
                    self.frame().await;
                    self.acc += (self.dt()).min(0.25);
                }
                if self.acc > TICK * 3.0 {
                    self.acc = 0.0;
                }
            }
        }
        self.ticks += 1;
        self.poll();
    }

    /// Waits `n` ticks; returns true when a key was pressed before the time ran out
    /// (the original's FUN_1000_2068).
    pub async fn wait_or_key(&mut self, n: u32) -> bool {
        let mut n = n;
        loop {
            if self.any() {
                return true;
            }
            n = n.saturating_sub(1);
            if n == 0 {
                return false;
            }
            self.tick().await;
        }
    }

    pub async fn wait_release(&mut self) {
        while self.any() {
            self.tick().await;
        }
    }

    pub async fn wait_press(&mut self) {
        while !self.any() {
            self.tick().await;
        }
    }

    /// Palette fade from black (FUN_1000_77f1, `glowin`): 11 steps, two display retraces each.
    pub async fn fade_in(&mut self) {
        let mut f: i32 = 60;
        while f >= 0 {
            self.scr.fade = f as u8;
            self.retrace_wait(2).await;
            f -= 6;
        }
        self.scr.fade = 0;
    }

    /// Palette fade to black (FUN_1000_77af).
    pub async fn fade_out(&mut self) {
        let mut f = 0u8;
        while f < 0x40 {
            self.scr.fade = f;
            self.retrace_wait(2).await;
            f += 6;
        }
        self.scr.fade = 63;
    }

    /// Every colour rises from black to white, one step of 3 per retrace (`glowto(63,63,63)`,
    /// used by the newspaper intro before it fades in).
    pub async fn glow_to_white(&mut self) {
        let mut i = 0u8;
        while i < 63 {
            self.scr.white = Some(i);
            self.retrace_wait(1).await;
            i += 3;
        }
        self.scr.white = None;
    }

    /// Waits `n` display retraces. The game's 320x220 Mode X runs the VGA at 59.94 Hz
    /// (25.175 MHz dot clock, 525 lines); the 640x350 EGA mode of the newspaper intro at 70.09 Hz.
    pub async fn retrace_wait(&mut self, n: u32) {
        if self.mode == Mode::SelfTest {
            return;
        }
        let hz = if self.scr.w == 640 { 70.09 } else { 59.94 };
        let t = n as f64 / hz;
        let mut waited = 0.0;
        while waited < t {
            self.frame().await;
            waited += (self.dt()).min(0.1);
        }
    }

    // ----- sound: one effect channel (like the SoundBlaster's single DMA channel) + music -----

    /// Plays an effect on the single voice: it only starts when its priority is at least that
    /// of the effect still sounding (the original's FUN_1000_4903).
    pub fn play(&mut self, s: &Snd) {
        if !self.save.sound || self.mode != Mode::Normal {
            return;
        }
        if let Some((snd, len, prio)) = s {
            if self.fx.is_some() && *prio < self.fx_prio {
                return;
            }
            self.fx_prio = *prio;
            if let Some(old) = self.fx.take() {
                stop_sound(&old);
            }
            play_sound(snd, PlaySoundParams { looped: false, volume: 1.0 });
            self.fx = Some(snd.clone());
            self.fx_until = self.time + len;
        }
    }

    pub fn play_music(&mut self, s: &Snd, looped: bool) {
        if !self.save.sound || self.mode != Mode::Normal {
            return;
        }
        if let Some((snd, len, _)) = s {
            self.stop_music();
            play_sound(snd, PlaySoundParams { looped, volume: 1.0 });
            self.music = Some(snd.clone());
            self.music_until = if looped { f64::MAX } else { self.time + len };
        }
    }

    pub fn stop_music(&mut self) {
        if let Some(old) = self.music.take() {
            stop_sound(&old);
        }
    }

    pub fn stop_fx(&mut self) {
        if let Some(old) = self.fx.take() {
            stop_sound(&old);
        }
    }

    /// Whether an effect started with `play` is still sounding (the original's FUN_1000_4bbf).
    pub fn fx_playing(&self) -> bool {
        self.fx.is_some()
    }

    pub fn music_playing(&self) -> bool {
        self.music.is_some()
    }
}

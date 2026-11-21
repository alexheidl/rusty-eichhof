//! Everything the game reads from the original BEER.DAT, decoded once into game-ready form.

use crate::dat::{self, Dat};
use crate::gfx::Spr;
use macroquad::audio::{load_sound_from_bytes, Sound};
use std::collections::HashMap;

/// A playable sound: the sound, its length in seconds and the original's priority.
pub type Snd = Option<(Sound, f64, u16)>;
pub type Pal = [[u8; 3]; 256];

#[cfg(feature = "embedded")]
pub static BEER_DAT: &[u8] = include_bytes!("../data/BEER.DAT");

/// The original archive: built in for private builds, otherwise read from `BEER.DAT` beside the
/// program (or next to the web page / in the Android app's assets).
pub async fn beer_dat() -> Option<&'static [u8]> {
    #[cfg(feature = "embedded")]
    {
        return Some(BEER_DAT);
    }
    #[allow(unreachable_code)]
    {
        for name in ["BEER.DAT", "beer.dat"] {
            #[allow(unused_mut)]
            let mut candidates = vec![name.to_string()];
            #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
            if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
                candidates.insert(0, dir.join(name).to_string_lossy().into_owned());
                // macOS app bundle: Contents/MacOS/<exe> -> also look in Contents/Resources and beside the .app
                candidates.push(dir.join("../Resources").join(name).to_string_lossy().into_owned());
                candidates.push(dir.join("../../..").join(name).to_string_lossy().into_owned());
            }
            for c in candidates {
                if let Ok(v) = macroquad::file::load_file(&c).await {
                    if v.starts_with(crate::dat::DAT_MAGIC) {
                        return Some(Box::leak(v.into_boxed_slice()));
                    }
                }
            }
        }
        None
    }
}

fn to6(p: &dat::Palette) -> Pal {
    let mut o = [[0u8; 3]; 256];
    for i in 0..256 {
        for c in 0..3 {
            o[i][c] = ((p[i][c] as u32 * 63 + 127) / 255) as u8;
        }
    }
    o
}

fn spr_from_frames(frames: Vec<dat::Frame>, speed: u16, xgran: i32) -> Spr {
    let (w, h) = frames.first().map(|f| (f.w, f.h)).unwrap_or((0, 0));
    Spr { w, h, frames: frames.into_iter().map(|f| f.pixels).collect(), speed, xgran }
}

/// A table of far-pointer records over one file, kept as 16-bit words so scripts can be
/// walked by word index (scripts loop back inside their own record).
pub struct Table {
    pub words: Vec<i16>,
    pub starts: Vec<usize>,
}

impl Table {
    fn parse(data: &[u8], stride: usize) -> Table {
        let words: Vec<i16> =
            data.chunks(2).map(|c| i16::from_le_bytes([c[0], *c.get(1).unwrap_or(&0)])).collect();
        let starts = dat::far_ptr_table(data, stride).into_iter().map(|p| p / 2).collect();
        Table { words, starts }
    }
    pub fn w(&self, at: usize) -> i16 {
        *self.words.get(at).unwrap_or(&(0x8000u16 as i16))
    }
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.starts.len()
    }
}

#[derive(Clone)]
pub struct Wpn {
    pub name: String,
    pub sprite: usize,
    pub shot: usize,
    pub price: u16,
    pub reload: i16,
    pub kind: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct TblEntry {
    pub t: u16,
    pub x: i16,
    pub y: i16,
    pub what: u16,
}

pub struct Level {
    #[allow(dead_code)]
    pub num: u16,
    pub name: String,
    pub goal: i16,
    pub score_bonus: u16,
    pub money: u16,
    pub sprites: Vec<Spr>,
    pub sounds: Vec<Snd>,
    pub foes: Table,
    pub exps: Table,
    pub tbl: Vec<TblEntry>,
    pub stars: Vec<dat::Star>,
}

pub struct Res {
    pub dat: Dat,
    pub game_pal: Pal,
    pub fonts: HashMap<&'static str, Spr>,
    pub weapons: Vec<Wpn>,
    pub wsprites: Vec<Spr>,
    pub shots: &'static Table,
    sounds: HashMap<String, Snd>,
}

pub fn u16_at(d: &[u8], p: usize) -> u16 {
    u16::from_le_bytes([*d.get(p).unwrap_or(&0), *d.get(p + 1).unwrap_or(&0)])
}

/// 8-bit unsigned mono samples to a 16-bit WAV file (plays the same everywhere). The 1993
/// samples use only a small part of the 8-bit range, so each sound is raised until its loudest
/// sample reaches about 90% of full scale (at most 4x); the waveform itself is unchanged.
pub fn wav16(rate: u32, samples: &[u8]) -> Vec<u8> {
    let peak = samples.iter().map(|&s| (s as i32 - 128).abs()).max().unwrap_or(0).max(1);
    let gain = (115.0 / peak as f32).clamp(1.0, 4.0);
    let n = samples.len() as u32 * 2;
    let mut v = Vec::with_capacity(44 + n as usize);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + n).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&rate.to_le_bytes());
    v.extend_from_slice(&(rate * 2).to_le_bytes());
    v.extend_from_slice(&2u16.to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&n.to_le_bytes());
    for &s in samples {
        let x = ((s as f32 - 128.0) * 256.0 * gain).clamp(-32768.0, 32767.0) as i16;
        v.extend_from_slice(&x.to_le_bytes());
    }
    v
}

async fn make_sound(s: &dat::Sound) -> Snd {
    if s.samples.is_empty() || s.rate == 0 {
        return None;
    }
    let wav = wav16(s.rate, &s.samples);
    let len = s.samples.len() as f64 / s.rate as f64;
    load_sound_from_bytes(&wav).await.ok().map(|snd| (snd, len, s.priority))
}

impl Res {
    pub fn load(beer_dat: &'static [u8]) -> Res {
        let dat = Dat::parse(beer_dat);
        let game_pal = to6(&dat::game_palette(&dat));
        let mut fonts = HashMap::new();
        for (key, file) in [
            ("font1", "FONT1.SPR"),
            ("font2", "FONT2.SPR"),
            ("font3", "FONT3.SPR"),
            ("mfont", "MFONT.SPR"),
            ("cfont", "CFONT.SPR"),
            ("ofont", "OFONT.SPR"),
            ("barfont", "BARFONT.SPR"),
            ("bar", "BAR.SPR"),
            ("go", "GO.SPR"),
            ("close", "CLOSE.SPR"),
            ("arrow", "ARROW.SPR"),
            ("mencur", "MENCUR.SPR"),
            ("mencur2", "MENCUR2.SPR"),
            ("menback", "MENBACK.SPR"),
            ("mentxt", "MENTXT.SPR"),
            ("exit", "EXIT.SPR"),
            ("exit2", "EXIT2.SPR"),
        ] {
            let data = dat.file(file).unwrap_or(&[]);
            fonts.insert(key, spr_from_frames(dat::decode_spr(data), 2, 2));
        }
        // Animation speeds the game gives these when it turns them into sprites.
        for (k, param) in [("mencur", 2u16), ("mencur2", 4), ("menback", 4), ("arrow", 2), ("exit", 4), ("exit2", 4), ("barfont", 2)] {
            if let Some(s) = fonts.get_mut(k) {
                s.speed = if param & 8 != 0 { 1 } else { 2 };
                s.xgran = if param & 7 >= 4 { 4 } else { 2 };
            }
        }
        let wpn = dat.file("WEAPONS.WPN").unwrap_or(&[]);
        let n = u16_at(wpn, 0) as usize;
        let weapons = (0..n)
            .map(|i| {
                let r = &wpn[2 + i * 30..2 + i * 30 + 30];
                let end = r[..20].iter().position(|&b| b == 0).unwrap_or(20);
                Wpn {
                    name: String::from_utf8_lossy(&r[..end]).into_owned(),
                    sprite: u16_at(r, 0x14) as usize,
                    shot: u16_at(r, 0x16) as usize,
                    price: u16_at(r, 0x18),
                    reload: u16_at(r, 0x1a) as i16,
                    kind: u16_at(r, 0x1c),
                }
            })
            .collect();
        let wsprites = sli_sprites(dat.file("WEAPONS.SLI").unwrap_or(&[]));
        let shots: &'static Table = Box::leak(Box::new(Table::parse(dat.file("WEAPONS.SHT").unwrap_or(&[]), 4)));
        Res { dat, game_pal, fonts, weapons, wsprites, shots, sounds: HashMap::new() }
    }

    pub fn font(&self, k: &str) -> &Spr {
        &self.fonts[k]
    }

    /// A full-screen picture: indices, width, height and its palette (6-bit).
    pub fn pcx(&self, name: &str) -> (Vec<u8>, usize, usize, Pal) {
        let p = dat::decode_pcx(self.dat.file(name).unwrap_or(&[]));
        (p.pixels, p.w, p.h, to6(&p.palette))
    }

    pub async fn sound(&mut self, name: &str) -> Snd {
        if let Some(s) = self.sounds.get(name) {
            return s.clone();
        }
        let s = match self.dat.file(name) {
            Some(d) => make_sound(&dat::decode_snd(d)).await,
            None => None,
        };
        self.sounds.insert(name.to_string(), s.clone());
        s
    }

    pub async fn level(&mut self, n: usize) -> Level {
        let f = |ext: &str| self.dat.file(&format!("LEVEL{}.{}", n, ext)).unwrap_or(&[]);
        let dsc = f("DSC");
        let (num, name) = dat::decode_dsc(dsc);
        let sprites = sli_sprites(f("SLI"));
        let bank = dat::decode_snd_bank(f("SND"));
        let mut sounds = vec![];
        for s in &bank {
            sounds.push(make_sound(s).await);
        }
        let tb = f("TBL");
        let cnt = u16_at(tb, 0) as usize;
        let tbl = (0..cnt)
            .filter(|i| 2 + i * 8 + 8 <= tb.len())
            .map(|i| {
                let a = 2 + i * 8;
                TblEntry {
                    t: u16_at(tb, a),
                    x: u16_at(tb, a + 2) as i16,
                    y: u16_at(tb, a + 4) as i16,
                    what: u16_at(tb, a + 6),
                }
            })
            .collect();
        Level {
            num,
            name,
            goal: u16_at(dsc, 0x2a) as i16,
            score_bonus: u16_at(dsc, 0x2c),
            money: u16_at(dsc, 0x2e),
            sprites,
            sounds,
            foes: Table::parse(f("FOE"), 4),
            exps: Table::parse(f("EXP"), 4),
            tbl,
            stars: dat::decode_sta(f("STA")),
        }
    }
}

fn sli_sprites(data: &[u8]) -> Vec<Spr> {
    dat::decode_sli(data)
        .into_iter()
        .map(|s| {
            let speed = if s.param & 8 != 0 { 1 } else { 2 };
            let xg = s.x_granularity() as i32;
            spr_from_frames(s.frames, speed, xg)
        })
        .collect()
}

//! Settings and high scores, stored in the original CONFIG.HIG layout: six key scan codes,
//! the sound switch, then the high-score table (8 x {score + 123456789, name with +3*i per
//! character}) and the winner text (+2*i per character) — FUN_1000_0825 / 08f2 / 3f36 / 3f9b.

use crate::ctx::{HighScore, Save};
use macroquad::prelude::KeyCode;

#[cfg(feature = "embedded")]
static DEFAULT: &[u8] = include_bytes!("../data/CONFIG.HIG");

/// Without the original CONFIG.HIG: the original's default keys (FUN_1000_0825), sound on and an
/// empty high-score table.
#[cfg(not(feature = "embedded"))]
fn fallback() -> Save {
    Save {
        keys: [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right, KeyCode::Space, KeyCode::P],
        sound: true,
        table: (0..8).map(|_| HighScore { score: 0, name: String::new() }).collect(),
        winner: String::new(),
    }
}

const SCANS: &[(u16, KeyCode)] = &[
    (0x01, KeyCode::Escape), (0x02, KeyCode::Key1), (0x03, KeyCode::Key2), (0x04, KeyCode::Key3),
    (0x05, KeyCode::Key4), (0x06, KeyCode::Key5), (0x07, KeyCode::Key6), (0x08, KeyCode::Key7),
    (0x09, KeyCode::Key8), (0x0a, KeyCode::Key9), (0x0b, KeyCode::Key0), (0x0c, KeyCode::Minus),
    (0x0d, KeyCode::Equal), (0x0e, KeyCode::Backspace), (0x0f, KeyCode::Tab), (0x10, KeyCode::Q),
    (0x11, KeyCode::W), (0x12, KeyCode::E), (0x13, KeyCode::R), (0x14, KeyCode::T), (0x15, KeyCode::Y),
    (0x16, KeyCode::U), (0x17, KeyCode::I), (0x18, KeyCode::O), (0x19, KeyCode::P),
    (0x1a, KeyCode::LeftBracket), (0x1b, KeyCode::RightBracket), (0x1c, KeyCode::Enter),
    (0x1d, KeyCode::LeftControl), (0x1e, KeyCode::A), (0x1f, KeyCode::S), (0x20, KeyCode::D),
    (0x21, KeyCode::F), (0x22, KeyCode::G), (0x23, KeyCode::H), (0x24, KeyCode::J), (0x25, KeyCode::K),
    (0x26, KeyCode::L), (0x27, KeyCode::Semicolon), (0x28, KeyCode::Apostrophe),
    (0x29, KeyCode::GraveAccent), (0x2a, KeyCode::LeftShift), (0x2b, KeyCode::Backslash),
    (0x2c, KeyCode::Z), (0x2d, KeyCode::X), (0x2e, KeyCode::C), (0x2f, KeyCode::V), (0x30, KeyCode::B),
    (0x31, KeyCode::N), (0x32, KeyCode::M), (0x33, KeyCode::Comma), (0x34, KeyCode::Period),
    (0x35, KeyCode::Slash), (0x36, KeyCode::RightShift), (0x37, KeyCode::KpMultiply),
    (0x38, KeyCode::LeftAlt), (0x39, KeyCode::Space), (0x3a, KeyCode::CapsLock),
    (0x3b, KeyCode::F1), (0x3c, KeyCode::F2), (0x3d, KeyCode::F3), (0x3e, KeyCode::F4),
    (0x3f, KeyCode::F5), (0x40, KeyCode::F6), (0x41, KeyCode::F7), (0x42, KeyCode::F8),
    (0x43, KeyCode::F9), (0x44, KeyCode::F10), (0x45, KeyCode::NumLock), (0x46, KeyCode::ScrollLock),
    (0x47, KeyCode::Home), (0x48, KeyCode::Up), (0x49, KeyCode::PageUp), (0x4a, KeyCode::KpSubtract),
    (0x4b, KeyCode::Left), (0x4c, KeyCode::Kp5), (0x4d, KeyCode::Right), (0x4e, KeyCode::KpAdd),
    (0x4f, KeyCode::End), (0x50, KeyCode::Down), (0x51, KeyCode::PageDown), (0x52, KeyCode::Insert),
    (0x53, KeyCode::Delete), (0x57, KeyCode::F11), (0x58, KeyCode::F12),
    (0x1d, KeyCode::RightControl), (0x38, KeyCode::RightAlt),
];

#[allow(dead_code)]
pub fn scan_to_key(s: u16) -> Option<KeyCode> {
    SCANS.iter().find(|e| e.0 == s).map(|e| e.1)
}

pub fn key_to_scan(k: KeyCode) -> u16 {
    SCANS.iter().find(|e| e.1 == k).map(|e| e.0).unwrap_or(0)
}

/// The name the original shows for a key (its key-name table: ESC, SPACE, ENTER, ... else the letter).
pub fn key_name(k: KeyCode) -> String {
    let s = format!("{:?}", k).to_uppercase();
    match k {
        KeyCode::Space => "SPACE".into(),
        KeyCode::Enter => "ENTER".into(),
        KeyCode::Escape => "ESC".into(),
        KeyCode::LeftShift => "LSHFT".into(),
        KeyCode::RightShift => "RSHFT".into(),
        KeyCode::LeftControl | KeyCode::RightControl => "CTRL".into(),
        KeyCode::LeftAlt | KeyCode::RightAlt => "ALT".into(),
        KeyCode::CapsLock => "CPSLCK".into(),
        KeyCode::NumLock => "NUMLCK".into(),
        KeyCode::ScrollLock => "SCRLCK".into(),
        _ => s.trim_start_matches("KEY").chars().take(6).collect(),
    }
}

#[allow(dead_code)]
fn parse(d: &[u8]) -> Option<Save> {
    if d.len() < 14 + 194 {
        return None;
    }
    let w = |p: usize| u16::from_le_bytes([d[p], d[p + 1]]);
    let mut keys = [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right, KeyCode::Space, KeyCode::P];
    for (i, k) in keys.iter_mut().enumerate() {
        if let Some(kc) = scan_to_key(w(i * 2)) {
            *k = kc;
        }
    }
    let sound = w(12) != 0;
    let hs = &d[14..];
    let mut table = vec![];
    for i in 0..8 {
        let r = &hs[i * 20..i * 20 + 20];
        let score = u32::from_le_bytes([r[0], r[1], r[2], r[3]]).wrapping_sub(123456789);
        let name: Vec<u8> = (0..16).map(|j| r[4 + j].wrapping_sub((3 * j) as u8)).collect();
        let end = name.iter().position(|&b| b == 0).unwrap_or(16);
        table.push(HighScore { score, name: String::from_utf8_lossy(&name[..end]).into_owned() });
    }
    let wt: Vec<u8> = (0..34).map(|j| hs[160 + j].wrapping_sub((2 * j) as u8)).collect();
    let end = wt.iter().position(|&b| b == 0).unwrap_or(34);
    Some(Save { keys, sound, table, winner: String::from_utf8_lossy(&wt[..end]).into_owned() })
}

fn encode(s: &Save) -> Vec<u8> {
    let mut d = vec![];
    for k in s.keys {
        d.extend_from_slice(&key_to_scan(k).to_le_bytes());
    }
    d.extend_from_slice(&(s.sound as u16).to_le_bytes());
    for i in 0..8 {
        let (score, name) = s.table.get(i).map(|h| (h.score, h.name.as_bytes().to_vec())).unwrap_or((0, vec![]));
        d.extend_from_slice(&score.wrapping_add(123456789).to_le_bytes());
        for j in 0..16 {
            let c = *name.get(j).unwrap_or(&0);
            d.push(c.wrapping_add((3 * j) as u8));
        }
    }
    let wt = s.winner.as_bytes();
    for j in 0..34 {
        let c = *wt.get(j).unwrap_or(&0);
        d.push(c.wrapping_add((2 * j) as u8));
    }
    d
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn path() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))?;
    Some(base.join("last-eichhof").join("CONFIG.HIG"))
}

pub fn load() -> Save {
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    if let Some(p) = path() {
        if let Ok(d) = std::fs::read(p) {
            if let Some(s) = parse(&d) {
                return s;
            }
        }
    }
    // A CONFIG.HIG the player put beside the program (their own original table).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        if let Some(s) = std::fs::read(dir.join("CONFIG.HIG")).ok().and_then(|d| parse(&d)) {
            return s;
        }
    }
    #[cfg(feature = "embedded")]
    return parse(DEFAULT).expect("default CONFIG.HIG");
    #[cfg(not(feature = "embedded"))]
    fallback()
}

pub fn store(s: &Save) {
    let _d = encode(s);
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    if let Some(p) = path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, _d);
    }
}

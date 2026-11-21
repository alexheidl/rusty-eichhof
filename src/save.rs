//! Settings and high scores, stored in the original CONFIG.HIG layout: six key scan codes,
//! the sound switch, then the high-score table (8 x {score + 123456789, name with +3*i per
//! character}) and the winner text (+2*i per character) — FUN_1000_0825 / 08f2 / 3f36 / 3f9b.

use crate::ctx::{HighScore, Save};
use macroquad::prelude::KeyCode;

/// A new player's settings, as the original starts without a CONFIG.HIG: arrows, Space and P
/// (MENU.C loadconfig), sound on, and the high-score table built into the game (HISCORE.C).
pub fn defaults() -> Save {
    let table = [
        (81000, "ATOMMUELL"),
        (29600, "POWER"),
        (14800, "JUBEL"),
        (8110, "ZYNAX"),
        (8099, "HEAVY"),
        (6367, "WASSERGLAS"),
        (4558, "TWEETY"),
        (1, "TRITONE"),
    ];
    Save {
        keys: [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right, KeyCode::Space, KeyCode::P],
        sound: true,
        table: table.iter().map(|&(score, name)| HighScore { score, name: name.into() }).collect(),
        winner: "EIN PROSIT AUF ALPHA-HELIX".into(),
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

/// MENU.C `ownkeycodes`: for each scan code, an entry of `KEYTABLE` (0..=37) or, above 37, a
/// font character (entry - 36 is the glyph, i.e. character entry - 4).
const OWNKEYCODES: [u8; 89] = [
    0, 0, 53, 54, 55, 56, 57, 58, 59, 60, 61, 52, 49, 47, 3, 4, 85, 91, 73, 86, 88, 93, 89, 77, 83, 84, 95, 97, 7, 8,
    69, 87, 72, 74, 75, 76, 78, 79, 80, 63, 38, 43, 12, 96, 94, 92, 71, 90, 70, 82, 81, 48, 50, 51, 13, 14, 15, 1, 2,
    24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 5, 6, 9, 10, 11, 36, 16, 17, 18, 37, 19, 20, 21, 22, 23, 0, 0, 96, 34, 35,
];

/// MENU.C `keytable`.
const KEYTABLE: [&str; 38] = [
    "ESC", "SPACE", "CPSLCK", "BKSP", "TAB", "NUMLCK", "SCRLCK", "ENTER", "CTRL", "HOME", "UP", "PGUP", "LSHFT",
    "RSHFT", "[*]", "ALT", "LEFT", "[5]", "RIGHT", "END", "DOWN", "PGDN", "INSERT", "DELETE", "F1", "F2", "F3", "F4",
    "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "[-]", "[+]",
];

/// The name the original shows for a key in "Choose keys" (MENU.C newkey).
pub fn key_name(k: KeyCode) -> String {
    let scan = key_to_scan(k) as usize;
    match OWNKEYCODES.get(scan) {
        Some(&e) if scan != 0 && e > 37 => ((e - 4) as char).to_string(),
        Some(&e) if scan != 0 => KEYTABLE[e as usize].into(),
        // A key the 1993 keyboard didn't have.
        _ => format!("{:?}", k).to_uppercase().chars().take(6).collect(),
    }
}

pub fn parse(d: &[u8]) -> Option<Save> {
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

pub fn encode(s: &Save) -> Vec<u8> {
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

/// Where CONFIG.HIG is kept: the user's settings folder on computers, the app's private folder
/// on Android.
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn path() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))?;
    Some(base.join("last-eichhof").join("CONFIG.HIG"))
}

#[cfg(target_os = "android")]
fn path() -> Option<std::path::PathBuf> {
    android_files_dir().map(|d| d.join("CONFIG.HIG"))
}

/// The app's private files folder (Context.getFilesDir()), asked through JNI.
#[cfg(target_os = "android")]
fn android_files_dir() -> Option<std::path::PathBuf> {
    use macroquad::miniquad::native::android::{attach_jni_env, ACTIVITY};
    unsafe {
        let env = attach_jni_env();
        let f = &**env;
        let get_class = f.GetObjectClass?;
        let get_method = f.GetMethodID?;
        let call = f.CallObjectMethod?;
        let chars = f.GetStringUTFChars?;
        let release = f.ReleaseStringUTFChars?;
        let mid = get_method(env, get_class(env, ACTIVITY), c"getFilesDir".as_ptr(), c"()Ljava/io/File;".as_ptr());
        let dir = call(env, ACTIVITY, mid);
        if dir.is_null() {
            return None;
        }
        let mid = get_method(env, get_class(env, dir), c"getAbsolutePath".as_ptr(), c"()Ljava/lang/String;".as_ptr());
        let jstr = call(env, dir, mid);
        if jstr.is_null() {
            return None;
        }
        let p = chars(env, jstr, std::ptr::null_mut());
        let s = std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned();
        release(env, jstr, p);
        Some(std::path::PathBuf::from(s))
    }
}

// Web: the page keeps CONFIG.HIG in the browser's localStorage (functions in web/index.html).
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn eich_config_load(ptr: *mut u8, max: u32) -> u32;
    fn eich_config_store(ptr: *const u8, len: u32);
}

/// Lets the page's storage plugin check it matches this program.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn eich_storage_crate_version() -> u32 {
    1
}

fn read_stored() -> Option<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut buf = vec![0u8; 512];
        let n = unsafe { eich_config_load(buf.as_mut_ptr(), buf.len() as u32) } as usize;
        buf.truncate(n.min(512));
        return if buf.is_empty() { None } else { Some(buf) };
    }
    #[cfg(not(target_arch = "wasm32"))]
    return path().and_then(|p| std::fs::read(p).ok());
}

pub fn load() -> Save {
    if let Some(s) = read_stored().and_then(|d| parse(&d)) {
        return s;
    }
    // A CONFIG.HIG the player put beside the program (their own original table).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        if let Some(s) = std::fs::read(dir.join("CONFIG.HIG")).ok().and_then(|d| parse(&d)) {
            return s;
        }
    }
    defaults()
}

/// Saves settings and high scores. Failing to save (a read-only folder, private browsing) only
/// costs the saved state, so it is reported and otherwise ignored.
pub fn store(s: &Save) {
    let d = encode(s);
    #[cfg(target_arch = "wasm32")]
    unsafe {
        eich_config_store(d.as_ptr(), d.len() as u32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(p) = path() {
        let res = p.parent().map(std::fs::create_dir_all).unwrap_or(Ok(())).and_then(|_| std::fs::write(&p, &d));
        if let Err(e) = res {
            eprintln!("could not save {}: {}", p.display(), e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trip() {
        let mut s = defaults();
        s.keys[4] = KeyCode::LeftControl;
        s.sound = false;
        s.table[3] = HighScore { score: 123456, name: "SCHNAPS".into() };
        let d = encode(&s);
        assert_eq!(d.len(), 208);
        let back = parse(&d).unwrap();
        assert_eq!(back.keys, s.keys);
        assert!(!back.sound);
        assert_eq!(back.table[3].score, 123456);
        assert_eq!(back.table[3].name, "SCHNAPS");
        assert_eq!(back.table[0].name, "ATOMMUELL");
        assert_eq!(back.winner, "EIN PROSIT AUF ALPHA-HELIX");
    }

    #[test]
    fn reads_the_original_config() {
        let s = parse(include_bytes!("../data/CONFIG.HIG")).unwrap();
        assert_eq!(s.table.len(), 8);
        assert_eq!(encode(&s), include_bytes!("../data/CONFIG.HIG").to_vec());
    }

    #[test]
    fn original_key_names() {
        assert_eq!(key_name(KeyCode::Space), "SPACE");
        assert_eq!(key_name(KeyCode::PageUp), "PGUP");
        assert_eq!(key_name(KeyCode::Backspace), "BKSP");
        assert_eq!(key_name(KeyCode::Kp5), "[5]");
        assert_eq!(key_name(KeyCode::KpMultiply), "[*]");
        assert_eq!(key_name(KeyCode::Minus), "-");
        assert_eq!(key_name(KeyCode::A), "A");
        assert_eq!(key_name(KeyCode::Key1), "1");
        assert_eq!(key_name(KeyCode::Up), "UP");
    }
}

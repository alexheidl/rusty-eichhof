//! Decoders for the assets in The Last Eichhof's `BEER.DAT` archive.
//!
//! Std only, no dependencies. Every decoder is defensive: truncated or odd
//! data yields shorter output instead of a panic. See `docs/FORMATS.md` for
//! the byte-level description of every format.

/// 256 RGB colours, 8 bits per channel.
pub type Palette = [[u8; 3]; 256];

// ------------------------------------------------------------------ helpers

fn u16_at(d: &[u8], pos: usize) -> Option<u16> {
    let b = d.get(pos..pos + 2)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
}

fn u32_at(d: &[u8], pos: usize) -> Option<u32> {
    let b = d.get(pos..pos + 4)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Scales a 6-bit VGA DAC value (0..63) to 8 bits (0..255).
pub fn vga6_to_8(v: u8) -> u8 {
    let v = v & 0x3f;
    (v << 2) | (v >> 4)
}

/// Builds a palette from 768 bytes of 6-bit VGA values.
pub fn palette_from_vga6(raw: &[u8]) -> Palette {
    let mut pal = [[0u8; 3]; 256];
    for (i, c) in pal.iter_mut().enumerate() {
        for (k, ch) in c.iter_mut().enumerate() {
            *ch = vga6_to_8(raw.get(i * 3 + k).copied().unwrap_or(0));
        }
    }
    pal
}

/// Reads a table of 16-bit far pointers (`u16 count`, then `count` entries of
/// `stride` bytes, each starting with `offset:u16, segment:u16`). Pointers are
/// relative to the byte after the count (file offset 2), so the absolute
/// position is `2 + segment * 16 + offset`. Entries pointing outside the data
/// are dropped.
pub fn far_ptr_table(data: &[u8], stride: usize) -> Vec<usize> {
    let n = u16_at(data, 0).unwrap_or(0) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let at = 2 + i * stride;
        let (Some(off), Some(seg)) = (u16_at(data, at), u16_at(data, at + 2)) else {
            break;
        };
        let pos = 2 + seg as usize * 16 + off as usize;
        if pos < data.len() {
            out.push(pos);
        }
    }
    out
}

// ------------------------------------------------------------------ archive

/// One directory entry of BEER.DAT.
#[derive(Clone, Debug)]
pub struct Entry {
    /// Upper-case 8.3 name as stored.
    pub name: String,
    pub offset: u32,
    pub size: u32,
    /// Cache hint for the game's EMS/XMS file cache (bit 0: keep a cached
    /// copy, bit 1: only if room is free). Never affects the data itself.
    pub flags: u16,
}

/// The BEER.DAT archive ("ALPHA-HELIX COMBINER VER 3.3").
pub struct Dat {
    bytes: &'static [u8],
    entries: Vec<Entry>,
}

pub const DAT_MAGIC: &[u8] = b"ALPHA-HELIX COMBINER VER 3.3";

impl Dat {
    /// Parses the directory. A file without the magic text or with a broken
    /// directory gives an archive with fewer (or no) entries.
    pub fn parse(bytes: &'static [u8]) -> Dat {
        let mut entries = Vec::new();
        if bytes.starts_with(DAT_MAGIC) {
            let n = u16_at(bytes, 0x20).unwrap_or(0) as usize;
            for i in 0..n {
                let at = 0x22 + i * 24;
                let Some(rec) = bytes.get(at..at + 24) else { break };
                let name_len = rec[..14].iter().position(|&b| b == 0).unwrap_or(14);
                let name = String::from_utf8_lossy(&rec[..name_len]).to_ascii_uppercase();
                let size = u32_at(rec, 14).unwrap_or(0);
                let flags = u16_at(rec, 18).unwrap_or(0);
                let offset = u32_at(rec, 20).unwrap_or(0);
                if (offset as usize).saturating_add(size as usize) <= bytes.len() {
                    entries.push(Entry { name, offset, size, flags });
                }
            }
        }
        Dat { bytes, entries }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The contents of a file, by case-insensitive name.
    pub fn file(&self, name: &str) -> Option<&'static [u8]> {
        let e = self.entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
        let bytes: &'static [u8] = self.bytes;
        bytes.get(e.offset as usize..(e.offset + e.size) as usize)
    }
}

// ------------------------------------------------------------------ palettes

/// File offset of the game's built-in palette in BEER.EXE (DS:0x0F5C, 768
/// bytes of 6-bit values; DS = 0x1A97 starts at file offset 0x200 + 0xA970).
pub const EXE_PALETTE_OFFSET: usize = 0x200 + 0xA970 + 0x0F5C;

/// The built-in palette read from BEER.EXE, if the file is long enough.
pub fn exe_palette(exe: &[u8]) -> Option<Palette> {
    exe.get(EXE_PALETTE_OFFSET..EXE_PALETTE_OFFSET + 768)
        .map(palette_from_vga6)
}

/// The palette active during gameplay (and for all sprites/fonts that have no
/// palette of their own). It is the EXE's built-in palette; MENU.PCX (and
/// most other PCX files) carry the identical palette, so it is taken from
/// there to avoid needing BEER.EXE.
pub fn game_palette(dat: &Dat) -> Palette {
    dat.file("MENU.PCX")
        .map(|d| decode_pcx(d).palette)
        .unwrap_or([[0; 3]; 256])
}

// ------------------------------------------------------------------ PCX

/// An 8-bit PCX picture.
pub struct Pcx {
    pub w: usize,
    pub h: usize,
    pub pixels: Vec<u8>,
    /// 8-bit RGB, computed the way the game does: each byte >> 2 into the
    /// 6-bit DAC, then scaled back up.
    pub palette: Palette,
}

/// Decodes a 320-wide, 8 bits/pixel, 1 plane PCX (version 5) with the
/// 256-colour palette at the end.
pub fn decode_pcx(data: &[u8]) -> Pcx {
    let xmin = u16_at(data, 4).unwrap_or(0) as usize;
    let ymin = u16_at(data, 6).unwrap_or(0) as usize;
    let xmax = u16_at(data, 8).unwrap_or(0) as usize;
    let ymax = u16_at(data, 10).unwrap_or(0) as usize;
    let w = (xmax + 1).saturating_sub(xmin);
    let h = (ymax + 1).saturating_sub(ymin);
    let bpl = match u16_at(data, 66).unwrap_or(0) as usize {
        0 => w,
        n => n.max(w),
    };
    let total = bpl * h;
    let mut raw = Vec::with_capacity(total);
    let mut p = 128;
    while raw.len() < total && p < data.len() {
        let b = data[p];
        p += 1;
        if b & 0xC0 == 0xC0 {
            let v = data.get(p).copied().unwrap_or(0);
            p += 1;
            let n = ((b & 0x3F) as usize).min(total - raw.len());
            raw.extend(std::iter::repeat(v).take(n));
        } else {
            raw.push(b);
        }
    }
    raw.resize(total, 0);
    let mut pixels = Vec::with_capacity(w * h);
    for y in 0..h {
        pixels.extend_from_slice(&raw[y * bpl..y * bpl + w]);
    }
    let mut palette = [[0u8; 3]; 256];
    if data.len() >= 769 && data[data.len() - 769] == 0x0C {
        let pal = &data[data.len() - 768..];
        for (i, c) in palette.iter_mut().enumerate() {
            for (k, ch) in c.iter_mut().enumerate() {
                *ch = vga6_to_8(pal[i * 3 + k] >> 2);
            }
        }
    }
    Pcx { w, h, pixels, palette }
}

// ------------------------------------------------------------------ sprites

/// One sprite frame: row-major palette indices. Index `transparent` (always
/// 0 in this game) is not drawn by the masked sprite routine.
#[derive(Clone, Debug)]
pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub pixels: Vec<u8>,
    pub transparent: u8,
}

impl Frame {
    /// RGBA8 pixels; the transparent index gets alpha 0.
    pub fn rgba(&self, palette: &Palette) -> Vec<u8> {
        rgba(&self.pixels, palette, Some(self.transparent))
    }

    /// RGBA8 pixels with every index opaque (how fonts, BAR.SPR, GO.SPR...
    /// are drawn: the opaque blitter also paints index 0, i.e. black).
    pub fn rgba_opaque(&self, palette: &Palette) -> Vec<u8> {
        rgba(&self.pixels, palette, None)
    }
}

/// Palette indices -> RGBA8. With `Some(t)`, index `t` becomes alpha 0.
pub fn rgba(pixels: &[u8], palette: &Palette, transparent: Option<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixels.len() * 4);
    for &i in pixels {
        let [r, g, b] = palette[i as usize];
        let a = if Some(i) == transparent { 0 } else { 255 };
        out.extend_from_slice(&[r, g, b, a]);
    }
    out
}

/// Decodes a sprite block at `pos`: `u16 w, u16 h, u16 frames`, then
/// `frames * w * h` bytes. Returns the frames and the end position.
pub fn decode_sprite_at(data: &[u8], pos: usize) -> (Vec<Frame>, usize) {
    let (Some(w), Some(h), Some(n)) = (u16_at(data, pos), u16_at(data, pos + 2), u16_at(data, pos + 4))
    else {
        return (Vec::new(), pos);
    };
    let (w, h, n) = (w as usize, h as usize, n as usize);
    let size = w * h;
    let mut frames = Vec::with_capacity(n);
    let mut q = pos + 6;
    for _ in 0..n {
        let Some(px) = data.get(q..q + size) else { break };
        frames.push(Frame { w, h, pixels: px.to_vec(), transparent: 0 });
        q += size;
    }
    (frames, q)
}

/// Decodes a stand-alone .SPR file (fonts, UI pieces). Fonts: frame =
/// character code - 0x20, advance = frame width.
pub fn decode_spr(data: &[u8]) -> Vec<Frame> {
    decode_sprite_at(data, 0).0
}

/// A sprite from a sprite library (.SLI) with its load parameter.
#[derive(Clone, Debug)]
pub struct SpriteSet {
    /// The raw parameter the game passes to its sprite builder.
    pub param: u16,
    pub w: usize,
    pub h: usize,
    pub frames: Vec<Frame>,
}

impl SpriteSet {
    /// Game ticks each animation frame is shown: 2 when bit 3 of `param` is
    /// set (animation counter advances by 1, frame = counter / 2), else 1.
    pub fn ticks_per_frame(&self) -> u32 {
        if self.param & 8 != 0 {
            2
        } else {
            1
        }
    }

    /// Horizontal position granularity the original renderer had: with
    /// `param & 7 == 4` the sprite only exists unshifted, so x is rounded
    /// down to a multiple of 4; with `param & 7 == 2` a copy shifted by 2
    /// pixels exists too, so x is rounded down to a multiple of 2.
    pub fn x_granularity(&self) -> u32 {
        if self.param & 7 >= 4 {
            4
        } else {
            2
        }
    }
}

/// Decodes a sprite library (LEVELn.SLI, WEAPONS.SLI): `u16 count`, then
/// `count` entries `{offset:u16, segment:u16, param:u16}` whose far pointer
/// (relative to file offset 2) addresses a sprite block.
pub fn decode_sli(data: &[u8]) -> Vec<SpriteSet> {
    let n = u16_at(data, 0).unwrap_or(0) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let at = 2 + i * 6;
        let (Some(off), Some(seg), Some(param)) =
            (u16_at(data, at), u16_at(data, at + 2), u16_at(data, at + 4))
        else {
            break;
        };
        let pos = 2 + seg as usize * 16 + off as usize;
        let w = u16_at(data, pos).unwrap_or(0) as usize;
        let h = u16_at(data, pos + 2).unwrap_or(0) as usize;
        let (frames, _) = decode_sprite_at(data, pos);
        out.push(SpriteSet { param, w, h, frames });
    }
    out
}

// ------------------------------------------------------------------ sound

/// A decoded sound: unsigned 8-bit mono PCM.
#[derive(Clone, Debug)]
pub struct Sound {
    pub rate: u32,
    pub samples: Vec<u8>,
    /// Playback priority: a sound starts only if its priority is >= that of
    /// the sound currently playing (one SoundBlaster voice, no mixing).
    pub priority: u16,
    /// The data was Creative 4-bit ADPCM (decoded here to 8-bit PCM).
    pub adpcm: bool,
}

/// Playback rate the game programs for a header rate field `khz`: DSP time
/// constant `256 - 1000 / khz`, i.e. `1_000_000 / (1000 / khz)` Hz
/// (15 -> 15151, 12 -> 12048, 10 -> 10000, 9 -> 9009).
pub fn snd_rate(khz: u16) -> u32 {
    let k = khz.max(4) as u32; // the DSP time constant must stay in a byte
    1_000_000 / (1000 / k)
}

/// Decodes one sound at `pos`: `u16 priority, u16 khz, u16 flags,
/// u32 length`, then `length` bytes of data (8-bit unsigned PCM, or Creative
/// 4-bit ADPCM with a reference byte when `flags & 1`).
pub fn decode_snd_at(data: &[u8], pos: usize) -> Sound {
    let priority = u16_at(data, pos).unwrap_or(0);
    let khz = u16_at(data, pos + 2).unwrap_or(15);
    let flags = u16_at(data, pos + 4).unwrap_or(0);
    let len = u32_at(data, pos + 6).unwrap_or(0) as usize;
    let start = (pos + 10).min(data.len());
    let end = start.saturating_add(len).min(data.len());
    let body = &data[start..end];
    let adpcm = flags & 1 != 0;
    let samples = if adpcm { decode_adpcm4(body) } else { body.to_vec() };
    Sound { rate: snd_rate(khz), samples, priority, adpcm }
}

/// Decodes a stand-alone .SND file (MENU.SND, TITLE.SND, ...).
pub fn decode_snd(data: &[u8]) -> Sound {
    decode_snd_at(data, 0)
}

/// Decodes a level sound bank (LEVELn.SND): a far-pointer table (4-byte
/// entries) of embedded sounds. Scripts refer to them by index.
pub fn decode_snd_bank(data: &[u8]) -> Vec<Sound> {
    far_ptr_table(data, 4)
        .into_iter()
        .map(|p| decode_snd_at(data, p))
        .collect()
}

/// Creative Labs 4-bit ADPCM (SoundBlaster DSP command 0x75/0x74): the first
/// byte is the initial sample, then each byte gives two samples, high nibble
/// first.
pub fn decode_adpcm4(data: &[u8]) -> Vec<u8> {
    const STEP: [i8; 64] = [
        0, 1, 2, 3, 4, 5, 6, 7, 0, -1, -2, -3, -4, -5, -6, -7, //
        1, 3, 5, 7, 9, 11, 13, 15, -1, -3, -5, -7, -9, -11, -13, -15, //
        2, 6, 10, 14, 18, 22, 26, 30, -2, -6, -10, -14, -18, -22, -26, -30, //
        4, 12, 20, 28, 36, 44, 52, 60, -4, -12, -20, -28, -36, -44, -52, -60,
    ];
    const ADJUST: [u8; 64] = [
        0, 0, 0, 0, 0, 16, 16, 16, 0, 0, 0, 0, 0, 16, 16, 16, //
        240, 0, 0, 0, 0, 16, 16, 16, 240, 0, 0, 0, 0, 16, 16, 16, //
        240, 0, 0, 0, 0, 16, 16, 16, 240, 0, 0, 0, 0, 16, 16, 16, //
        240, 0, 0, 0, 0, 0, 0, 0, 240, 0, 0, 0, 0, 0, 0, 0,
    ];
    let Some((&first, rest)) = data.split_first() else {
        return Vec::new();
    };
    let mut sample = first as i32;
    let mut scale: u8 = 0;
    let mut out = Vec::with_capacity(rest.len() * 2);
    for &b in rest {
        for nib in [b >> 4, b & 15] {
            let idx = (nib as usize + scale as usize).min(63);
            sample = (sample + STEP[idx] as i32).clamp(0, 255);
            scale = scale.wrapping_add(ADJUST[idx]);
            out.push(sample as u8);
        }
    }
    out
}

/// A complete 8-bit mono PCM WAV file in memory.
pub fn wav_bytes(s: &Sound) -> Vec<u8> {
    let n = s.samples.len() as u32;
    let pad = (n & 1) as usize;
    let mut v = Vec::with_capacity(44 + n as usize + pad);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + n + pad as u32).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes()); // PCM
    v.extend_from_slice(&1u16.to_le_bytes()); // mono
    v.extend_from_slice(&s.rate.to_le_bytes());
    v.extend_from_slice(&s.rate.to_le_bytes()); // byte rate
    v.extend_from_slice(&1u16.to_le_bytes()); // block align
    v.extend_from_slice(&8u16.to_le_bytes()); // bits
    v.extend_from_slice(b"data");
    v.extend_from_slice(&n.to_le_bytes());
    v.extend_from_slice(&s.samples);
    if pad == 1 {
        v.push(0x80);
    }
    v
}

// ------------------------------------------------------------------ level extras

/// One background star from LEVELn.STA.
#[derive(Clone, Copy, Debug)]
pub struct Star {
    /// The game draws stars in plane 0 only, so the visible x is `x & !3`.
    pub x: i16,
    pub y: i16,
    pub colour: u8,
    /// Rows moved down per frame; wraps at the bottom of the play field.
    pub speed: i16,
}

/// LEVELn.STA: `u16 count`, then `count` x `{x, y, colour, speed}` (i16 each).
pub fn decode_sta(data: &[u8]) -> Vec<Star> {
    let n = u16_at(data, 0).unwrap_or(0) as usize;
    (0..n)
        .map_while(|i| {
            let at = 2 + i * 8;
            Some(Star {
                x: u16_at(data, at)? as i16,
                y: u16_at(data, at + 2)? as i16,
                colour: u16_at(data, at + 4)? as u8,
                speed: u16_at(data, at + 6)? as i16,
            })
        })
        .collect()
}

/// LEVELn.DSC: level number and name (shown on the "GO" screen).
pub fn decode_dsc(data: &[u8]) -> (u16, String) {
    let num = u16_at(data, 0).unwrap_or(0);
    let name = data.get(2..).unwrap_or(&[]);
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    (num, String::from_utf8_lossy(&name[..end]).into_owned())
}

// ------------------------------------------------------------------ intro

/// BLICK.PAK: the 640x350 16-colour newspaper picture of the intro (EGA mode
/// 0x10). Returns a frame of indices 0..15 (use the first 16 entries of the
/// game palette).
pub fn decode_pak(data: &[u8]) -> Frame {
    const PLANE: usize = 640 * 350 / 8;
    let mut pixels = vec![0u8; 640 * 350];
    let mut p = 0usize;
    for plane in 0..4 {
        let mut buf = Vec::with_capacity(PLANE + 128);
        while buf.len() < PLANE && p < data.len() {
            let c = data[p] as usize;
            p += 1;
            if c < 0x80 {
                let v = data.get(p).copied().unwrap_or(0);
                p += 1;
                buf.extend(std::iter::repeat(v).take(c + 1));
            } else {
                let n = c - 0x7F;
                let end = (p + n).min(data.len());
                buf.extend_from_slice(&data[p..end]);
                p = end;
            }
        }
        buf.resize(PLANE, 0);
        for (i, &b) in buf.iter().take(PLANE).enumerate() {
            for bit in 0..8 {
                if b & (0x80 >> bit) != 0 {
                    pixels[i * 8 + bit] |= 1 << plane;
                }
            }
        }
    }
    Frame { w: 640, h: 350, pixels, transparent: 0 }
}

/// PAPER.FNT: the intro's 1-bit scroller font, 40x40 glyphs for characters
/// 0x20.. (glyph = char - 0x20). Set pixels are index 15, clear ones 0.
pub fn decode_paper_font(data: &[u8]) -> Vec<Frame> {
    data.chunks_exact(200)
        .map(|g| {
            let mut pixels = vec![0u8; 40 * 40];
            for col in 0..5 {
                for row in 0..40 {
                    let b = g[col * 40 + row];
                    for bit in 0..8 {
                        if b & (0x80 >> bit) != 0 {
                            pixels[row * 40 + col * 8 + bit] = 15;
                        }
                    }
                }
            }
            Frame { w: 40, h: 40, pixels, transparent: 0 }
        })
        .collect()
}

// ------------------------------------------------------------------ tests

#[cfg(all(test, feature = "embedded"))]
mod tests {
    use super::*;

    static DATA: &[u8] = include_bytes!("../data/BEER.DAT");

    #[test]
    fn decodes_whole_archive() {
        let dat = Dat::parse(DATA);
        assert_eq!(dat.entries().len(), 78);
        assert!(dat.file("font1.spr").is_some());
        let pal = game_palette(&dat);
        assert_ne!(pal, [[0; 3]; 256]);
        for e in dat.entries() {
            let d = dat.file(&e.name).unwrap();
            let ext = e.name.rsplit('.').next().unwrap();
            match ext {
                "PCX" => {
                    let p = decode_pcx(d);
                    assert_eq!(p.w, 320, "{}", e.name);
                    assert!(p.h >= 200, "{}", e.name);
                    assert_eq!(p.pixels.len(), p.w * p.h);
                }
                "SPR" => {
                    let f = decode_spr(d);
                    assert!(!f.is_empty(), "{}", e.name);
                    assert_eq!(f[0].rgba(&pal).len(), f[0].w * f[0].h * 4);
                }
                "SLI" => {
                    let sets = decode_sli(d);
                    assert!(!sets.is_empty(), "{}", e.name);
                    for s in &sets {
                        assert!(!s.frames.is_empty(), "{}", e.name);
                        assert!(s.ticks_per_frame() >= 1 && s.x_granularity() >= 2);
                    }
                }
                "SND" if e.name.starts_with("LEVEL") => {
                    let bank = decode_snd_bank(d);
                    assert!(!bank.is_empty(), "{}", e.name);
                    for s in &bank {
                        assert!(s.rate > 8000 && !s.samples.is_empty(), "{}", e.name);
                    }
                }
                "SND" => {
                    let s = decode_snd(d);
                    assert!(s.rate > 8000 && !s.samples.is_empty(), "{}", e.name);
                    let w = wav_bytes(&s);
                    assert_eq!(&w[..4], b"RIFF");
                    assert_eq!(u32_at(&w, 4).unwrap() as usize + 8, w.len());
                }
                "STA" => assert!(!decode_sta(d).is_empty()),
                _ => {}
            }
        }
        assert_eq!(decode_pak(dat.file("BLICK.PAK").unwrap()).pixels.len(), 640 * 350);
        assert_eq!(decode_paper_font(dat.file("PAPER.FNT").unwrap()).len(), 59);
        assert_eq!(decode_dsc(dat.file("LEVEL1.DSC").unwrap()).1, "FELDSCHLOESSCHEN");
        let title = decode_snd(dat.file("TITLE.SND").unwrap());
        assert_eq!(title.rate, 15151);
        assert_eq!(decode_snd(dat.file("LONGTIME.SND").unwrap()).rate, 12048);
    }
}

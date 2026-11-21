//! The 320x220 256-colour screen of the original (VGA Mode X), kept as palette indices and
//! turned into a texture once per displayed frame.

use macroquad::prelude::*;

pub const W: usize = 320;
pub const H: usize = 220;

/// A sprite file: frames of `w` x `h` palette indices; index 0 is see-through when drawn as a
/// sprite (the original's sprite blitter) and black when drawn as a tile (its font blitter).
#[derive(Clone, Debug, Default)]
pub struct Spr {
    pub w: usize,
    pub h: usize,
    /// Shared, so copying a sprite set (the screens and the status bar do) costs nothing.
    pub frames: std::rc::Rc<[Vec<u8>]>,
    /// Animation step per tick: 2 = a new frame every tick, 1 = every second tick.
    pub speed: u16,
    /// Horizontal placement step of the original's pre-shifted sprite copies (2 or 4 px).
    pub xgran: i32,
}

impl Spr {
    pub fn count(&self) -> usize {
        self.frames.len()
    }
}

pub struct Screen {
    pub w: usize,
    pub h: usize,
    pub pix: Vec<u8>,
    /// Current palette, 6-bit VGA values (0..63) like the original.
    pub pal: [[u8; 3]; 256],
    /// Subtracted from every palette component (the original fades this way).
    pub fade: u8,
    /// When set, every colour shows as this grey level instead (the intro's flash to white).
    pub white: Option<u8>,
    /// Clip rectangle (inclusive left/top, exclusive right/bottom for our blits).
    pub clip: (i32, i32, i32, i32),
    tex: Texture2D,
    rgba: Vec<u8>,
    /// What the texture currently shows, to skip uploads when nothing changed.
    shown_pix: Vec<u8>,
    shown_look: Option<([[u8; 3]; 256], u8, Option<u8>)>,
    /// The picture enlarged by whole pixels, then smoothly fitted to the window ("sharp
    /// bilinear"): every original pixel stays crisp and equally wide at any window size.
    big: Option<(RenderTarget, u32, u32)>,
}

impl Screen {
    pub fn new() -> Self {
        let tex = Texture2D::from_rgba8(W as u16, H as u16, &vec![0u8; W * H * 4]);
        tex.set_filter(FilterMode::Nearest);
        Screen {
            w: W,
            h: H,
            pix: vec![0; W * H],
            pal: [[0; 3]; 256],
            fade: 0,
            white: None,
            clip: (0, 0, W as i32, H as i32),
            tex,
            rgba: vec![0; W * H * 4],
            shown_pix: vec![],
            shown_look: None,
            big: None,
        }
    }

    /// Switches the screen size (320x220 for the game, 640x350 for the newspaper intro).
    pub fn set_size(&mut self, w: usize, h: usize) {
        if w == self.w && h == self.h {
            return;
        }
        self.w = w;
        self.h = h;
        self.pix = vec![0; w * h];
        self.rgba = vec![0; w * h * 4];
        self.tex = Texture2D::from_rgba8(w as u16, h as u16, &self.rgba);
        self.tex.set_filter(FilterMode::Nearest);
        self.clip = (0, 0, w as i32, h as i32);
        self.shown_look = None;
    }

    pub fn clear(&mut self, c: u8) {
        self.pix.iter_mut().for_each(|p| *p = c);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        for yy in y.max(0)..(y + h).min(self.h as i32) {
            for xx in x.max(0)..(x + w).min(self.w as i32) {
                self.pix[yy as usize * self.w + xx as usize] = c;
            }
        }
    }

    pub fn set_pal6(&mut self, pal: &[[u8; 3]; 256]) {
        self.pal = *pal;
    }

    pub fn reset_clip(&mut self) {
        self.clip = (0, 0, self.w as i32, self.h as i32);
    }

    /// Sprite blit: index 0 is transparent; clipped to the clip rectangle.
    pub fn blit(&mut self, s: &Spr, frame: usize, x: i32, y: i32) {
        if s.frames.is_empty() {
            return;
        }
        let f = &s.frames[frame % s.frames.len()];
        let (cl, ct, cr, cb) = self.clip;
        for sy in 0..s.h as i32 {
            let py = y + sy;
            if py < ct || py >= cb {
                continue;
            }
            for sx in 0..s.w as i32 {
                let px = x + sx;
                if px < cl || px >= cr {
                    continue;
                }
                let c = f[(sy as usize) * s.w + sx as usize];
                if c != 0 {
                    self.pix[py as usize * self.w + px as usize] = c;
                }
            }
        }
    }

    /// Solid silhouette in one colour (the hit flash).
    pub fn blit_flat(&mut self, s: &Spr, frame: usize, x: i32, y: i32, col: u8) {
        if s.frames.is_empty() {
            return;
        }
        let f = &s.frames[frame % s.frames.len()];
        let (cl, ct, cr, cb) = self.clip;
        for sy in 0..s.h as i32 {
            let py = y + sy;
            if py < ct || py >= cb {
                continue;
            }
            for sx in 0..s.w as i32 {
                let px = x + sx;
                if px < cl || px >= cr {
                    continue;
                }
                if f[(sy as usize) * s.w + sx as usize] != 0 {
                    self.pix[py as usize * self.w + px as usize] = col;
                }
            }
        }
    }

    /// Opaque blit (the original's font/tile routine writes every pixel, 0 included).
    pub fn blit_opaque(&mut self, s: &Spr, frame: usize, x: i32, y: i32) {
        if s.frames.is_empty() {
            return;
        }
        let f = &s.frames[frame % s.frames.len()];
        for sy in 0..s.h as i32 {
            let py = y + sy;
            if py < 0 || py >= self.h as i32 {
                continue;
            }
            for sx in 0..s.w as i32 {
                let px = x + sx;
                if px < 0 || px >= self.w as i32 {
                    continue;
                }
                self.pix[py as usize * self.w + px as usize] = f[(sy as usize) * s.w + sx as usize];
            }
        }
    }

    /// Draws text with a font sprite whose frame n is character 0x20+n (opaque, like the original).
    pub fn text(&mut self, font: &Spr, x: i32, y: i32, s: &str) {
        let mut x = x;
        for ch in s.bytes() {
            let idx = ch.saturating_sub(0x20) as usize;
            if idx < font.count() {
                self.blit_opaque(font, idx, x, y);
            }
            x += font.w as i32;
        }
    }

    #[allow(dead_code)]
    pub fn text_width(font: &Spr, s: &str) -> i32 {
        font.w as i32 * s.len() as i32
    }

    /// Draws a full indexed picture at a vertical offset.
    pub fn picture(&mut self, pixels: &[u8], pw: usize, ph: usize, yoff: i32, scroll: i32) {
        for y in 0..self.h as i32 {
            let sy = y - yoff + scroll;
            if sy < 0 || sy >= ph as i32 {
                continue;
            }
            let src = &pixels[sy as usize * pw..sy as usize * pw + pw.min(self.w)];
            self.pix[y as usize * self.w..y as usize * self.w + src.len()].copy_from_slice(src);
        }
    }

    /// Uploads the indexed picture through the palette (only when the picture or the palette
    /// changed since the last upload).
    pub fn upload(&mut self) {
        let look = (self.pal, self.fade, self.white);
        if self.shown_look == Some(look) && self.shown_pix == self.pix {
            return;
        }
        self.shown_look = Some(look);
        self.shown_pix.clone_from(&self.pix);
        let mut lut = [[0u8; 4]; 256];
        for (i, c) in self.pal.iter().enumerate() {
            let f = |v: u8| -> u8 {
                let v = v.saturating_sub(self.fade).min(63) as u32;
                ((v * 255 + 31) / 63) as u8
            };
            lut[i] = match self.white {
                Some(v) => {
                    let g = ((v.min(63) as u32 * 255 + 31) / 63) as u8;
                    [g, g, g, 255]
                }
                None => [f(c[0]), f(c[1]), f(c[2]), 255],
            };
        }
        for (i, &p) in self.pix.iter().enumerate() {
            self.rgba[i * 4..i * 4 + 4].copy_from_slice(&lut[p as usize]);
        }
        self.tex.update_from_bytes(self.w as u32, self.h as u32, &self.rgba);
    }

    /// The largest 320:220-ish area (pixels shown 1:1.2 tall like a 4:3 CRT) inside the window.
    pub fn view_rect(&self) -> Rect {
        let sw = screen_width();
        let sh = screen_height();
        // 320x220 Mode X uses the square-pixel 240-line timing; the 640x350 EGA intro filled a 4:3 screen.
        let aspect = if self.w == 640 { 4.0 / 3.0 } else { self.w as f32 / self.h as f32 };
        let (w, h) = if sw / sh > aspect { (sh * aspect, sh) } else { (sw, sw / aspect) };
        Rect::new((sw - w) / 2.0, (sh - h) / 2.0, w, h)
    }

    pub fn present(&mut self) {
        self.upload();
        clear_background(BLACK);
        let mut r = self.view_rect();
        // Work in real screen pixels (HiDPI screens have more than the window's logical size).
        let dpi = screen_dpi_scale();
        r.x = (r.x * dpi).round() / dpi;
        r.y = (r.y * dpi).round() / dpi;
        let (pw, ph) = (r.w * dpi, r.h * dpi);
        let kx = ((pw / self.w as f32).floor() as u32).max(1);
        let ky = ((ph / self.h as f32).floor() as u32).max(1);
        let whole = (kx as f32 * self.w as f32 - pw).abs() < 0.5 && (ky as f32 * self.h as f32 - ph).abs() < 0.5;
        if whole {
            draw_texture_ex(&self.tex, r.x, r.y, WHITE, DrawTextureParams { dest_size: Some(vec2(r.w, r.h)), ..Default::default() });
            return;
        }
        let (tw, th) = (self.w as u32 * kx, self.h as u32 * ky);
        if self.big.as_ref().map(|b| (b.1, b.2)) != Some((tw, th)) {
            let rt = render_target(tw, th);
            rt.texture.set_filter(FilterMode::Linear);
            self.big = Some((rt, tw, th));
        }
        let rt = self.big.as_ref().map(|b| b.0.clone()).unwrap();
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, tw as f32, th as f32));
        cam.render_target = Some(rt.clone());
        set_camera(&cam);
        clear_background(BLACK);
        draw_texture_ex(&self.tex, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(tw as f32, th as f32)), ..Default::default() });
        set_default_camera();
        // Render targets come out upside down in macroquad's default camera.
        draw_texture_ex(&rt.texture, r.x, r.y, WHITE, DrawTextureParams { dest_size: Some(vec2(r.w, r.h)), flip_y: true, ..Default::default() });
    }
}

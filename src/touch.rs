//! Touch controls for phones, tablets and touch screens (the original only had a keyboard).
//!
//! * Right half: drag to steer. The finger's movement is turned into arrow-key steps, so the
//!   bottle follows the finger 1:1 at its normal speed (relative steering: the finger never
//!   covers the bottle). In menus a swipe of `step` pixels moves the cursor one place.
//! * Left half: touch and hold to fire (like holding Space). In menus a tap selects.
//! * A quick tap anywhere also counts as Fire (select).
//! * Top-left corner: back (Esc). Top-right corner: pause (P).

use crate::ctx::G;
use macroquad::prelude::*;

#[derive(Default)]
pub struct Touch {
    /// Whether this device has been touched at all (the overlay is only drawn then).
    pub seen: bool,
    steer_id: Option<u64>,
    last: Vec2,
    acc: Vec2,
    fire_held: bool,
    tap_pulse: u8,
    back_pulse: u8,
    pause_pulse: u8,
    starts: Vec<(u64, Vec2, f64)>,
    pending: Vec<G>,
    /// Screen pixels of the 320x220 game per arrow step (the bottle's speed in game).
    pub step: f32,
    pub view: Rect,
}

impl Touch {
    fn to_game(&self, p: Vec2) -> Vec2 {
        let r = self.view;
        if r.w <= 0.0 {
            return p;
        }
        vec2((p.x - r.x) * 320.0 / r.w, (p.y - r.y) * 220.0 / r.h)
    }

    pub fn update(&mut self) {
        let ts = touches();
        self.view = {
            let sw = screen_width();
            let sh = screen_height();
            let aspect = 320.0 / 220.0;
            let (w, h) = if sw / sh > aspect { (sh * aspect, sh) } else { (sw, sw / aspect) };
            Rect::new((sw - w) / 2.0, (sh - h) / 2.0, w, h)
        };
        if self.step <= 0.0 {
            self.step = 12.0;
        }
        self.fire_held = false;
        let half = screen_width() / 2.0;
        let corner = (screen_width().min(screen_height()) * 0.12).max(48.0);
        let now = get_time();
        let mut steer_alive = false;
        for t in &ts {
            self.seen = true;
            match t.phase {
                TouchPhase::Started => {
                    if t.position.y < corner && t.position.x < corner {
                        self.back_pulse = 2;
                        continue;
                    }
                    if t.position.y < corner && t.position.x > screen_width() - corner {
                        self.pause_pulse = 2;
                        continue;
                    }
                    self.starts.push((t.id, t.position, now));
                    if t.position.x >= half && self.steer_id.is_none() {
                        self.steer_id = Some(t.id);
                        self.last = self.to_game(t.position);
                    }
                    if t.position.x < half {
                        self.fire_held = true;
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if Some(t.id) == self.steer_id {
                        steer_alive = true;
                        let p = self.to_game(t.position);
                        self.acc += p - self.last;
                        self.last = p;
                    } else if t.position.x < half {
                        self.fire_held = true;
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if let Some(i) = self.starts.iter().position(|s| s.0 == t.id) {
                        let (_, p0, t0) = self.starts.remove(i);
                        if (t.position - p0).length() < 12.0 && now - t0 < 0.35 {
                            self.tap_pulse = 2;
                        }
                    }
                    if Some(t.id) == self.steer_id {
                        self.steer_id = None;
                        self.acc = Vec2::ZERO;
                    }
                }
            }
            if Some(t.id) == self.steer_id && t.phase == TouchPhase::Started {
                steer_alive = true;
            }
        }
        if !steer_alive && self.steer_id.is_some() && ts.iter().all(|t| Some(t.id) != self.steer_id) {
            self.steer_id = None;
            self.acc = Vec2::ZERO;
        }
    }

    /// The keys the touches hold for this game step. Called once per tick.
    pub fn held(&mut self) -> Vec<G> {
        let mut v = std::mem::take(&mut self.pending);
        let s = self.step.max(1.0);
        if self.acc.x >= s * 0.5 {
            v.push(G::Right);
            self.acc.x = (self.acc.x - s).max(-s);
        } else if self.acc.x <= -s * 0.5 {
            v.push(G::Left);
            self.acc.x = (self.acc.x + s).min(s);
        }
        if self.acc.y >= s * 0.5 {
            v.push(G::Down);
            self.acc.y = (self.acc.y - s).max(-s);
        } else if self.acc.y <= -s * 0.5 {
            v.push(G::Up);
            self.acc.y = (self.acc.y + s).min(s);
        }
        if self.fire_held || self.tap_pulse > 0 {
            v.push(G::Fire);
        }
        if self.tap_pulse > 0 {
            v.push(G::Enter);
            self.tap_pulse -= 1;
        }
        if self.back_pulse > 0 {
            v.push(G::Esc);
            self.back_pulse -= 1;
        }
        if self.pause_pulse > 0 {
            v.push(G::Pause);
            self.pause_pulse -= 1;
        }
        v
    }

    /// Faint hints for the touch areas, only once the device has been touched.
    pub fn draw(&self) {
        if !self.seen {
            return;
        }
        let corner = (screen_width().min(screen_height()) * 0.12).max(48.0);
        let c = Color::new(1.0, 1.0, 1.0, 0.18);
        draw_rectangle_lines(4.0, 4.0, corner - 8.0, corner - 8.0, 2.0, c);
        draw_line(16.0, 16.0, corner - 16.0, corner - 16.0, 2.0, c);
        draw_line(corner - 16.0, 16.0, 16.0, corner - 16.0, 2.0, c);
        let x0 = screen_width() - corner;
        draw_rectangle_lines(x0 + 4.0, 4.0, corner - 8.0, corner - 8.0, 2.0, c);
        draw_rectangle(x0 + corner * 0.35, 14.0, corner * 0.1, corner - 28.0, c);
        draw_rectangle(x0 + corner * 0.55, 14.0, corner * 0.1, corner - 28.0, c);
        let h = screen_height();
        draw_text("HOLD: FIRE", 12.0, h - 14.0, 22.0, c);
        let tw = measure_text("DRAG: MOVE", None, 22, 1.0).width;
        draw_text("DRAG: MOVE", screen_width() - tw - 12.0, h - 14.0, 22.0, c);
    }

    /// Queues a key for the next tick (used by on-screen buttons).
    #[allow(dead_code)]
    pub fn press(&mut self, g: G) {
        self.pending.push(g);
    }
}

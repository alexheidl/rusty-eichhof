//! The Last Eichhof (Alpha-Helix, 1993) for today's computers, phones and browsers.
//! Plays from the original BEER.DAT; rules re-implemented from the original program
//! (see MECHANICS.md).

#[allow(dead_code)]
mod dat;
mod ctx;
mod game;
mod gfx;
mod res;
mod save;
mod screens;
mod selftest;
mod touch;

use ctx::{Ctx, Mode};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    let shot = std::env::args().any(|a| a == "--screenshot" || a == "--self-test");
    Conf {
        window_title: "The Last Eichhof".to_owned(),
        window_width: 960,
        window_height: 660,
        high_dpi: true,
        window_resizable: true,
        // Checks run unattended; keep the window plain and small then.
        fullscreen: false,
        sample_count: 1,
        platform: miniquad::conf::Platform {
            // Checks must keep drawing even when the window is covered: don't wait for the
            // screen refresh then.
            swap_interval: if shot { Some(0) } else { None },
            ..Default::default()
        },
        ..Default::default()
    }
}

fn arg_after(flag: &str) -> Option<String> {
    let a: Vec<String> = std::env::args().collect();
    a.iter().position(|x| x == flag).and_then(|i| a.get(i + 1).cloned())
}

#[macroquad::main(window_conf)]
async fn main() {
    let started = std::time::Instant::now();
    let shot = arg_after("--screenshot");
    if std::env::args().any(|a| a == "--self-test" || a == "--screenshot") {
        eprintln!("window up after {:?}", started.elapsed());
    }
    let test = arg_after("--self-test");
    let mode = if test.is_some() {
        Mode::SelfTest
    } else if shot.is_some() {
        Mode::Screenshot
    } else {
        Mode::Normal
    };
    let t0 = std::time::Instant::now();
    let Some(data) = res::beer_dat().await else {
        missing_data().await;
        return;
    };
    let res = res::Res::load(data);
    if mode != Mode::Normal {
        eprintln!("resources loaded in {:?}", t0.elapsed());
    }
    let mut ctx = Ctx::new(res, mode);
    // The original's cheat: an argument like "007.5" (bits: 1 lives, 2 money, 4 invulnerable).
    for a in std::env::args() {
        if let Some(i) = a.find("007.") {
            ctx.cheat = a.as_bytes().get(i + 4).map(|b| b.wrapping_sub(b'0') as u16).unwrap_or(0) & 7;
        }
    }

    if let Some(path) = test {
        selftest::run(&mut ctx, &path).await;
        return;
    }
    if let Some(path) = shot {
        ctx.on_frame = Some(Box::new(move |c: &mut Ctx| {
            if c.frames == c.shot_frame {
                // Draw this frame, then read it back.
                c.scr.upload();
                c.scr.present();
                let img = get_screen_data();
                let (w, h) = (img.width as usize, img.height as usize);
                // macroquad already returns the rows top to bottom.
                let mut flipped = img.bytes.clone();
                for px in flipped.chunks_mut(4) {
                    px[3] = 255;
                }
                let out = Image { bytes: flipped, width: w as u16, height: h as u16 };
                if let Some(dir) = std::path::Path::new(&path).parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                out.export_png(&path);
                std::process::exit(0);
            }
        }));
    }

    if let Some(scene) = arg_after("--scene") {
        screens::scene(&mut ctx, &scene).await;
        return;
    }
    screens::newspaper(&mut ctx).await;
    while !ctx.quit {
        screens::title(&mut ctx).await;
        screens::main_menu(&mut ctx).await;
    }
}

/// Shown by release builds when the player's own BEER.DAT isn't beside the program.
async fn missing_data() {
    let checking = std::env::args().any(|a| a == "--screenshot" || a == "--self-test");
    let mut frames = 0;
    loop {
        frames += 1;
        if checking && frames > 120 {
            return;
        }
        clear_background(BLACK);
        let lines = [
            "THE LAST EICHHOF",
            "",
            "BEER.DAT was not found.",
            "Copy BEER.DAT from your own copy of the original game",
            "into the same folder as this program, then start it again.",
            "",
            "Press Esc to close.",
        ];
        for (i, l) in lines.iter().enumerate() {
            draw_text(l, 30.0, 60.0 + i as f32 * 34.0, 30.0, if i == 0 { YELLOW } else { WHITE });
        }
        if is_key_pressed(KeyCode::Escape) {
            return;
        }
        next_frame().await;
    }
}

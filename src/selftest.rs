//! `--self-test <file.json>`: starts a game like a player (new game, level 1), steers and fires
//! with scripted keys, and reports from the game's own state whether moving, shooting and the
//! status bar work.

use crate::ctx::{Ctx, G};
use crate::game::{Field, Hero, Outcome};

fn keys(t: u64) -> Vec<G> {
    let mut v = vec![G::Fire];
    match (t / 30) % 4 {
        0 => v.push(G::Left),
        1 => v.push(G::Right),
        2 => v.push(G::Right),
        _ => v.push(G::Left),
    }
    v
}

pub async fn run(ctx: &mut Ctx, path: &str) {
    let t0 = std::time::Instant::now();
    eprintln!("selftest: start");
    ctx.script = Some(keys);
    // New game, like choosing "1 play!" in the menu (FUN_1000_1fa3 / 14d8), invulnerable so
    // the measurement isn't cut short by a collision.
    let mut hero = Hero::new(&ctx.res.weapons, 4);
    let level = ctx.res.level(0).await;
    eprintln!("selftest: level loaded {:?}", t0.elapsed());
    let level_name = level.name.clone();
    let mut field = Field::new(ctx, level, &mut hero);
    let start_score = hero.score;
    let outcome = field.play_for(ctx, &mut hero, 700).await;
    eprintln!("selftest: played {:?}", t0.elapsed());
    let st = field.stats.clone();
    // HUD: the status bar row and score digits are on the screen.
    let w = ctx.scr.w;
    let bar_px = ctx.scr.pix[211 * w..220 * w].iter().filter(|&&p| p != 0).count();
    let steps = vec![
        (
            "move",
            st.moved > 20,
            format!("the bottle moved {} px in total with the arrow keys", st.moved),
        ),
        (
            "act",
            st.shots_fired > 5 && st.hits > 0,
            format!(
                "fired {} shots, {} hits, {} enemies destroyed, score {} -> {}",
                st.shots_fired, st.hits, st.kills, start_score, hero.score
            ),
        ),
        (
            "hud",
            bar_px > 300,
            format!("status bar with score and spare bottles drawn ({} lit pixels in rows 211-219), level '{}', outcome {:?}", bar_px, level_name, outcome),
        ),
    ];
    let mut json = String::from("{\"version\": 1, \"steps\": [");
    for (i, (s, ok, d)) in steps.iter().enumerate() {
        if i > 0 {
            json.push_str(", ");
        }
        json.push_str(&format!(
            "{{\"step\": \"{}\", \"passed\": {}, \"detail\": \"{}\"}}",
            s,
            ok,
            d.replace('"', "'")
        ));
    }
    json.push_str("]}\n");
    if let Some(dir) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, json);
    let _ = Outcome::Won;
    eprintln!("selftest: written");
}

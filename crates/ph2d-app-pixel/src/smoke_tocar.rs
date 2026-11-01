//! **A cena 17 — tags dentro de tags** (P5): a entrada `s1` que o oráculo do Aseprite Trial tocou
//! (`docs/PixelArt/ferramentas/trial_p5_play.py`): 8 quadros de 100 ms, cada um com o quadrado de
//! uma cor do arco-íris; a tag «out» sobre 2..7 (para a frente, para sempre) e, dentro dela, «in»
//! sobre 3..5 (Ping-pong, 2 passadas). Em Draw no quadro 1: o Enter toca
//! `1 2 3 4 5 4 3 6 7 8 | 1 …` (base 1), como o Trial.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::{Background, ColorMode, Direction, NewSprite, Rect};

use crate::PixelApp;
use crate::smoke_quadros::{passo, patch};

pub const SIZE: (u32, u32) = (16, 16);
pub const COLORS: [[u8; 4]; 8] = [
    [220, 50, 50, 255],
    [240, 140, 40, 255],
    [235, 200, 40, 255],
    [40, 170, 70, 255],
    [40, 180, 190, 255],
    [50, 110, 230, 255],
    [150, 70, 200, 255],
    [230, 90, 170, 255],
];
/// As tags (posições, base 0): o nome, de, até, a direcção e as passadas.
pub const TAGS: [(&str, u32, u32, Direction, u16); 2] = [
    ("out", 1, 6, Direction::Forward, 0),
    ("in", 2, 4, Direction::PingPong, 2),
];

pub(crate) fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u32> {
    let (w, h) = SIZE;
    let spec = NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let a = app.docs.get(&key)?.sprite.layers()[0].id;
    let mut ids = vec![app.docs.get(&key)?.sprite.frames()[0].id];
    for _ in 1..COLORS.len() {
        let after = *ids.last()?;
        let (op, id) =
            frames::new_frame(&mut app.docs.get_mut(&key)?.sprite, after, NewFrame::Empty)?;
        app.commit(key, op).ok()?;
        ids.push(id);
    }
    for (i, &f) in ids.iter().enumerate() {
        patch(app, key, a, f, Rect::new(2, 2, 12, 12), |_, _| COLORS[i])?;
    }
    passo(app, key, |s| frames::set_duration(s, &ids, 100))?;
    for (i, &(nome, de, ate, d, r)) in TAGS.iter().enumerate() {
        passo(app, key, |s| frames::new_tag(s, nome, de, ate))?;
        passo(app, key, |s| {
            frames::edit_tag(s, i, |t| {
                t.direction = d;
                t.repeat = r;
            })
        })?;
    }
    let doc = app.docs.get_mut(&key)?;
    (doc.frame, doc.layer) = (ids[0], a);
    Some(key)
}

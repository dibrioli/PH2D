//! **As cenas de smoke** (`PH2D_PIXEL_SMOKE=n`).
//!
//! 1 — um Pixel Sprite 32 × 32 já em Edit, com uma cara desenhada em QUATRO traços (quatro passos
//! do desfazer, para o `Ctrl+Z` ter o que mostrar). O dono desenha por cima, desfaz, grava e reabre.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::{NewSprite, Stroke};

use crate::PixelApp;

pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_PIXEL_SMOKE",
    max_level: 1,
}];

/// O nível armado em `PH2D_PIXEL_SMOKE`.
#[must_use]
pub fn armed_level() -> Option<u32> {
    std::env::var(ROUTERS[0].env)
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
}

/// Os quatro traços da cara (células de um sprite 32 × 32) e a cor de cada um (índice da DB32):
/// olho esquerdo e direito (preto), o sorriso (vermelho), a moldura (azul).
pub const FACE: [(&[(i32, i32)], usize); 4] = [
    (&[(11, 11), (11, 13)], 0),
    (&[(20, 11), (20, 13)], 0),
    (&[(9, 19), (12, 22), (19, 22), (22, 19)], 27),
    (&[(6, 6), (25, 6), (25, 25), (6, 25), (6, 6)], 17),
];

/// Encena o nível `level`. `false` = nível que este roteador não conhece.
pub fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    level: u32,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> bool {
    match level {
        1 => {
            let Ok(bits) = crate::object_add::spawn(sim, app, NewSprite::default(), spawn_root)
            else {
                return false;
            };
            let Some(key) = crate::input::key_of(sim.world(), bits) else {
                return false;
            };
            for (path, color) in FACE {
                if !draw_path(app, key, color, path) {
                    return false;
                }
            }
            app.fg = [0, 0, 0, 255];
            app.bg = [255, 255, 255, 255];
            true
        }
        _ => false,
    }
}

/// Um traço pela `path` com a cor `color` da paleta, como UM passo do desfazer.
fn draw_path(app: &mut PixelApp, key: u32, color: usize, path: &[(i32, i32)]) -> bool {
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    let s = &doc.sprite;
    let Some(&rgba) = s.palette().get(color) else {
        return false;
    };
    let Some(&(x0, y0)) = path.first() else {
        return false;
    };
    let mut st = Stroke::begin(s, doc.layer, doc.frame, s.value_of_rgba(rgba), x0, y0);
    for &(x, y) in &path[1..] {
        st.to(s, x, y);
    }
    let Some(op) = st.op(s) else {
        return false;
    };
    app.commit(key, op).is_ok()
}

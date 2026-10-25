//! **As cenas de smoke** (`PH2D_PIXEL_SMOKE=n`).
//!
//! 1 — um Pixel Sprite 32 × 32 já em Draw, com uma cara desenhada em QUATRO traços (quatro passos
//! do desfazer, para o `Ctrl+Z` ter o que mostrar). O dono desenha por cima, desfaz, grava e reabre.
//! 2 — P1: um Pixel Sprite 64 × 48 em Draw com uma casa feita SÓ com as ferramentas novas, cada
//! forma um passo do desfazer: paredes (Rectangle), telhado (Polygon), porta (Filled Rectangle de
//! canto redondo), sol (Filled Ellipse), chão (Line), fumo (Curve). As paredes FECHAM — o balde
//! enche a casa sem vazar —, e a Rectangle fica na mão.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::{NewSprite, Paint, Stroke};

use crate::PixelApp;

pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_PIXEL_SMOKE",
    max_level: 2,
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
        2 => stage_house(app, sim, spawn_root),
        _ => false,
    }
}

/// As formas da casa da cena 2: a ferramenta, as células que a definem e a cor (índice da DB32).
type Peca = (crate::Tool, &'static [(i32, i32)], usize);

pub const HOUSE: [Peca; 6] = [
    (crate::Tool::Line, &[(0, 40), (63, 40)], 12),
    (crate::Tool::Rectangle, &[(14, 22), (40, 40)], 4),
    (crate::Tool::Polygon, &[(11, 22), (27, 9), (43, 22)], 27),
    (crate::Tool::FilledRectangle, &[(24, 30), (30, 40)], 3),
    (crate::Tool::FilledEllipse, &[(50, 3), (59, 12)], 8),
    (crate::Tool::Curve, &[(34, 13), (28, 1), (44, 2)], 2),
];

fn stage_house(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> bool {
    let spec = NewSprite {
        width: 64,
        height: 48,
        ..NewSprite::default()
    };
    let Ok(bits) = crate::object_add::spawn(sim, app, spec, spawn_root) else {
        return false;
    };
    let Some(key) = crate::input::key_of(sim.world(), bits) else {
        return false;
    };
    app.options.corner_radius = 2;
    for (tool, pts, color) in HOUSE {
        if !draw_shape(app, key, tool, pts, color) {
            return false;
        }
    }
    app.options.corner_radius = 0;
    app.tool = crate::Tool::Rectangle;
    app.fg = [0, 0, 0, 255];
    app.bg = [255, 255, 255, 255];
    true
}

/// Uma forma da cena pela MESMA lei do gesto ([`crate::gesture::reshape`]), como UM passo.
fn draw_shape(
    app: &mut PixelApp,
    key: u32,
    tool: crate::Tool,
    pts: &[(i32, i32)],
    color: usize,
) -> bool {
    use crate::gesture::{Kind, Mods, paint, reshape};
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    let s = &doc.sprite;
    let Some(&rgba) = s.palette().get(color) else {
        return false;
    };
    let kind = match (tool, pts) {
        (crate::Tool::Polygon, _) => Kind::Polygon(pts.to_vec()),
        (crate::Tool::Curve, [a, c, b]) => Kind::Curve {
            from: *a,
            to: *b,
            ctrl: Some(*c),
        },
        (_, [a, b]) => Kind::Box { from: *a, to: *b },
        _ => return false,
    };
    let mut st = Stroke::new(
        doc.layer,
        doc.frame,
        s.value_of_rgba(rgba),
        paint(&app.options, doc),
    );
    reshape(&mut st, s, tool, &kind, &app.options, Mods::default());
    let Some(op) = st.op(s) else {
        return false;
    };
    app.commit(key, op).is_ok()
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
    let mut st = Stroke::begin(
        s,
        doc.layer,
        doc.frame,
        s.value_of_rgba(rgba),
        Paint::default(),
        (x0, y0),
    );
    for &(x, y) in &path[1..] {
        st.to(s, x, y);
    }
    let Some(op) = st.op(s) else {
        return false;
    };
    app.commit(key, op).is_ok()
}

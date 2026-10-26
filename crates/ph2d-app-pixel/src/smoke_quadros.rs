//! **A cena 8 — os frames que SE VÊEM** (P4). Um Pixel Sprite RGBA 64 × 32 de céu transparente,
//! 16 frames, em Draw no frame 1 com o papel vegetal ligado (vermelho atrás, azul à frente):
//!
//! - uma **bola** atravessa o sprite em quatro trechos de quatro frames, cada trecho com a sua cor e
//!   a sua **tag**: vermelho «Forward» (2 passadas; o frame 4 dura 480 ms, os outros 120 — a bola
//!   PÁRA ali), verde «Reverse» (2 passadas; a bola tem **z +1** e passa À FRENTE do pilar), azul
//!   «Ping-pong» (2 passadas = ida e volta; o cel da bola a **50 %**), amarelo «Ping-pong Reverse»
//!   (para sempre). `Enter` no frame 1 toca a sequência inteira e fica no amarelo;
//! - o **chão** é UMA imagem ligada nos 16 frames (a camada é *continuous*: o New Frame liga-a) — na
//!   linha do tempo uma pílula só; pintar o chão num frame muda-o em todos;
//! - o **pilar** é copiado em cada frame (pontos separados): por trás dele a bola some, salvo no
//!   verde.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::{ColorMode, Direction, FrameId, LayerId, NewSprite, Op, Patch, Rect, layers};

use crate::PixelApp;

/// O tamanho da cena 8 e o número de frames.
pub const SIZE: (u32, u32) = (64, 32);
pub const FRAMES: usize = 16;
/// O pilar e o chão.
pub const PILLAR: Rect = Rect::new(22, 6, 5, 20);
pub const GROUND: Rect = Rect::new(0, 26, 64, 6);
/// O raio da bola e o centro dela na posição `k` (0..4) de cada trecho.
pub const BALL_R: i32 = 4;
#[must_use]
pub const fn ball_center(k: usize) -> (i32, i32) {
    (8 + k as i32 * 16, 17)
}
/// A cor de cada trecho (a da tag e a da bola).
pub const COLORS: [[u8; 4]; 4] = [
    [220, 50, 50, 255],
    [40, 170, 70, 255],
    [50, 110, 230, 255],
    [235, 200, 40, 255],
];
/// As tags: (nome, direcção, passadas).
pub const TAGS: [(&str, Direction, u16); 4] = [
    ("Forward", Direction::Forward, 2),
    ("Reverse", Direction::Reverse, 2),
    ("Ping-pong", Direction::PingPong, 2),
    ("Ping-pong Reverse", Direction::PingPongReverse, 0),
];
/// A duração de cada frame e a do frame que segura a bola (o 4.º).
pub const FRAME_MS: u32 = 120;
pub const HOLD_MS: u32 = 480;
pub const HOLD_INDEX: usize = 3;

fn patch(app: &mut PixelApp, key: u32, layer: LayerId, frame: FrameId, r: Rect, f: impl Fn(i32, i32) -> [u8; 4]) -> Option<()> {
    let mut pixels = Vec::with_capacity(r.area() * 4);
    for y in r.y..r.y + r.h as i32 {
        for x in r.x..r.x + r.w as i32 {
            pixels.extend_from_slice(&f(x, y));
        }
    }
    app.commit(key, Op::Patch(Patch { layer, frame, bbox: Some(r), rect: r, pixels })).ok()
}

/// Um passo construído sobre o sprite de agora (`None` do construtor = nada a fazer).
fn passo(app: &mut PixelApp, key: u32, f: impl FnOnce(&mut ph2d_pixel_core::Sprite) -> Option<Op>) -> Option<()> {
    let op = f(&mut app.docs.get_mut(&key)?.sprite);
    match op {
        Some(op) => app.commit(key, op).ok(),
        None => Some(()),
    }
}

fn camada(app: &mut PixelApp, key: u32, above: LayerId, name: &str) -> Option<LayerId> {
    let (op, id) = layers::new_layer(&mut app.docs.get_mut(&key)?.sprite, above)?;
    app.commit(key, op).ok()?;
    passo(app, key, |s| layers::edit(s, id, |l| name.clone_into(&mut l.name)))?;
    Some(id)
}

fn bola(c: [u8; 4]) -> impl Fn(i32, i32) -> [u8; 4] {
    move |x, y| {
        let (dx, dy) = (x - BALL_R, y - BALL_R);
        if dx * dx + dy * dy <= BALL_R * BALL_R { c } else { [0; 4] }
    }
}

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
        background: ph2d_pixel_core::Background::Transparent,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let chao = app.docs.get(&key)?.sprite.layers()[0].id;
    passo(app, key, |s| {
        layers::edit(s, chao, |l| {
            "Ground".clone_into(&mut l.name);
            l.continuous = true;
        })
    })?;
    let f0 = app.docs.get(&key)?.sprite.frames()[0].id;
    patch(app, key, chao, f0, GROUND, |x, y| {
        if (x + y) % 5 == 0 { [120, 80, 40, 255] } else { [150, 105, 60, 255] }
    })?;
    let ball = camada(app, key, chao, "Ball")?;
    let pilar = camada(app, key, ball, "Pillar")?;
    patch(app, key, pilar, f0, PILLAR, |_, _| [110, 110, 125, 255])?;
    let mut ids = vec![f0];
    for _ in 1..FRAMES {
        let after = *ids.last()?;
        let (op, id) = frames::new_frame(&mut app.docs.get_mut(&key)?.sprite, after, NewFrame::Copy)?;
        app.commit(key, op).ok()?;
        ids.push(id);
    }
    for (i, &f) in ids.iter().enumerate() {
        let (cx, cy) = ball_center(i % 4);
        let r = Rect::new(cx - BALL_R, cy - BALL_R, (2 * BALL_R + 1) as u32, (2 * BALL_R + 1) as u32);
        patch(app, key, ball, f, r, |x, y| bola(COLORS[i / 4])(x - r.x, y - r.y))?;
        match i / 4 {
            1 => passo(app, key, |s| frames::set_cel(s, (ball, f), None, Some(1)))?,
            2 => passo(app, key, |s| frames::set_cel(s, (ball, f), Some(128), None))?,
            _ => {}
        }
    }
    passo(app, key, |s| frames::set_duration(s, &ids, FRAME_MS))?;
    passo(app, key, |s| frames::set_duration(s, &[ids[HOLD_INDEX]], HOLD_MS))?;
    for (t, &(name, dir, repeat)) in TAGS.iter().enumerate() {
        let from = (t * 4) as u32;
        passo(app, key, |s| frames::new_tag(s, name, from, from + 3))?;
        passo(app, key, |s| {
            frames::edit_tag(s, t, |x| (x.direction, x.repeat, x.color) = (dir, repeat, COLORS[t]))
        })?;
    }
    let doc = app.docs.get_mut(&key)?;
    (doc.frame, doc.layer) = (f0, ball);
    app.onion_on = true;
    app.onion.tint = true;
    Some(bits as u32)
}

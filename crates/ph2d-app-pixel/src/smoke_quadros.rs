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
use ph2d_pixel_core::{ColorMode, FrameId, LayerId, NewSprite, Op, Rect, layers};

use crate::PixelApp;

pub use ph2d_pixel_cenas::quadros::*;

pub(crate) fn patch(
    app: &mut PixelApp,
    key: u32,
    layer: LayerId,
    frame: FrameId,
    r: Rect,
    f: impl Fn(i32, i32) -> [u8; 4],
) -> Option<()> {
    app.commit(key, ph2d_pixel_cenas::patch::op(layer, frame, r, r, f))
        .ok()
}

/// Um passo construído sobre o sprite de agora (`None` do construtor = nada a fazer).
pub(crate) fn passo(
    app: &mut PixelApp,
    key: u32,
    f: impl FnOnce(&mut ph2d_pixel_core::Sprite) -> Option<Op>,
) -> Option<()> {
    let op = f(&mut app.docs.get_mut(&key)?.sprite);
    match op {
        Some(op) => app.commit(key, op).ok(),
        None => Some(()),
    }
}

pub(crate) fn camada(app: &mut PixelApp, key: u32, above: LayerId, name: &str) -> Option<LayerId> {
    let (op, id) = layers::new_layer(&mut app.docs.get_mut(&key)?.sprite, above)?;
    app.commit(key, op).ok()?;
    passo(app, key, |s| {
        layers::edit(s, id, |l| name.clone_into(&mut l.name))
    })?;
    Some(id)
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
        if (x + y) % 5 == 0 {
            [120, 80, 40, 255]
        } else {
            [150, 105, 60, 255]
        }
    })?;
    let ball = camada(app, key, chao, "Ball")?;
    let pilar = camada(app, key, ball, "Pillar")?;
    patch(app, key, pilar, f0, PILLAR, |_, _| [110, 110, 125, 255])?;
    let mut ids = vec![f0];
    for _ in 1..FRAMES {
        let after = *ids.last()?;
        let (op, id) =
            frames::new_frame(&mut app.docs.get_mut(&key)?.sprite, after, NewFrame::Copy)?;
        app.commit(key, op).ok()?;
        ids.push(id);
    }
    for (i, &f) in ids.iter().enumerate() {
        let (cx, cy) = ball_center(i % 4);
        let r = Rect::new(
            cx - BALL_R,
            cy - BALL_R,
            (2 * BALL_R + 1) as u32,
            (2 * BALL_R + 1) as u32,
        );
        patch(app, key, ball, f, r, |x, y| {
            bola(COLORS[i / 4])(x - r.x, y - r.y)
        })?;
        match i / 4 {
            1 => passo(app, key, |s| frames::set_cel(s, (ball, f), None, Some(1)))?,
            2 => passo(app, key, |s| frames::set_cel(s, (ball, f), Some(128), None))?,
            _ => {}
        }
    }
    passo(app, key, |s| frames::set_duration(s, &ids, FRAME_MS))?;
    passo(app, key, |s| {
        frames::set_duration(s, &[ids[HOLD_INDEX]], HOLD_MS)
    })?;
    for (t, &(name, dir, repeat)) in TAGS.iter().enumerate() {
        let from = (t * 4) as u32;
        passo(app, key, |s| frames::new_tag(s, name, from, from + 3))?;
        passo(app, key, |s| {
            frames::edit_tag(s, t, |x| {
                (x.direction, x.repeat, x.color) = (dir, repeat, COLORS[t])
            })
        })?;
    }
    let doc = app.docs.get_mut(&key)?;
    (doc.frame, doc.layer) = (f0, ball);
    app.onion_on = true;
    app.muda_prefs(key, |p| p.onion.tint = true);
    Some(bits as u32)
}

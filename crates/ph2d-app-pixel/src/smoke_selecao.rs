//! **A cena 9 — a selecção e a transformação que SE VÊEM** (P6). Um Pixel Sprite RGBA 128 × 64,
//! em Draw, com a ferramenta Rectangular Marquee na mão e UMA selecção activa feita de quatro
//! peças (as formigas marcham à volta de cada uma):
//!
//! - em cima, da esquerda para a direita: um **retângulo** de marquee atravessado por um **traço
//!   que só pintou dentro dele** (o traço foi de ponta a ponta com aquela selecção activa), uma
//!   **elipse**, uma **estrela de laço** e a **região da varinha** (o losango verde, um clique);
//!   à direita a mesma imagem 3 × 3 aumentada para 12 × 12 em **Nearest** e em **Bilinear**;
//! - em baixo: a **figura** original, a mesma figura **girada 30°** em **Fast** e em
//!   **RotSprite**, a figura **espelhada**, e um **ladrilho deslocado com volta** (Shift).

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::affine::{Affine, Linear, Resample};
use ph2d_pixel_core::selection::{self, Mask, SelectMode, Wand};
use ph2d_pixel_core::transform::{Floating, Image, flip_h, resize_bilinear, resize_nearest};
use ph2d_pixel_core::{ColorMode, NewSprite, Op, Paint, Patch, Rect, Stroke, edit};

use crate::PixelApp;

pub use ph2d_pixel_cenas::selecao::*;

fn assenta(app: &mut PixelApp, key: u32, f: &Floating) -> Option<()> {
    let doc = app.docs.get(&key)?;
    let op = edit::paste(&doc.sprite, (doc.layer, doc.frame), f, 0)?;
    app.commit(key, op).ok()
}

/// Um passo (`None` = nada a mudar, como a selecção que o Paste já deixou).
fn passo(app: &mut PixelApp, key: u32, op: Option<Op>) -> Option<()> {
    match op {
        Some(op) => app.commit(key, op).ok(),
        None => Some(()),
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
    let (layer, frame) = {
        let d = app.docs.get(&key)?;
        (d.layer, d.frame)
    };
    // O losango que a varinha apanha.
    let ((cx, cy), r) = WAND;
    let caixa = Rect::new(cx - r, cy - r, (2 * r + 1) as u32, (2 * r + 1) as u32);
    let mut pixels = Vec::with_capacity(caixa.area() * 4);
    for y in caixa.y..caixa.y + caixa.h as i32 {
        for x in caixa.x..caixa.x + caixa.w as i32 {
            pixels.extend_from_slice(&if in_wand_blob(x, y) {
                WAND_COLOR
            } else {
                [0; 4]
            });
        }
    }
    let p = Patch {
        layer,
        frame,
        bbox: Some(caixa),
        rect: caixa,
        pixels,
    };
    app.commit(key, Op::Patch(p)).ok()?;
    // As cópias: aumentadas, a figura e as giradas, a espelhada, o ladrilho com volta.
    let s = small();
    assenta(
        app,
        key,
        &solta(
            NEAREST_AT.0,
            NEAREST_AT.1,
            resize_nearest(&s, SCALED, SCALED),
        ),
    )?;
    assenta(
        app,
        key,
        &solta(
            BILINEAR_AT.0,
            BILINEAR_AT.1,
            resize_bilinear(&s, SCALED, SCALED),
        ),
    )?;
    assenta(app, key, &solta(ORIGINAL_AT.0, ORIGINAL_AT.1, figure()))?;
    assenta(app, key, &rotated(FAST_AT, Resample::Fast)?)?;
    assenta(app, key, &rotated(ROTSPRITE_AT, Resample::RotSprite)?)?;
    assenta(app, key, &solta(FLIP_AT.0, FLIP_AT.1, flip_h(&figure())))?;
    assenta(app, key, &solta(TILE_AT.0, TILE_AT.1, tile()))?;
    // O Shift com volta, um degrau de cada vez, com o ladrilho seleccionado (como o artista faz).
    let t = Mask::from_rect(w, h, Rect::new(TILE_AT.0, TILE_AT.1, FIGURE, FIGURE));
    let sel = |app: &PixelApp, m: &Mask| {
        selection::select(&app.docs.get(&key)?.sprite, m, SelectMode::Replace)
    };
    passo(app, key, sel(app, &t))?;
    for (dx, dy) in std::iter::repeat_n((1, 0), SHIFT.0 as usize)
        .chain(std::iter::repeat_n((0, 1), SHIFT.1 as usize))
    {
        let doc = app.docs.get(&key)?;
        passo(
            app,
            key,
            edit::shift(&doc.sprite, (layer, frame), dx, dy, 0),
        )?;
    }
    // O traço de ponta a ponta com a marquee activa: só pinta dentro dela.
    passo(app, key, sel(app, &Mask::from_rect(w, h, RECT)))?;
    let doc = app.docs.get(&key)?;
    let s = &doc.sprite;
    let v = s.value_of_rgba(STROKE);
    let mut st = Stroke::begin(s, layer, frame, v, Paint::default(), (0, STROKE_Y));
    st.to(s, STROKE_X1, STROKE_Y);
    passo(app, key, st.op(s))?;
    // A selecção final: as quatro peças.
    let m = mask(&app.docs.get(&key)?.sprite, (layer, frame));
    passo(app, key, sel(app, &m))?;
    app.tool = crate::Tool::RectMarquee;
    Some(bits as u32)
}

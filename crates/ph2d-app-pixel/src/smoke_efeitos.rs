//! **A cena 19 — a P8** (Gradient, Blur/Jumble, Text, Edit ▸ Fill/Stroke, Adjustments, FX). Um
//! Pixel Sprite RGBA 64 × 48 de fundo branco em Draw: um boneco de três cores ([`boneco`]), uma
//! selecção sobre a zona livre ([`SELECTION`]) e a rampa de seis tons ([`RAMP`]) no fim da paleta,
//! com a frente na ponta escura e o fundo na clara — o Gradient vê-se e o «Palette ramp» anda nela.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::selection::{Mask, SelectMode, select};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Rect};

use crate::{PixelApp, Tool};

/// O tamanho da cena 19.
pub const SIZE: (u32, u32) = (64, 48);
/// A selecção feita (x, y, w, h): o quarto de cima à direita, onde não há boneco.
pub const SELECTION: (i32, i32, u32, u32) = (32, 6, 26, 20);
/// A pele, a camisa e as calças do boneco.
pub const SKIN: [u8; 4] = [240, 190, 140, 255];
pub const SHIRT: [u8; 4] = [50, 100, 210, 255];
pub const PANTS: [u8; 4] = [60, 50, 70, 255];
/// A rampa (roxo → amarelo), acrescentada depois das cores de fábrica da paleta.
pub const RAMP: [[u8; 4]; 6] = [
    [40, 20, 80, 255],
    [100, 30, 120, 255],
    [180, 50, 110, 255],
    [235, 100, 80, 255],
    [250, 170, 60, 255],
    [255, 235, 120, 255],
];

/// A cor do boneco na célula `(x, y)`: cabeça, braços e tronco, pernas.
#[must_use]
pub fn boneco(x: i32, y: i32) -> Option<[u8; 4]> {
    match (x, y) {
        (10..=17, 6..=13) => Some(SKIN),
        (6..=7 | 20..=21, 15..=24) => Some(SKIN),
        (8..=19, 15..=28) => Some(SHIRT),
        (9..=12 | 15..=18, 29..=40) => Some(PANTS),
        _ => None,
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
        background: Background::White,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let (layer, frame, mut colors, transparent) = {
        let d = app.docs.get(&key)?;
        let s = &d.sprite;
        (
            d.layer,
            d.frame,
            s.palette().to_vec(),
            s.transparent_index(),
        )
    };
    colors.extend(RAMP);
    app.commit(
        key,
        Op::Palette {
            colors,
            transparent,
        },
    )
    .ok()?;
    let r = Rect::new(0, 0, w, h);
    let pixels = (0..h as i32)
        .flat_map(|y| (0..w as i32).flat_map(move |x| boneco(x, y).unwrap_or([255; 4])))
        .collect();
    let p = Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels,
    };
    app.commit(key, Op::Patch(p)).ok()?;
    let (x, y, sw, sh) = SELECTION;
    let m = Mask::from_rect(w, h, Rect::new(x, y, sw, sh));
    let op = select(&app.docs.get(&key)?.sprite, &m, SelectMode::Replace)?;
    app.commit(key, op).ok()?;
    crate::input::set_tool(app, Tool::Gradient);
    app.fg = RAMP[0];
    app.bg = RAMP[RAMP.len() - 1];
    Some(key)
}

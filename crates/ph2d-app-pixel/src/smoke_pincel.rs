//! **A cena 18 — os pincéis e a dinâmica** (P7b). Um Pixel Sprite RGBA 96 × 64 de fundo branco em
//! Draw: no canto, um desenho de seis cores ([`DRAWING`]) JÁ seleccionado, o lápis na
//! mão com Size 16 e a Dynamics em Size ← Velocity (Min 1): o Ctrl+B faz dele o pincel, e um traço
//! lento sai fino e um rápido grosso.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::{PixelSensor, PixelThreshold};
use ph2d_pixel_core::selection::{Mask, SelectMode, select};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Rect};

use crate::{PixelApp, Tool};

pub use ph2d_pixel_cenas::pincel::*;

/// As pegas da velocidade da cena, em ‰: as de fábrica (100/890 = 5,6 e 57,7 px de ecrã por evento)
/// pedem um rato muito rápido; com 0/300 (−1 e 18,8 px por evento) um gesto normal já mostra o fino
/// e o grosso — e a barra fica a ensinar-se a si mesma.
pub const VELOCITY: PixelThreshold = PixelThreshold { lo: 0, hi: 300 };

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
    let (layer, frame) = {
        let d = app.docs.get(&key)?;
        (d.layer, d.frame)
    };
    let (dw, dh) = (DRAWING[0].len() as u32, DRAWING.len() as u32);
    let caixa = Rect::new(AT.0, AT.1, dw, dh);
    let branco = [255; 4];
    let pixels = DRAWING
        .iter()
        .flat_map(|linha| linha.iter().flat_map(|c| c.unwrap_or(branco)))
        .collect();
    let p = Patch {
        layer,
        frame,
        // O cel de fundo é a imagem inteira: o patch não o encolhe à caixa do desenho.
        bbox: Some(Rect::new(0, 0, w, h)),
        rect: caixa,
        pixels,
    };
    app.commit(key, Op::Patch(p)).ok()?;
    let m = Mask::from_rect(w, h, caixa);
    let op = select(&app.docs.get(&key)?.sprite, &m, SelectMode::Replace)?;
    app.commit(key, op).ok()?;
    crate::input::set_tool(app, Tool::Pencil);
    app.options.size = BRUSH_SIZE;
    app.options.dynamics.size.sensor = PixelSensor::Velocity;
    app.options.dynamics.size.min = 1;
    app.options.dynamics.velocity = VELOCITY;
    app.fg = [0, 0, 0, 255];
    app.bg = branco;
    Some(key)
}

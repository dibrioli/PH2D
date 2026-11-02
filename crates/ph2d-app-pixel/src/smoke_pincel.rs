//! **A cena 18 — os pincéis e a dinâmica** (P7b). Um Pixel Sprite RGBA 96 × 64 de fundo branco em
//! Draw: no canto, um desenho de seis cores ([`DRAWING`]) JÁ seleccionado, o lápis na
//! mão com Size 16 e a Dynamics em Size ← Velocity (Min 1): o Ctrl+B faz dele o pincel, e um traço
//! lento sai fino e um rápido grosso.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::{PixelSensor, PixelThreshold};
use ph2d_pixel_core::selection::{Mask, SelectMode, select};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Rect};

use crate::{PixelApp, Tool};

/// O tamanho da cena 18.
pub const SIZE: (u32, u32) = (96, 64);
/// Onde o desenho está (o canto de cima à esquerda).
pub const AT: (i32, i32) = (6, 6);
/// O desenho: 6 × 4, uma cor por coluna.
pub const DRAWING: [[Option<[u8; 4]>; 6]; 4] = {
    const R: Option<[u8; 4]> = Some([230, 40, 40, 255]);
    const O: Option<[u8; 4]> = Some([245, 150, 30, 255]);
    const Y: Option<[u8; 4]> = Some([240, 220, 40, 255]);
    const G: Option<[u8; 4]> = Some([40, 170, 70, 255]);
    const B: Option<[u8; 4]> = Some([40, 90, 220, 255]);
    const P: Option<[u8; 4]> = Some([150, 50, 190, 255]);
    [
        [R, O, Y, G, B, P],
        [R, O, Y, G, B, P],
        [R, O, Y, G, B, P],
        [R, O, Y, G, B, P],
    ]
};
/// As pegas da velocidade da cena, em ‰: as de fábrica (100/890 = 5,6 e 57,7 px de ecrã por evento)
/// pedem um rato muito rápido; com 0/300 (−1 e 18,8 px por evento) um gesto normal já mostra o fino
/// e o grosso — e a barra fica a ensinar-se a si mesma.
pub const VELOCITY: PixelThreshold = PixelThreshold { lo: 0, hi: 300 };
/// O Size da barra (o máximo da dinâmica).
pub const BRUSH_SIZE: u32 = 16;

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

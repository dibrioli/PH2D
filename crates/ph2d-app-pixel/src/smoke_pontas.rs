//! **A cena 15 — as pontas soltas das formas e da selecção** (P6g). Um Pixel Sprite RGBA 128 × 80
//! em Draw, com DUAS camadas — «Figures» (activa) por baixo e uma vazia por cima:
//!
//! - à esquerda, o «F» da cena 9 levantado numa moldura solta já INCLINADA ([`SKEW`] graus: a barra
//!   diz `Skew 20`); fora do meio de cada lado do gizmo há uma zona de inclinar (a alça do lado vira
//!   losango sob o cursor);
//! - à direita, o losango verde: a varinha (que a cena deixa com «Visible layers») apanha-o mesmo
//!   com a camada vazia activa;
//! - o «F» está copiado: o painel Layers mostra «Paste as New Sprite»;
//! - o Transform na mão com «Auto Select Layer» ligado, e o flyout das selecções aberto: as 4 numa
//!   linha.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::{PixelToolGroup, PixelTransformField};
use ph2d_pixel_core::selection::{Mask, SelectMode, select};
use ph2d_pixel_core::transform::Floating;
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Rect, edit, layers};

use crate::edicao::Prancheta;
use crate::smoke_selecao::{FIGURE, WAND_COLOR, figure};
use crate::{PixelApp, Tool};

/// O tamanho da cena 15.
pub const SIZE: (u32, u32) = (128, 80);
/// Onde o «F» está (o canto da caixa dele, [`FIGURE`] × [`FIGURE`]).
pub const FIGURE_AT: (i32, i32) = (16, 24);
/// O nome da camada de baixo (a das figuras).
pub const NAME: &str = "Figures";
/// A inclinação da moldura solta (graus).
pub const SKEW: i32 = 20;
/// O losango verde: o centro e o raio (Manhattan).
pub const BLOB: ((i32, i32), i32) = ((92, 40), 12);

/// O losango verde.
#[must_use]
pub fn in_blob(x: i32, y: i32) -> bool {
    let ((cx, cy), r) = BLOB;
    (x - cx).abs() + (y - cy).abs() <= r
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
        background: Background::Transparent,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let (layer, frame) = {
        let d = app.docs.get(&key)?;
        (d.layer, d.frame)
    };
    let ((cx, cy), r) = BLOB;
    let caixa = Rect::new(cx - r, cy - r, (2 * r + 1) as u32, (2 * r + 1) as u32);
    let pixels = (caixa.y..caixa.y + caixa.h as i32)
        .flat_map(|y| (caixa.x..caixa.x + caixa.w as i32).map(move |x| (x, y)))
        .flat_map(|(x, y)| if in_blob(x, y) { WAND_COLOR } else { [0; 4] })
        .collect();
    let p = Patch {
        layer,
        frame,
        bbox: Some(caixa),
        rect: caixa,
        pixels,
    };
    app.commit(key, Op::Patch(p)).ok()?;
    let image = figure();
    let keep = vec![true; (image.w * image.h) as usize];
    let f = Floating {
        x: FIGURE_AT.0,
        y: FIGURE_AT.1,
        image,
        keep,
    };
    let doc = app.docs.get(&key)?;
    let op = edit::paste(&doc.sprite, (layer, frame), &f, 0)?;
    app.commit(key, op).ok()?;
    let s = &app.docs.get(&key)?.sprite;
    let op = layers::edit(s, layer, |l| NAME.clone_into(&mut l.name))?;
    app.commit(key, op).ok()?;
    // A camada vazia por cima («Figures» continua a activa: é dela que a moldura sai).
    let doc = app.docs.get_mut(&key)?;
    let (op, _) = layers::new_layer(&mut doc.sprite, layer)?;
    app.commit(key, op).ok()?;
    if let Some(d) = app.docs.get_mut(&key) {
        d.layer = layer;
    }
    let (fw, fh) = (FIGURE, FIGURE);
    let m = Mask::from_rect(w, h, Rect::new(FIGURE_AT.0, FIGURE_AT.1, fw, fh));
    let s = &app.docs.get(&key)?.sprite;
    // `None` = a selecção que o Paste já deixou.
    if let Some(op) = select(s, &m, SelectMode::Replace) {
        app.commit(key, op).ok()?;
    }
    let s = &app.docs.get(&key)?.sprite;
    app.clip = Some(Prancheta::de(s, edit::copy(s, (layer, frame))?));
    app.options.auto_select_layer = true;
    app.options.wand_sample_visible = true;
    crate::input::set_tool(app, Tool::Transform);
    if !app.levanta(key) {
        return None;
    }
    app.campo_da_moldura(PixelTransformField::Skew, SKEW);
    app.groups.open = Some(PixelToolGroup::Select);
    Some(key)
}

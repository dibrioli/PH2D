//! **A cena 11 — as três correcções do smoke da P6b que SE VÊEM** (P6c, dono 26/10). Um Pixel
//! Sprite RGBA 128 × 80 em Draw, com a ELIPSE na mão e a caixa «Fill» ligada:
//!
//! - na coluna: o **botão das selecções** com a cara do laço poligonal (o ícone próprio: arestas
//!   rectas, vértices marcados, o nó do laço); a varinha e o **Transform** soltos — **sem o Move**;
//!   o **botão das formas** com o triângulo, e as quatro (Retângulo, Elipse, Polígono, Estrela)
//!   abertas ao lado; Linha, Curva, Contorno e Polígono livre soltos;
//! - em cima, da esquerda para a direita: retângulo oco, retângulo cheio, elipse oca, elipse cheia —
//!   a MESMA ferramenta, a caixa «Fill» desligada e ligada;
//! - em baixo: a figura com uma selecção poligonal à volta (as formigas). `V` escolhe o Transform:
//!   a figura levanta no gizmo, e arrastar por dentro move-a.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::PixelToolGroup;
use ph2d_pixel_core::selection::{self, Mask, SelectMode};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, edit};

use crate::{PixelApp, Tool};

/// O tamanho da cena 11.
pub const SIZE: (u32, u32) = (128, 80);
/// As quatro formas: a ferramenta, a caixa (cantos inclusivos), a cor e a caixa «Fill».
pub const SHAPES: [(Tool, (i32, i32), (i32, i32), [u8; 4], bool); 4] = [
    (
        Tool::Rectangle,
        (4, 6),
        (27, 29),
        [70, 110, 220, 255],
        false,
    ),
    (
        Tool::Rectangle,
        (34, 6),
        (57, 29),
        [70, 110, 220, 255],
        true,
    ),
    (Tool::Ellipse, (64, 6), (87, 29), [240, 200, 60, 255], false),
    (Tool::Ellipse, (94, 6), (117, 29), [240, 200, 60, 255], true),
];
/// Onde a figura fica e os vértices da selecção poligonal à volta dela.
pub const FIGURE_AT: (i32, i32) = (57, 50);
pub const POLYGON: [(i32, i32); 5] = [(52, 47), (70, 44), (76, 58), (66, 68), (51, 64)];

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
    for (tool, a, b, rgba, fill) in SHAPES {
        app.options.shape_fill = fill;
        crate::smoke_formas::forma(app, key, tool, a, b, rgba)?;
    }
    let n = crate::smoke_selecao::FIGURE;
    let doc = app.docs.get(&key)?;
    let f = ph2d_pixel_core::transform::Floating {
        x: FIGURE_AT.0,
        y: FIGURE_AT.1,
        keep: vec![true; (n * n) as usize],
        image: crate::smoke_selecao::figure(),
    };
    let op = edit::paste(&doc.sprite, (doc.layer, doc.frame), &f, 0)?;
    app.commit(key, op).ok()?;
    let m = Mask::polygon(w, h, &POLYGON);
    let op = selection::select(&app.docs.get(&key)?.sprite, &m, SelectMode::Replace)?;
    app.commit(key, op).ok()?;
    // A elipse na mão com o Fill ligado; a cara das selecções é o laço poligonal; as formas abertas.
    app.groups.chose(Tool::PolygonLasso);
    crate::input::set_tool(app, Tool::Ellipse);
    app.groups.open = Some(PixelToolGroup::Shape);
    Some(key)
}

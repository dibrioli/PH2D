//! **P13 U-Pintura: os ids do ink Dither e do Push** (pintar, registar e despachar leem ESTA
//! tabela).

use ph2d_a11y::NodeId;
use ph2d_editor_core::pixel_panel::{PixelGradientDither, PixelOptions};
use ph2d_tool_registry::hash_node_id;

pub const DITHER_LEVEL: NodeId = hash_node_id("pixel.opt.ink.dither.level");
pub const PUSH_STRENGTH: NodeId = hash_node_id("pixel.opt.smudge.push.strength");

/// As matrizes do ink Dither (sem `None`: o ink sem padrão é a Simple).
pub const DITHER_MATRICES: [(PixelGradientDither, NodeId, &str); 3] = [
    (
        PixelGradientDither::Bayer2,
        hash_node_id("pixel.opt.ink.dither.bayer2"),
        "pixel.ink.dither.matrix.bayer2",
    ),
    (
        PixelGradientDither::Bayer4,
        hash_node_id("pixel.opt.ink.dither.bayer4"),
        "pixel.ink.dither.matrix.bayer4",
    ),
    (
        PixelGradientDither::Bayer8,
        hash_node_id("pixel.opt.ink.dither.bayer8"),
        "pixel.ink.dither.matrix.bayer8",
    ),
];

/// Os campos desta fatia: id, faixa `(min, max, passo)`.
pub const NUMBERS: [(NodeId, (f64, f64, f64)); 2] = [
    (DITHER_LEVEL, (0.0, 64.0, 1.0)), // LITERAL-PX-OK: os 65 níveis do PincelDither (domínio do campo)
    (PUSH_STRENGTH, (0.0, 255.0, 1.0)), // LITERAL-PX-OK: byte da força (forca/255)
];

/// A ferramenta escreve a cor pronta (ou a borracha, sempre Simple): sem fila de tintas.
#[must_use]
pub fn sem_tinta(t: ph2d_editor_core::pixel_panel::PixelTool) -> bool {
    use ph2d_editor_core::pixel_panel::PixelTool as T;
    matches!(t, T::Eraser | T::Gradient | T::Blur | T::Jumble | T::Push)
}

/// `id` é um campo numérico das opções (o da tabela de sempre ou um destes).
#[must_use]
pub fn e_numero(id: NodeId) -> bool {
    crate::ids::NUMBERS
        .iter()
        .chain(&NUMBERS)
        .any(|(n, _)| *n == id)
}

/// O valor do campo `id`, se é desta fatia.
#[must_use]
pub fn numero(o: &PixelOptions, id: NodeId) -> Option<f64> {
    match id {
        DITHER_LEVEL => Some(f64::from(o.dither_level)),
        PUSH_STRENGTH => Some(f64::from(o.push_strength)),
        _ => None,
    }
}

/// Grava o campo `id` (já preso à faixa) — `false` se não é desta fatia.
pub fn grava(o: &mut PixelOptions, id: NodeId, u: u32) -> bool {
    let b = u8::try_from(u).unwrap_or(u8::MAX);
    match id {
        DITHER_LEVEL => o.dither_level = b,
        PUSH_STRENGTH => o.push_strength = b,
        _ => return false,
    }
    true
}

/// A matriz `id`, se é desta fatia.
pub fn segmento(o: &mut PixelOptions, id: NodeId) -> bool {
    match DITHER_MATRICES.iter().find(|(_, s, _)| *s == id) {
        Some((d, ..)) => {
            o.dither_matrix = *d;
            true
        }
        None => false,
    }
}

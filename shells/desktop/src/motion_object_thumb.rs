//! **A MINIATURA de um assado** — irmã do [`crate::motion_object_bake`] por RESPONSABILIDADE (e
//! pelo tecto de LOC): aquele produz os pixels de um objecto; isto reduz-os ao que cabe num cartão
//! de painel, e não sabe nada sobre cenas, afins ou GPU.

use ph2d_panel_motion_graph::PreviewThumb;

/// Downsample straight RGBA8 (`w`×`h`) to a card thumbnail (doc 86 A5): at most
/// [`THUMB_MAX`] on its long side, aspect preserved, never upscaled. Box-average in
/// PREMULTIPLIED space (`Σ c·a / Σ a`) so a transparent edge does not bleed a dark
/// halo into the shrunk shape — the premul trap the overlay lesson names (ADR-0120
/// neighbourhood). One pass per bake; the result is cached with the tile.
pub(crate) fn thumbnail(rgba: &[u8], w: u32, h: u32) -> PreviewThumb {
    let (rgba, w, h) = crate::thumbnail::reduce(rgba, w, h);
    PreviewThumb { rgba, w, h }
}

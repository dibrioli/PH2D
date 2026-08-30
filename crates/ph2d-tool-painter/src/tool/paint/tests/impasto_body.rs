//! **O CORPO da tinta (impasto) — a raiz do assunto e as fixturas que os quatro lados partilham.**
//! A tela armada para esculpir, a leitura do plano de relevo e a leitura do quadro iluminado; os
//! quatro módulos abaixo são os lados desse mesmo corpo.

use super::*;

mod deposit;
mod light;
mod material;
mod sculpting;

// ── Impasto (#16) — the height channel rides the SHARED dab list ──────────────────────────────────

/// The relief the artist would see on the active layer (committed + the open stroke's envelope).
fn relief(t: &PainterTool) -> Vec<f32> {
    let id = t.layers.active().expect("a layer is active");
    t.layer_height_view(id).unwrap_or_default()
}

/// A canvas with an impasto brush ready to sculpt. Hard disk ⇒ a deterministic, level plateau.
fn impasto_canvas(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        // The REFERENCE radius (`height::IMPASTO_REFERENCE_RADIUS_PX`): the deposit's height now scales with
        // brush size (Enio 2026-07-14), so a fixture at any other radius would fold that scale into every
        // height it asserts. Pinning it here keeps these gates about Depth / Body / Grain / the ceiling, and
        // leaves the size-scaling to its own gate (`the_relief_height_scales_with_the_brush_size`).
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.3],
        space_attenuation: false,
        impasto: true,
        impasto_depth: 0.5,
        // The artist's defaults (Depth 1 / Body 0 / Smoothing 1, Enio 2026-07-12) are for PAINTING; a
        // fixture that inherited them would be asserting about the settle blur and the round profile
        // in gates that are about neither. Pin the two that would blur the claim, per-gate.
        impasto_smoothing: 0.0,
        impasto_body: 1.0,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t
}

// ── Impasto (#16) — the light pass ────────────────────────────────────────────────────────────────

/// The composited, LIT preview.
fn lit(t: &mut PainterTool) -> Vec<u8> {
    let (rgba, _, _) = t.take_preview_arc().expect("a preview");
    (*rgba).clone()
}

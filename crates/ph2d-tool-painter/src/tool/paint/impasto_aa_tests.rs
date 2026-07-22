//! Screen-space anti-aliasing of the impasto silhouette (the impasto half of BUGS #16, Enio
//! 2026-07-20 — thin impasto strokes shared the watercolor's "borda dura pixelada").
//!
//! The impasto FILM (`height_film::film_of`) hardens the soft falloff silhouette so the paint is
//! opaque right up to where the body ends — a palette-knife edge by design. Its transition band is a
//! fixed interval of `t = distance/radius`, so its screen width shrinks with the radius and a thin
//! stroke's edge snaps to a binary stair-step. Unlike the watercolor (whose optical model saturates),
//! the impasto pigment alpha COMPOSITES LINEARLY — so the film's fractional texel-area coverage can
//! feed the same `w` both consumers already read, through the same door, and pigment + light stay in
//! agreement about where the paint ends.
//!
//! Gates render real strokes; the hard mode (checkbox off) is pinned byte-for-byte to the pre-AA
//! fingerprints captured before the AA existed.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A `PainterTool` over a white opaque raster with an IMPASTO brush (the product's arm: `impasto`
/// on + the Sphere falloff the toggle installs), black pigment, given radius.
fn imp_tool(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: radius,
        impasto: true,
        falloff: Falloff::Sphere,
        color: [0.0, 0.0, 0.0],
        ..Default::default()
    };
    let seed = t.paint.brush;
    for slot in &mut t.paint.brush_by_mode {
        *slot = seed;
    }
    t
}

fn imp_stroke(size: u32, radius: f32, pts: &[[f32; 2]]) -> Vec<u8> {
    let mut t = imp_tool(size, radius);
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
    t.canvas_rgba.to_vec()
}

fn lum(canvas: &[u8], size: u32, x: u32, y: u32) -> u8 {
    let i = ((y * size + x) * 4) as usize;
    canvas[i].max(canvas[i + 1]).max(canvas[i + 2])
}

fn canvas_hash(canvas: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in canvas {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

// MEASUREMENT note: the thin-stroke edge profiles + whole-canvas fingerprints below became the
// hard-mode (checkbox OFF) oracles; the standalone measurement scaffold was retired.

/// RED-FIRST product gate (the impasto half of Enio's report): a thin impasto stroke's outer edge
/// must CLIMB, never jump paper→solid in one texel. Pre-AA every radius was binary (r=10 jumped
/// `255 → 0`, 255 bytes in one texel; r=20 `255 → 226 → 2`, 224); with the film's fractional area
/// the profiles climb `255 → 115 → 0` (r4) … `255 → 182 → 55 → 3` (r20).
#[test]
fn a_thin_impasto_stroke_edge_climbs_instead_of_jumping() {
    for r in [4.0f32, 6.0, 10.0, 20.0] {
        let size = 128u32;
        let cy = 64.0;
        let pts = [[24.0, cy], [104.0, cy]];
        let c = imp_stroke(size, r, &pts);
        let y0 = 64 - (r as u32) - 6;
        let prof: Vec<u8> = (y0..64).map(|y| lum(&c, size, 64, y)).collect();
        let dark_at = prof
            .iter()
            .enumerate()
            .min_by_key(|&(_, &l)| l)
            .map(|(i, _)| i)
            .unwrap();
        let max_step = prof[..=dark_at]
            .windows(2)
            .map(|w| (i32::from(w[0]) - i32::from(w[1])).abs())
            .max()
            .unwrap();
        assert!(
            max_step < 160,
            "radius {r}: impasto outer edge must climb, not jump (max step {max_step}, profile {prof:?})"
        );
    }
}

/// The hard mode (checkbox off) is the pre-AA render **byte-for-byte** — four whole-canvas
/// fingerprints captured before the AA existed, one per radius. And smooth is the default.
#[test]
fn impasto_smooth_edges_off_is_the_pre_aa_render_byte_for_byte() {
    assert!(
        BrushSpec::default().impasto_smooth_edges,
        "Smooth Edges must be the impasto default"
    );
    for (r, expect) in [
        (4.0f32, 0xb8cdb68d5b78af05u64),
        (6.0, 0x18f7c0a2b10064b5),
        (10.0, 0x4c1f656d3af21b05),
        (20.0, 0x258096fd961a7d3d),
    ] {
        let size = 128u32;
        let cy = 64.0;
        let pts = [[24.0, cy], [104.0, cy]];
        let mut t = imp_tool(size, r);
        t.paint.brush.impasto_smooth_edges = false;
        for slot in &mut t.paint.brush_by_mode {
            slot.impasto_smooth_edges = false;
        }
        t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
        for p in &pts[1..] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
        assert_eq!(
            canvas_hash(&t.canvas_rgba),
            expect,
            "radius {r}: impasto Smooth Edges OFF must render the pre-AA composite byte-for-byte"
        );
    }
}

/// The film invariant survives the AA: on a GRAINLESS impasto stroke, pigment support and the
/// light's coverage support are the SAME set — a texel the light weighs carries pigment, and a texel
/// with pigment is weighed (both halves take the fraction from the same door, `FilmAa::film_at`).
/// (With a Grain, a rim texel in a deep valley can quantize its pigment away — the bare-canvas gate
/// covers that class with its neighbourhood test.)
#[test]
fn the_films_two_halves_agree_on_the_rim() {
    let size = 128u32;
    let pts = [[24.0, 64.0], [104.0, 64.0]];
    let mut t = imp_tool(size, 8.0);
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    t.on_canvas_pointer(cp(pts[1], PointerPhase::Move));
    t.on_canvas_pointer(cp(pts[1], PointerPhase::Up));
    let active = t.layers.active().expect("layer");
    let cov = t
        .covers
        .get(&active)
        .map(|c| c.as_ref().clone())
        .expect("impasto stroke has a covers plane");
    let mut disagree = 0u32;
    for (i, &c) in cov.iter().enumerate().take(size as usize * size as usize) {
        let has_pigment = t.canvas_rgba[i * 4 + 1] != 255; // black over white: green drops
        let has_cover = c > 0;
        if has_pigment != has_cover {
            disagree += 1;
        }
    }
    assert_eq!(
        disagree, 0,
        "{disagree} texels where pigment and the light's coverage disagree about where the paint ends"
    );
}

/// Tool half of the checkbox seam (the panel half is the two `PAINTER_IMPASTO_CLICKS` sweeps): the
/// forwarded Click reaches `route_brush_impasto_event` and flips the mode both ways.
#[test]
fn the_impasto_smooth_edges_click_flips_the_mode() {
    let mut t = imp_tool(16, 4.0);
    assert!(t.paint.brush.impasto_smooth_edges, "default is smooth");
    let ev = ph2d_editor_core::tool::PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_IMPASTO_SMOOTH_EDGES,
    );
    assert!(t.route_brush_impasto_event(&ev), "click must be consumed");
    assert!(
        !t.paint.brush.impasto_smooth_edges,
        "first click turns the AA off"
    );
    assert!(t.route_brush_impasto_event(&ev));
    assert!(
        t.paint.brush.impasto_smooth_edges,
        "second click turns it back on"
    );
}

/// Toggling **Repeat Image** hands the preview slot to the other producer (`gpu_eligible` refuses
/// while the tile preview is on), so the toggle must leave the composite DIRTY — the incoming
/// producer publishes the very next frame. Without it the handoff waited for the next stroke: the
/// painting vanished on the toggle and the tiles only appeared after the first dab (Enio 2026-07-20).
/// The fixture is the reported shape exactly: paint, drain (clean), toggle → a drain must produce.
#[test]
fn toggling_repeat_image_reprimes_the_preview() {
    let mut t = imp_tool(64, 6.0);
    t.on_canvas_pointer(cp([20.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Up));
    assert!(
        t.take_preview_arc().is_some(),
        "fixture: the stroke dirtied the preview"
    );
    assert!(t.take_preview_arc().is_none(), "fixture: drained clean");
    t.toggle_repeat_image();
    assert!(
        t.take_preview_arc().is_some(),
        "toggling Repeat Image ON must re-prime the preview (the CPU producer takes over NOW, \
         not at the next stroke)"
    );
    assert!(t.take_preview_arc().is_none(), "drained clean again");
    t.toggle_repeat_image();
    assert!(
        t.take_preview_arc().is_some(),
        "toggling it OFF hands the slot back — the outgoing frame must re-prime just the same"
    );
}

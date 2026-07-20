//! Screen-space anti-aliasing of the watercolor silhouette (thin-stroke hard-edge fix, Enio 2026-07-20).
//!
//! The composite hardens the feathered coverage through `smoothstep(SS0, SS1, cov)`
//! ([`super::watercolor_render`]). That window lives in **coverage** units, so its transition width in
//! screen texels shrinks with the brush radius; on a thin stroke the silhouette crosses it in well under
//! a texel and snaps to a binary, stair-stepped edge (Enio's report: thin watercolor strokes have a
//! "borda dura pixelada" while the plain painter — whose alpha is the raw soft falloff — stays smooth at
//! any size). [`super::watercolor_field::aa_hardened_coverage`] reconstructs that sub-texel edge by
//! averaging the hardened coverage over a sub-texel grid, gated to steep edges so a thick stroke is
//! byte-identical.
//!
//! These gates render real strokes (over a white opaque raster, so the silhouette AA shows as grey rim
//! texels in RGB, not in alpha) — they contain the phenomenon, and pin the byte-identity of thick strokes.

use super::watercolor_field::{aa_hardened_coverage, sample_bilinear, smoothstep};
use super::*;
use ph2d_editor_core::tool::RasterEditTool;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A `PainterTool` over a white opaque `size`×`size` raster with a black brush of the given radius.
/// `watercolor` selects the wet-media render-path; `warp` sets the organic-edge amplitude (6.0 = default).
fn wc_tool(size: u32, radius: f32, watercolor: bool, warp: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: radius,
        watercolor,
        color: [0.0, 0.0, 0.0],
        warp,
        ..Default::default()
    };
    let seed = t.paint.brush;
    for slot in &mut t.paint.brush_by_mode {
        *slot = seed;
    }
    t
}

/// Paint a straight stroke through `pts` (image-space) and return the composited canvas.
fn wc_stroke(size: u32, radius: f32, watercolor: bool, warp: f32, pts: &[[f32; 2]]) -> Vec<u8> {
    let mut t = wc_tool(size, radius, watercolor, warp);
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

/// FNV-1a over the whole canvas — a fingerprint for the byte-identity (thick-stroke) promise.
fn canvas_hash(canvas: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in canvas {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Count "cliff" adjacencies — a paper texel (`≥ PAPER`) directly touching a solid-ish texel
/// (`≤ SOLID`) with no intermediate between them (horizontal + vertical pairs). This IS the aliased
/// hard edge; anti-aliasing inserts a grey texel and removes the cliff. Fewer = smoother.
fn count_cliffs(canvas: &[u8], size: u32) -> usize {
    const PAPER: i32 = 230;
    const SOLID: i32 = 60;
    let g = |x: u32, y: u32| lum(canvas, size, x, y) as i32;
    let mut cliffs = 0;
    for y in 0..size {
        for x in 0..size {
            let l = g(x, y);
            if x + 1 < size {
                let r = g(x + 1, y);
                if (l >= PAPER && r <= SOLID) || (l <= SOLID && r >= PAPER) {
                    cliffs += 1;
                }
            }
            if y + 1 < size {
                let d = g(x, y + 1);
                if (l >= PAPER && d <= SOLID) || (l <= SOLID && d >= PAPER) {
                    cliffs += 1;
                }
            }
        }
    }
    cliffs
}

/// RED-FIRST product gate: a thin diagonal watercolor stroke (radius 4, default warp) must have an
/// anti-aliased edge — few paper↔solid cliffs. Before the fix the single-sample hardening posts ~290
/// cliffs (a stair-stepped binary edge); the reconstruction brings it well under half of that.
#[test]
fn a_thin_watercolor_stroke_edge_is_antialiased() {
    let pts = [[24.0, 24.0], [104.0, 104.0]];
    let canvas = wc_stroke(128, 4.0, true, 6.0, &pts);
    let cliffs = count_cliffs(&canvas, 128);
    assert!(
        cliffs < 180,
        "thin watercolor diagonal must be anti-aliased (cliffs = {cliffs}; the un-reconstructed \
         single-sample edge posts ~290)"
    );
}

/// The "sem prejuízo ao que já conseguimos" promise: a THICK watercolor stroke (radius 40) is
/// **byte-identical** to the pre-fix render — its shallow coverage rim never trips the gradient gate,
/// so every texel takes the single-sample path. Fingerprint pinned from the pre-fix code.
#[test]
fn a_thick_watercolor_stroke_is_byte_identical() {
    let pts = [[70.0, 128.0], [186.0, 128.0]];
    let canvas = wc_stroke(256, 40.0, true, 6.0, &pts);
    assert_eq!(
        canvas_hash(&canvas),
        0xc5ebf8cf645fb6f6,
        "the thick watercolor stroke must render byte-for-byte as before the AA fix"
    );
}

/// Mechanism, unit-level: on a SHALLOW field (transition ≥ ~2 texels — the thick-stroke rim) the
/// reconstruction is **byte-identical** to the plain `smoothstep(e0, e1, sample_bilinear(…))` — the
/// gradient gate keeps the single-sample path. A 21-wide field ramping 0→1 has grad 0.05 ≪ band/2.
#[test]
fn aa_hardened_coverage_is_identical_on_a_shallow_edge() {
    let field: Vec<f32> = (0..21).map(|i| i as f32 / 20.0).collect(); // grad 0.05 per texel
    for i in 40..160 {
        let sx = i as f32 * 0.1; // 4.0 .. 15.9, away from the clamped borders
        let aa = aa_hardened_coverage(&field, 21, 1, sx, 0.0, 0.12, 0.60);
        let plain = smoothstep(0.12, 0.60, sample_bilinear(&field, 21, 1, sx, 0.0));
        assert_eq!(aa.to_bits(), plain.to_bits(), "shallow edge must be untouched at sx={sx}");
    }
}

/// Mechanism, unit-level: on a STEEP field (a 0→1 step within one texel) the reconstruction produces a
/// genuinely intermediate coverage at the boundary — where the single-sample hardening snaps to 0 or 1.
/// This is the anti-aliasing the thin-stroke silhouette was missing.
#[test]
fn aa_hardened_coverage_softens_a_steep_step() {
    // A vertical field: two paper rows, then a hard step to solid — the sub-texel edge a thin rim makes.
    let field = [0.0f32, 0.0, 1.0, 1.0]; // 1×4 down the y axis
    // At the boundary texel (sy = 1.0, the last "outside" row) the single sample is exactly 0.
    let plain = smoothstep(0.12, 0.60, sample_bilinear(&field, 1, 4, 0.0, 1.0));
    assert_eq!(plain, 0.0, "single sample snaps the boundary to 0");
    let aa = aa_hardened_coverage(&field, 1, 4, 0.0, 1.0, 0.12, 0.60);
    assert!(
        aa > 0.02,
        "the reconstruction must give the boundary texel fractional coverage (got {aa})"
    );
}

//! Watercolor **canvas-anchored noise** — the deterministic integer-hash value noise behind the
//! RaggedEdge warp, the built-in paper tooth and the backrun jag, plus the [`NoiseTile`] sprite-wrap
//! that makes every one of them SEAMLESS across the tile seam (doc 13 #2). Split from
//! `watercolor_field.rs` for the workspace LOC cap; re-exported there so callers keep using
//! `watercolor_field::{value_noise_tiled, warp_axis, NoiseTile, …}`. HR-5: transcendental-free.

// ── Deterministic value noise (integer hash; HR-5 transcendental-free) ───────────────────────────────
// Distinct seeds keep the two warp axes + the paper grain decorrelated (else the boundary would ripple
// along the diagonal and the granulation would track the warp).
pub(super) const SEED_WARP_X_A: u32 = 0x1111_1111;
pub(super) const SEED_WARP_X_B: u32 = 0x2222_2222;
pub(super) const SEED_WARP_Y_A: u32 = 0x3333_3333;
pub(super) const SEED_WARP_Y_B: u32 = 0x4444_4444;
pub(super) const SEED_GRAIN: u32 = 0x5555_5555;
pub(super) const SEED_GRAIN_FINE: u32 = 0x6666_6666;

/// Smoothstep-in-`[0,1]` fade (the value-noise interpolation weight; a cubic, so no transcendental).
#[inline]
pub(super) fn smooth01(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Integer hash of a lattice cell → `[0, 1)` (a `lowbias32`-style avalanche; deterministic + fast).
#[inline]
pub(super) fn hash2(ix: i32, iy: i32, seed: u32) -> f32 {
    let mut h = (ix as u32).wrapping_mul(0x8DA6_B343)
        ^ (iy as u32).wrapping_mul(0xD816_3841)
        ^ seed.wrapping_mul(0x1B56_C4E9);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// Sprite-tiling context for canvas-anchored noise (doc 13 #2): the axis PERIODS (the sprite w/h in
/// px) and which axes wrap. When an axis tiles, [`value_noise_tiled`] wraps its lattice at a WHOLE
/// number of cells spanning the period, so the field is periodic (`noise(x) == noise(x + period)`) and
/// every canvas-anchored texture (RaggedEdge warp, paper granulation, backrun jag) is seamless across
/// the sprite seam. [`NoiseTile::NONE`] (no axis) ⇒ the plain non-periodic noise, byte-identical.
#[derive(Clone, Copy)]
pub(super) struct NoiseTile {
    period: [f32; 2],
    on: [bool; 2],
}

impl NoiseTile {
    /// No axis tiles — the historical non-periodic noise.
    pub(super) const NONE: Self = Self {
        period: [0.0, 0.0],
        on: [false, false],
    };

    /// From the sprite size (px) + the per-axis Tiling flags.
    #[inline]
    pub(super) fn new(size: (usize, usize), on: [bool; 2]) -> Self {
        Self {
            period: [size.0 as f32, size.1 as f32],
            on,
        }
    }

    /// Effective cell + wrap period (in WHOLE cells) for axis `i`: rounds the cell count so an integer
    /// number of cells spans the period exactly — the seam only glues on a whole cell count, and the
    /// effective cell is nudged to `period/cells` so pixel `period` lands on cell `0` again. A non-tiled
    /// axis returns `(cell, 0)` = the input cell verbatim, no wrap (⇒ byte-identical to the old noise).
    #[inline]
    fn axis(self, cell: f32, i: usize) -> (f32, i32) {
        if self.on[i] && self.period[i] > 0.0 {
            let cells = (self.period[i] / cell).round().max(1.0);
            (self.period[i] / cells, cells as i32)
        } else {
            (cell, 0)
        }
    }
}

/// Wrap a lattice index into `[0, p)` when `p > 0` (a tiled axis); `p == 0` (non-tiled) ⇒ verbatim.
#[inline]
fn wrap_cell(i: i32, p: i32) -> i32 {
    if p > 0 { i.rem_euclid(p) } else { i }
}

/// Bilinear value noise with `cell`-px features at `(x, y)` → `[0, 1]` (wet_edges `valueNoise`),
/// optionally PERIODIC per axis (`tile`) so a tiled sprite is seamless (doc 13 #2). `NoiseTile::NONE`
/// ⇒ the plain non-periodic noise (effective cell = `cell`, no wrap — the historical field).
#[inline]
pub(super) fn value_noise_tiled(x: f32, y: f32, cell: f32, seed: u32, tile: NoiseTile) -> f32 {
    let (cx, px) = tile.axis(cell, 0);
    let (cy, py) = tile.axis(cell, 1);
    let fx = x / cx;
    let fy = y / cy;
    let x0 = fx.floor();
    let y0 = fy.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let sx = smooth01(fx - x0);
    let sy = smooth01(fy - y0);
    let a = hash2(wrap_cell(ix, px), wrap_cell(iy, py), seed);
    let b = hash2(wrap_cell(ix + 1, px), wrap_cell(iy, py), seed);
    let c = hash2(wrap_cell(ix, px), wrap_cell(iy + 1, py), seed);
    let d = hash2(wrap_cell(ix + 1, px), wrap_cell(iy + 1, py), seed);
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

/// Fractal displacement field (two octaves, cells 22/8 px) in `[-1, 1]` for a warp axis (wet_edges
/// warp). `tile` makes it seamless across the sprite seam (doc 13 #2); `NONE` ⇒ non-periodic (old).
#[inline]
pub(super) fn warp_axis(x: f32, y: f32, sa: u32, sb: u32, tile: NoiseTile) -> f32 {
    (value_noise_tiled(x, y, 22.0, sa, tile) * 0.65
        + value_noise_tiled(x, y, 8.0, sb, tile) * 0.35
        - 0.5)
        * 2.0
}

/// Both warp axes at `(x, y)` as a raw displacement `(dx, dy) ∈ [-1, 1]²` (pre-amplitude) — the two
/// decorrelated `warp_axis` reads the composite adds to the sample position; seamless when `tile` wraps.
#[inline]
pub(super) fn warp_offset(x: f32, y: f32, tile: NoiseTile) -> (f32, f32) {
    (
        warp_axis(x, y, SEED_WARP_X_A, SEED_WARP_X_B, tile),
        warp_axis(x, y, SEED_WARP_Y_A, SEED_WARP_Y_B, tile),
    )
}

/// Paper-tooth granulation height at `(x, y)` in `[0, 1]` — the built-in fallback when no Paper
/// slot is set. TWO octaves (5 px + 2.5 px, doc 12 GRAN-3): the audit measured the old single
/// octave as mid-band and uniform (good) but mono-scale full-range — the "digital" side of the
/// default look; the fine second octave breaks the single-frequency signature.
#[inline]
pub(super) fn paper_height(x: f32, y: f32, tile: NoiseTile) -> f32 {
    0.65 * value_noise_tiled(x, y, 5.0, SEED_GRAIN, tile)
        + 0.35 * value_noise_tiled(x, y, 2.5, SEED_GRAIN_FINE, tile)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Value noise is deterministic, in `[0, 1]`, and varies across cells (not a constant field).
    #[test]
    fn value_noise_is_deterministic_and_bounded() {
        let a = value_noise_tiled(12.3, 45.6, 5.0, SEED_GRAIN, NoiseTile::NONE);
        let b = value_noise_tiled(12.3, 45.6, 5.0, SEED_GRAIN, NoiseTile::NONE);
        assert_eq!(a, b, "same input ⇒ same value (deterministic)");
        assert!((0.0..=1.0).contains(&a), "in range");
        let c = value_noise_tiled(112.3, 245.6, 5.0, SEED_GRAIN, NoiseTile::NONE);
        assert!(
            (a - c).abs() > 1e-4,
            "distant cells differ (it actually varies)"
        );
    }

    /// **Tiling (doc 13 #2): the canvas-anchored noise is SEAMLESS across the sprite period.** A tiled
    /// axis wraps the lattice at a whole number of cells spanning the period, so `noise(x) == noise(x +
    /// period)` for every cell size the wash uses (warp 22/8, paper 5/2.5, jag) — the RaggedEdge lines up
    /// at the seam. `NoiseTile::NONE` (Tiling off) stays NON-periodic, guarding the byte-identical path.
    #[test]
    fn tiled_noise_is_seamless_across_the_sprite_period() {
        let (pw, ph) = (64.0f32, 48.0f32);
        let tile = NoiseTile::new((pw as usize, ph as usize), [true, true]);
        for &cell in &[22.0f32, 8.0, 5.0, 2.5] {
            for k in 0..53 {
                let x = k as f32 * 1.7;
                let y = k as f32 * 1.1;
                let vx = value_noise_tiled(x, y, cell, SEED_GRAIN, tile);
                let vx2 = value_noise_tiled(x + pw, y, cell, SEED_GRAIN, tile);
                assert!(
                    (vx - vx2).abs() < 1e-5,
                    "X seam discontinuous (cell={cell}, k={k}): {vx} vs {vx2}"
                );
                let vy = value_noise_tiled(x, y + ph, cell, SEED_GRAIN, tile);
                assert!(
                    (vx - vy).abs() < 1e-5,
                    "Y seam discontinuous (cell={cell}, k={k}): {vx} vs {vy}"
                );
            }
        }
        // warp_axis (the RaggedEdge boundary) wraps too — the visible bug in the smoke.
        for k in 0..64 {
            let y = k as f32 * 0.9;
            let w = warp_axis(3.0, y, SEED_WARP_X_A, SEED_WARP_X_B, tile);
            let w2 = warp_axis(3.0 + pw, y, SEED_WARP_X_A, SEED_WARP_X_B, tile);
            assert!((w - w2).abs() < 1e-5, "warp seam discontinuous (k={k})");
        }
        // NONE must NOT be periodic (the historical non-tiled noise — no accidental tiling).
        let none = NoiseTile::NONE;
        let differs = (0..64).any(|k| {
            let y = k as f32;
            (value_noise_tiled(1.0, y, 8.0, SEED_GRAIN, none)
                - value_noise_tiled(1.0 + pw, y, 8.0, SEED_GRAIN, none))
            .abs()
                > 1e-4
        });
        assert!(
            differs,
            "NoiseTile::NONE must stay non-periodic (byte-identical path)"
        );
    }
}

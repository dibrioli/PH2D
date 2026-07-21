//! Brush stamp (port of `brush.js`, SPEC §7): radial falloff x bristle texture.
//!
//! Porosity is load-bearing. The bristle texture is ~94% near-zero felt with
//! ~950 tiny bright tips, so a painted blob is a SIEVE of wet channels, not a
//! uniform film: the solver can nucleate discrete rivulets at its edge and the
//! reservoir keeps its pigment. The texture is anchored to the dab center in
//! canvas pixels (1 texel = 1 px at every brush size), so grain has constant
//! physical size and the tips streak with the stroke.
//!
//! Two tip populations, BOTH placed uniformly at random over the whole tile
//! (the SPEC §7 uniformity clause is binding: any deliberate region-based
//! placement makes the stroke profile a function of the brush radius):
//! FINE tips are the sub-gate sieve (visual felt between lanes); CORE tips
//! (a fraction of the count) are the gate-clearing bristle lanes.

use crate::jsmath::{js_round, wrap_mask};
use crate::rng::Rng;

pub const BRISTLE_SIZE: usize = 128;
const BMASK: i32 = BRISTLE_SIZE as i32 - 1;
const BRISTLE_SEED: u32 = 0x7a5b_0010;

// Tip-distribution calibration (SPEC §7, deposit-integral bullet). Tuned so
// the defaults hit: tile mean ~0.02, ~6-7% of texels > 0.04, origin deposit
// integral ~4.0 with 13-23 gate-clearing texels, 3x3-grid integral mean
// 4.0 +-25% with min >= 1.5, and the §18.11 budgets at both r=12 and r=25.
const FINE_RADIUS_MIN: f64 = 0.4;
const FINE_RADIUS_SPAN: f64 = 0.4;
const FINE_STRENGTH_MIN: f64 = 0.03;
const FINE_STRENGTH_SPAN: f64 = 0.05; // sub-gate grain: visual tooth, no deposit
const CORE_FRACTION: f64 = 0.26; // share of the bristle count drawn as core tips
const CORE_RADIUS_MIN: f64 = 0.9;
const CORE_RADIUS_SPAN: f64 = 0.6;
const CORE_STRENGTH_MIN: f64 = 0.16;
const CORE_STRENGTH_SPAN: f64 = 0.16;
const TIP_PLATEAU: f64 = 0.55; // fraction of the radius at full strength

/// The paint-group knobs the texture build reads.
#[derive(Clone, Copy)]
pub struct BristleKnobs {
    pub felt: f64,
    pub bristle_count: f64,
    pub bristle_strength: f64,
    pub bristle_size: f64,
}

impl Default for BristleKnobs {
    fn default() -> Self {
        BristleKnobs { felt: 0.01, bristle_count: 950.0, bristle_strength: 1.0, bristle_size: 1.0 }
    }
}

/// Build the 128x128 bristle tile. Deterministic: same knobs -> same texture.
/// - felt: near-zero noise floor between the hairs, mean = felt knob;
/// - tips: plateau-topped round dots (fine sieve + core blobs), max-blended.
pub fn build_bristle_texture(knobs: BristleKnobs) -> Vec<f32> {
    let felt = knobs.felt;
    let count = js_round(knobs.bristle_count).max(0.0) as i64;
    let tip_strength = knobs.bristle_strength;
    let tip_size = knobs.bristle_size;
    let mut rng = Rng::new(BRISTLE_SEED);
    let mut t = vec![0.0f32; BRISTLE_SIZE * BRISTLE_SIZE];
    for v in t.iter_mut() {
        *v = (rng.next() * 2.0 * felt) as f32; // mean = felt
    }
    let inv_fall = 1.0 / (1.0 - TIP_PLATEAU);
    let core_count = js_round(count as f64 * CORE_FRACTION) as i64;
    for b in 0..count {
        let rr = rng.next();
        let rs = rng.next();
        let is_core = b < core_count;
        let radius = if is_core {
            CORE_RADIUS_MIN + rr * CORE_RADIUS_SPAN
        } else {
            FINE_RADIUS_MIN + rr * FINE_RADIUS_SPAN
        } * tip_size;
        let strength = if is_core {
            CORE_STRENGTH_MIN + rs * CORE_STRENGTH_SPAN
        } else {
            FINE_STRENGTH_MIN + rs * FINE_STRENGTH_SPAN
        } * tip_strength;
        // Uniform random placement for EVERY tip — no reserved regions, no
        // re-draws (SPEC §7 uniformity clause).
        let cx = rng.next() * BRISTLE_SIZE as f64;
        let cy = rng.next() * BRISTLE_SIZE as f64;
        let reach = radius.ceil() as i32;
        let inv = 1.0 / (radius + 0.0001);
        let x0 = cx.floor() as i32 - reach;
        let x1 = cx.floor() as i32 + reach;
        let y0 = cy.floor() as i32 - reach;
        let y1 = cy.floor() as i32 + reach;
        for yy in y0..=y1 {
            for xx in x0..=x1 {
                let dx = xx as f64 - cx;
                let dy = yy as f64 - cy;
                let d = (dx * dx + dy * dy).sqrt() * inv;
                if d >= 1.0 {
                    continue;
                }
                // Plateau top with a smoothstep shoulder: round, not skirt-heavy.
                let mut w = 1.0;
                if d > TIP_PLATEAU {
                    let u = (d - TIP_PLATEAU) * inv_fall;
                    w = 1.0 - u * u * (3.0 - 2.0 * u);
                }
                let v = (strength * w) as f32;
                let idx = wrap_mask(yy, BMASK) * BRISTLE_SIZE + wrap_mask(xx, BMASK);
                if v > t[idx] {
                    t[idx] = v; // max-blend keeps tips crisp
                }
            }
        }
    }
    t
}

/// Nearest-texel wrap sample, offsets in canvas px from the dab center.
#[inline]
pub fn sample_bristle(tex: &[f32], dx_px: f64, dy_px: f64) -> f64 {
    tex[wrap_mask(js_round(dy_px) as i32, BMASK) * BRISTLE_SIZE
        + wrap_mask(js_round(dx_px) as i32, BMASK)] as f64
}

/// Radial falloff: a = (1-d)*hardness; 1 when a>=1, else smoothstep(a).
#[inline]
pub fn radial_falloff(d: f64, hardness: f64) -> f64 {
    let a = (1.0 - d) * hardness;
    if a >= 1.0 {
        return 1.0;
    }
    if a <= 0.0 {
        return 0.0;
    }
    a * a * (3.0 - 2.0 * a)
}

/// Brush footprint shapes. Flat/fan are ellipses in the stroke frame
/// (along/across axes in units of the brush radius).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrushShape {
    Round,
    Flat,
    Fan,
}

impl BrushShape {
    fn axes(self) -> Option<[f64; 2]> {
        match self {
            BrushShape::Round => None,
            BrushShape::Flat => Some([1.0, 0.35]),
            BrushShape::Fan => Some([1.4, 0.5]),
        }
    }
}

/// Iterate every stamp pixel of one dab, restricted to the brushable area
/// [2..W-1] x [2..H-1]. Calls cb(i, x, y, falloff, texture) — the caller
/// multiplies them (plus its own intensity/clamps); they stay separate
/// because the wet-edge-softening extension keys on the falloff alone.
///
/// Takes the grid DIMENSIONS, not the grid: the callback is where deposits
/// mutate cell arrays, so the iterator must not hold a borrow of them.
///
/// Flat/fan is an ellipse in the stroke frame (dir = stroke direction unit
/// vector); with no direction yet it falls back to the disc. Texture coords
/// for shaped brushes are the ROTATED px offsets, so the grain rides the
/// stroke frame.
#[allow(clippy::too_many_arguments)]
pub fn for_each_stamp_pixel(
    s: usize,
    grid_w: usize,
    grid_h: usize,
    tex: &[f32],
    cx: f64,
    cy: f64,
    r: f64,
    hardness: f64,
    shape: BrushShape,
    dir_x: f64,
    dir_y: f64,
    mut cb: impl FnMut(usize, i32, i32, f64, f64),
) {
    let axes = shape.axes();
    let has_dir = axes.is_some() && (dir_x != 0.0 || dir_y != 0.0);
    let reach = if has_dir { r * axes.unwrap()[0] } else { r };
    let x0 = ((cx - reach).ceil() as i32).max(2);
    let x1 = ((cx + reach).floor() as i32).min(grid_w as i32 - 1);
    let y0 = ((cy - reach).ceil() as i32).max(2);
    let y1 = ((cy + reach).floor() as i32).min(grid_h as i32 - 1);
    if x1 < x0 || y1 < y0 {
        return;
    }
    let inv_r = 1.0 / r;
    let (mut inv_a, mut inv_b) = (0.0, 0.0);
    if has_dir {
        let ax = axes.unwrap();
        inv_a = 1.0 / (ax[0] * r);
        inv_b = 1.0 / (ax[1] * r);
    }
    for y in y0..=y1 {
        let mut i = x0 as usize + y as usize * s;
        for x in x0..=x1 {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let (d2, tx, ty);
            if has_dir {
                let along = dx * dir_x + dy * dir_y;
                let across = -dx * dir_y + dy * dir_x;
                let na = along * inv_a;
                let nb = across * inv_b;
                d2 = na * na + nb * nb;
                tx = along;
                ty = across;
            } else {
                let nx = dx * inv_r;
                let ny = dy * inv_r;
                d2 = nx * nx + ny * ny;
                tx = dx;
                ty = dy;
            }
            if d2 < 1.0 {
                // reject on squared distance before any sqrt
                let fall = radial_falloff(d2.sqrt(), hardness);
                if fall > 0.0 {
                    let texv = sample_bristle(tex, tx, ty);
                    if texv > 0.0 {
                        cb(i, x, y, fall, texv);
                    }
                }
            }
            i += 1;
        }
    }
}

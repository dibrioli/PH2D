//! **Impasto** — the deposit settling under its own weight, and the two constants that bound it.
//!
//! Split from [`super::impasto`] for the workspace file-LOC cap: that module is the tool-side plumbing
//! (when the relief is deposited, committed, re-derived, dirtied); this one is the material behaviour.

use super::Region;

/// Below this, a change in relief cannot move an output byte — the threshold the commit's dirty-rect
/// diff uses.
/// A 16-bit tick of the height range: finer than any 8-bit composite can show, so nothing visible is
/// ever missed, and the float noise of a re-derive that landed on the same field costs no repaint.
pub(super) const RELIEF_EPS: f32 = 1.0 / 65_536.0; // CLAMP-OK: sub-visible threshold, not a design value

/// Radius, in pixels, of the settling blur at Smoothing = 1. Thick paint slumps a little, not into a
/// puddle — past a few pixels the ridges stop reading as brush-marks. // CLAMP-OK
const SETTLE_MAX_PX: f32 = 4.0;

/// How far the settle can push relief beyond the paint that made it — the pad that turns a stroke's dab
/// footprint into the window the commit is allowed to work in. It is exactly [`SETTLE_MAX_PX`] (the box
/// blur's largest radius), so a window grown by it has a border of zeros, and the blur of zeros is zero.
/// That is what makes the crop **byte-identical** rather than an approximation. // CLAMP-OK
pub(super) const SETTLE_REACH_PX: u32 = SETTLE_MAX_PX as u32;

/// Take ownership of a shared plane: free when nobody else holds it (the common case), a copy when an
/// undo snapshot does. Copy-on-write, and the `Arc` is exactly what buys it — the snapshot that used to
/// deep-clone 80 MB of relief + coverage per stroke at 4096` now bumps a refcount.
pub(super) fn owned<T: Clone>(a: std::sync::Arc<Vec<T>>) -> Vec<T> {
    std::sync::Arc::try_unwrap(a).unwrap_or_else(|a| a.as_ref().clone())
}

/// Visit every canvas index inside `rect` on a `w`-wide canvas, row by row.
#[inline]
pub(super) fn for_each_in(rect: Region, w: u32, mut f: impl FnMut(usize)) {
    for y in rect.y..rect.y + rect.h {
        let row = (y as usize) * (w as usize);
        for x in rect.x..rect.x + rect.w {
            f(row + x as usize);
        }
    }
}

/// Let a height field **settle** under its own weight: a separable box blur, applied in place.
///
/// Binomial-ish by repetition (two box passes ≈ a triangle kernel), which is what a viscous medium
/// relaxing actually looks like — and it is transcendental-free (HR-5) and O(n) in the radius, unlike
/// a true Gaussian. The blur is signed, so a carved groove softens exactly as a raised ridge does.
pub(super) fn settle(field: &mut [f32], w: u32, h: u32, amount: f32) {
    let r = (amount.clamp(0.0, 1.0) * SETTLE_MAX_PX).round() as i64;
    if r < 1 || w == 0 || h == 0 || field.len() < (w as usize) * (h as usize) {
        return;
    }
    let (wi, hi) = (w as i64, h as i64);
    let mut tmp = vec![0.0f32; field.len()];
    let inv = 1.0 / (2 * r + 1) as f32;
    // Horizontal pass.
    for y in 0..hi {
        let row = (y * wi) as usize;
        for x in 0..wi {
            let mut sum = 0.0;
            for k in -r..=r {
                let sx = (x + k).clamp(0, wi - 1) as usize;
                sum += field[row + sx];
            }
            tmp[row + x as usize] = sum * inv;
        }
    }
    // Vertical pass.
    for y in 0..hi {
        for x in 0..wi {
            let mut sum = 0.0;
            for k in -r..=r {
                let sy = (y + k).clamp(0, hi - 1) as usize;
                sum += tmp[sy * (w as usize) + x as usize];
            }
            field[(y * wi + x) as usize] = sum * inv;
        }
    }
}

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
/// Two box passes ≈ a triangle kernel, which is what a viscous medium relaxing actually looks like — and
/// it is transcendental-free (HR-5). The blur is signed, so a carved groove softens exactly as a raised
/// ridge does.
pub(super) fn settle(field: &mut [f32], w: u32, h: u32, amount: f32) {
    let r = (amount.clamp(0.0, 1.0) * SETTLE_MAX_PX).round() as u32;
    box_blur(field, w, h, r);
}

/// Separable box blur of radius `r`, in place, clamping at the buffer's edge.
///
/// **Each output texel re-sums its window from scratch** — O(n·r), and deliberately so. The obvious
/// speed-up is a sliding window that carries the running sum from one texel to the next (O(n) at any
/// radius); it was written, and the byte-identity gate rejected it. A running sum accumulates float
/// rounding *along the row*, so its result depends on how far the row has run — and the whole crop of
/// the commit (§11) rests on the blur of a WINDOW being bit-for-bit the blur of the CANVAS. A faster
/// blur that silently made the window a different number was not faster, it was wrong; the window is
/// small, and the constant factor is the price of a property worth having.
pub(super) fn box_blur(field: &mut [f32], w: u32, h: u32, r: u32) {
    let (wi, hi) = (w as usize, h as usize);
    if r < 1 || wi == 0 || hi == 0 || field.len() < wi * hi {
        return;
    }
    let mut tmp = vec![0.0f32; wi * hi];
    // Horizontal.
    blur_rows(&field[..wi * hi], &mut tmp, wi, hi, r as usize);
    // Vertical — done as a HORIZONTAL pass on the transpose, then transposed back.
    //
    // The arithmetic is untouched — the same taps, summed in the same order — so this changes no bit of
    // the output; what it changes is the MEMORY. The obvious vertical pass reads `tmp[y*w + x]` for 2r+1
    // values of `y`: one cache line per tap, 25 of them per texel at the displacement's radius. That
    // stride was almost the whole cost of Push (**+22 ms on the pen-up at 4096², now +10**). Transposed,
    // every read is sequential. (What is GATED here is the property that depends on it: the blur of a
    // WINDOW is bit-for-bit the blur of the CANVAS, §11 — which is why a sliding sum, which drifts along
    // the row, was rejected and this, which does not, was not.)
    let mut t2 = vec![0.0f32; wi * hi];
    transpose(&tmp, &mut t2, wi, hi);
    let mut t3 = vec![0.0f32; wi * hi];
    blur_rows(&t2, &mut t3, hi, wi, r as usize);
    transpose(&t3, &mut field[..wi * hi], hi, wi);
}

/// One separable pass: box-blur every row of `src` (a `w × h` buffer) into `dst`, clamping at the edges.
/// Each output re-sums its own window, in a fixed order — so the result never depends on how far the row
/// has run (a sliding sum would drift, and the crop's byte-identity rests on it not drifting).
fn blur_rows(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: usize) {
    let inv = 1.0 / (2 * r + 1) as f32;
    let last = w - 1;
    for y in 0..h {
        let row = y * w;
        for x in 0..w {
            let mut sum = 0.0;
            for k in 0..=2 * r {
                let sx = (x + k).saturating_sub(r).min(last);
                sum += src[row + sx];
            }
            dst[row + x] = sum * inv;
        }
    }
}

/// Transpose a `w × h` buffer into an `h × w` one.
fn transpose(src: &[f32], dst: &mut [f32], w: usize, h: usize) {
    for y in 0..h {
        for x in 0..w {
            dst[x * h + y] = src[y * w + x];
        }
    }
}

/// The union of two dab dirty-rects (the deposit's and the displacement's — the latter reaches further).
pub(super) fn union_dirty(
    a: ph2d_painter_brush::dab::DirtyRect,
    b: ph2d_painter_brush::dab::DirtyRect,
) -> ph2d_painter_brush::dab::DirtyRect {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.w).max(b.x + b.w);
    let y1 = (a.y + a.h).max(b.y + b.h);
    ph2d_painter_brush::dab::DirtyRect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

//! **The Blob's footprint is a morphological CLOSING, not a dilation** (Enio, 2026-07-17: *"não protege as
//! bordas do traço que não deveriam nem ser tocados … os cruzamentos também são muito erodidos"*).
//!
//! The bounded ball (`sculpt_offset`) fixed the junction GASH, but a raw dilation grows the footprint the
//! SAME `ρ` in every direction — into the concave armpits (wanted: it fills the gash) AND off every convex
//! flank into bare canvas (a translucent skirt, then — once the matter was made opaque — a clean but
//! unwanted fattening). Enio's two complaints are the two faces of that one isotropy.
//!
//! A **closing** (dilate by the ball, then erode by the same ball) is exactly *fill concavities up to the
//! ball, leave convex boundaries where they are*: on a straight or convex edge the erosion undoes the
//! dilation (net zero — the edge is not touched); in an armpit narrower than `2ρ` the dilation bridges the
//! notch and the eroder ball cannot fit back in, so a rounded fillet stays (the gash fills). No thresholds,
//! no angular heuristic (the enclosure signal cannot tell a thick straight flank from an armpit — both wrap
//! more than 180° — and tuning it is the *"difícil de ajustar = bug de design"* trap); `ρ` is the whole
//! parameter, and it is the ball the artist already set.
//!
//! The HEIGHT keeps its dilation (the rounded dome Enio approved, 2026-07-14); only the MATTER's footprint is
//! closed. On a convex flank the height then grows past the coverage, onto zero-coverage canvas, where the
//! light does not weigh it — so the dome rounds up in place without the silhouette moving.
//!
//! Both morphological steps are done with an **exact squared Euclidean distance transform** (Felzenszwalb &
//! Huttenlocher 2004 — the same author whose parabola the ball replaced), `O(area)` and separable:
//! `dilate(P) = {dist²(·,P) ≤ ρ²}`, then `close = erode(dilate) = {dist(·, ¬dilate) ≥ ρ}`, anti-aliased over
//! the last texel of that distance so the fill's free edge (the armpit's mouth) is smooth, not a cookie-cut.

use super::Region;
use rayon::prelude::*;

/// A distance that stands in for infinity in the squared EDT — larger than any `d²` a real canvas produces.
const INF: f32 = 1.0e20;

/// The anti-alias width of the closing's free boundary, in texels. Kept small: on a convex flank the closing
/// boundary sits ON the paint edge, so this band is the ONLY growth a convex edge sees — below a texel of it,
/// invisible, while still feathering the armpit's mouth where the fill meets bare canvas.
const CLOSE_AA: f32 = 1.5;

/// The paint-mask threshold: a texel counts as "paint" for the footprint if its frozen coverage clears this.
/// Low, so the deposit's whole soft body is the footprint (a higher bar would erode the form from within).
const COVER_MASK: u8 = 24;

/// **1-D squared Euclidean distance transform** (Felzenszwalb–Huttenlocher). `f[q]` is the squared distance
/// budget already at column `q` (`0` on a mask cell, `INF` off it); `d[q]` becomes `min_p f[p] + (q − p)²`.
/// `v`/`z` are the lower-envelope scratch (parabola sites and their boundaries), passed in so the 2-D driver
/// reuses them across rows and columns.
fn edt_1d(f: &[f32], d: &mut [f32], v: &mut [usize], z: &mut [f32]) {
    let n = f.len();
    if n == 0 {
        return;
    }
    let mut k = 0usize;
    v[0] = 0;
    z[0] = -INF;
    z[1] = INF;
    for q in 1..n {
        // Intersection of the parabola from `q` with the one currently on top of the envelope; pop sites the
        // new parabola hides.
        let mut s;
        loop {
            let vk = v[k];
            s = ((f[q] + (q * q) as f32) - (f[vk] + (vk * vk) as f32)) / (2 * q - 2 * vk) as f32;
            if s <= z[k] && k > 0 {
                k -= 1;
            } else {
                break;
            }
        }
        k += 1;
        v[k] = q;
        z[k] = s;
        z[k + 1] = INF;
    }
    k = 0;
    for (q, out) in d.iter_mut().enumerate() {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let dq = q as f32 - v[k] as f32;
        *out = dq * dq + f[v[k]];
    }
}

/// Below this many texels a region is done SERIALLY: rayon's split/wake overhead (six parallel calls per
/// closing) does not pay on the local `cr` of a brush dab (~62k texels), where it made the Inflate *slower*.
/// A whole-layer filter (millions of texels — Enio's real case) clears it comfortably and runs parallel.
const PAR_MIN: usize = 1 << 18;

/// The 1-D transform along every ROW of a `len × rows` buffer. Parallel over rows (disjoint — the ADR-0109
/// property `blob_ball` uses; `for_each_init` gives each worker one reusable scratch set) when `par`, else
/// serial with a single scratch set.
fn edt_rows(d: &mut [f32], len: usize, par: bool) {
    let scratch = || {
        (
            vec![0.0f32; len],
            vec![0.0f32; len],
            vec![0usize; len],
            vec![0.0f32; len + 1],
        )
    };
    let work = |(f, out, v, z): &mut (Vec<f32>, Vec<f32>, Vec<usize>, Vec<f32>),
                row: &mut [f32]| {
        f.copy_from_slice(row);
        edt_1d(f, out, v, z);
        row.copy_from_slice(out);
    };
    if par {
        d.par_chunks_mut(len).for_each_init(scratch, work);
    } else {
        let mut s = scratch();
        d.chunks_mut(len).for_each(|row| work(&mut s, row));
    }
}

/// Transpose a `w × h` row-major buffer into an `h × w` one, so the column pass becomes a row pass on the
/// transpose — cache-friendly and reusing [`edt_rows`]'s parallelism instead of a strided (unshareable)
/// column walk.
fn transpose(src: &[f32], w: usize, h: usize, par: bool) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    let fill = |x: usize, col: &mut [f32]| {
        for (y, cell) in col.iter_mut().enumerate() {
            *cell = src[y * w + x];
        }
    };
    if par {
        out.par_chunks_mut(h)
            .enumerate()
            .for_each(|(x, col)| fill(x, col));
    } else {
        out.chunks_mut(h)
            .enumerate()
            .for_each(|(x, col)| fill(x, col));
    }
    out
}

/// One separable pass of the squared EDT that leaves its result in the TRANSPOSE: horizontal EDT in the given
/// layout, then transpose, then the other-axis EDT — returning the full 2-D transform, but transposed. Fusing
/// the two closing EDTs through this (threshold in the transposed layout, run the next EDT there) halves the
/// transposes from four to two — the whole of the 4096² cost.
fn edt_half(mut d: Vec<f32>, w: usize, h: usize, par: bool) -> Vec<f32> {
    edt_rows(&mut d, w, par); // one axis, in the current layout
    let mut dt = transpose(&d, w, h, par);
    edt_rows(&mut dt, h, par); // the other axis — result is the full 2-D EDT, transposed
    dt
}

/// **The closing of the paint footprint, as a per-texel fill fraction over `cr`** (`cw × ch`, row-major).
///
/// `1` inside the closing (the paint and the concavities the ball fills), feathering to `0` over `CLOSE_AA`
/// texels at the closing's free boundary, `0` on convex exterior canvas. The caller multiplies the Blob's
/// (opaque, height-anti-aliased) coverage by this, so a convex edge grows nothing and an armpit fills.
///
/// `pre_cover` is the whole-canvas frozen coverage; `w` its stride; `rho` the ball radius in texels. `cr`
/// carries the margin (`sculpt_inflate` grows it past the write region by more than `2ρ`), so the dilation
/// and erosion a written texel needs never read past `cr`.
pub(super) fn closing_fill(pre_cover: &[u8], w: usize, cr: Region, rho: f32) -> Vec<f32> {
    let (cw, ch) = (cr.w as usize, cr.h as usize);
    let (cx, cy) = (cr.x as usize, cr.y as usize);
    let n = cw * ch;
    // Parallel only above `PAR_MIN` — the whole-layer filter clears it; a brush dab's local `cr` does not,
    // and there the rayon overhead cost more than the transform.
    let par = n >= PAR_MIN;
    // The paint footprint over `cr`, as the EDT's seed directly: 0 on paint, INF off it. While building it,
    // note whether ANY texel is bare and whether ANY is paint — because the closing of a region that is ALL
    // paint is all paint (`cfill = 1`), and of one with NO paint is nothing (`cfill = 0`), and either way the
    // two distance transforms have nothing to find. A stroke landing inside solid paint (there is no bare
    // canvas to preserve a convex edge from) hits the first case and skips the EDT entirely.
    let mut seed = vec![INF; n];
    let (mut any_bare, mut any_paint) = (false, false);
    for ry in 0..ch {
        let gy = (cy + ry) * w + cx;
        let row = &mut seed[ry * cw..ry * cw + cw];
        for (rx, cell) in row.iter_mut().enumerate() {
            if pre_cover[gy + rx] > COVER_MASK {
                *cell = 0.0;
                any_paint = true;
            } else {
                any_bare = true;
            }
        }
    }
    if !any_bare {
        return vec![1.0; n]; // all paint — the closing is the whole region
    }
    if !any_paint {
        return vec![0.0; n]; // nothing to grow from
    }
    let rho2 = rho * rho;
    // Dilate: D = {dist² to paint ≤ ρ²}. Then erode D: the closing is {dist to ¬D ≥ ρ}. The two EDTs are
    // fused through `edt_half` — the first leaves `dtp2` in the transposed layout, the ¬D threshold is applied
    // there (elementwise, layout-agnostic), and the second EDT starts in that same layout, so only two
    // transposes run instead of four.
    let dtp2_t = edt_half(seed, cw, ch, par); // dtp2, transposed (ch-wide rows)
    // ¬D seed in the transposed layout: 0 where `q` is NOT in the dilation, INF where it is.
    let notd_seed: Vec<f32> = if par {
        dtp2_t
            .par_iter()
            .map(|&d2| if d2 > rho2 { 0.0 } else { INF })
            .collect()
    } else {
        dtp2_t
            .iter()
            .map(|&d2| if d2 > rho2 { 0.0 } else { INF })
            .collect()
    };
    let dtnd2 = edt_half(notd_seed, ch, cw, par); // transpose of a transpose ⇒ back to cw-wide rows
    // Anti-alias the erosion's boundary over the last `CLOSE_AA` texels of the distance to ¬D: `0` at
    // `ρ − CLOSE_AA`, `1` at `ρ` (and beyond, clamped). On a convex flank ¬D sits `ρ` out, so the band lands
    // on the paint edge and the exterior stays 0; in an armpit ¬D is far, so the fill is solid.
    let lo = (rho - CLOSE_AA).max(0.0);
    let span = (rho - lo).max(1.0e-4);
    let fill = |d2: f32| ((d2.sqrt() - lo) / span).clamp(0.0, 1.0);
    if par {
        dtnd2.par_iter().map(|&d2| fill(d2)).collect()
    } else {
        dtnd2.iter().map(|&d2| fill(d2)).collect()
    }
}

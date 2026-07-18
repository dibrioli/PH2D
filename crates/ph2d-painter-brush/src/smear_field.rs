//! **Smear as a displacement FIELD** — the transport that carries mass instead of a filament.
//!
//! The lift-and-blend kernels next door ([`crate::smear_dab`] and friends) are correct for ONE dab and
//! wrong for a stroke. Each dab re-reads the *result of the previous dab* and lerps toward it:
//!
//! ```text
//! dst ← dst + (src_one_step_back − dst)·w
//! ```
//!
//! so what survives `n` steps is `h·wⁿ` — a **product** over the dab list. The Smear route's spacing is
//! ~1 px, so a 170 px drag is ~170 steps. Exactly on the drag axis `t = 0`, `w = 1` and nothing decays;
//! six pixels off it `w ≈ 0.8` and `0.8¹⁵⁰ ≈ 0`. The stroke therefore delivers a **one-texel needle** and
//! no body at all — measured on the product (`push_look_probe` scene 13: across the trail at x=250,
//! `y194 h0.00 · y200 h3.73 · y206 h0.00`, with a brush of radius 10).
//!
//! This is the third time this line has met the same disease — the bow wave's bite
//! (`the_trench_is_a_fact_of_the_path_not_of_the_dab_spacing`) and the relief capsule were the first two.
//! The cure is always the same shape, and `warp/apply.rs` already states it: **a sequential accumulation
//! is sampling-dependent; an accumulated displacement applied ONCE to a frozen source is not.**
//!
//! So a dab here writes no pixels. It advances a per-stroke **backward map** — for each destination
//! texel, where in the frozen source its content came from — and the render resolves
//! `out[p] = pre[p − disp[p]]` exactly once, from the pixels frozen at pen-down. The same map moves the
//! colour and the impasto planes through one door (`warp/relief.rs`), which is what makes the pigment and
//! the body physically unable to disagree about where the paint went.
//!
//! ## Why the map is COMPOSED and not summed — the bucket brigade
//!
//! The obvious accumulation, `disp[i] += step · w(i)`, is wrong, and wrong in a way that is worth
//! recording because it looks right and it *measures* right on a short drag. A texel only accumulates
//! while the brush is over it, so the total displacement it can reach is bounded by roughly
//! *brush diameter × mean weight* — about 20 px for a 32 px knife, **no matter how far you drag**. Past
//! that the render samples a frozen source at a point that was never painted, and the trail simply stops.
//! Measured: colour and relief both fell to zero ~35 px past the ridge, on a 160 px drag.
//!
//! But a smear is a **relay**. Dab *k* hands its content to dab *k+1*, which hands it on again; content
//! near the axis stays under the moving brush and therefore travels the *whole stroke*, while content off
//! the axis is passed over once and left behind. That is a **composition of maps**, not a sum of offsets:
//!
//! ```text
//! φ_new(p) = φ_old(p − v(p))          v(p) = step · w(p)
//! ```
//!
//! *"what is at `p` now came from wherever the thing at `p − v` came from"* — semi-Lagrangian
//! backtracking, the standard formulation. In displacement form (`φ(p) = p − disp(p)`) that is
//!
//! ```text
//! disp_new(p) = v(p) + disp_old(p − v(p))
//! ```
//!
//! which relays without bound on the axis and settles to a finite partial offset off it — the trail with
//! mass, instead of either a needle or a stub.
//!
//! **This is still "accumulate, then apply ONCE to a frozen source", and that is the point.** What gets
//! resampled repeatedly is the *map* — a smooth, locally near-affine coordinate field, which bilinear
//! interpolation reproduces almost exactly. The IMAGE is resampled a single time, at the end. The kernel
//! this replaces resampled the image itself on every one of ~170 steps, which is why its off-axis mass
//! died as `wⁿ`. Resampling a coordinate field and resampling a picture are not the same operation, and
//! the difference between them is this whole module.
//!
//! ## Why this rides `walk_dab` and not a footprint of its own
//!
//! Because the Smear gets Tiling, Symmetry, the shape editors, pressure, Jitter, the **Shape**
//! silhouette and the **Grain** for free by hanging off the one dab list, and a warp session with its own
//! geometry inherits none of it. `walk_dab` is documented as *"the ONE place the swept body, the
//! silhouette, the Grain and the Selection are resolved"*; this module is simply its third rider, beside
//! the sculpt intensity and the plane fit. A dab shaped differently for the smear than for everything
//! else is how "Tiling doesn't work in Smear" gets born.

use crate::dab::DirtyRect;
use crate::height::HeightDab;
use crate::spec::BrushSpec;

/// Accumulate ONE smear dab into the session's cumulative displacement map.
///
/// `disp` is canvas-sized (`width · height`), in **pixels**, and is the source of truth for the whole
/// stroke: the renderer resolves each texel as `bilinear(pre, p − disp[p])`. `step` is this dab's motion
/// (`to − from`) in canvas pixels — **not** rounded to whole texels the way the lift-and-blend kernel
/// rounds it, because a displacement is resampled bilinearly and has no reason to quantise. `mask` is the
/// Selection's per-texel coverage, folded per dab as it lands (never onto the running total — see
/// [`crate::sculpt::accumulate_dab_sculpt`], which explains why that compounds under a Feather).
///
/// Returns the touched rect, or `None` if the dab wrote nothing.
///
/// ## The fold is the SMEAR's fold, and that is deliberate
///
/// [`crate::sculpt::walk_dab`] folds `coverage × flow × strength`, because that is what the deposit and
/// the colour routes do. The Smear route has never folded **Flow** — its weight is `coverage × strength`
/// (`stamp_dabs_smear`: `amount = strength · d.coverage`). Whether Flow *ought* to mean something on a
/// knife is a real question, and it is not this fix's to answer: turning an inert slider live would
/// change every existing smear drawing for a reason nobody asked for. So the caller hands us a spec whose
/// `flow` is `1.0` and the fold reduces to exactly the one the route has always applied.
#[must_use]
pub fn accumulate_dab_smear(
    out: SmearOut<'_>,
    step: [f32; 2],
    mask: Option<&[u8]>,
    width: u32,
    height: u32,
    spec: &BrushSpec,
    dab: &HeightDab<'_>,
) -> Option<DirtyRect> {
    let SmearOut { disp, scratch } = out;
    let n = (width as usize) * (height as usize);
    if disp.len() < n {
        return None;
    }
    // A dab that does not move transports nothing. (The lift-and-blend kernel early-outs on the same
    // fact; here it also keeps a zero-length step from marking the rect dirty for no reason.)
    if step[0] == 0.0 && step[1] == 0.0 {
        return None;
    }
    // Pass 1 — resolve the footprint ONCE (the expensive part: falloff, Shape image, Grain, Selection)
    // and park `(index, weight)`. The composition needs the dab's rect before it can snapshot, and
    // walking the silhouette twice would double the dab's cost.
    scratch.pairs.clear();
    let pairs = &mut scratch.pairs;
    let rect = crate::sculpt::walk_dab(mask, width, height, spec, dab, |i, _dx, _dy, add| {
        pairs.push((i as u32, add));
    })?;

    // Snapshot the OLD map over the rect grown by the furthest this dab can backtrack. The update reads
    // `disp_old(p − v)` for neighbours of `p`, so reading and writing the same buffer in place would let
    // a texel updated earlier in the scan pollute one updated later — the very sequential dependence
    // this kernel exists to remove.
    let reach = step[0].abs().max(step[1].abs()).ceil() as i64 + 2;
    let wi = width as i64;
    let hi = height as i64;
    let wx0 = (rect.x as i64 - reach).max(0);
    let wy0 = (rect.y as i64 - reach).max(0);
    let wx1 = (rect.x as i64 + rect.w as i64 + reach).min(wi);
    let wy1 = (rect.y as i64 + rect.h as i64 + reach).min(hi);
    let ww = (wx1 - wx0) as usize;
    let wh = (wy1 - wy0) as usize;
    scratch.win.clear();
    scratch.win.reserve(ww * wh);
    for y in wy0..wy1 {
        let row = (y * wi) as usize;
        scratch
            .win
            .extend_from_slice(&disp[row + wx0 as usize..row + wx1 as usize]);
    }

    // Pass 2 — compose. `disp_new(p) = v(p) + disp_old(p − v(p))`.
    for &(i, add) in &scratch.pairs {
        let i = i as usize;
        let px = (i % width as usize) as f32;
        let py = (i / width as usize) as f32;
        let v = [step[0] * add, step[1] * add];
        let back = sample_window(
            &scratch.win,
            ww,
            wh,
            wx0 as f32,
            wy0 as f32,
            px - v[0],
            py - v[1],
        );
        disp[i] = [v[0] + back[0], v[1] + back[1]];
    }
    Some(rect)
}

/// The map being advanced, plus its caller-owned scratch — bundled the way [`crate::sculpt::PlaneOut`]
/// bundles the plane family's outputs, so the kernel keeps one output parameter instead of two.
pub struct SmearOut<'a> {
    /// The session's cumulative backward map, canvas-sized.
    pub disp: &'a mut [[f32; 2]],
    /// Reused per-dab scratch (see [`SmearScratch`]).
    pub scratch: &'a mut SmearScratch,
}

/// Caller-owned scratch so a hot stroke allocates nothing: the resolved `(index, weight)` pairs of one
/// dab, and the snapshot of the old map over that dab's window.
#[derive(Default)]
pub struct SmearScratch {
    pairs: Vec<(u32, f32)>,
    win: Vec<[f32; 2]>,
}

/// Bilinear-sample the windowed map snapshot at canvas coords `(x, y)`, clamping to the window's edge.
///
/// Clamping is safe because the window is grown by the dab's maximum backtrack, so a clamp can only bite
/// at the true canvas edge — where extending the map is exactly the policy the pixel and relief samplers
/// already use (`bilinear_clamped`: extend, never wrap).
#[inline]
fn sample_window(
    win: &[[f32; 2]],
    ww: usize,
    wh: usize,
    ox: f32,
    oy: f32,
    x: f32,
    y: f32,
) -> [f32; 2] {
    if ww == 0 || wh == 0 {
        return [0.0, 0.0];
    }
    let lx = x - ox;
    let ly = y - oy;
    let x0f = lx.floor();
    let y0f = ly.floor();
    let fx = lx - x0f;
    let fy = ly - y0f;
    let x0 = (x0f as i64).clamp(0, ww as i64 - 1) as usize;
    let y0 = (y0f as i64).clamp(0, wh as i64 - 1) as usize;
    let x1 = (x0 + 1).min(ww - 1);
    let y1 = (y0 + 1).min(wh - 1);
    let at = |xi: usize, yi: usize| win[yi * ww + xi];
    let (a, b, c, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
    let top = [a[0] + (b[0] - a[0]) * fx, a[1] + (b[1] - a[1]) * fx];
    let bot = [c[0] + (d[0] - c[0]) * fx, c[1] + (d[1] - c[1]) * fx];
    [
        top[0] + (bot[0] - top[0]) * fy,
        top[1] + (bot[1] - top[1]) * fy,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::height::HeightDab;

    fn dab_at(center: [f32; 2], radius: f32) -> HeightDab<'static> {
        HeightDab {
            center,
            radius,
            coverage: 1.0,
            footprint: crate::footprint::FootprintDeform::identity(),
            prev_center: None,
            shape: None,
            grain: None,
            grain_image: None,
        }
    }

    fn spec(radius: f32) -> BrushSpec {
        BrushSpec {
            radius_px: radius,
            flow: 1.0,
            strength: 1.0,
            ..Default::default()
        }
    }

    /// **The law the whole fix rests on: transport is a SUM, so it does not depend on how finely the
    /// motion was sampled.** Walk the same 32 px with 32 one-pixel steps and with 8 four-pixel steps —
    /// the displacement that lands on the axis must be the same distance, not a different one.
    ///
    /// The kernel this replaces fails exactly here: `h·wⁿ` depends on `n`.
    ///
    /// **Mutation that must bleed:** make the sink `disp[i] = step·add` (assign, not add) — the coarse
    /// walk then reports one step's worth and the two disagree by 4×.
    #[test]
    fn transport_is_a_sum_so_the_dab_spacing_cannot_change_it() {
        let (w, h) = (64u32, 64u32);
        let n = (w * h) as usize;
        let s = spec(8.0);
        let walk = |stride: f32| {
            let mut sc = SmearScratch::default();
            let mut disp = vec![[0.0f32; 2]; n];
            let steps = (32.0 / stride) as u32;
            for k in 0..steps {
                let x = 16.0 + stride * (k as f32 + 1.0);
                let _ = accumulate_dab_smear(
                    SmearOut {
                        disp: &mut disp,
                        scratch: &mut sc,
                    },
                    [stride, 0.0],
                    None,
                    w,
                    h,
                    &s,
                    &dab_at([x, 32.0], 8.0),
                );
            }
            disp
        };
        let fine = walk(1.0);
        let coarse = walk(4.0);
        // On the axis, mid-trail: the brush centre passed over this texel, so the weight is ~1 and the
        // displacement should be ~the distance travelled while it was under the brush.
        let i = (32 * w + 32) as usize;
        assert!(
            fine[i][0] > 8.0,
            "the fine walk must actually transport (got {})",
            fine[i][0]
        );
        let ratio = coarse[i][0] / fine[i][0];
        assert!(
            (0.8..1.25).contains(&ratio),
            "same 32 px of motion, 4× the sampling: fine displaced {} px, coarse {} px (ratio {ratio}). \
             Transport that depends on the spacing is the product law this kernel exists to replace",
            fine[i][0],
            coarse[i][0]
        );
    }

    /// **The trail has the brush's WIDTH.** Off the drag axis the falloff makes the displacement
    /// smaller, but it must not make it *zero* — that is the filament. Across the trail, the band of
    /// texels that moved at all should span roughly the brush's diameter.
    ///
    /// **Mutation that must bleed:** restore a per-step lerp toward the previous result — the off-axis
    /// column collapses to the centre texel.
    #[test]
    fn the_displaced_band_is_as_wide_as_the_brush() {
        let (w, h) = (96u32, 96u32);
        let n = (w * h) as usize;
        let s = spec(10.0);
        let mut sc = SmearScratch::default();
        let mut disp = vec![[0.0f32; 2]; n];
        for k in 0..48u32 {
            let x = 20.0 + k as f32;
            let _ = accumulate_dab_smear(
                SmearOut {
                    disp: &mut disp,
                    scratch: &mut sc,
                },
                [1.0, 0.0],
                None,
                w,
                h,
                &s,
                &dab_at([x, 48.0], 10.0),
            );
        }
        // Cut across the trail at mid-drag and count what moved by a visible amount.
        let moved = (0..h)
            .filter(|&y| disp[(y * w + 44) as usize][0] > 0.5)
            .count();
        assert!(
            moved >= 14,
            "the knife is 20 px across but only {moved} px of the cross-section moved — the transport \
             narrowed to a filament"
        );
    }

    /// A dab that does not move writes nothing at all — and, in particular, does not dirty a rect.
    #[test]
    fn a_still_dab_transports_nothing() {
        let (w, h) = (32u32, 32u32);
        let mut disp = vec![[0.0f32; 2]; (w * h) as usize];
        assert!(
            accumulate_dab_smear(
                SmearOut {
                    disp: &mut disp,
                    scratch: &mut SmearScratch::default()
                },
                [0.0, 0.0],
                None,
                w,
                h,
                &spec(6.0),
                &dab_at([16.0, 16.0], 6.0)
            )
            .is_none()
        );
        assert!(disp.iter().all(|d| *d == [0.0, 0.0]));
    }

    /// The Selection attenuates the transport where it is partial — the knife cannot drag paint out of a
    /// region the artist masked off.
    #[test]
    fn the_selection_attenuates_the_transport() {
        let (w, h) = (48u32, 48u32);
        let n = (w * h) as usize;
        let s = spec(8.0);
        // Left half fully selected, right half not at all.
        let mut mask = vec![0u8; n];
        for y in 0..h {
            for x in 0..(w / 2) {
                mask[(y * w + x) as usize] = 255;
            }
        }
        let mut sc = SmearScratch::default();
        let mut disp = vec![[0.0f32; 2]; n];
        for k in 0..24u32 {
            let _ = accumulate_dab_smear(
                SmearOut {
                    disp: &mut disp,
                    scratch: &mut sc,
                },
                [1.0, 0.0],
                Some(&mask),
                w,
                h,
                &s,
                &dab_at([12.0 + k as f32, 24.0], 8.0),
            );
        }
        let inside = disp[(24 * w + 12) as usize][0];
        let outside = disp[(24 * w + 40) as usize][0];
        assert!(
            inside > 0.5,
            "inside the selection the knife drags ({inside})"
        );
        assert_eq!(
            outside, 0.0,
            "outside the selection nothing moves (got {outside})"
        );
    }
}

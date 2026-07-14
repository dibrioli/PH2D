//! **Sculpt** — the brush as a LOCAL operator on the height field (`docs/Painter/18_plano_sculpt_relevo.md`).
//!
//! A sculpt dab neither deposits nor displaces. It *marks how hard it touched* each texel, and the
//! reshaping is done once, afterwards, from a frozen copy of the relief. This module owns exactly that
//! mark — the per-stroke **intensity** field — and nothing else; the kernel that consumes it
//! (blur / unsharp) and the memo that makes it affordable live tool-side, because they need the
//! canvas, the selection and the stroke's window.
//!
//! ## Why a dab does not simply blur the height under it
//!
//! Because the obvious version is wrong twice, and both failures look like features at first:
//!
//! 1. **It is not idempotent.** The shape editors (Line / Curve / Ellipse / Polygon / Free Hand)
//!    re-stamp the WHOLE stroke on every pointer move. A dab that blurred `h` in place would melt the
//!    relief further on every frame while the artist merely *looked* at the curve. This is not
//!    hypothetical: it is the exact reason `height_push`'s displacement is idempotent by construction.
//! 2. **It composes into diffusion.** At 10% spacing a texel is under ~10 dabs. Ten blurs of radius `r`
//!    are not one blur of radius `r` scaled by ten — they are a diffusion of radius `r√10`, so the
//!    *scale* of the smoothing would be set by the SPACING slider rather than by the artist's intent.
//!
//! Accumulating the intensity and applying the kernel exactly once fixes both, and it is the same
//! discipline the Deform warp already uses (`tool::paint::warp::apply` accumulates displacement and
//! re-renders from the frozen stroke-start pixels rather than re-gathering its own output).
//!
//! ## Isolation (ADR-0107)
//!
//! A sibling of [`crate::height`], append-only: it reuses that module's [`HeightDab`] — the already-frozen
//! carrier of the dab list's resolved frames — and adds one function. `height.rs` is not touched, so a
//! concurrent line editing it cannot collide with this one.

use crate::height::{HeightDab, sweep_axis, sweep_residual};

/// Accumulate ONE sculpt dab's **intensity** into `amount` (canvas-sized, `width × height`).
///
/// The walk is the same walk the deposit does — the same swept body, the same silhouette (falloff or
/// Shape image), the same Grain frame — because it is handed the same [`HeightDab`] the colour kernel
/// gets. That is what makes Symmetry, Tiling, the shape editors, Jitter and pressure sculpt the relief
/// without a line of code each (`docs/Painter/18…` §10.1).
///
/// Dabs **SUM** here (a slow pass smooths more than a fast one, which is what a hand expects) and the
/// consumer clamps the total to `1`. Returns the touched rect, or `None` if nothing was written.
///
/// ## The fold is the house's fold — and that was worth measuring rather than reading
///
/// `dab.coverage × Flow × Strength`. It looks like a double-count, because `Dab::coverage` is documented
/// as *"brush strength × pressure coverage-scale × space-attenuation"* — the Strength is already inside
/// it — and this line multiplies by Strength again. The first cut of this kernel duly "fixed" that, and
/// was wrong.
///
/// Measured on the real product (Strength 1.0 vs 0.5, one dab, Depth 1, Body 0):
///
/// ```text
///   relief   0.5 / 1.0  =  0.250     ← quadratic
///   pigment  0.5 / 1.0  =  0.251     ← quadratic, to three figures
/// ```
///
/// The COLOUR routes fold it the same way (`stamp_color_cache`, `stamp_color_dynamic`:
/// `d.coverage * brush.flow * brush.strength`), so the pigment and the body respond to the slider
/// identically, and `height.rs`'s claim of *"the same fold the colour kernel applies"* is simply true.
/// Whether a quadratic Strength is the RIGHT curve is an app-wide question and not a sculpt wave's to
/// answer — but a spatula whose Strength slider behaved differently from every other tool's would be a
/// surprise the artist feels and cannot name, which is worse than a curve they have already learned.
///
/// So: the same fold, deliberately. **A convention is not a bug just because you misread it.**
///
/// The consequence the artist sees is that Strength is baked into the gesture, like every other thing the
/// hand did. **And so is the rest of the card** — Radius and Smooth↔Sharpen arm the NEXT stroke; they do not
/// re-render the last one. They did, briefly, riding the deposit's "Adjust Last Stroke"; that was a design
/// bug, and Enio's smoke found it in one move (reaching for Sharpen, to sharpen elsewhere, turned the Smooth
/// behind him into its opposite). Paint is a substance and has properties you can keep tuning; a smoothing
/// is a verb that already happened. See `tool::paint::sculpt::SculptState` in the tool crate.
///
/// The `impasto_depth` gate is not here either: a sculpt brush deposits no body, so a brush with Depth 0
/// — or with the Impasto checkbox off entirely — still sculpts. The checkbox says *this brush lays
/// thickness*; it says nothing about a brush that only reshapes the thickness already there.
/// ## The selection is folded HERE, per dab — not onto the running total
///
/// `mask` is the Selection's per-texel coverage (`0..=255`, canvas-sized), or `None` when nothing is
/// selected. It multiplies each dab's **own** contribution as it lands.
///
/// Attenuating the accumulated `amount` afterwards instead — which is the obvious place, and where this
/// started — silently compounds: `amount` carries every earlier batch, so a texel touched by *k* pointer
/// batches gets its first contribution scaled *k* times (`((a₁·s) + a₂)·s`, not `(a₁ + a₂)·s`). With a
/// HARD selection `s ∈ {0, 1}` and the multiply is idempotent, so it looks fine and every hard-edged gate
/// stays green. With **Feather** — a real, shipped slider — the boundary band carries partial coverage,
/// and there the spatula's strength becomes a function of how many pointer events the OS happened to
/// deliver. It is the polling-rate bug of §4.2 wearing a different hat, which is exactly why it belongs in
/// the same place the rest of the dab's weight does: right here, once, as the dab lands.
#[must_use]
pub fn accumulate_dab_sculpt(
    amount: &mut [f32],
    mask: Option<&[u8]>,
    width: u32,
    height: u32,
    spec: &crate::BrushSpec,
    dab: &HeightDab<'_>,
) -> Option<crate::dab::DirtyRect> {
    let n = (width as usize) * (height as usize);
    if amount.len() < n {
        return None;
    }
    walk_dab(mask, width, height, spec, dab, |i, _dx, _dy, add| {
        // SUM, not envelope. The deposit takes a max (one pass of a loaded brush leaves ONE thickness);
        // a sculpt has no such conservation to respect — dwelling on a spot really does smooth it more,
        // and that is the whole feel of the tool. The consumer clamps at 1.
        amount[i] += add;
    })
}

/// Accumulate ONE **plane-family** dab (Flatten / Scrape / Fill): fit the local plane to the surface under
/// the footprint, then write both the intensity AND the plane's height at each texel
/// (`docs/Painter/18…` §7).
///
/// `pre` is the FROZEN relief the stroke started from — never the live one. Reading the live surface would
/// let each dab fit a plane to what the previous dab already flattened, and the flattening would then
/// depend on the spacing and the mouse's polling rate (§4.2, and the gate
/// `a_faster_mouse_does_not_sculpt_deeper` is written against exactly that).
///
/// ## Two outputs, and why the second is a SUM rather than a plane
///
/// * `amount[i] += w` — the same intensity [`accumulate_dab_sculpt`] writes. Nothing here changes it.
/// * `plane_sum[i] += w · plane(i)` — the same weight, times *this* dab's plane at that texel.
///
/// The consumer divides: `target[i] = plane_sum[i] / amount[i]`, the **coverage-weighted mean of every
/// plane that touched the texel**. Storing the mean directly would be wrong the moment a second dab
/// arrives (you cannot average an average without its weight), and storing a plane *per dab* would need
/// the dab list at render time — which the shape editors throw away and rebuild every frame.
///
/// Note what the division buys: the target is **independent of Strength and Flow**, because they scale
/// numerator and denominator alike. Strength decides *how far* you travel toward the plane; it does not
/// decide *where the plane is*. Those are different questions and a spatula that confused them would go
/// deeper as you leaned harder — which is Scrape's job, and it does it through `k`, not by moving the floor.
///
/// The **plane Offset** is deliberately NOT folded in here. It is a rigid shift, so
/// `Σ w·(plane + off) = plane_sum + off·amount` — the consumer adds it at render time, which is what keeps
/// the Offset slider live on an open shape without re-stamping a single dab.
///
/// `scratch` is caller-owned so a hot stroke allocates nothing: the walk samples the silhouette (the
/// expensive part — falloff, Shape image, Grain) exactly ONCE and parks `(index, weight)` there, then the
/// second pass reads it back to write the outputs. A second silhouette pass would have doubled the dab's cost.
#[must_use]
pub fn accumulate_dab_plane(
    out: PlaneOut<'_>,
    mask: Option<&[u8]>,
    width: u32,
    height: u32,
    spec: &crate::BrushSpec,
    dab: &HeightDab<'_>,
) -> Option<crate::dab::DirtyRect> {
    let PlaneOut {
        amount,
        plane_sum,
        pre,
        scratch,
    } = out;
    let n = (width as usize) * (height as usize);
    if amount.len() < n || plane_sum.len() < n || pre.len() < n {
        return None;
    }
    scratch.clear();
    let mut fit = crate::plane::PlaneFit::default();
    let rect = walk_dab(mask, width, height, spec, dab, |i, dx, dy, add| {
        // The fit is weighted by the dab's OWN mask, so the Shape image / falloff / Grain decide which
        // part of the surface the spatula reads — exactly as they decide which part it touches. A chisel
        // reads with its edge.
        fit.add(dx, dy, pre[i], add);
        scratch.push((i as u32, add));
    })?;
    let plane = fit.solve()?;
    let (cx, cy) = (dab.center[0], dab.center[1]);
    let w_us = width as usize;
    for &(i, add) in scratch.iter() {
        let i = i as usize;
        // Recover (dx, dy) from the index rather than parking two more floats: the walk's own arithmetic,
        // and it is exact (the same `+ 0.5` texel centre).
        let dx = ((i % w_us) as f32 + 0.5) - cx;
        let dy = ((i / w_us) as f32 + 0.5) - cy;
        amount[i] += add;
        plane_sum[i] += add * plane.at(dx, dy);
    }
    Some(rect)
}

/// What a plane-family dab writes, and what it reads to decide. Grouped because they travel together and
/// mean nothing apart — and because six loose buffer arguments is a call site nobody can read.
pub struct PlaneOut<'a> {
    /// The intensity, exactly as [`accumulate_dab_sculpt`] writes it. `Σ w`.
    pub amount: &'a mut [f32],
    /// `Σ w · plane(i)` — the un-normalised target. The consumer divides by `amount`.
    pub plane_sum: &'a mut [f32],
    /// The **frozen** relief the stroke started from. The plane is fitted to THIS, never to the live
    /// surface: fitting to the live one would let each dab flatten what the last dab flattened, making the
    /// result a function of the spacing and of how fast the artist moved.
    pub pre: &'a [f32],
    /// Caller-owned `(index, weight)` scratch, so the silhouette is sampled once and the fit still gets a
    /// second pass. Cleared on entry; never read across dabs.
    pub scratch: &'a mut Vec<(u32, f32)>,
}

/// The dab's footprint walk — the ONE place the swept body, the silhouette, the Grain and the Selection are
/// resolved. Both accumulators above ride it, so a change to how a dab is shaped cannot reach one of them
/// and miss the other.
///
/// Calls `f(index, dx, dy, weight)` for every texel the dab actually touches, where `weight` is the fully
/// folded contribution (`silhouette × grain × coverage × flow × strength × selection`) and `(dx, dy)` is
/// the offset from the dab centre. Returns the touched rect, or `None` if nothing was written.
fn walk_dab(
    mask: Option<&[u8]>,
    width: u32,
    height: u32,
    spec: &crate::BrushSpec,
    dab: &HeightDab<'_>,
    mut f: impl FnMut(usize, f32, f32, f32),
) -> Option<crate::dab::DirtyRect> {
    let n = (width as usize) * (height as usize);
    if width == 0 || height == 0 {
        return None;
    }
    // A mask that does not describe this canvas is not a mask — refuse it rather than index into it.
    let mask = mask.filter(|m| m.len() >= n);
    // The SAME fold the deposit and the colour both apply (see `accumulate_dab_sculpt`: it was measured,
    // not read). It is also what makes Strength 0 cost nothing at all — the walk never starts, so nothing
    // is written, so the layer's relief plane is never forked. Not an optimisation: a consequence.
    let coverage =
        dab.coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 {
        return None;
    }
    let radius = dab.radius.max(0.5);
    let (cx, cy) = (dab.center[0], dab.center[1]);
    // The bbox covers the whole SWEPT body, not just the disc at the centre — otherwise the intensity
    // would bead at every pointer event, at a rhythm chosen by the mouse's polling rate.
    let reach = radius + crate::height::sweep_len(dab);
    let x0 = (cx - reach).floor().max(0.0) as i64;
    let y0 = (cy - reach).floor().max(0.0) as i64;
    let x1 = ((cx + reach).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + reach).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let sweep = sweep_axis(dab);
    let mut touched = false;
    for py in y0..y1 {
        let dy = (py as f32 + 0.5) - cy;
        for px in x0..x1 {
            let dx = (px as f32 + 0.5) - cx;
            let (rx, ry) = sweep_residual(dx, dy, sweep);
            let t = dab.footprint.falloff_t(rx * inv_radius, ry * inv_radius);
            let w = crate::dab::silhouette_at(spec, dab.shape, t, px, py, dab.center, radius);
            if w <= 0.0 {
                continue;
            }
            // The **Grain**, when the brush has one: a spatula with a grain is a textured spatula, and
            // that falls out of riding the dab list. Folded by the SAME groove law the deposit uses
            // (`height::derive_height`) rather than a bare multiply — a grain's samples average well
            // under half, so `× g` would not texture the touch, it would silently remove two thirds of
            // it. No grain ⇒ exactly `1.0` ⇒ byte-identical to a brush without one.
            let g = match dab.grain {
                Some(b) => {
                    let s =
                        crate::dab::grain_at(spec, b, dab.grain_image, px, py, dab.center, radius)
                            .clamp(0.0, 1.0);
                    crate::height::grain_groove(s)
                }
                None => 1.0,
            };
            let i = (py as usize) * (width as usize) + px as usize;
            // The **Selection**, folded into THIS dab's contribution (see `accumulate_dab_sculpt`'s docs:
            // attenuating the running total instead compounds once per pointer batch, and a Feather makes
            // that visible).
            let sel = match mask {
                Some(m) => f32::from(m[i]) / 255.0,
                None => 1.0,
            };
            let add = w * g * coverage * sel;
            if add <= 0.0 {
                continue;
            }
            f(i, dx, dy, add);
            touched = true;
        }
    }
    if !touched {
        return None;
    }
    Some(crate::dab::DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

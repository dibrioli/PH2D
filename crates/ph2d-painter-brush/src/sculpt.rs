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
    if width == 0 || height == 0 || amount.len() < n {
        return None;
    }
    // A mask that does not describe this canvas is not a mask — refuse it rather than index into it.
    let mask = mask.filter(|m| m.len() >= n);
    // The SAME fold the deposit and the colour both apply (see the doc comment: it was measured, not
    // read). It is also what makes Strength 0 cost nothing at all — the walk never starts, so `amount` is
    // never written, so the layer's relief plane is never forked. Not an optimisation: a consequence.
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
            // The **Selection**, folded into THIS dab's contribution (see the doc comment: attenuating the
            // running total instead compounds once per pointer batch, and a Feather makes that visible).
            let sel = match mask {
                Some(m) => f32::from(m[i]) / 255.0,
                None => 1.0,
            };
            let add = w * g * coverage * sel;
            if add <= 0.0 {
                continue;
            }
            // SUM, not envelope. The deposit takes a max (one pass of a loaded brush leaves ONE
            // thickness); a sculpt has no such conservation to respect — dwelling on a spot really does
            // smooth it more, and that is the whole feel of the tool. The consumer clamps at 1.
            amount[i] += add;
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

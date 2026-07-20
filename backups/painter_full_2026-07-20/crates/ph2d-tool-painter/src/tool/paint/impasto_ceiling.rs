//! The **glass ceiling** — how accumulated paint tops out.
//!
//! A sibling of [`super::impasto_settle`], and for the same reason: this is the *physics of the material*,
//! apart from the plumbing that schedules it. The settle says how paint relaxes sideways under its own
//! weight; this says how it stops rising.
//!
//! The whole module is one function, and the argument for it is in that function's doc — including why the
//! `clamp` that used to do this job was not a ceiling but an eraser.

/// Where the paint stops rising **linearly** — the knee of the far-field guard, in units of a full-Depth
/// stroke.
///
/// ## It is not a ceiling the artist hits — it is a runaway guard (Enio, 2nd smoke of 2026-07-14)
///
/// Enio: *"o fato de ficar progressivamente mais difícil de subir não é desejável — não poderia … subir na
/// proporção real do peso da ferramenta?"* Right: paint should stack in proportion to its weight, with no
/// glass to press against. So below this knee the relief is **exactly linear** — a stroke adds its full
/// weight, ten strokes add ten, byte for byte.
///
/// The knee sits at **24 loads**, which is 384 px of relief (`× DEPTH_UNIT_PX`). Nothing an artist does in
/// the reachable range comes near it: the whole point of the number is to be *out of reach*, so that the one
/// thing the compression still buys — a bound on the height the LIGHT can see, so a single runaway texel
/// cannot hand it an infinite slope — never costs the artist a linear load. The old knee was **2**, and with
/// the new radius-scaled deposit a big brush cleared it in a single dab; that is the wall Enio saw.
/// // CLAMP-OK
pub(super) const H_KNEE: f32 = 24.0;

/// The height the **apparent** relief approaches and never reaches — the far-field guard's asymptote. So far
/// past the knee that the band between them is itself effectively unreachable; it exists only so the
/// approach is a smooth curve rather than a hard wall if a pathological pile ever climbs into it. // CLAMP-OK
pub(super) const H_ASYMPTOTE: f32 = 128.0;

/// A sanity bound on the **stored** field — not a design value, a guard. Nothing an artist can do reaches
/// it; it exists so that a pathological accumulation cannot walk off into infinity and take the normal with
/// it. // CLAMP-OK
pub(super) const H_MAX: f32 = 128.0;

/// The **glass ceiling** — and it is a ceiling of the *appearance*, not of the data.
///
/// ## The hard clamp was not glass. It was an eraser. (Enio, smoke of 2026-07-14)
///
/// This used to be `h.clamp(-2.0, 2.0)`, applied to the stored field, and the doc quoted Corel Painter for
/// it: *"the accumulated artwork will begin to top out and appear as if the strokes are pressed against
/// glass"*. **Top out** — gradually. A `clamp` does not do that. It maps **everything** above the ceiling to
/// **exactly the same number**, which means:
///
/// * the brush-marks up there are not compressed, they are **deleted** — every one of them becomes `2.0`;
/// * a plateau of one constant has **zero gradient**, and the light shades from the gradient
///   (`∇h × DEPTH_UNIT_PX`);
/// * so precisely where the artist worked hardest, the surface renders as **a dead flat plate**.
///
/// Two strokes of Inflate and the sculpture was a mesa. The screenshot was unanswerable.
///
/// ## What it is now
///
/// A **C¹ asymptotic compression**, and it lives at the **light** (`ReliefFields::height_at`) rather than at
/// the buffer. Two consequences worth stating out loud:
///
/// * **The data stays honest.** Smooth, Flatten, Scrape and Inflate all fit planes and roll balls over the
///   relief the artist actually built. A clamp in the buffer corrupts the geometry those verbs reason about;
///   a display transform cannot.
/// * **Nothing ever goes flat.** `soft'(h) = 1/(1+t)²` is small on a huge pile but never zero, so the fine
///   marks on top of it survive — with less contrast, which is exactly what *pressed against glass* means and
///   what the clamp never delivered.
///
/// Identity below [`H_KNEE`] (so every canvas painted before this is **byte-identical**), unit slope at the
/// knee (no crease where the two halves meet), monotone, bounded by [`H_ASYMPTOTE`], and symmetric — a gouge
/// bottoms out the same way a pile tops out. Algebraic, so it costs a divide and no transcendental (HR-5).
#[inline]
pub(super) fn soft_ceiling(h: f32) -> f32 {
    let a = h.abs();
    if a <= H_KNEE {
        return h;
    }
    let span = H_ASYMPTOTE - H_KNEE;
    let t = (a - H_KNEE) / span;
    // t/(1+t): zero at the knee with slope 1, → 1 as t → ∞. The whole ceiling is this one fraction.
    (H_KNEE + span * (t / (1.0 + t))).copysign(h)
}

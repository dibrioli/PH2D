//! **The film** — where a body of paint ENDS, and how thick it is at that edge.
//!
//! One curve ([`body_profile`]) on one pair of thresholds ([`W_TAIL`] / [`W_SOLID`]), read by three
//! things that must never disagree about what counts as paint:
//!
//! - the **relief** ([`crate::height::derive_height`]) — the body it stands up,
//! - the **light** (`impasto_light::paint_body`) — the shading it is allowed to weigh,
//! - the **pigment** ([`film_coverage`]) — the colour the brush is allowed to lay.
//!
//! The third one is new (Enio, 2026-07-12). Until it existed the first two agreed and the pigment did
//! not: the colour ran out to the dab's geometric rim, so every impasto stroke wore a skirt of paint the
//! light was RIGHT to refuse to model — a haze around the ridge, whose width was a pure function of the
//! falloff. Sibling module of [`crate::height`], for the file-LOC cap and because it is its own idea.

/// Coverage below which the paint carries NO body: everything thinner is the STAIN — pigment rubbed
/// into the paper, not paint you could stand a wall on. It is deliberately high: the wall must rise
/// INSIDE the pigmented part of the stroke (Photoshop's bevel runs from the matte's edge *inward*,
/// over solid pixels, for the same reason) — a wall standing on the translucent rim gets its strong
/// lighting multiplied into pixels that are mostly PAPER, which is the white-canvas halo all over
/// again (`impasto_light_shades_the_paint_not_the_paper_showing_through_it` refuses it). // CLAMP-OK
pub const W_TAIL: f32 = 0.35;

/// Coverage at which the paint is a SOLID film: full thickness from here inward. Shared with the
/// light pass (its coverage weighting rides the same [`body_profile`]), so "solid paint" is one
/// concept with one pair of numbers on both sides of the pipeline. // CLAMP-OK
pub const W_SOLID: f32 = 0.75;

/// The **body curve**: how the dab's silhouette becomes the paint's *body*.
///
/// `h = depth × coverage × w` copies the colour's own soft profile into the relief — and for the
/// default brush (hardness 0) that is a dome the full width of the stroke, which reads as a blur, not
/// as paint (measured: shading peak 7.3 levels at 31% of the half-width; 1 level at the visible
/// edge). Nobody in the state of the art does that: Photoshop's bevel is a distance profile from the
/// coverage EDGE (Chisel), Blender's Layer brush caps at a fixed height ("creates the appearance of a
/// flat layer"), Hertzmann (NPAR 2002) gives height its own texture, and Painter documents Uniform as
/// "even depth". See `docs/Painter/17_impasto_deposito_pesquisa2.md` §3.
///
/// So the height gets its own profile: a **plateau** wherever the paint is solid (`w ≥ W_SOLID`),
/// **nothing** over the stain (`w ≤ W_TAIL` — the translucent rim keeps its pigment and stays flat),
/// and the **shoulder** — the wall the light lives on — in between, standing on pigment-backed
/// pixels a little inside the stain's edge, the way a real paint film ends inside its own smear.
/// Because a falloff is monotone in distance, remapping `w` IS a profile-in-distance (the bevel), it
/// commutes with the stroke envelope's `max`, and rule 1 still holds: the height consumes exactly
/// the silhouette the colour consumes. The shoulder's width follows the brush's own softness — a
/// soft brush lays a softer body edge — which is the coupling that should exist; the dome was the
/// one that shouldn't.
#[inline]
#[must_use]
pub fn body_profile(w: f32) -> f32 {
    let t = ((w - W_TAIL) / (W_SOLID - W_TAIL)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The **film** the brush actually lays: a dab's coverage, cut to the paint that carries a BODY.
///
/// ## The bug this closes
///
/// Enio, 2026-07-12, two screenshots of the same crossing strokes: *"o efeito leva em consideração os
/// limites do pincel e não o peso do relevo. Este falloff (smooth) pinta tinta fora do relevo. Usando o
/// falloff Sphere fica mais preciso e a tinta corresponde ao relevo."*
///
/// He had found a real seam. Two things already agreed on where paint stops carrying a body — the relief
/// ([`body_profile`], zero below [`W_TAIL`]) and the light (it weighs its shading by the same curve, so
/// it will not bleach the paper showing through a translucent rim). The **pigment** knew nothing about
/// it: the colour kernel deposited all the way out to the dab's geometric rim. So every impasto stroke
/// carried a skirt of paint that the light was *right* to refuse to model, and it read as a haze around
/// the ridge.
///
/// Its width is a pure function of the falloff, which is exactly why the falloff looked like the culprit:
/// coverage `W_TAIL` sits at `t = 0.61` under `Smooth` (39% of the radius — 16 px on a 40 px brush) and
/// at `t = 0.94` under `Sphere` (6%). Sphere was not more *precise*; it simply has almost no skirt.
///
/// ## The rule
///
/// **A brush that lays no body lays no paint.** One curve, one threshold, one definition of "paint":
/// wherever the light gives a pixel no shading, the brush gives it no pigment either.
///
/// It closes exactly because the light's weight is `body_profile(cover)` and `cover` is the RAW paint
/// (`silhouette × dynamics`), which this does not touch: the film is that same curve on that same
/// quantity, so the pigment's support and the lit region are the *same set*, at every falloff and every
/// pressure. The stroke does not come out narrower — the lit ridge was already only `t < 0.61` wide; what
/// goes is the haze around it.
///
/// It also leaves the ingredients alone, so the whole Body card stays live: Depth, Body, Depth Source,
/// Smoothing and Push still re-derive the last stroke. `Body` shapes the *relief's profile* (dome ↔ slab)
/// over a film whose edge is fixed — which is the right split, because a film's edge is a fact of the
/// paint that is already down, and its cross-section is not.
///
/// ## The cut belongs to the SILHOUETTE — not to the grain, and not to the dynamics
///
/// `sil` is the dab's bare silhouette (falloff, or a Shape image) — **before** the Grain multiplies it and
/// **before** the dab's pressure × Flow × Strength scale it. Both exclusions were paid for:
///
/// - **Not the dynamics.** Cutting the dab's *full* coverage silently kills the brush: at Strength 0.5 the
///   peak is 0.25, under [`W_TAIL`], so the curve returns zero for every texel and the stroke deposits
///   **nothing at all** (`the_film_never_starves_the_brush_at_low_strength`). The physics agrees: a film's
///   edge is a property of the tip, not of how hard you press. A light touch lays a THINNER film, not a
///   film with a different outline.
/// - **Not the grain.** The relief the light weighs (`cover`) is the silhouette × dynamics — the Grain is
///   deliberately *not* in it, because a Grain textures the pigment and does not carve the body
///   ([`DepthSource::Uniform`]). Cut the film through the grain and its valleys lose their pigment while
///   keeping their full body: the light then shines, at full strength, on bare paper. Measured at 124
///   levels over 1694 px before this was moved (`impasto_light_does_not_shade_paint_that_is_not_there`).
///
/// So the film reshapes the **silhouette**, once, and everything downstream — grain, dynamics, the
/// Accumulate-OFF cap, the ramps, the per-layer colour — consumes the reshaped one and needs no
/// arithmetic of its own. The relief consumes the RAW silhouette (`cover = sil × dynamics`), so the
/// light's weight `body_profile(cover)` is the film's own alpha: the pigment's support and the lit region
/// are the same set, which is the property the gate states.
#[inline]
#[must_use]
pub fn film_coverage(lays_body: bool, sil: f32) -> f32 {
    if lays_body { body_profile(sil) } else { sil }
}

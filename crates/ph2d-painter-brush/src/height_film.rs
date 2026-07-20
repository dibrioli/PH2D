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

/// The normalised dab distance `t` at which a bare-falloff silhouette's BODY ends — where its
/// coverage first drops to [`W_TAIL`], the SAME threshold [`film_of`] uses to decide a texel carries
/// paint at all. Inside this `t` the paint has a body; beyond it there is only the translucent stain,
/// then bare canvas.
///
/// It is the anchor the displaced paint must bank from: **a brush shoves paint to where the paint
/// STOPS**, which is the body's edge — not the dab's geometric rim (`t = 1`). Anchoring the rim at
/// `t = 1` on a soft falloff stood a hard, perfectly circular collar of banked paint a whole
/// silhouette's-width of BARE CANVAS out from the soft paint it was meant to be shoved by — Enio's
/// smoke, 2026-07-15: *"é usada a circunferência do gizmo do brush para empurrar a massa e não o
/// alpha do falloff"*. Under `Smooth`, `W_TAIL` sits at `t ≈ 0.61` (§14.1 of doc 16): on a 40 px
/// brush the visible paint ends at ~24 px and the old rim was born at 40 px — 16 px of naked canvas
/// between them. This closes that gap; the ridge's foot now rises from the body's edge.
///
/// A **hard-edged** falloff (Constant, or hardness ≥ 1) carries its body right out to the geometric
/// rim, so its edge IS `t = 1` and the rim anchors exactly where it always did — **byte-identical**,
/// and there was no soft skirt to leave a gap in the first place. The [`RIM_PROBE`] guard returns
/// exactly `1.0` for those, so the bisection never approximates a value a hair below it.
///
/// The silhouette is monotone in `t`, so this is a bisection on [`crate::BrushSpec::falloff_weight`]
/// (which folds in the brush's hardness) — cheap, and meant to be hoisted **once per stroke** (the
/// falloff and hardness do not change mid-stroke). The **Shape** slot has no radial body edge (an
/// Image tip is a stamp, a procedural one is masked per-pixel); the caller keeps `t = 1` there — see
/// [`crate::height_push::rim_t0`].
#[must_use]
pub fn body_edge_t(spec: &crate::BrushSpec) -> f32 {
    /// A hair inside the geometric rim: if the silhouette is still solid here, the body reaches the
    /// edge (Constant / hardness ≥ 1) and the anchor is exactly `t = 1`. No soft falloff comes within
    /// `W_TAIL` of the rim (Sphere, the flattest, is ~0.045 here), so this fires only for hard tips.
    const RIM_PROBE: f32 = 0.999;
    if spec.falloff_weight(RIM_PROBE) >= W_TAIL {
        return 1.0;
    }
    // Bisection: `falloff_weight` decreases monotonically from 1 at the centre to 0 at the rim, so the
    // invariant `weight(lo) ≥ W_TAIL > weight(hi)` holds from `lo = 0` / `hi = 1` and is preserved.
    // 24 halvings ⇒ ~6e-8 px of `t`, far under a texel; `lo` is the last distance that still has body.
    let mut lo = 0.0f32;
    let mut hi = 1.0f32;
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        if spec.falloff_weight(mid) >= W_TAIL {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
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
    if lays_body { film_of(sil) } else { sil }
}

/// The film a bare silhouette leaves: its body ([`body_profile`]) seen as an OPACITY
/// ([`film_opacity`]). The two ends are constants, and skipping them is not a micro-optimisation — the
/// tail is most of a dab's bounding box, and this runs per texel per dab in the height kernel.
#[inline]
#[must_use]
pub fn film_of(sil: f32) -> f32 {
    if sil <= W_TAIL {
        return 0.0; // the stain: no body, so no paint
    }
    if sil >= W_SOLID {
        return 1.0; // solid paint, and opaque long before this
    }
    film_opacity(body_profile(sil))
}

/// How **opaque** a film of paint of this thickness is.
///
/// ## Opacity is not thickness — and getting that wrong left the haze standing
///
/// Enio, 2026-07-12, third smoke: *"regrediu ao deixar a tinta extravasar o relevo e não resolveu a
/// distância da tinta levantada."* The film was cutting the pigment at the body's edge, and the support
/// of the two matched exactly — the gate said so and the gate was right. It was also **too weak to see
/// what he saw**: the alpha *ramped* with the body over ~8 px, so the stroke ended in a soft gradient of
/// pale red carrying no 3D form at all. A soft gradient with no form IS a haze. Measured, on the smoke's
/// own brush: ink 227 → 0 across `t = 0.38 … 0.57`, with the shading fading in lockstep — the two agree
/// perfectly and the edge still reads as fog.
///
/// The physics I had wrong: a film's **opacity saturates long before its thickness does**
/// (Beer–Lambert). Oil paint at a tenth of full thickness is already all but opaque — that is why a
/// palette knife leaves an EDGE and not a gradient. Modelling alpha as proportional to body was
/// modelling paint as glass.
///
/// So the deposit's alpha is the film's opacity: `1 − (1 − d)⁸`, saturating fast, transcendental-free
/// (HR-5 — three squarings, no `exp`). The paint is opaque right up to where the body ends and then it
/// **stops**, over the ~1 px the silhouette needs to fall the rest of the way. That is also why Enio's
/// `Sphere` looked right: its silhouette is nearly flat to the rim, so its film reached full body over a
/// pixel or two — it was already doing this, by accident of shape. Now every falloff does.
///
/// It closes the halo from the other side too, and that is not a coincidence: the pass may not bleach the
/// **paper seen through translucent paint**, and this is the function that says there is barely any
/// translucent paint to see through. // CLAMP-OK
#[inline]
#[must_use]
pub fn film_opacity(d: f32) -> f32 {
    let q = (1.0 - d.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let q2 = q * q;
    let q4 = q2 * q2;
    1.0 - q4 * q4
}

/// How much **solid paint** one dab lays at a texel — the quantity the light weighs its shading by.
///
/// `sil` is the bare silhouette, `dynamics` the dab's folded pressure × Flow × Strength. It is exactly
/// [`film_coverage`] scaled by the dynamics — the film's own alpha — and the factorisation is the point.
///
/// ## The hole it closes (Enio's bug, at partial pressure)
///
/// The light used to weigh by `body_profile(cover)` where `cover` was the raw paint, `sil × dynamics`.
/// The dynamics were INSIDE the curve, so they could starve it: at Flow × Strength × pressure ≈ 0.35 the
/// argument falls under [`W_TAIL`] for every texel and the light models **nothing anywhere on the
/// stroke** — while the pigment (cut on the silhouette, [`film_coverage`]) is still there. That is the
/// haze all over again, and it is the exact species of bug the film was written to kill; it merely hid
/// behind the mouse, which always presses at 1.0.
///
/// It is the same theorem as the film's, applied to the other side: **the threshold belongs to the
/// silhouette; the dynamics multiply afterwards.** A light touch lays a THINNER film — less pigment, less
/// body, less light — not a film the light refuses to look at.
///
/// At full dynamics `dynamics × body_profile(sil)` and `body_profile(sil × dynamics)` are the same
/// number, so the light is unchanged for every stroke drawn with a mouse — and every gate that pins its
/// look was drawn with one.
#[inline]
#[must_use]
pub fn solid_paint(sil: f32, dynamics: f32) -> f32 {
    (dynamics * film_of(sil)).clamp(0.0, 1.0)
}

// ── Screen-space AA of the film silhouette (BUGS #16, impasto half — Enio 2026-07-20) ──────────────

/// Sub-texel offsets of the AA reconstruction grid (3×3, ±0.667 texel) — the same grid the
/// watercolor AA settled on (`watercolor_field::AA_SS`): wider than the unit box so a footprint of
/// ~1.3 texels reconstructs a sub-texel film step as a soft ramp. `LITERAL-PX-OK`: AA geometry.
const AA_SS: [f32; 3] = [-0.667, 0.0, 0.667]; // LITERAL-PX-OK

/// Screen-space anti-aliasing of the film's silhouette, hoisted **once per dab**.
///
/// The film's transition occupies a FIXED interval of `t = distance/radius` (a property of the
/// falloff shape), so its screen width shrinks with the radius: a thin impasto stroke's edge crosses
/// it in well under a texel and snaps to a binary stair-step — the impasto half of the watercolor's
/// "borda dura pixelada" (BUGS #16; measured: radius 10 jumps `255 → 0` in one texel). Unlike the
/// watercolor, the impasto pigment's alpha **composites linearly** — so the film's fractional
/// texel-area coverage can replace `film_of(sil)` directly, in BOTH consumers (the pigment funnel in
/// `dab.rs` and the light's coverage envelope in `height.rs`), and the invariant the film exists for
/// — pigment support == lit region — holds texel-for-texel by construction.
///
/// `None` (checkbox off / no body laid / a Shape image tip) = the callers keep the single-sample
/// `film_of`, byte-for-byte. A Shape image is a STAMP and its edge is hard by design (the same
/// precedent as `body_edge_t` / `rim_t0`: an image tip has no radial body edge).
pub struct FilmAa {
    /// `t` below which the film is solidly 1 for every subsample (band start − sub-texel reach).
    t_lo: f32,
    /// `t` above which the film is solidly 0 for every subsample (band end + sub-texel reach).
    t_hi: f32,
}

impl FilmAa {
    /// Build the per-dab AA plan, or `None` when the single-sample path applies (see type docs).
    #[must_use]
    pub fn for_dab(spec: &crate::BrushSpec, shape_active: bool, radius: f32) -> Option<Self> {
        if !spec.film_aa_wanted(shape_active) || radius <= 0.0 {
            return None;
        }
        // Sub-texel reach in `t` units: the grid's diagonal extent (0.667·√2 texels) over the radius.
        let reach_t = 0.943 / radius; // LITERAL-PX-OK: 0.667·√2, the AA grid's diagonal reach
        // The band's inner edge is where the film SATURATES, not where it reaches `W_SOLID`:
        // `film_of` is sub-quantum from 1.0 (< 1/512 in u8) by sil ≈ 0.58, well before the body is
        // solid at 0.75 — supersampling the stretch between the two averages a constant, and at a
        // big radius that stretch is most of the ring (Sphere r=100: 28 → 12.5 texels, measured
        // ~+2 ms/move of pure waste in the perf kill gate).
        const FILM_SATURATED_SIL: f32 = 0.58; // LITERAL-PX-OK: film_of ≥ 1 − 5.2e-4 here (u8-exact)
        let (t_solid, t_tail) = if matches!(spec.falloff, crate::Falloff::Custom) {
            // A Custom curve need not be monotone, so a bisection could find A crossing instead of
            // THE band — take the whole dab as the band (correct, just unhoisted).
            (0.0, 1.0)
        } else {
            (cross_t(spec, FILM_SATURATED_SIL), cross_t(spec, W_TAIL))
        };
        Some(Self {
            t_lo: t_solid - reach_t,
            t_hi: t_tail + reach_t,
        })
    }

    /// The film at a texel: outside the rim band, exactly `film_of(sil_centre)` — the old single
    /// sample, byte-identical for the whole interior and exterior; inside, the **mean of `film_of`
    /// over the sub-texel grid** — the texel's fractional area under the film. `sil_at(ox, oy)` is
    /// the caller's own silhouette chain evaluated at the sub-texel offset, so the fraction follows
    /// whatever geometry the caller stamps (disc or swept capsule) with no second formula.
    #[inline]
    #[must_use]
    pub fn film_at(&self, t: f32, sil_centre: f32, sil_at: impl Fn(f32, f32) -> f32) -> f32 {
        if t <= self.t_lo || t >= self.t_hi {
            return film_of(sil_centre);
        }
        let mut acc = 0.0;
        for &oy in &AA_SS {
            for &ox in &AA_SS {
                acc += film_of(sil_at(ox, oy));
            }
        }
        let frac = acc * (1.0 / (AA_SS.len() * AA_SS.len()) as f32);
        // The toe is cut at two u8 quanta, at the DOOR: below it the pigment blend and the film
        // byte quantize INDEPENDENTLY (one rounds to nothing, the other to 1) and the light shades
        // bare canvas — the support equality the film exists for, broken by rounding. Both
        // consumers inherit the same cut, so the supports stay one set.
        if frac < 2.0 / 255.0 { 0.0 } else { frac }
    }

    /// Whether the AA is active for this dab — the bbox pads by one texel so the outermost
    /// fractional ring is not clipped at small radii (a texel whose CENTRE lies past the geometric
    /// rim can still be partially covered).
    #[must_use]
    pub fn pad_px(aa: &Option<Self>) -> f32 {
        if aa.is_some() { 1.0 } else { 0.0 }
    }
}

/// The `t` where [`crate::BrushSpec::falloff_weight`] crosses `v` — the [`body_edge_t`] bisection,
/// generalised to any threshold (same hard-tip probe, same convergence: ~6e-8 of `t`).
fn cross_t(spec: &crate::BrushSpec, v: f32) -> f32 {
    const RIM_PROBE: f32 = 0.999;
    if spec.falloff_weight(RIM_PROBE) >= v {
        return 1.0; // hard tip: the body reaches the geometric rim, the crossing IS t = 1
    }
    let mut lo = 0.0f32;
    let mut hi = 1.0f32;
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        if spec.falloff_weight(mid) >= v {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

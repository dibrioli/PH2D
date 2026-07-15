//! Gates for **Inflate** — rewritten after Enio's smoke of 2026-07-14 (*"parece fazer a mesma coisa de
//! Layer"*), because the ones that were here **pinned the bug**.
//!
//! ## What went wrong, so that the shape of these gates makes sense
//!
//! Inflate shipped as `toward = pre + Depth · n_z` — "raise along the normal, and a height field can only
//! move in `z`". Two things were wrong with it, and only the second one is interesting.
//!
//! 1. **It was upside down.** Re-project the offset graph back into a height field and the raise is
//!    `Depth · S` where `S = √(1+|∇h|²) = 1/n_z` — the *secant*, not the cosine. Steep places must move
//!    **more**, which is how a wall shifts sideways and a form gets *fatter*. It moved them **less**, which
//!    rounds a crest off. Inflate was a worse Smooth. See [`super::super::sculpt_offset`].
//!
//! 2. **Fixing the sign would not have fixed the tool.** Over the relief the *deposit actually lays*, the
//!    median painted texel has `n_z = 1.000` — a stroke's interior is dead flat — so `Depth·n_z`,
//!    `Depth/n_z` and plain `Depth` are the same number, and Inflate is Layer to the bit over every texel
//!    the artist is looking at. A per-texel formula **cannot** inflate: `h + d·S` is one Euler step of the
//!    offset PDE, and one step cannot move material *sideways*, which is the whole word.
//!
//! ## And the gate was green
//!
//! The old gate was `inflate_rounds_the_crest_instead_of_translating_it`, run on a synthetic mesa whose wall
//! fell a full load in 6 px. Two failures, both in the fixture, both mine:
//!
//! * **The name asserted the wrong intention.** "Rounds the crest off" *is* the bug. A gate cannot save you
//!   from a wrong idea; it can only hold you to it.
//! * **The fixture could not occur.** The deposit cannot make a wall that steep — the settle blurs it. So the
//!   gate proved a true thing about a canvas the product never produces
//!   ([[feedback_a_gate_only_proves_what_its_fixture_contains]]). I even *noticed* the gentle ridge made the
//!   tool look like a uniform raise, and concluded the ridge was hiding the bug. It was reporting it.
//!
//! So the load-bearing gate here — [`the_inflate_fattens_the_form_where_the_layer_only_raises_it`] — runs on
//! a stroke laid by the **real deposit**, and its oracle is the thing Enio was looking at: *how wide is the
//! paint*. The analytic one beside it pins the arithmetic exactly, on a ramp, where the offset of a plane has
//! a closed form.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::{BrushSpec, Falloff};
use std::sync::Arc;

const LAYER: u8 = 6;
const INFLATE: u8 = 7;

/// The light's height→pixel gain — the constant that makes the two axes of a height field comparable, and
/// therefore the one every geometric quantity over `h` has to cross. Re-read from the product so this file
/// cannot drift from it.
const UNIT: f32 = super::super::impasto_light::DEPTH_UNIT_PX;

/// Depth `+0.5` loads on the `0..1` track (`depth = (t − 0.5) · 2`).
const DEPTH_UP: f32 = 0.75;
/// Depth `−0.5` loads.
const DEPTH_DOWN: f32 = 0.25;

// ── The analytic oracle: a ramp, where the offset of a plane is a closed form ────────────────────────

/// **Inflate raises a slope by the SECANT of it, not the cosine.**
///
/// Offsetting a *plane* by a ball of radius `ρ` is exact and has no discretisation to argue about: the
/// supporting sphere touches at `u = ρG/S` and the plane comes up by
///
/// ```text
/// max over |u| ≤ ρ of [ G·u_x + √(ρ² − |u|²) ] = ρ·√(1 + G²) = ρ·S        (G = the slope, in PIXELS/pixel)
/// ```
///
/// which, back in paint-loads, is `Depth · S`. This gate builds the 3-4-5 triangle — `G = 0.75`, so
/// `S = 1.25` exactly — and asks for it.
///
/// The shipped formula computed `Depth / S`. At this slope that is `0.4` where the truth is `0.625`: the two
/// answers differ by 56%, and no tolerance can hide the difference between a number and its reciprocal.
///
/// **Mutation that must bleed:** drop a `DEPTH_UNIT_PX` from the curvature `a_curv = 1/(2·|Depth|·unit²)` in
/// `render_inflate`, which makes the ball sixteen times too tall and the raise sixteen times too large. (The
/// lift *sign* is not what this gate catches — a ramp is symmetric enough that the centre still rises by
/// ≈ Depth·S with the sign flipped; that inversion is pinned by
/// [`the_parabolic_blob_matches_the_brute_force_dilation`] instead. Verified: the flip leaves this one green.)
#[test]
fn the_inflate_raises_a_ramp_by_the_secant_of_its_slope() {
    let size = 96u32;
    let (mut t, layer, _) = sculpt_canvas(size);

    // A ramp of slope G = 0.75 as the LIGHT sees it, so S = √(1 + 0.75²) = 1.25 exactly.
    let g_px = 0.75 / UNIT; // loads per texel
    let c = 48.0f32;
    let n = (size * size) as usize;
    let before: Vec<f32> = (0..n)
        .map(|i| ((i as u32 % size) as f32 - c) * g_px)
        .collect();
    t.heights.insert(layer, Arc::new(before.clone()));
    t.covers.insert(layer, Arc::new(vec![255u8; n]));
    t.sync_relief_flags();

    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    t.set_sculpt_depth(DEPTH_UP); // +0.5 loads ⇒ a ball of 8 px
    drag(&mut t, &[[c, 40.0], [c, 48.0], [c, 56.0]]);

    let after = heights_of(&t, layer);
    let at = |x: u32, y: u32| -> f32 {
        let i = (y * size + x) as usize;
        after[i] - before[i]
    };

    let expected = 0.5 * 1.25; // Depth · S
    let got = at(48, 48);
    assert!(
        (got - expected).abs() < 0.02,
        "Inflate raised a slope-0.75 ramp by {got:.4} loads; the offset of a plane by a ball of Depth 0.5 \
         raises it by Depth·S = {expected:.4}, exactly. {}",
        if got < 0.5 {
            "It came out BELOW the Depth, which means the normal is being applied upside down (Depth/S, the \
             cosine) — the steep places are moving LESS, which rounds crests off instead of pushing the \
             form outward. That is a Smooth, and we have Smooth."
        } else {
            "The ball's cap is in the wrong unit, or the kernel is not a ball at all."
        }
    );
}

/// **On a flat, Inflate makes a ROUNDED DOME — not a flat Layer raise.** (Enio's 3rd smoke, 2026-07-14:
/// *"parece uma mistura de inflate com layer … estude o Blob do Blender."*)
///
/// The old gate here asserted the OPPOSITE — that Inflate equals Layer on a flat — and it was the behaviour
/// Enio rejected. Inflate is the **Blob** now: a ball whose radius follows the falloff, so on a flat it lifts
/// a smooth spherical mound — a rounded peak at the centre, tapering to nothing at the brush edge — instead
/// of the flat, falloff-scaled raise a Layer gives. That rounded centre is the whole of the fix.
///
/// The gate reads the profile down a radius: it must rise ≈ Depth at the centre, fall **monotonically and
/// convexly** (no flat plateau — the flat top is precisely the *"mistura com layer"* complaint), and reach
/// zero by the brush edge (*"na borda nada se move"*).
///
/// **Mutation that must bleed:** make `render_inflate`'s ball radius constant (`depth.abs()·unit` instead of
/// `·amount·unit`) — the dome flat-tops and the plateau check fails.
#[test]
fn on_a_flat_the_inflate_is_a_rounded_dome_not_a_layer_raise() {
    let size = 160u32;
    let n = (size * size) as usize;
    let (mut t, layer, _) = sculpt_canvas(size);
    t.heights.insert(layer, Arc::new(vec![0.5f32; n])); // dead-flat paint, half a load thick
    t.covers.insert(layer, Arc::new(vec![255u8; n]));
    t.sync_relief_flags();

    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    let mut b = t.paint.brush;
    b.radius_px = 40.0;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth; // a soft brush: the falloff is the ball's size profile
    t.paint.brush = b;
    t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
    t.set_sculpt_depth(DEPTH_UP); // +0.5 loads
    let c = 80.0f32;
    drag(&mut t, &[[c, c]]);

    let h = heights_of(&t, layer);
    let rise = |r: u32| -> f32 { h[(80 * size + 80 + r) as usize] - 0.5 };

    // Rises ≈ Depth at the centre.
    assert!(
        (rise(0) - 0.5).abs() < 0.08,
        "the blob's centre rose {:.3}, not ≈ 0.5 (Depth)",
        rise(0)
    );
    // Tapers to nothing at the brush edge — *"na borda nada se move"*.
    assert!(
        rise(40).abs() < 0.02,
        "the blob is still {:.3} loads high at the brush edge (40 px); it must taper to zero",
        rise(40)
    );
    // Monotonically decreasing AND rounded (no flat top): the drop from centre to quarter-radius is real,
    // which a flat Layer plateau would not have. This is the assertion the old "agree with Layer" gate had
    // exactly backwards.
    let (r0, r8, r16) = (rise(0), rise(8), rise(16));
    assert!(
        r0 > r8 && r8 > r16 && r16 > rise(24),
        "the blob's profile is not a monotone dome (centre {r0:.3}, 8px {r8:.3}, 16px {r16:.3})"
    );
    assert!(
        (r0 - r8) > 0.01,
        "the blob's top is FLAT (centre {r0:.3} vs 8 px {r8:.3}) — a flat top is the *mistura com layer* \
         Enio rejected; the ball radius must follow the falloff so the centre is a rounded peak"
    );
}

/// **The offset reaches ρ√2 and then STOPS — it is a ball, not a runaway rectangle.** (Enio's 3rd smoke,
/// 2026-07-14: *"funciona mas com um falloff de influência retangular bizarra"*.)
///
/// The Blob dilates the absolute height `pre + lift`, which is right — that is what rolls the ball over the
/// existing form and pushes its rim out. But the separable engine is a **parabola**, and a parabola has no
/// support: a source of height `H` lifts everything within `√(H/a)` of it, which for THICK built-up paint is a
/// hundred texels. Written only inside `kr = brush + 2ρ`, that runaway is clipped to the rectangle — the hard
/// square Enio saw around the dome.
///
/// A true ball of radius ρ reaches ρ, no matter how tall the cliff it rolls against. The fix caps the composed
/// argmax distance at ρ√2 (`dx² + dy² ≤ 2ρ²`, the radius where the parabola has fallen the full |Depth| and a
/// real ball ends) and falls the rest back to `pre`. It is **circular** — it bounds `dx²+dy²`, not each axis —
/// so it can never itself draw a square.
///
/// The fixture is the screenshot: a tall isolated plateau (thick paint) with bare canvas around it, inflated.
/// The near probe (inside ρ√2) is the **presence sibling** — the form genuinely fattens by the ball's radius,
/// so the far probe reading `pre` proves the reach is BOUNDED, not that Inflate does nothing. The far probe
/// (past ρ√2, still inside `kr`) must stay at `pre`: no shelf, no square.
///
/// **Mutation that must bleed:** let the clamp never fire (widen `reach2`) — the far probe rises to ~20 loads
/// (verified: 19.78), the plateau's parabola clipped to `kr`, which is exactly the shelf Enio saw. The erode
/// sibling below is what proves the *rim re-sample* (rather than a blunt fall-to-`pre`) is the right fall-back:
/// snapping the argmin to `pre` there would cancel the dig on a thin ridge.
#[test]
fn the_inflate_offset_reach_is_bounded_not_a_runaway_rectangle() {
    let size = 160u32;
    let n = (size * size) as usize;
    let (mut t, layer, _) = sculpt_canvas(size);
    // A tall plateau — thick built-up paint — of radius 10, over bare canvas. The height (20 loads) is what
    // makes the parabola's reach run away: √(20/a) ≈ 100 texels at Depth 1.
    let c = 80i32;
    let plateau: Vec<f32> = (0..n)
        .map(|i| {
            let (x, y) = ((i as i32 % size as i32) - c, (i as i32 / size as i32) - c);
            if x * x + y * y <= 100 { 20.0 } else { 0.0 }
        })
        .collect();
    t.heights.insert(layer, Arc::new(plateau.clone()));
    t.covers.insert(layer, Arc::new(vec![0u8; n]));
    if let Some(cov) = t.covers.get_mut(&layer) {
        let cov = Arc::make_mut(cov);
        for (i, c0) in cov.iter_mut().enumerate() {
            let (x, y) = ((i as i32 % size as i32) - c, (i as i32 / size as i32) - c);
            if x * x + y * y <= 100 {
                *c0 = 255;
            }
        }
    }
    t.sync_relief_flags();

    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    b.falloff = Falloff::Constant;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
    t.set_sculpt_depth(1.0); // Depth +1.0 ⇒ ρ = 16 px, ρ√2 ≈ 22.6, reach² = 512
    drag(&mut t, &[[c as f32, c as f32]]);

    let after = heights_of(&t, layer);
    let at = |dx: i32| -> f32 { after[((c + dx) + c * size as i32) as usize] };

    // NEAR (18 px out, inside ρ√2): the plateau grew outward — the form is genuinely fatter here. This is the
    // presence sibling: it must be lifted so the far probe's `pre` reads as BOUNDED, not as a dead tool.
    assert!(
        at(18) > 5.0,
        "the offset did not fatten the plateau at 18 px (inside ρ√2): rose {:.2}, expected the ball to carry \
         the form outward. If this is ~0 the offset is not reaching at all and the far-probe check below is \
         vacuous.",
        at(18)
    );
    // FAR (35 px out, past ρ√2 but still inside kr = brush + 2ρ): must be untouched canvas. The runaway
    // parabola would put ~18 loads of shelf here, and the rectangular kr would clip it into the square.
    assert!(
        at(35) < 1.0,
        "35 px past the plateau — beyond the ball's ρ√2 reach but inside the write region — rose to {:.2} \
         loads. A ball of radius ρ cannot reach here; this is the unbounded parabola's runaway skirt, and \
         clipped to the rectangular kr it is exactly the *falloff retangular bizarra* Enio reported.",
        at(35)
    );
}

/// **The separable parabolic dilation equals the brute-force `O(N²)` one — the fast path is the true one.**
///
/// The Blob's engine is Felzenszwalb's `O(N)` lower-envelope sweep, twice (x then y). A subtle sign or
/// intersection error would give a plausible-but-wrong dome — the first cut inverted the lift sign and
/// turned a flat top into a peak (the fatten gates caught it, but only on a shaped fixture). This pins the
/// algorithm itself: for a random field, the swept result must match the naïve `max over all p of
/// [f(p) − a·|q−p|²]`, to the visible bit.
///
/// **Mutation that must bleed:** flip the `out_val` lift sign in `ParabolaScratch::transform` (`sign` →
/// `−sign`); the dome inverts and this diverges everywhere.
#[test]
fn the_parabolic_blob_matches_the_brute_force_dilation() {
    use super::super::sculpt_offset::{blob_dilate, unpack_src};
    let (w, h) = (24u32, 18u32);
    let (wu, hu) = (w as usize, h as usize);
    // A deterministic, bumpy field (HR-5: no RNG in a gate).
    let g: Vec<f32> = (0..wu * hu)
        .map(|i| {
            let x = (i % wu) as f32;
            let y = (i / wu) as f32;
            0.3 * (x * 0.7).fract() + 0.5 * (y * 0.4 + x * 0.1).fract()
        })
        .collect();
    let a = 1.0 / (2.0 * 0.5 * 16.0 * 16.0); // the Blob's curvature at Depth 0.5

    let (fast, src) = blob_dilate(&g, w, h, a, true);

    // The naïve dilation: for each output, the max over EVERY source of the parabola, and its argmax.
    let mut worst = 0.0f32;
    let mut src_mismatch = 0u32;
    for qy in 0..hu {
        for qx in 0..wu {
            let mut best = f32::NEG_INFINITY;
            for py in 0..hu {
                for px in 0..wu {
                    let d2 = ((qx as f32 - px as f32).powi(2)) + ((qy as f32 - py as f32).powi(2));
                    best = best.max(g[py * wu + px] - a * d2);
                }
            }
            let o = qy * wu + qx;
            worst = worst.max((fast[o] - best).abs());
            // The argmax can tie; only count a mismatch when the fast pick's VALUE is worse than the
            // brute pick's (a real error), not when two equal-value sources were chosen differently.
            let (dx, dy) = unpack_src(src[o]);
            let (sx, sy) = (qx as i64 + dx, qy as i64 + dy);
            if sx >= 0 && sy >= 0 && (sx as usize) < wu && (sy as usize) < hu {
                let picked = g[(sy as usize) * wu + (sx as usize)]
                    - a * ((qx as f32 - sx as f32).powi(2) + (qy as f32 - sy as f32).powi(2));
                if picked < best - 1e-4 {
                    src_mismatch += 1;
                }
            }
        }
    }
    assert!(
        worst < 1e-4,
        "the separable parabolic dome differs from the brute-force one by {worst:e} — the O(N) sweep is not \
         the O(N²) truth, so the fast Blob is a different (wrong) shape"
    );
    assert_eq!(
        src_mismatch, 0,
        "the composed argmax pointed at a source the brute force beats — the paint would follow the wrong \
         texel and the fattening would carry the wrong colour"
    );
}

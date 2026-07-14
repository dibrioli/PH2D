//! Gates for **Wave 3** — the Chisel, the Layer and the Inflate
//! (`docs/Painter/18_plano_sculpt_relevo.md` W3).
//!
//! Three verbs, and each one is here because it does something none of the other five can:
//!
//! * **Chisel** — the knife on its edge. It is Scrape plus one `abs`, and at Angle 0 it *is* Scrape, to the
//!   byte. Both halves of that sentence are gated, and the second is the sharper one: a degenerate case that
//!   quietly stopped being degenerate is how a "new" verb silently becomes a slightly-wrong old one.
//! * **Layer** — bounded build-up. `k ≤ 1` and the target is the constant `pre + Depth`, so the coat is one
//!   Depth thick however long the artist dwells. The gate dwells.
//! * **Inflate** — raise along the surface normal. Its entire reason to exist is that flats rise and walls do
//!   not, so the gate measures exactly that ratio — and it uses a surface with a REAL wall, because on a
//!   gentle one every implementation (including a plain uniform raise) looks identical.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use std::sync::Arc;

const SCRAPE: u8 = 3;
const CHISEL: u8 = 5;
const LAYER: u8 = 6;
const INFLATE: u8 = 7;

/// A **plateau**: a flat mesa of paint with a genuinely steep wall, and nothing else.
///
/// The wall is what makes an Inflate gate mean anything. On the gentle ridge of `sculpt_canvas` the slope is
/// 0.5 loads over 60 px, so `n_z = 0.99` and a correct Inflate is indistinguishable from a uniform raise —
/// the gate would be green against a kernel that ignores the normal entirely. Here the wall falls a full load
/// over 6 px, which the light reads as a 2.7:1 slope and the normal as `n_z = 0.35`.
fn plateau_canvas(size: u32) -> (PainterTool, crate::tool::RtLayerId, Vec<f32>) {
    let (mut t, layer, _) = sculpt_canvas(size);
    let n = (size * size) as usize;
    let c = size as f32 * 0.5;
    let relief: Vec<f32> = (0..n)
        .map(|i| {
            let x = (i as u32 % size) as f32;
            let y = (i as u32 / size) as f32;
            let r = ((x - c) * (x - c) + (y - c) * (y - c)).sqrt();
            // Flat top out to r = 30, then a wall falling one full load over 6 px.
            (1.0 - ((r - 30.0) / 6.0).clamp(0.0, 1.0)).clamp(0.0, 1.0)
        })
        .collect();
    t.heights.insert(layer, Arc::new(relief.clone()));
    t.covers.insert(layer, Arc::new(vec![255u8; n]));
    t.sync_relief_flags();
    (t, layer, relief)
}

/// **At Angle 0 the Chisel IS Scrape — byte for byte.**
///
/// The flat blade is not a special case in the code: `tilt = tan(0) = 0`, the `+ tilt·|lateral|` term
/// vanishes, and the same plane accumulator runs. Gating it is what keeps that true. A degenerate case that
/// quietly stops being degenerate is how a new verb turns into a slightly-wrong copy of an old one, and
/// nobody notices because both look plausible.
///
/// It also means the Angle slider's bottom end is honest: it is the flat knife, not a dead zone.
///
/// **Mutation that must bleed:** floor the tilt (`tan(angle).max(0.05)`), or drop the `Chisel` arm from the
/// `delta.min(0.0)` clamp so it stops being a scrape at all.
#[test]
fn the_chisel_at_zero_degrees_is_byte_identical_to_scrape() {
    let size = 200u32;
    let path = [[100.0, 60.0], [100.0, 100.0], [100.0, 140.0]];

    let run = |mode: u8, angle: f32| -> Vec<f32> {
        let (mut t, layer, _) = sculpt_canvas(size);
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        t.set_sculpt_offset(0.3); // a real bite, so there is something to be identical ABOUT
        t.set_sculpt_angle(angle);
        drag(&mut t, &path);
        heights_of(&t, layer)
    };
    let scrape = run(SCRAPE, 0.0);
    let chisel = run(CHISEL, 0.0);

    let differing = scrape
        .iter()
        .zip(&chisel)
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    assert_eq!(
        differing, 0,
        "the Chisel at 0° left a different relief from Scrape in {differing} texels. They are the same \
         kernel — the V's term is `tilt · |lateral|` and `tan(0)` is zero — so the flat blade must be the \
         flat blade, not a near-miss of it."
    );
    // …and the fixture actually cut something, or the equality above is the equality of two no-ops.
    let cut = scrape.iter().filter(|v| **v != 0.0).count();
    assert!(cut > 200, "fixture: the Scrape removed nothing to compare");
}

/// **The Chisel carves a crease: it spares the flanks, and the sparing grows with distance from the axis.**
///
/// That IS the V. The knife's two faces meet on the line the brush is travelling along, and they rise away
/// from it — so relative to a flat blade at the same Offset, the chisel takes off just as much on the axis
/// and progressively less to either side. What is left is a groove with a crease at the bottom.
///
/// The gate measures the *sparing profile* rather than an absolute shape, because the surface underneath is
/// bumpy and the absolute shape is the surface's business. The sparing is the knife's.
///
/// **Mutation that must bleed:** drop the `+ v` from `plane_sum` in `accumulate_dab_plane` (the chisel
/// becomes a scrape and spares nothing), or use `local · dir` instead of `local · perp` (the V folds about
/// the wrong axis — along the stroke instead of across it — and the sparing stops growing sideways).
#[test]
fn the_chisel_carves_a_crease() {
    let size = 200u32;
    // Straight DOWN the canvas, so the stroke's axis is vertical and "lateral" is the x offset.
    //
    // **x = 100.5, not 100.** A texel's centre sits at `x + 0.5`, so a stroke down x = 100 passes half a
    // texel to the side of texel 100 — and the V's whole point is that half a texel off the axis is already
    // off the axis. The gate would then be asserting that a chisel does not cut where it is not. Put the
    // axis ON a texel centre and the on-axis assert below means what it says.
    let path = [[100.5, 50.0], [100.5, 100.0], [100.5, 150.0]];

    let run = |mode: u8, angle: f32| -> Vec<f32> {
        let (mut t, layer, _) = sculpt_canvas(size);
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        t.set_sculpt_offset(0.25); // sink the plane: the knife has to bite before it can crease
        t.set_sculpt_angle(angle);
        drag(&mut t, &path);
        heights_of(&t, layer)
    };
    let flat = run(SCRAPE, 0.0);
    let chisel = run(CHISEL, 0.6); // ~36°

    // How much MORE relief the chisel left standing, as a function of |x − 100|, on the row the brush swept.
    let spared = |dx: usize| -> f64 {
        let y = 100usize;
        let mut s = 0.0;
        for &x in &[100 - dx as i64, 100 + dx as i64] {
            let i = y * size as usize + x as usize;
            s += f64::from(chisel[i] - flat[i]);
        }
        s * 0.5 // the mean of the two sides — the V is symmetric about the axis
    };

    let on_axis = spared(0);
    let near = spared(4);
    let far = spared(9);

    assert!(
        on_axis.abs() < 1e-4,
        "on its own axis the chisel spared {on_axis:.4} loads. The V's two faces MEET there — the tilt term \
         is `tilt · |lateral|` and lateral is zero — so the knife must cut exactly as deep as a flat blade."
    );
    assert!(
        far > near && near > 0.03,
        "the chisel did not open a V: it spared {near:.4} loads at 4 px from the axis and {far:.4} at 9 px. \
         The sparing has to GROW with the lateral distance — that growth IS the groove's wall, and without \
         it the tool is a scrape wearing a new name."
    );
}

/// **Layer lays one coat, however long you dwell.**
///
/// The bound is not a clamp bolted on; it falls out of the kernel. `k = clamp(amount, 0, 1)` and the target
/// is the CONSTANT `pre + Depth`, so `h = pre + k·Depth` can approach `pre + Depth` and can never pass it.
/// That is the one thing neither the deposit (which accumulates, and is supposed to) nor the plane verbs
/// (which level toward a surface, not a thickness) can do: *this much paint, no more.*
///
/// The gate dwells — one stroke that passes over the same band three times, so `amount` there is well past 1
/// — and then asks for the ceiling back. It also asks that the coat actually ARRIVED (a kernel that laid
/// nothing would honour the ceiling triumphantly).
///
/// The ceiling is measured against `pre`, the relief this STROKE found, and a second stroke lays a second
/// coat. That is the semantics, not a leak: "one coat per pass" is what a palette knife does.
///
/// **Mutation that must bleed:** make the target `p + depth * a` (the un-clamped accumulation) — the triple
/// pass then lays three coats and blows the ceiling.
#[test]
fn layer_lays_one_coat_however_long_you_dwell() {
    let size = 200u32;
    let (mut t, layer, before) = sculpt_canvas(size);
    arm_sculpt(&mut t, LAYER, 0.5, 1.0);
    t.set_sculpt_depth(0.75); // +0.5 loads

    // ONE stroke, three passes over the same band: right, back, right again.
    drag(
        &mut t,
        &[
            [60.0, 100.0],
            [140.0, 100.0],
            [60.0, 100.0],
            [140.0, 100.0],
            [60.0, 100.0],
        ],
    );
    let after = heights_of(&t, layer);

    let risen: Vec<f32> = before.iter().zip(&after).map(|(b, a)| a - b).collect();
    let peak = risen.iter().copied().fold(f32::MIN, f32::max);
    assert!(
        peak <= 0.5 + 1e-5,
        "three passes of Layer at Depth 0.50 laid {peak:.4} loads. The coat is supposed to be BOUNDED — \
         `k ≤ 1` against a constant target — and a bound that a slow hand can exceed is not a bound, it is \
         a suggestion."
    );
    assert!(
        peak > 0.45,
        "Layer at Depth 0.50 laid only {peak:.4} loads even after three passes — the ceiling holds because \
         nothing reaches it, which is a no-op wearing a bound's clothes"
    );
    // Nothing was carved: Layer's Δ is positive at a positive Depth.
    let dug = risen.iter().filter(|d| **d < -1e-5).count();
    assert_eq!(dug, 0, "a positive Layer carved {dug} texels");
}

/// **Inflate rounds the crest off; it does not translate the paint upward.**
///
/// The whole tool is one line — `toward = pre + Depth · n_z` — and its entire behaviour lives in `n_z`. On a
/// flat the normal points straight up (`n_z = 1`) and the paint rises the full Depth; on a wall the normal
/// lies over (`n_z → 0`) and the paint barely moves. So a mesa's TOP comes up while its EDGE stays put, and
/// what was a sharp lip becomes a shoulder. That is inflation, in the only sense a height field can offer it
/// (§1.3 — and the docs say so rather than shipping a 3D name over a 2D operation).
///
/// **The normal is the LIGHT's normal**, gained by `DEPTH_UNIT_PX`, and that is the load-bearing part: the
/// height buffer's unit is not pixels, so without the gain a real wall reads `n_z = 0.999` and Inflate is a
/// uniform raise that no gate written in "loads per texel" would ever catch. A normal the light does not use
/// is a normal the artist cannot see ([[feedback_oracle_must_model_appearance_not_implementation]]).
///
/// **Mutations that must bleed** (both checked): return `1.0` from `inflate_nz`; and drop the
/// `* DEPTH_UNIT_PX` gain from its two differences.
#[test]
fn inflate_rounds_the_crest_instead_of_translating_it() {
    let size = 160u32;
    let (mut t, layer, before) = plateau_canvas(size);
    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    t.set_sculpt_depth(0.75); // +0.5 loads

    let c = f32::from(80u8);
    // A big brush parked on the mesa's edge, so the same dab covers flat top AND wall.
    let mut b = t.paint.brush;
    b.radius_px = 40.0;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;
    drag(&mut t, &[[c, c], [c + 1.0, c]]);

    let after = heights_of(&t, layer);
    let rise = |x: u32, y: u32| -> f32 {
        let i = (y * size + x) as usize;
        after[i] - before[i]
    };

    // The mesa's flat top (r ≈ 10 from the centre) — normal straight up.
    let flat = rise(90, 80);
    // Its wall (r ≈ 33, mid-fall) — the normal lies right over.
    let wall = rise(113, 80);

    assert!(
        flat > 0.4,
        "Inflate barely raised the flat top ({flat:.4} of a 0.50 Depth) — on a surface whose normal points \
         straight up it is supposed to lay the whole Depth"
    );
    assert!(
        wall < flat * 0.6,
        "Inflate raised the WALL by {wall:.4} and the flat top by {flat:.4} — nearly the same. It is not \
         reading the normal (or it is reading one without the light's `DEPTH_UNIT_PX` gain, which on a \
         height field is the same thing as not reading one): what it is doing is translating the paint \
         upward, which is Draw, and we have Draw."
    );
}

/// **A Height-family stroke costs 8 B/px — it holds no target buffer at all.**
///
/// Layer's target is the constant `pre + Depth`; Inflate's is `pre + Depth · n_z(pre)`, four reads and a
/// square root. Neither is worth a canvas-sized plane, and the family therefore carries `pre` + `amount` and
/// nothing else — a third less memory than the other six verbs, for the two that are cheapest anyway.
///
/// Counted on the real thing rather than reasoned about: *a rule that never observes cannot fire*
/// ([[feedback_a_rule_that_never_observes_cannot_fire]]).
///
/// **Mutation that must bleed:** size `plane_sum` for the Height family in `ensure_family_target`.
#[test]
fn a_layer_stroke_costs_eight_bytes_per_pixel() {
    let size = 128u32;
    let n = (size * size) as usize;
    let (mut t, _layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, LAYER, 0.5, 1.0);

    t.on_canvas_pointer(cp([40.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([90.0, 64.0], PointerPhase::Move));

    let s = &t.paint.sculpt;
    assert!(s.layer.is_some(), "fixture: the gesture is in flight");
    assert!(
        s.plane_sum.is_empty() && s.blurred.is_empty() && s.blur_done.is_empty(),
        "a Layer stroke allocated another family's target: {} B of plane_sum + {} B of blur memo. It reads \
         neither — its target is `pre` and a knob.",
        s.plane_sum.len() * 4,
        s.blurred.len() * 4
    );
    let live = (s.pre.len() + s.amount.len()) * 4;
    assert_eq!(
        live / n,
        8,
        "a live Layer stroke costs {} B/px, not 8",
        live / n
    );
}

/// **Switching INTO the Height family frees the other families' targets.**
///
/// The sibling of `switching_family_mid_shape_rebuilds_the_target` (Wave 2), from the other side: there, the
/// switch had to BUILD a target that only a re-stamp can produce. Here it has to RELEASE one — and if it does
/// not, the 8 B/px above is a number that is true only when you enter Layer first and never leave it, which
/// is the same as not being true.
///
/// **Mutation that must bleed:** make `drop_stale_family_target` keep the plane target in the Height arm.
#[test]
fn switching_into_the_height_family_frees_the_other_targets() {
    let size = 128u32;
    let (mut t, _layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, SCRAPE, 0.5, 1.0);

    t.on_canvas_pointer(cp([40.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([90.0, 64.0], PointerPhase::Move));
    assert!(
        !t.paint.sculpt.plane_sum.is_empty(),
        "fixture: a Scrape must have a plane target to release"
    );

    t.set_sculpt_mode(LAYER);
    assert!(
        t.paint.sculpt.plane_sum.is_empty(),
        "leaving Scrape for Layer kept the plane target alive ({} B) — the Height family never reads it, and \
         a session that hoards every family's buffer costs 16 B/px whatever the comments say",
        t.paint.sculpt.plane_sum.len() * 4
    );
}

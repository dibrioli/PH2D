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
//!
//! **Inflate has moved out** to [`super::inflate`], and its old gates here were DELETED rather than ported:
//! they pinned the bug Enio's smoke found (the verb was Layer wearing a normal's name), and the synthetic
//! cliff they ran on is a canvas the deposit cannot make. Read that module before touching this one — the
//! two most expensive mistakes of this wave are written up there.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

const SCRAPE: u8 = 3;
const CHISEL: u8 = 5;
const SMOOTH: u8 = 0;
const INFLATE: u8 = 7;
const LAYER: u8 = 6;
const SHARPEN: u8 = 1;
const FLATTEN: u8 = 2;
const FILL: u8 = 4;

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

    // The two verbs read the Offset track through DIFFERENT maps — the Chisel's runs `−MAX..=0`, because a
    // chisel cuts (see `SculptState::plane_offset`). So the fixture asks for the same offset in LOADS, which
    // is the unit the kernel works in, and then checks that it got it. Comparing at equal slider positions
    // would compare two different tools and call the difference a bug.
    let run = |mode: u8, angle: f32, norm: f32| -> (Vec<f32>, f32) {
        let (mut t, layer, _) = sculpt_canvas(size);
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        t.set_sculpt_offset(norm);
        t.set_sculpt_angle(angle);
        drag(&mut t, &path);
        (heights_of(&t, layer), t.sculpt_plane_offset())
    };
    let (scrape, off_s) = run(SCRAPE, 0.0, 0.3); // (0.3 − 0.5)·2 = −0.4 loads
    let (chisel, off_c) = run(CHISEL, 0.0, 0.6); // (0.6 − 1.0)   = −0.4 loads
    assert!(
        (off_s - off_c).abs() < 1e-6,
        "fixture: the two verbs were given different offsets ({off_s} vs {off_c} loads), so any difference \
         below would be the fixture's and not the kernel's"
    );

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

    // Same bite in LOADS, not the same slider position: the Chisel's Offset track is negative-only
    // (`−MAX..=0`) and Scrape's is two-sided, so equal `norm`s are different tools. What the kernel sees has
    // to match, or the "sparing" below is just the two runs cutting to different depths.
    let run = |mode: u8, angle: f32, norm: f32| -> (Vec<f32>, f32) {
        let (mut t, layer, _) = sculpt_canvas(size);
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        t.set_sculpt_offset(norm); // sink the plane: the knife has to bite before it can crease
        t.set_sculpt_angle(angle);
        drag(&mut t, &path);
        (heights_of(&t, layer), t.sculpt_plane_offset())
    };
    let (flat, off_f) = run(SCRAPE, 0.0, 0.25); // (0.25 − 0.5)·2 = −0.5 loads
    let (chisel, off_c) = run(CHISEL, 0.6, 0.5); // (0.50 − 1.0)   = −0.5 loads  ·  angle ≈ 36°
    assert!(
        (off_f - off_c).abs() < 1e-6,
        "fixture: the blade and the knife were sunk to different depths ({off_f} vs {off_c} loads)"
    );

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

/// **A Height-family stroke costs 8 B/px — it holds no target buffer at all.**
///
/// Layer's target is the constant `pre + Depth`, evaluated per texel in the render, so the family carries
/// `pre` + `amount` and nothing else — a third less memory than the other seven verbs, for the one that is
/// cheapest anyway. (Inflate USED to be here, on the strength of a per-texel formula that turned out not to
/// inflate anything; its target is a neighbourhood kernel and it now pays the memo's 4 B/px like Smooth.
/// See [`super::inflate`].)
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
        s.plane_sum.is_empty() && s.memo.is_empty() && s.memo_done.is_empty(),
        "a Layer stroke allocated another family's target: {} B of plane_sum + {} B of kernel memo. It reads \
         neither — its target is `pre` and a knob.",
        s.plane_sum.len() * 4,
        s.memo.len() * 4
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

// ── The Chisel's two corrections (Enio's smoke, 2026-07-14) ──────────────────────────────────────────

/// **The Chisel's Offset never goes positive — the whole track cuts.**
///
/// A chisel that cannot reach below the surface is not a chisel. Its target is `plane + V + offset` and it
/// only removes (`min(Δ, 0)`), and the plane is the least-squares fit *through* the paint — so at `offset ≥ 0`
/// the only material the blade can reach is whatever already stands above that fit. It nicks the brush marks.
/// It cannot open a groove, which is the one thing this verb exists to do, and the upper half of its track was
/// a slow fade into "Scrape, but weaker".
///
/// Swept across the whole slider rather than spot-checked at the ends: the map is a formula, and a formula is
/// wrong in the middle just as easily.
///
/// **Mutation that must bleed:** give the Chisel the two-sided map (delete its arm from `plane_offset`).
#[test]
fn the_chisels_offset_track_is_negative_all_the_way_across() {
    let (mut t, _layer, _) = sculpt_canvas(64);
    arm_sculpt(&mut t, CHISEL, 0.5, 1.0);

    for i in 0..=20u32 {
        let norm = f32::from(u16::try_from(i).unwrap_or(0)) / 20.0;
        t.set_sculpt_offset(norm);
        let off = t.sculpt_plane_offset();
        assert!(
            off <= 0.0,
            "at slider {norm:.2} the Chisel's Offset is {off:+.3} loads. A positive offset lifts the blade \
             clear of the fitted plane, and `min(Δ, 0)` then throws away everything it might have cut: the \
             knife stops carving and starts nicking."
        );
    }
    // The far end still DOES something (a track whose bottom is a no-op is the bug from the other side).
    t.set_sculpt_offset(0.0);
    assert!(
        (t.sculpt_plane_offset() + super::super::sculpt::PLANE_OFFSET_MAX).abs() < 1e-6,
        "the Chisel's deepest cut is not the full PLANE_OFFSET_MAX below the surface"
    );
    // …and Scrape, which is NOT a chisel, keeps both halves: a positive offset there is a gentler skim.
    t.set_sculpt_mode(SCRAPE);
    t.set_sculpt_offset(1.0);
    assert!(
        t.sculpt_plane_offset() > 0.0,
        "the negative-only clamp leaked out of the Chisel and onto Scrape, whose upper half is a real tool \
         (skim only what stands proudest of the plane)"
    );
}

/// **The Chisel asks the stroke engine for a heading — and only the Chisel does.**
///
/// The engine holds a stroke's opening dabs until travel has settled a direction (the rake **warm-up**), but
/// it only knew to do that for the two texture slots. The Chisel is a third reader of `Dab::dir`, so without
/// this its first dab carves a flat scrape (gated at the source in `ph2d_painter_brush::stroke::tests`).
///
/// The policy is the gate: the flag must be **on for the Chisel, off for everything else, and off the moment
/// the artist leaves Sculpt** — otherwise a plain paint brush inherits a warm-up it has no use for, on the
/// strength of a chisel the artist picked up ten minutes ago.
///
/// **Mutations that must bleed** (both checked): drop the `sync_stroke_heading_need()` call from
/// `set_sculpt_mode` (the flag never arms); and drop it from `set_paint_tool_mode` (it never disarms — the
/// live brush keeps it across the mode switch, because the slot swap restores a spec that was saved with it).
#[test]
fn only_the_chisel_asks_the_stroke_engine_for_a_heading() {
    let (mut t, _layer, _) = sculpt_canvas(64);

    arm_sculpt(&mut t, CHISEL, 0.5, 1.0);
    assert!(
        t.paint.brush.needs_heading,
        "the Chisel did not ask for a heading, so its opening dabs will arrive with `dir = [0, 0]` and carve \
         no V at all — the groove starts blunt on every stroke"
    );

    t.set_sculpt_mode(SCRAPE);
    assert!(
        !t.paint.brush.needs_heading,
        "Scrape kept the Chisel's heading need. It reads no direction — the flag is an enumeration of the \
         readers of `Dab::dir`, and a stale member is how it stops being one."
    );

    t.set_sculpt_mode(CHISEL);
    t.set_paint_tool_mode("brush");
    assert!(
        !t.paint.brush.needs_heading,
        "leaving Sculpt left the heading need on the PAINT brush. The slot swap restores a spec that was \
         saved while the Chisel was armed, so the answer has to be re-asked after the swap, not before it."
    );

    t.set_paint_tool_mode("sculpt");
    assert!(
        t.paint.brush.needs_heading,
        "coming back to Sculpt (still on the Chisel) did not restore the heading need"
    );
}

/// **The Sculpt brush rakes by default** (Enio 2026-07-14) — and that is a DEFAULT, not the mechanism.
///
/// The verbs that carve are directional: the Chisel's groove runs along the stroke, and a blade-shaped Shape
/// image has to turn with it or the silhouette points one way while the cut goes another. So the box is
/// ticked when the artist arrives.
///
/// The sibling assertion is the load-bearing one: **unticking it must not take the Chisel's V away.** That is
/// what `needs_heading` is for. If these two ever collapse into one flag, the artist who wants a fixed-angle
/// blade silhouette silently loses the start of every groove — which is the bug this pair exists to prevent
/// from coming back ([[feedback_two_doors_to_the_same_question_diverge]]).
#[test]
fn the_sculpt_brush_rakes_by_default_but_the_chisel_does_not_depend_on_it() {
    // A bare tool, NOT `sculpt_canvas` — that fixture stamps its own spec into every brush slot, which would
    // overwrite the very default this gate exists to read. A fixture that clobbers the thing under test is a
    // gate that can only ever agree with itself.
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    t.set_paint_tool_mode("sculpt");
    assert!(
        t.paint.brush.shape.rake,
        "the Sculpt brush arrived with Rake unticked"
    );

    t.set_sculpt_mode(CHISEL);
    // The artist unticks Rake (they want a fixed blade silhouette).
    let mut b = t.paint.brush;
    b.shape.rake = false;
    t.paint.brush = b;
    assert!(
        t.paint.brush.needs_heading,
        "unticking Rake took the Chisel's heading away with it. The checkbox governs a SILHOUETTE IMAGE's \
         rotation; the V's axis is not its business, and coupling them means the groove's start depends on a \
         box about something else."
    );
}

// ── Rake: the two ways to hold a knife (Enio 2026-07-14, second correction) ──────────────────────────

/// A quarter-circle stroke — the only path that can tell the two Rake modes apart.
///
/// On a STRAIGHT stroke the heading never turns, so "follow it live" and "lock it at the start" are the same
/// instruction and produce the same groove, to the byte. That identity is a gate below, not an inconvenience:
/// it is what proves the difference measured here is the CURVE and not noise.
fn quarter_circle(cx: f32, cy: f32, r: f32) -> Vec<[f32; 2]> {
    (0..=24u8)
        .map(|i| {
            let t = f32::from(i) * (std::f32::consts::FRAC_PI_2 / 24.0);
            [cx + r * t.cos(), cy + r * t.sin()]
        })
        .collect()
}

/// Run a chisel stroke along `path` with Rake on or off, and hand back the relief **and the angle the knife
/// was holding at the last dab**.
///
/// The lock has to be read from INSIDE the gesture: the session dies at pen-up (`end_sculpt_session`), and
/// with it the lock — as it must, because the angle belongs to the stroke and the stroke is over. So the
/// walk is spelled out here rather than going through `drag`, which finishes the stroke.
fn chisel_run(size: u32, rake: bool, path: &[[f32; 2]]) -> (Vec<f32>, Option<[f32; 2]>) {
    let (mut t, layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, CHISEL, 0.5, 1.0);
    t.set_sculpt_offset(0.5); // −0.5 loads: a real bite
    t.set_sculpt_angle(0.8); // ~48°: a V steep enough to read
    assert!(t.sculpt_rake(), "fixture: Rake is meant to default ON");
    if !rake {
        t.toggle_sculpt_rake();
    }
    assert_eq!(t.sculpt_rake(), rake, "fixture: the toggle did not take");

    t.on_canvas_pointer(cp(path[0], PointerPhase::Down));
    for p in &path[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
    }
    let lock = t.paint.sculpt.locked_dir; // …while the gesture is still alive
    t.on_canvas_pointer(cp(*path.last().unwrap(), PointerPhase::Up));
    (heights_of(&t, layer), lock)
}

/// **Rake ON: the knife turns with the stroke, in real time. Rake OFF: it holds the angle it entered at.**
///
/// Enio's own words (2026-07-14): *"o tool identifica a direção do traço e mantém o ângulo … até o final.
/// Isso é bom e deve permanecer assim — mas esse é o modo onde Rake NÃO está checado. Com Rake checado … o
/// ângulo se ajusta em tempo real."*
///
/// Both modes read the stroke; the difference is **when**. So the gate drags a quarter-circle — where the
/// heading sweeps 90° — and asks that the two grooves are not the same groove. On a straight line they would
/// be identical, which is the sibling below.
///
/// **Mutations that must bleed** (checked): make `chisel_dir` ignore the lock (`return dir` always — Rake off
/// becomes Rake on); and set `locked_dir` on every dab batch instead of only the first, so a "locked" knife
/// silently re-aims at each pointer move.
///
/// A third one does **not** bleed, and knowing why is worth more than the mutation: deleting the
/// `if rake { return dir }` early-return — "make Rake on behave like Rake off" — changes nothing, because the
/// lock is only ever *populated* while Rake is OFF (`stamp_dabs_sculpt` guards on `!rake`). So
/// `locked_dir.unwrap_or(dir)` already IS `dir` when raking. Two independent things hold the live path up,
/// and the `lock_on.is_none()` assert above is what pins the second of them. (The mutation that *does* break
/// the live path is `if rake { return [1.0, 0.0] }` — a knife that ignores the stroke — and it takes
/// [`the_chisel_carves_a_crease`] down with it.)
#[test]
fn the_knife_turns_with_the_stroke_only_when_it_rakes() {
    let size = 200u32;
    let curve = quarter_circle(100.0, 60.0, 55.0);

    let (raked, lock_on) = chisel_run(size, true, &curve);
    let (locked, lock_off) = chisel_run(size, false, &curve);

    assert!(
        lock_on.is_none(),
        "a RAKING knife locked an angle ({lock_on:?}). It has no business remembering one — every dab reads \
         its own heading, which is what 'in real time' means."
    );
    let lock = lock_off.expect(
        "the un-raked knife never locked an angle, so it has nothing to hold and every dab falls back to \
         its own heading — which IS raking, wearing the other checkbox",
    );
    // …and it locked onto the direction the stroke SET OFF in (+y at the top of this quarter-circle), not
    // some later one. A lock that drifts is not a lock.
    assert!(
        lock[1] > 0.8 && lock[0].abs() < 0.4,
        "the knife locked onto {lock:?}, which is not the direction the stroke entered at (≈ +y). It is \
         holding an angle from the middle of the gesture — the artist set off one way and the tool aimed \
         another."
    );

    let differing = raked
        .iter()
        .zip(&locked)
        .filter(|(a, b)| (*a - *b).abs() > 1e-3)
        .count();
    assert!(
        differing > 400,
        "the raked and the locked knife cut the SAME groove on a quarter-circle ({differing} texels apart). \
         One is supposed to follow the curve and the other to hold the angle it started at; over 90° of \
         sweep those cannot agree."
    );
}

/// **On a straight stroke, Rake changes nothing — to the byte.**
///
/// The presence sibling of the gate above, and it is the one that gives it meaning. "Follow the heading" and
/// "hold the first heading" are the *same instruction* when the heading never changes, so if these two runs
/// differed the difference above would be measuring the mechanism (a stale lock, a lost heading), not the
/// curve. It also pins the promise the artist actually cares about: unticking Rake must not change how a
/// straight cut looks.
///
/// **Mutation that must bleed:** give the un-raked knife any axis that is not the stroke's — e.g. the brush's
/// `dab_angle_deg`, which is what the first cut of this checkbox did, and which Enio corrected.
#[test]
fn on_a_straight_stroke_the_rake_makes_no_difference_at_all() {
    let size = 200u32;
    let straight = [[60.0, 100.5], [100.0, 100.5], [140.0, 100.5]];

    let (raked, _) = chisel_run(size, true, &straight);
    let (locked, lock) = chisel_run(size, false, &straight);

    assert!(
        lock.is_some(),
        "fixture: even here the un-raked knife must LOCK a heading — otherwise this passes by both runs \
         falling through to the same fallback, and proves nothing"
    );
    // Below the product's OWN "this height did not move" threshold — not byte-identity, and the difference
    // between those two claims is a measurement rather than a preference. The engine's heading is a
    // length-weighted EMA that re-normalises every step, so on a straight run its unit vector wobbles in the
    // last bits; the locked copy is one particular sample of it. Measured, that wobble reaches **4.8e-7
    // loads** — a thousandth of `RELIEF_EPS`, which is the number the deposit itself uses to decide a texel
    // is unchanged. Demanding `to_bits()` equality here would be demanding that an EMA be idempotent, and
    // the gate would be pinning the arithmetic of the smoother instead of the behaviour of the knife.
    let worst = raked
        .iter()
        .zip(&locked)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < super::super::impasto_settle::RELIEF_EPS,
        "Rake changed a STRAIGHT cut by {worst:e} loads (the product's own no-change threshold is {:e}). The \
         heading never turns here, so following it and holding its first value are the same instruction — a \
         difference this size means the un-raked knife is aiming at something other than the stroke.",
        super::super::impasto_settle::RELIEF_EPS
    );
    // …and the fixture cut something, or the equality above is the equality of two no-ops.
    let cut = raked.iter().filter(|v| **v != 0.0).count();
    assert!(cut > 200, "fixture: the chisel removed nothing to compare");
}

/// **Inflate arms its own defaults: falloff Sharper, shoulder 8 px** (Enio 2026-07-18).
///
/// The Blob offsets the surface by a ball whose radius rides the dab's weight, so the falloff IS the
/// dome's profile — a soft shoulder spreads the ball over the whole footprint and the result reads as a
/// swell instead of something inflated.
///
/// ⚠️ **`Falloff::Pow4`, whose UI name is "Sharper".** The enum also has a `Sharp`, which is a different
/// and gentler curve (`p²` against `p⁴`); the request came in the UI's vocabulary, and picking the variant
/// whose *identifier* matches the word would have armed the wrong one. This gate names both so the next
/// reader does not have to re-derive which is which.
///
/// **Mutations that must bleed:** drop the `arm_inflate_defaults()` call from `set_sculpt_mode`; arm
/// `Falloff::Sharp` instead of `Pow4`; leave `smooth_norm` at `0.0`.
#[test]
fn inflate_arms_sharper_and_an_eight_pixel_shoulder() {
    use ph2d_painter_brush::Falloff;
    // A plain tool, NOT `sculpt_canvas`: that fixture pins `Falloff::Constant` for its own reasons, and a
    // fixture that overrides the very field under test cannot see the field being set.
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 120 * 120 * 4], 120, 120);
    arm_sculpt(&mut t, SMOOTH, 0.5, 1.0);
    assert_eq!(
        t.paint.brush.falloff,
        Falloff::Smooth,
        "fixture: the sculpt brush is meant to start at the factory Smooth — without that the swap below \
         has nothing to swap and this passes on a tool that never arms anything"
    );
    t.set_sculpt_mode(INFLATE);
    assert_eq!(
        t.paint.brush.falloff,
        Falloff::Pow4,
        "picking Inflate did not arm the Sharper falloff. `Pow4` IS \"Sharper\" in the UI; `Falloff::Sharp` \
         is a different curve and arming it would be the wrong default wearing the right word."
    );
    assert_eq!(
        t.sculpt_smooth_px(),
        8,
        "Inflate's shoulder is meant to default to 8 px, not {} — the ball's hard edge is the thing it \
         softens, and 0 ships the edge Enio rejected",
        t.sculpt_smooth_px()
    );

    // …and a DELIBERATE choice is never overridden — the law `toggle_brush_impasto` already obeys.
    let mut t2 = PainterTool::default();
    t2.set_source(vec![255u8; 120 * 120 * 4], 120, 120);
    arm_sculpt(&mut t2, SMOOTH, 0.5, 1.0);
    // The artist's pick goes through the REAL setter — it is the setter that records the falloff as
    // deliberately chosen (`falloff_armed = false`), which is what the arming law honours. A direct
    // field write models nothing the product can do.
    t2.set_brush_falloff(Falloff::Sphere.to_u8());
    t2.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = t2.paint.brush;
    t2.set_sculpt_mode(INFLATE);
    assert_eq!(
        t2.paint.brush.falloff,
        Falloff::Sphere,
        "arming a default overrode a falloff the artist had chosen. It arms a default; it does not \
         enforce a policy."
    );
}

/// **Every impasto tool is born with the falloff its verb wants** (Enio 2026-07-20). The table and
/// the measurements behind it live on `arm_tool_falloff_defaults`; this gate walks the VERBS through
/// the real seam (`set_sculpt_mode`) and the KNIFE through the real mode switch, asserting each arm —
/// and that an armed default is re-armable (Flatten's Sphere gives way to Inflate's Sharper) while an
/// artist's pick still survives every switch (the sibling assertion above).
#[test]
fn every_impasto_tool_is_born_with_its_own_falloff() {
    use ph2d_painter_brush::Falloff;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 120 * 120 * 4], 120, 120);
    arm_sculpt(&mut t, SMOOTH, 0.5, 1.0);
    assert_eq!(
        t.paint.brush.falloff,
        Falloff::Smooth,
        "Smooth verb keeps the soft profile"
    );
    for (verb, want) in [
        (FLATTEN, Falloff::Smoother),
        (SCRAPE, Falloff::Smoother),
        (FILL, Falloff::Smoother),
        (CHISEL, Falloff::Smoother),
        (LAYER, Falloff::Smoother),
        (INFLATE, Falloff::Pow4),
        // …and back down: an ARMED default is re-armable, verb after verb.
        (SHARPEN, Falloff::Smooth),
        (FLATTEN, Falloff::Smoother),
    ] {
        t.set_sculpt_mode(verb);
        assert_eq!(
            t.paint.brush.falloff, want,
            "verb {verb}: the arm did not install its falloff (an armed default must give way to \
             the next verb's default)"
        );
    }
    // The Knife arms Smoother on the mode switch — the smear handoff's "agulha do pincel padrão",
    // closed as a default (trail 6 px vs the old Smooth's 4-px needle; Sphere's 12 was traded away
    // for the soft rim — Enio 2026-07-20, "Knife Smoother").
    let mut k = PainterTool::default();
    k.set_source(vec![255u8; 120 * 120 * 4], 120, 120);
    k.set_paint_tool_mode("knife");
    assert_eq!(
        k.paint.brush.falloff,
        Falloff::Smoother,
        "entering the Knife did not arm Smoother — the needle-trail default the smear handoff left open"
    );
}

/// The direction of the groove AT a point, read off the RENDERED relief.
///
/// A chisel cuts a V whose crease runs along the stroke, so from inside the trench the relief varies least
/// **along** the groove and most **across** it. Sweeping a probe line through the point and taking the
/// direction of least variation therefore recovers the groove's axis from the pixels themselves — no
/// knowledge of `Dab::dir`, of `chisel_dir`, or of any intermediate the tool computed.
///
/// That independence is the whole point: the gate above measures that two grooves DIFFER, which a knife
/// rotated to any wrong angle also satisfies. This measures the thing the artist actually sees.
fn groove_angle_deg(h: &[f32], size: u32, cx: usize, cy: usize, arm: usize) -> f32 {
    let at = |x: f32, y: f32| -> f32 {
        let (xi, yi) = (x.round() as i64, y.round() as i64);
        if xi < 0 || yi < 0 || xi >= i64::from(size) || yi >= i64::from(size) {
            return 0.0;
        }
        h[(yi as usize) * (size as usize) + xi as usize]
    };
    let (mut best, mut best_var) = (0.0f32, f32::MAX);
    // 1° steps over a half-turn: a groove has no head or tail, so θ and θ+180° are the same axis.
    for step in 0..180 {
        let th = (step as f32).to_radians();
        let (dx, dy) = (th.cos(), th.sin());
        let (mut sum, mut sum2, mut n) = (0.0f32, 0.0f32, 0.0f32);
        for k in -(arm as i32)..=(arm as i32) {
            let v = at(cx as f32 + dx * k as f32, cy as f32 + dy * k as f32);
            sum += v;
            sum2 += v * v;
            n += 1.0;
        }
        let var = sum2 / n - (sum / n) * (sum / n);
        if var < best_var {
            best_var = var;
            best = step as f32;
        }
    }
    best
}

/// **The groove ends up PARALLEL to the stroke — measured on the pixels, not on the tool's intermediates.**
///
/// This is the gate that was missing while the defect lived, and the reason it lived: its neighbour
/// `the_knife_turns_with_the_stroke_only_when_it_rakes` asserts only that the raked and locked grooves
/// **differ**, which is equally true of a knife rotated to the wrong angle. Enio reported exactly that
/// (*"Rake não consegue rotacionar o brush para que a vala seja sempre paralela à direção do traço"*) with
/// the whole Chisel suite green: the axis was the SMOOTHED tangent (`Dab::dir`, which the Rake texture
/// rides and wants), and a smoothed tangent trails — measured up to **52° behind at a 60-px brush**,
/// because the smoothing window scales with the radius.
///
/// So this reads the groove's direction out of the RENDERED height and compares it to the path's own
/// tangent. It knows nothing about `chisel_dir`, and that is what makes it an oracle rather than a mirror.
///
/// **Mutation that must bleed:** take the axis from `d.dir` again in `stamp_dabs_sculpt` (i.e. undo
/// `path_axis`) — the lag returns and the assert names the angle.
#[test]
fn the_raked_groove_runs_parallel_to_the_stroke() {
    let size = 200u32;
    let curve = quarter_circle(100.0, 60.0, 55.0);
    let (raked, _) = chisel_run(size, true, &curve);

    // Sample WELL INSIDE the run: the first dabs are the heading warm-up and the last is the pen-up.
    let mut worst = 0.0f32;
    for k in [8usize, 12, 16] {
        let p = curve[k];
        let (prev, next) = (curve[k - 1], curve[k + 1]);
        let want = (next[1] - prev[1]).atan2(next[0] - prev[0]).to_degrees();
        let got = groove_angle_deg(&raked, size, p[0] as usize, p[1] as usize, 7);
        // Both are AXES, not directions: fold the difference into [0, 90].
        let mut d = (want - got).rem_euclid(180.0);
        if d > 90.0 {
            d = 180.0 - d;
        }
        assert!(
            d < 12.0,
            "at station {k} the stroke runs at {want:.1}° and the groove at {got:.1}° — {d:.1}° apart. \
             The V is folded about the stroke's axis, so a groove that is not parallel to the path is a \
             knife pointing somewhere the artist never pointed it. (`Dab::dir` is the SMOOTHED tangent \
             and trails by roughly the brush radius; the axis has to come from the dab CENTRES.)"
        );
        worst = worst.max(d);
    }
    assert!(
        worst > 0.0,
        "fixture: every station measured exactly 0° off, which a groove read from real pixels does not \
         do — the probe is reading a constant, not the relief"
    );
}

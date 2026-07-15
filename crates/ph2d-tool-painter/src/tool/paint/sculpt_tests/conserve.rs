//! Gates for **W5 — Conserve, the bow wave** (plan §6): where the scraped paint goes.
//!
//! Blender deletes the volume its Scrape removes. Paint does not do that — so with the flag armed, the
//! down-only verbs (Scrape / Chisel) bank what they take onto the spatula's rim, through the SAME
//! conservative engine the deposit's Push uses (`bank_dab_push`). The bite is counted incrementally
//! inside the plane walk (`PlaneBite`), so summed over the stroke it telescopes to exactly what the
//! final render removes.
//!
//! The flag is **opt-in** (default off): the pile's mechanics are gated here; its DRAWING is judged by
//! the smoke, which is where the Push — the same problem from the deposit's side — earned its scars.

use super::*;
use std::sync::Arc;

const SCRAPE: u8 = 3;

/// A live Scrape gesture (session still open — the committed session takes its `amount` to the grave):
/// Down + Moves across the ridge, NO Up. The caller ends it when the gate is done measuring.
fn scrape_live(t: &mut PainterTool, conserve: bool) {
    arm_sculpt(t, SCRAPE, 0.5, 1.0);
    if conserve {
        t.toggle_sculpt_conserve();
        assert!(t.paint.sculpt.conserve, "fixture: the flag did not arm");
    }
    let mut b = t.paint.brush;
    b.radius_px = 14.0;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;
    t.on_canvas_pointer(cp([80.0, 100.0], PointerPhase::Down));
    for i in 1..=5 {
        t.on_canvas_pointer(cp([80.0 + (i as f32) * 8.0, 100.0], PointerPhase::Move));
    }
}

/// Split a height diff into (removed, banked) volume — the two sides of the conservation ledger.
fn ledger(before: &[f32], after: &[f32]) -> (f64, f64) {
    let (mut removed, mut banked) = (0.0f64, 0.0f64);
    for (b, a) in before.iter().zip(after) {
        let d = f64::from(a - b);
        if d < 0.0 {
            removed -= d;
        } else {
            banked += d;
        }
    }
    (removed, banked)
}

/// **What the Scrape takes, the rim receives — the ledger balances.**
///
/// The conservation law, measured where the artist would ask for it: after a conserving Scrape, the
/// volume that left the channel equals the volume standing in the ridge, to well under a percent (the
/// bite telescopes the render's own expression, so the residue is float bookkeeping, not physics).
/// The presence sibling inside: paint genuinely LEFT (the fixture scraped something) and the ridge
/// stands on texels the brush never touched (it is a bow wave, not a smear inside the cut).
///
/// **Mutation that must bleed:** drop the `+ bank_v` term in `render_sculpt` — the removal stays, the
/// ridge never lands, and the ledger loses its whole credit side (this is exactly Blender's deletion,
/// which is the behaviour the flag exists to replace).
#[test]
fn what_the_scrape_takes_the_rim_receives() {
    let size = 200u32;
    let (mut t, layer, before) = sculpt_canvas(size);

    scrape_live(&mut t, true);
    let touched: Vec<f32> = (*t.paint.sculpt.amount).clone();
    let after = heights_of(&t, layer);
    t.on_canvas_pointer(cp([120.0, 100.0], PointerPhase::Up));

    let (removed, banked) = ledger(&before, &after);
    assert!(
        removed > 20.0,
        "fixture: the Scrape removed only {removed:.1} loads·px² — nothing to conserve, the balance \
         below is vacuous"
    );
    let net = banked - removed;
    assert!(
        net.abs() < removed * 0.01,
        "the ledger does not balance: removed {removed:.1} loads·px², banked {banked:.1} (net \
         {net:+.2}). Conserve's one promise is that paint is never destroyed — the bite counted by the \
         plane walk must land, whole, on the rim."
    );
    // The ridge is a BOW WAVE: it stands beyond the brush's own footprint.
    let rim_rise = (0..before.len())
        .filter(|i| touched[*i] <= 0.0)
        .map(|i| after[i] - before[i])
        .fold(0.0f32, f32::max);
    assert!(
        rim_rise > 0.05,
        "no ridge stands outside the touched footprint (max rim rise {rim_rise:.3} loads) — the bank \
         landed inside the channel, where the next dab scrapes it off again"
    );
}

/// **Off is OFF: the flag unarmed is today's Scrape, and the rim stays untouched to the bit.**
///
/// The escape half. Blender-style non-conservation is the shipped default, and arming W5 must not have
/// changed it: with the flag off the net volume is strictly negative (the paint really is deleted) and
/// not one texel beyond the footprint moves.
///
/// **Mutation that must bleed:** make `SculptState::conserve_active` return `true` — the one door both
/// the accumulation and the render ask (it is ONE door precisely so one mutation convicts both sides);
/// the rim rises with the flag off.
#[test]
fn conserve_off_is_todays_scrape_to_the_bit() {
    let size = 200u32;
    let (mut t, layer, before) = sculpt_canvas(size);

    scrape_live(&mut t, false);
    let touched: Vec<f32> = (*t.paint.sculpt.amount).clone();
    let after = heights_of(&t, layer);
    t.on_canvas_pointer(cp([120.0, 100.0], PointerPhase::Up));

    let (removed, banked) = ledger(&before, &after);
    assert!(removed > 20.0, "fixture: nothing scraped — vacuous");
    assert!(
        banked < removed * 0.01,
        "the flag is OFF and {banked:.1} loads·px² still piled up somewhere — off must mean off"
    );
    let mut rim_moved = 0usize;
    for i in 0..before.len() {
        if touched[i] <= 0.0 && after[i].to_bits() != before[i].to_bits() {
            rim_moved += 1;
        }
    }
    assert_eq!(
        rim_moved, 0,
        "{rim_moved} texels beyond the footprint moved with Conserve OFF — the bank is leaking past \
         its flag"
    );
}

/// **The bow wave obeys the re-stamp law — the pile is a function of the SHAPE, not of the drag.**
///
/// `bank` is a fact about the dab list, exactly like `plane_sum`: the shape editors rebuild that list on
/// every pointer move, so the reset that precedes each re-stamp must take the pile with it — or every
/// frame of LOOKING at an open shape would raise the ridge a little more (the §4 melt, inverted). Same
/// harness as the §4 gate: the same Line, reached straight and reached through a detour (each drag frame
/// a full re-stamp), must leave the same relief to the bit — ridge included.
///
/// **Mutation that must bleed:** drop the bank-zeroing in `restamp_reset_sculpt` — every re-stamp of the
/// detour run piles the same removal again, and the two runs diverge by a ridge's worth.
#[test]
fn the_bow_wave_obeys_the_restamp_law() {
    let size = 200u32;
    let a = [60.0, 100.0];
    let b = [140.0, 100.0];
    let c = [90.0, 60.0];
    let d = [170.0, 150.0];

    let line_scrape = |t: &mut PainterTool| {
        arm_sculpt(t, SCRAPE, 0.5, 1.0);
        t.toggle_sculpt_conserve();
        assert!(t.paint.sculpt.conserve, "fixture: the flag did not arm");
        let mut br = t.paint.brush;
        br.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
        t.paint.brush = br;
        t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = br;
    };
    let place = |t: &mut PainterTool, p: [f32; 2]| {
        t.on_canvas_pointer(cp(p, PointerPhase::Down));
        t.on_canvas_pointer(cp(p, PointerPhase::Up));
    };

    // Run 1 — the line A→B, drawn straight.
    let (mut t1, layer1, before) = sculpt_canvas(size);
    line_scrape(&mut t1);
    place(&mut t1, a);
    place(&mut t1, b);
    let straight = heights_of(&t1, layer1);
    let (removed, banked) = ledger(&before, &straight);
    assert!(
        removed > 5.0 && banked > removed * 0.5,
        "fixture: the Line laid no conserving scrape (removed {removed:.1}, banked {banked:.1}) — the \
         comparison below would pass on any engine"
    );

    // Run 2 — the SAME line, endpoint dragged there through a detour: every drag frame re-stamps the
    // whole shape (the detour never returns through the grab's own slop — see the §4 gate's note).
    let (mut t2, layer2, _) = sculpt_canvas(size);
    line_scrape(&mut t2);
    place(&mut t2, a);
    place(&mut t2, c);
    t2.on_canvas_pointer(cp(c, PointerPhase::Down));
    t2.on_canvas_pointer(cp(d, PointerPhase::Move));
    t2.on_canvas_pointer(cp(b, PointerPhase::Move));
    t2.on_canvas_pointer(cp(b, PointerPhase::Up));
    let dragged = heights_of(&t2, layer2);

    let differing = straight
        .iter()
        .zip(&dragged)
        .filter(|(x, y)| x.to_bits() != y.to_bits())
        .count();
    assert_eq!(
        differing, 0,
        "the same conserving Line, reached two different ways, left two different reliefs ({differing} \
         texels). The bow wave is carrying the HISTORY of the drag: each re-stamp piled the same removal \
         again, and an open shape grows a ridge just from being edited."
    );
}

/// **The undo snapshot carries the bow wave — restore, re-render, and the ridge is still there.**
///
/// `plane_sum`'s exact situation (§10.4): the bank is derived from the dab list, which the restore
/// cannot rebuild. A snapshot that dropped it would un-pile the ridge on the first undo-then-re-render
/// while the channel it balances stays cut — conservation broken by the undo button.
///
/// **Mutation that must bleed:** hand back an empty `bank` in `sculpt_for_snapshot` (or drop it in
/// `restore_sculpt`) — the re-render after the restore loses the ridge and the fields diverge.
#[test]
fn the_undo_snapshot_carries_the_bow_wave() {
    let size = 200u32;
    let (mut t, layer, _) = sculpt_canvas(size);
    scrape_live(&mut t, true);
    let h1 = heights_of(&t, layer);
    let snap = t.sculpt_for_snapshot();
    assert!(
        snap.bank.iter().any(|v| *v != 0.0),
        "fixture: the snapshot's bank is empty — the restore below proves nothing"
    );

    // A second batch moves things on…
    for i in 1..=4 {
        t.on_canvas_pointer(cp([120.0 + (i as f32) * 6.0, 100.0], PointerPhase::Move));
    }
    assert_ne!(
        h1,
        heights_of(&t, layer),
        "fixture: the second batch did nothing"
    );

    // …and the undo path puts the world AND the session back, then the standing gesture re-renders.
    if let Some(entry) = t.heights.get_mut(&layer) {
        *Arc::make_mut(entry) = h1.clone();
    }
    t.restore_sculpt(snap);
    t.refresh_live_sculpt();
    let after = heights_of(&t, layer);
    t.on_canvas_pointer(cp([150.0, 100.0], PointerPhase::Up));

    let diff = h1
        .iter()
        .zip(&after)
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    assert_eq!(
        diff, 0,
        "restore + re-render diverged from the snapshotted state at {diff} texels — the session \
         snapshot is not carrying the bow wave (or not putting it back), so undo un-piles the ridge \
         while the channel stays cut"
    );
}

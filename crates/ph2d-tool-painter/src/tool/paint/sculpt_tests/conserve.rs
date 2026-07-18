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

// ── Flatten and Fill: the same law, the other sign ──────────────────────────────────────────────────

/// A layer carrying one REAL deposited patch — paint with EDGES, and bare canvas all around it.
///
/// The synthetic `sculpt_canvas` cannot serve here: it sets coverage to 255 over the whole plane, so
/// "off the paint" does not exist in it and a moat digging into nothing would look identical to one
/// digging into paint. A fixture that cannot contain the phenomenon cannot gate it.
fn deposited_patch(size: u32) -> (PainterTool, crate::tool::RtLayerId, Vec<f32>, Vec<u8>) {
    use ph2d_painter_brush::{BrushSpec, Falloff};
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 18.0,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        impasto: true,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
    let layer = t.layers.active().expect("a layer");
    t.on_canvas_pointer(cp([80.0, 100.0], PointerPhase::Down));
    for i in 1..=5 {
        t.on_canvas_pointer(cp([80.0 + (i as f32) * 8.0, 100.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([120.0, 100.0], PointerPhase::Up));
    let relief = heights_of(&t, layer);
    let cover: Vec<u8> = t
        .covers
        .get(&layer)
        .map(|c| (**c).clone())
        .unwrap_or_default();
    assert!(
        cover.iter().any(|c| *c > 0) && cover.contains(&0),
        "fixture: the patch must have BOTH paint and bare canvas, or the moat gate proves nothing"
    );
    (t, layer, relief, cover)
}

/// Run one live plane-verb gesture over the patch at `offset`, Conserve optionally armed.
fn plane_live(t: &mut PainterTool, mode: u8, offset: f32, conserve: bool) {
    arm_sculpt(t, mode, 0.5, 1.0);
    t.set_sculpt_offset(offset);
    if conserve {
        t.toggle_sculpt_conserve();
        assert!(
            t.paint.sculpt.conserve_active(),
            "fixture: the flag did not arm for this verb"
        );
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

/// **What the Fill adds, the moat supplies — the ledger balances the OTHER way.**
///
/// The mirror of `what_the_scrape_takes_the_rim_receives`, and deliberately the same law rather than a
/// second one: Conserve means the stroke's net volume change is zero and the rim settles the difference
/// with the ledger's SIGN. Scrape's ledger is always negative, which is why the first cut could hard-code
/// a ridge; Fill's is always positive, so the rim owes paint instead of receiving it.
///
/// **Mutations that must bleed** (both checked): make `PlaneBite` measure `Travel::Down` for every verb
/// (Fill's travel is upward, so the bite reads zero and no moat forms); or restore the caller's
/// `displaced > 0.0` guard, which drops every adder's moat on the floor while the checkbox says otherwise.
#[test]
fn what_the_fill_adds_the_moat_supplies() {
    let (mut t, layer, before, _) = deposited_patch(200);
    plane_live(&mut t, 4, 0.5, true); // Fill, offset centred
    let after = heights_of(&t, layer);
    let (removed, banked) = ledger(&before, &after);
    assert!(
        banked > 5.0,
        "fixture: the Fill raised only {banked:.1} loads·px² — nothing happened, so a balanced ledger \
         below would be two zeroes agreeing"
    );
    assert!(
        removed > 5.0,
        "the Fill added {banked:.1} loads·px² and the moat gave up {removed:.1}. With Conserve armed the \
         paint has to come from somewhere: a rim that never sinks is matter created out of nothing, which \
         is the very thing this flag exists to refuse."
    );
    let net = (banked - removed).abs();
    assert!(
        net / banked.max(removed) < 0.02,
        "the ledger does not balance: added {banked:.1} loads·px², withdrawn {removed:.1} (net \
         {:+.1}). Conserve promises the stroke's NET volume change is zero.",
        banked - removed
    );
}

/// **A moat may only take paint that is actually standing.**
///
/// The gate this whole wave turned on, and it was born from a measurement rather than a worry.
///
/// The rim deliberately prefers texels the stroke has not worked over (`1 − paint`), so a ridge is never
/// banked into its own channel. Reverse the sign and that same preference steers the moat straight OFF the
/// paint — and `h` going negative under zero coverage renders **nothing**, because the light weights by
/// coverage. Measured on this fixture: **83% of the withdrawal landed on bare canvas while the ledger read
/// ±0.0**. A balanced ledger over a visible lie is worse than an unbalanced one, because nothing complains.
///
/// The cure is that a withdrawal is weighted by the material standing there — `height × coverage`, the
/// quantity the light integrates ([`ph2d_painter_brush::height_push::Supply`]). Relief alone is not enough
/// and was measured too: the deposit feathers height into a halo wider than the paint, and weighting by it
/// still put **56%** into the invisible.
///
/// **Mutation that must bleed:** pass `None` for the supply at the sculpt's `bank_dab_push` call, or drop
/// the `cover` factor from `Supply::at`. Both put the moat back in the void (56% and 83%), and the ledger
/// stays green through either — which is exactly why this is measured HERE and not there.
#[test]
fn a_moat_takes_only_paint_that_is_standing() {
    for (label, mode, offset) in [("FILL", 4u8, 0.5f32), ("FLATTEN+CLAY", 2, 0.75)] {
        let (mut t, layer, before, cover) = deposited_patch(200);
        plane_live(&mut t, mode, offset, true);
        let after = heights_of(&t, layer);
        let (mut on_paint, mut on_void) = (0.0f64, 0.0f64);
        for i in 0..after.len() {
            let d = f64::from(after[i] - before[i]);
            if d >= 0.0 {
                continue;
            }
            if cover.get(i).copied().unwrap_or(0) > 0 {
                on_paint -= d;
            } else {
                on_void -= d;
            }
        }
        assert!(
            on_paint > 5.0,
            "fixture ({label}): the moat withdrew only {on_paint:.1} loads·px² from the paint — with \
             nothing withdrawn anywhere, 'none of it went into the void' is vacuously true"
        );
        assert_eq!(
            on_void.round(),
            0.0,
            "{label}: the moat took {on_void:.1} loads·px² out of BARE CANVAS ({:.0}% of the \
             withdrawal). There is no paint there to drag into the hollow, and relief under zero \
             coverage renders nothing — so that volume balances the ledger while the artist watches \
             paint appear from nowhere.",
            100.0 * on_void / (on_paint + on_void)
        );
    }
}

/// **On Flatten the Offset is the volume knob, and Conserve is its counterweight.**
///
/// Measured before this wave was built, and it changed what the flag MEANS on this verb: at the centre of
/// its track Flatten is **already** conservative to +1.2%, and not by luck — the least-squares plane passes
/// through the weighted centroid, so `Σ w·(plane − h) = 0` by construction and what comes off the peaks is
/// already in the valleys. Off centre the Offset creates or destroys about **twelve times** the entire
/// redistribution.
///
/// So this gate asserts the thing that is actually true and actually useful: the flag is live exactly where
/// the verb stops being neutral. A gate written on the centred case would pass on an implementation that
/// did nothing at all.
///
/// **Mutation that must bleed:** drop `Flatten` from `SculptMode::conserves()` — the armed run stops
/// cancelling and lands on the unarmed number.
#[test]
fn on_flatten_the_offset_is_the_volume_knob_and_conserve_is_its_counterweight() {
    let net_of = |conserve: bool| {
        let (mut t, layer, before, _) = deposited_patch(200);
        plane_live(&mut t, 2, 0.75, conserve); // Flatten, plane lifted — this is Clay
        let after = heights_of(&t, layer);
        let (removed, banked) = ledger(&before, &after);
        (banked - removed, banked.max(removed))
    };
    let (loose, loose_scale) = net_of(false);
    let (held, held_scale) = net_of(true);
    assert!(
        loose / loose_scale > 0.5,
        "fixture: a lifted Flatten added only {loose:.1} loads·px² net — the Offset is supposed to be \
         the volume knob here, and if it is not, the armed run below has nothing to cancel"
    );
    assert!(
        held.abs() / held_scale < 0.02,
        "Conserve did not counterweight the Offset: the armed stroke still moved {held:+.1} \
         loads·px² net (the unarmed one moves {loose:+.1}). Clay is Flatten with the plane lifted, and \
         with this flag armed the lift has to come from the surrounding paint rather than from nowhere."
    );
}

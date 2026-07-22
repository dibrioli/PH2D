//! Doc 23 gates — the pigment answers the tools. The reported dysfunction
//! ("paint with pure paint, pass the Wet brush over it — nothing happens")
//! is the first gate, driven through the PRODUCT path (`e.tool` + the same
//! scripted-stroke driver the fingerprint session uses). The sisters pin
//! the off-switches: `wetLift = 0` is the old model (fingerprint.rs) and
//! `staining = 1` pins dried paint down across every active lift.

mod util;

use ph2d_wet_paint::painter::{Engine, Tool};
use ph2d_wet_paint::tuning::Knob;
use util::drive_stroke;

const W: usize = 160;
const H: usize = 120;

/// Paint a PURE stroke (water slider 0 — the reported scenario) along
/// y = 60, then fast-dry: everything the stroke left is SETTLED.
fn dried_stroke_engine() -> Engine {
    let mut e = Engine::new(W, H);
    e.color = [190.0, 40.0, 30.0];
    e.sliders.water = 0.0;
    drive_stroke(&mut e, 30.0, 60.0, 130.0, 60.0, 4.0, 10);
    e.fast_dry_now();
    e.sliders.water = 1.0; // the WET brush carries its own water again
    e
}

fn sums(e: &Engine) -> (f64, f64) {
    let g = e.active_grid();
    let (mut susp, mut sett) = (0.0f64, 0.0f64);
    for v in &g.susp {
        susp += *v as f64;
    }
    for v in &g.sett {
        sett += *v as f64;
    }
    (susp, sett)
}

/// TOTAL pigment center of mass (suspended + settled): the appearance
/// oracle — blown paint that re-settles keeps the displacement.
fn pigment_com_x(e: &Engine) -> f64 {
    let g = e.active_grid();
    let (mut m, mut mx) = (0.0f64, 0.0f64);
    for y in 0..H {
        for x in 0..W {
            let v = (g.susp[x + y * g.s] + g.sett[x + y * g.s]) as f64;
            m += v;
            mx += v * x as f64;
        }
    }
    mx / m.max(1e-9)
}

fn sett_bits(e: &Engine) -> Vec<u32> {
    e.active_grid().sett.iter().map(|v| v.to_bits()).collect()
}

/// The reported case, literally: pure dried paint answers the Wet brush —
/// the settled layer dissolves back into suspension (P1), and the freshly
/// suspended pigment ANSWERS a Blow (it re-entered the flow). Mutation
/// that bleeds: dropping the active-lift call in `apply_wet`.
#[test]
fn wet_over_dry_pure_paint_reactivates_it() {
    let mut e = dried_stroke_engine();
    let (susp0, sett0) = sums(&e);
    assert!(susp0 < 1.0, "precondition: fully dried (susp {susp0})");
    assert!(sett0 > 1000.0, "precondition: paint on the sheet ({sett0})");

    // Measured WHILE WET (sim_after 0): letting the sheet dry untouched
    // re-settles the lift — which is honest watercolor — so the gate reads
    // the state the artist acts on, not the one after walking away.
    e.tool = Tool::Wet;
    for _ in 0..2 {
        drive_stroke(&mut e, 30.0, 60.0, 130.0, 60.0, 4.0, 0);
    }
    let (susp1, sett1) = sums(&e);
    assert!(
        sett1 < 0.75 * sett0,
        "two Wet passes must dissolve dried paint: sett {sett0} -> {sett1}"
    );
    assert!(
        susp1 > 0.9 * (sett0 - sett1),
        "the dissolved mass must be IN SUSPENSION: susp {susp1}"
    );

    // The re-suspended pigment answers a Blow: the TOTAL-pigment center
    // of mass shifts — and STAYS shifted after the film dies and the flow
    // re-settles. Measured: 0.034 px against a no-blow control drift of
    // 0.0002 px (the drip-fed film is thin, so the nudge is honest, not
    // dramatic); threshold 0.01 = 3.4x moat over the effect, 50x over the
    // drift.
    let com_before = pigment_com_x(&e);
    e.set_knob(Knob::Blow, 1.0);
    e.tool = Tool::Blow;
    for _ in 0..5 {
        drive_stroke(&mut e, 40.0, 60.0, 120.0, 60.0, 6.0, 10);
    }
    let moved = (pigment_com_x(&e) - com_before).abs();
    assert!(
        moved > 0.01,
        "blowing re-wetted paint must move it (com moved {moved:.4} px)"
    );
}

/// `staining = 1` pins dried paint down: the same two Wet passes leave the
/// settled plane BIT-identical (the multiplier is exactly 0, active and
/// passive). Mutation that bleeds: `active_lift_gain` ignoring staining.
#[test]
fn staining_one_pins_the_paint_down() {
    let mut e = dried_stroke_engine();
    e.set_knob(Knob::ExtStaining, 1.0);
    let before = sett_bits(&e);
    e.tool = Tool::Wet;
    for _ in 0..2 {
        drive_stroke(&mut e, 30.0, 60.0, 130.0, 60.0, 4.0, 5);
    }
    assert_eq!(
        before,
        sett_bits(&e),
        "staining=1 must leave the settled plane untouched by Wet passes"
    );
}

/// P2: the Smear drags DRY paint — settled pigment crosses the original
/// footprint edge carrying its color — while `staining = 1` restores the
/// suspended-only smear to the byte. Mutation that bleeds: dropping the
/// settled block from `apply_smear`.
#[test]
fn smear_drags_dry_paint_when_not_staining() {
    // A SHORT dried stroke, so clean canvas remains to the right; the
    // stroke's true reach (release tail included) is MEASURED, not guessed.
    let dried_short = || {
        let mut e = Engine::new(W, H);
        e.color = [190.0, 40.0, 30.0];
        e.sliders.water = 0.0;
        drive_stroke(&mut e, 30.0, 60.0, 70.0, 60.0, 4.0, 10);
        e.fast_dry_now();
        e
    };
    let edge_of = |e: &Engine| {
        let g = e.active_grid();
        (0..W)
            .rev()
            .find(|x| (50..70).any(|y| g.sett[x + y * g.s] > 0.0))
            .expect("paint on the sheet")
    };
    let band = |e: &Engine, x0: usize| {
        let g = e.active_grid();
        let mut s = 0.0f64;
        for y in 50..70 {
            for x in x0..(x0 + 14).min(W - 2) {
                s += g.sett[x + y * g.s] as f64;
            }
        }
        s
    };

    // staining 0: dried paint smears outward like any raster smudge.
    let mut e = dried_short();
    e.set_knob(Knob::ExtStaining, 0.0);
    let edge = edge_of(&e);
    assert!(edge + 20 < W - 2, "fixture: no clean canvas left ({edge})");
    let probe = edge + 4;
    assert!(
        band(&e, probe) < 1.0,
        "precondition: clean band past the edge"
    );
    // Rubbing spreads by REPETITION (the front advances a few px per
    // pass, exponentially tapered — that is what a smudge is); five passes
    // measured 23.1 into the band, threshold 10 = 2.3x moat.
    e.tool = Tool::Smear;
    for _ in 0..5 {
        drive_stroke(
            &mut e,
            edge as f64 - 30.0,
            60.0,
            (probe + 10) as f64,
            60.0,
            4.0,
            0,
        );
    }
    let dragged = band(&e, probe);
    assert!(
        dragged > 10.0,
        "a smear must drag dried paint past the footprint edge ({dragged})"
    );

    // staining 1: the settled layer is pinned — the whole settled plane is
    // bit-identical after the same smear (susp is empty, film is zero, so
    // the legacy smear body has nothing to move either).
    let mut e = dried_short();
    e.set_knob(Knob::ExtStaining, 1.0);
    let edge = edge_of(&e);
    let before = sett_bits(&e);
    e.tool = Tool::Smear;
    drive_stroke(
        &mut e,
        edge as f64 - 30.0,
        60.0,
        (edge + 14) as f64,
        60.0,
        4.0,
        0,
    );
    assert_eq!(
        before,
        sett_bits(&e),
        "staining=1 must restore the suspended-only smear"
    );
}

/// P3: a WET blend re-suspends what a dry blend only re-mixes in place —
/// water + Blend puts pigment back into suspension; the same blend on the
/// dry sheet leaves suspension empty (the pre-doc-23 behavior). Mutation
/// that bleeds: dropping the lift block from `transfer_blend`.
#[test]
fn a_wet_blend_resuspends_what_a_dry_blend_only_remixes() {
    // Dry blend: no water anywhere — settled paint re-mixes but nothing
    // returns to suspension.
    let mut e = dried_stroke_engine();
    e.tool = Tool::Blend;
    drive_stroke(&mut e, 40.0, 60.0, 120.0, 60.0, 4.0, 0);
    let (susp_dry, _) = sums(&e);
    assert!(
        susp_dry < 1.0,
        "a dry blend must not create suspension ({susp_dry})"
    );

    // Wet the area first: the same blend now lifts through the door.
    let mut e = dried_stroke_engine();
    e.tool = Tool::Wet;
    drive_stroke(&mut e, 40.0, 60.0, 120.0, 60.0, 4.0, 0);
    let (susp_after_wet, sett_after_wet) = sums(&e);
    e.tool = Tool::Blend;
    drive_stroke(&mut e, 40.0, 60.0, 120.0, 60.0, 4.0, 0);
    let (susp_wet, sett_wet) = sums(&e);
    assert!(
        sett_wet < sett_after_wet - 1000.0,
        "a wet blend must LIFT settled pigment ({sett_after_wet} -> {sett_wet})"
    );
    assert!(
        susp_wet > susp_after_wet * 1.05,
        "the lifted mass must be IN SUSPENSION \
         ({susp_after_wet} -> {susp_wet})"
    );
}

/// P4: dried staining paint resists the eraser — the same erase stroke
/// (sim paused, zero drying passes: the erase is the ONLY writer) leaves
/// more settled pigment at staining 1 than at staining 0. Mutation that
/// bleeds: erasing `sett` with the un-resisted `k`.
#[test]
fn dried_staining_paint_resists_the_eraser() {
    let run = |staining: f64| -> f64 {
        let mut e = dried_stroke_engine();
        e.set_knob(Knob::ExtStaining, staining);
        e.sliders.erase = 1.0;
        e.tool = Tool::Erase;
        for _ in 0..3 {
            drive_stroke(&mut e, 30.0, 60.0, 130.0, 60.0, 4.0, 0);
        }
        sums(&e).1
    };
    let loose = run(0.0);
    let pinned = run(1.0);
    assert!(
        pinned > loose * 1.15,
        "staining paint must survive the eraser better: {loose} vs {pinned}"
    );
}

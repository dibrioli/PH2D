//! Doc 21 Stage 2+ gates — the commit-deposit door. Every gate names the
//! mutation that bleeds it; the fixtures speak POINTERS (the product's own
//! doors), never synthetic internals, except where the oracle needs the grid.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_painter_brush::{Falloff, StrokeMethod};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A white opaque canvas + a red brush, already in Wet Paint.
fn wet_tool() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 200 * 120 * 4], 200, 120);
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.8, 0.1, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("wetpaint");
    t
}

/// Total suspended+settled pigment mass in the live session's grid (0.0 with no session).
fn grid_mass(t: &PainterTool) -> f64 {
    t.paint.wetpaint.session.as_ref().map_or(0.0, |s| {
        let g = &s.engine.layers[0].grid;
        g.susp.iter().map(|&v| f64::from(v)).sum::<f64>()
            + g.sett.iter().map(|&v| f64::from(v)).sum::<f64>()
    })
}

/// Draw an ellipse (centre-out drag) — leaves the editor OPEN.
fn draw_ellipse(t: &mut PainterTool) {
    t.set_brush_stroke_method(StrokeMethod::Ellipse.to_u8());
    t.on_canvas_pointer(cp([60.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([90.0, 75.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([100.0, 80.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([100.0, 80.0], PointerPhase::Up));
}

/// Doc 21 G2 (red-first): the commit deposits EXACTLY once, and only the
/// commit — N authoring refills put nothing in the fluid; Enter deposits;
/// a second commit call adds zero. Mutation that bleeds it: depositing the
/// stash from a refill (I2 returns — mass grows while the artist looks).
#[test]
fn the_commit_deposits_exactly_once_and_it_is_the_preview() {
    let mut t = wet_tool();
    draw_ellipse(&mut t);
    // More authoring: nudge a handle (refills re-run; still zero deposit).
    t.on_canvas_pointer(cp([100.0, 80.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([104.0, 82.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([104.0, 82.0], PointerPhase::Up));
    assert_eq!(
        grid_mass(&t),
        0.0,
        "authoring refills deposited into the fluid before the commit"
    );
    assert!(t.commit_open_shape(), "fixture: a shape was open");
    let m1 = grid_mass(&t);
    assert!(m1 > 1.0, "the commit deposited nothing (mass {m1})");
    t.commit_drag_preview(); // stash is empty — must be a no-op on the grid
    assert_eq!(grid_mass(&t), m1, "a second commit deposited again");
}

/// Doc 21 G3 (red-first via mutation): the flat preview is PEELED before
/// the deposit — after Enter, pristine paper shows through where the fluid
/// laid nothing (the bristle sieve's gaps), instead of the flat sketch
/// remaining underneath. Mutation that bleeds it: skipping the peel in
/// `wetpaint_commit_deposit` (every flat pixel then stays painted).
#[test]
fn the_preview_is_peeled_before_the_deposit() {
    let mut t = wet_tool();
    draw_ellipse(&mut t);
    let flat: Vec<bool> = t
        .canvas_rgba
        .chunks_exact(4)
        .map(|p| p[0] != 255 || p[1] != 255 || p[2] != 255)
        .collect();
    assert!(
        flat.iter().any(|&b| b),
        "fixture: the flat preview painted something"
    );
    assert!(t.commit_open_shape());
    let after: Vec<bool> = t
        .canvas_rgba
        .chunks_exact(4)
        .map(|p| p[0] != 255 || p[1] != 255 || p[2] != 255)
        .collect();
    let returned = flat
        .iter()
        .zip(after.iter())
        .filter(|&(&f, &a)| f && !a)
        .count();
    assert!(
        returned > 10,
        "no flat-painted texel returned to pristine paper — the sketch was \
         never peeled, the deposit landed ON TOP of it ({returned})"
    );
}

/// Doc 21 G6: mouse-up IS the DragDot commit — nothing in the fluid during
/// the drag, the release batch after up. Sibling assert for Anchored (whose
/// single-dab transfer G7 pins engine-side). Mutation that bleeds it:
/// stashing without the commit branch in `commit_drag_preview`.
#[test]
fn mouse_up_is_the_drag_dot_and_anchored_commit() {
    for method in [StrokeMethod::DragDot, StrokeMethod::Anchored] {
        let mut t = wet_tool();
        t.set_brush_stroke_method(method.to_u8());
        t.on_canvas_pointer(cp([60.0, 60.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([100.0, 60.0], PointerPhase::Move));
        assert_eq!(
            grid_mass(&t),
            0.0,
            "{method:?}: the drag deposited before the release"
        );
        t.on_canvas_pointer(cp([100.0, 60.0], PointerPhase::Up));
        let m = grid_mass(&t);
        assert!(m > 1.0, "{method:?}: the release deposited nothing ({m})");
    }
}

/// Doc 21 G9 (graft D3): the ONE-SHOT deposit is bit-equal to the same dab
/// list fed incrementally in batches — proves the engine's per-dab water
/// law holds for big lists (no flood, no rate term) and that the door's
/// single replay is the live feed's equal. Mutation that bleeds it: any
/// per-deposit scaling in the door.
#[test]
fn the_one_shot_deposit_matches_the_live_feed() {
    let dabs: Vec<Dab> = (0..12)
        .map(|i| Dab {
            center: [40.0 + 10.0 * i as f32, 60.0],
            radius_px: 10.0,
            coverage: 0.9,
            color: [0.8, 0.1, 0.1],
            rotation: [1.0, 0.0],
            dir: [1.0, 0.0],
            arc_len: 10.0 * i as f32,
            stroke_radius_px: 10.0,
        })
        .collect();
    // One shot, through the commit door.
    let mut a = wet_tool();
    a.paint.wetpaint.pending_deposit = dabs.clone();
    a.wetpaint_commit_deposit();
    // The live feed: same list in three batches inside one gesture.
    let mut b = wet_tool();
    b.paint.wetpaint.live_gesture = true;
    b.stamp_dabs(&dabs[..4]);
    b.stamp_dabs(&dabs[4..9]);
    b.stamp_dabs(&dabs[9..]);
    b.wetpaint_stroke_end();
    let ga = &a.paint.wetpaint.session.as_ref().expect("A session").engine;
    let gb = &b.paint.wetpaint.session.as_ref().expect("B session").engine;
    assert_eq!(
        ga.layers[0].grid.susp, gb.layers[0].grid.susp,
        "the one-shot deposit diverged from the live feed"
    );
}

/// Doc 21 G11: the stash rides the preview — `pending_deposit` non-empty ⇒
/// a preview record exists, across the verb battery; and CANCEL clears it.
/// Mutation that bleeds it: dropping the stash-clear from the peel door.
#[test]
fn cancel_leaves_no_pending_deposit_and_the_stash_rides_the_preview() {
    let inv = |t: &PainterTool, ctx: &str| {
        assert!(
            t.paint.wetpaint.pending_deposit.is_empty() || t.paint.drag_preview.is_some(),
            "stash without a live preview after {ctx}"
        );
    };
    // Draw → invariant → Esc clears.
    let mut t = wet_tool();
    draw_ellipse(&mut t);
    inv(&t, "draw");
    assert!(
        !t.paint.wetpaint.pending_deposit.is_empty(),
        "fixture: stash filled"
    );
    assert!(t.cancel_open_shape());
    assert!(
        t.paint.wetpaint.pending_deposit.is_empty(),
        "Esc left a stash a later commit would deposit"
    );
    // Draw → leave the mode (bake) clears.
    let mut t = wet_tool();
    draw_ellipse(&mut t);
    t.set_paint_tool_mode("smear");
    assert!(
        t.paint.wetpaint.pending_deposit.is_empty(),
        "leaving the mode kept the stash"
    );
    // Draw → a fresh freehand gesture clears (paint_begin).
    let mut t = wet_tool();
    draw_ellipse(&mut t);
    t.commit_open_shape();
    t.set_brush_stroke_method(StrokeMethod::Space.to_u8());
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    inv(&t, "fresh gesture");
    t.on_canvas_pointer(cp([40.0, 20.0], PointerPhase::Up));
}

/// Doc 21 G15: Tiling and Symmetry reach the commit deposit exactly as they
/// reach the preview — the stash is the UNtiled batch and the dispatcher
/// re-tiles/mirrors at deposit. An ellipse near the right seam with Tiling-X
/// must land fluid mass on BOTH sides. Mutation that bleeds it: stashing the
/// tiled list (double-wrap) or bypassing `stamp_dabs` at the door.
#[test]
fn the_commit_deposit_is_tiled_like_the_preview() {
    let mut t = wet_tool();
    t.paint.tiling[0] = true;
    t.set_brush_stroke_method(StrokeMethod::Ellipse.to_u8());
    t.on_canvas_pointer(cp([190.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([205.0, 75.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([205.0, 75.0], PointerPhase::Up));
    assert!(t.commit_open_shape());
    let sess = t.paint.wetpaint.session.as_ref().expect("session");
    let g = &sess.engine.layers[0].grid;
    let side = |x0: usize, x1: usize| -> f64 {
        let mut m = 0.0;
        for cy in 1..=g.h {
            for cx in x0..=x1 {
                m += f64::from(g.susp[cx + cy * g.s]) + f64::from(g.sett[cx + cy * g.s]);
            }
        }
        m
    };
    assert!(side(150, 200) > 1.0, "no fluid near the right seam");
    assert!(
        side(1, 50) > 1.0,
        "the wrapped copy never reached the fluid — Tiling lost at the deposit"
    );
}

/// Doc 21 G16 (§F layer 3): `deposit_pass` is FALSE outside the door across
/// the whole verb battery — a leak turns every refill into a fluid deposit
/// (I2 resurrected, presenting as "extra juicy", not as an error). Mutation
/// that bleeds it: any second writer of the flag.
#[test]
fn the_deposit_pass_is_false_outside_the_door() {
    let mut t = wet_tool();
    assert!(!t.paint.wetpaint.deposit_pass);
    draw_ellipse(&mut t);
    assert!(!t.paint.wetpaint.deposit_pass, "after authoring");
    t.commit_open_shape();
    assert!(!t.paint.wetpaint.deposit_pass, "after commit");
    draw_ellipse(&mut t);
    t.cancel_open_shape();
    assert!(!t.paint.wetpaint.deposit_pass, "after cancel");
    t.set_paint_tool_mode("smear");
    assert!(!t.paint.wetpaint.deposit_pass, "after mode leave");
}

/// Doc 21 G17: the boolean MULTI-SHAPE batch reaches the stash and the grid —
/// the composite's traced contours are dabs like any other (no region-fill
/// door needed), and the parked+boolean funnel (`restamp_shapes_preview`)
/// stashes the ONE combined batch. Mutation that bleeds it: the wet branch
/// of `stamp_drag_preview` skipping the shapes funnel's batches.
#[test]
fn the_boolean_multi_shape_batch_reaches_the_stash_and_the_grid() {
    let mut t = wet_tool();
    draw_ellipse(&mut t);
    t.set_stroke_op_mode(1); // Add — the boolean composite
    t.set_brush_stroke_method(StrokeMethod::Polygon.to_u8());
    t.on_canvas_pointer(cp([100.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([130.0, 85.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([130.0, 85.0], PointerPhase::Up));
    assert!(
        !t.paint.wetpaint.pending_deposit.is_empty(),
        "the boolean multi-shape preview stashed nothing"
    );
    assert!(t.commit_open_shape());
    let m = grid_mass(&t);
    assert!(
        m > 1.0,
        "the boolean multi-shape set never reached the fluid ({m})"
    );
}

/// Doc 21 G4 — THE differential: a LIVE session survives authoring and the
/// commit deposit FUSES with the old water. Oracle is `sess.base` POINTER
/// identity (not appearance — a fresh session looks plausibly similar; the
/// silent theft). While held: zero sim steps (grid bit-frozen) and the flat
/// preview survives the ticks (no composite tears it). Mutations that bleed
/// it: (a) dropping the re-arm in the stash tail ⇒ the guard kills the
/// session at the next tick; (b) dropping the hold ⇒ a tick composite
/// erases preview pixels.
#[test]
fn a_live_session_survives_authoring_and_the_deposit_fuses() {
    let mut t = wet_tool();
    // Live water: an incremental stroke, then a few ticks of flow.
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
    for k in 1..=8 {
        t.on_canvas_pointer(cp([30.0 + 10.0 * k as f32, 30.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([110.0, 30.0], PointerPhase::Up));
    for _ in 0..4 {
        t.paint_tick(1.0 / 40.0);
    }
    let base_ptr = {
        let sess = t.paint.wetpaint.session.as_ref().expect("live water");
        std::sync::Arc::as_ptr(&sess.base)
    };
    // Author an ellipse OVER the live water.
    draw_ellipse(&mut t);
    let frozen = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("survived")
        .engine
        .layers[0]
        .grid
        .susp
        .clone();
    let preview_px: Vec<bool> = t
        .canvas_rgba
        .chunks_exact(4)
        .map(|p| p[0] != 255 || p[1] != 255 || p[2] != 255)
        .collect();
    for _ in 0..6 {
        t.paint_tick(1.0 / 40.0);
    }
    {
        let sess = t.paint.wetpaint.session.as_ref().expect(
            "the live session DIED under the authoring preview — the re-arm is gone \
             (the wet-on-wet differential silently stolen)",
        );
        assert_eq!(
            std::sync::Arc::as_ptr(&sess.base),
            base_ptr,
            "the session was rebuilt mid-authoring — not the same water"
        );
        assert_eq!(
            sess.engine.layers[0].grid.susp, frozen,
            "the sim STEPPED while the artist authored — the hold is gone"
        );
    }
    let still: Vec<bool> = t
        .canvas_rgba
        .chunks_exact(4)
        .map(|p| p[0] != 255 || p[1] != 255 || p[2] != 255)
        .collect();
    assert_eq!(
        preview_px, still,
        "a tick composite tore the flat preview off the canvas"
    );
    // Enter: the deposit lands in the SAME session — the fusion oracle.
    let m0 = grid_mass(&t);
    assert!(t.commit_open_shape());
    let sess = t.paint.wetpaint.session.as_ref().expect("fused session");
    assert_eq!(
        std::sync::Arc::as_ptr(&sess.base),
        base_ptr,
        "the deposit landed in a FRESH session — nothing fuses with the old water"
    );
    assert!(grid_mass(&t) > m0 + 1.0, "the commit deposited nothing");
}

/// Doc 21 G5: Esc returns the water ALIVE and untouched — the cancel peel is
/// an OWNED write (the guard re-arms), the hold releases, and the canvas is
/// byte-equal to the pre-shape composite. Mutation that bleeds it: peeling
/// without the re-arm (session dies at the next tick).
#[test]
fn esc_returns_the_water_alive() {
    let mut t = wet_tool();
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
    for k in 1..=8 {
        t.on_canvas_pointer(cp([30.0 + 10.0 * k as f32, 30.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([110.0, 30.0], PointerPhase::Up));
    for _ in 0..4 {
        t.paint_tick(1.0 / 40.0);
    }
    let before = t.canvas_rgba.as_ref().clone();
    draw_ellipse(&mut t);
    assert!(t.cancel_open_shape(), "fixture: a shape was open to cancel");
    assert_eq!(
        &*t.canvas_rgba, &before,
        "Esc did not return the canvas to the pre-shape composite"
    );
    for _ in 0..4 {
        t.paint_tick(1.0 / 40.0);
    }
    assert!(
        t.paint.wetpaint.session.is_some(),
        "the water DIED across an Esc — the peel was read as foreign"
    );
}

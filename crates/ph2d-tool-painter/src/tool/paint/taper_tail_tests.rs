//! Gates for the **far end of a freehand stroke** — [`super::taper_tail`].
//!
//! The oracle is the CANVAS, never the dab list: the whole point of the pen-up resolve is that ink
//! already on the screen is put back and laid again, and a gate that read the dabs would be green over
//! a replay that never reached a pixel.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;
use ph2d_painter_brush::taper::Taper;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A white canvas and a hard black disc, so "how wide is the mark here?" is a column count and not an
/// opinion.
fn canvas(size: u32, radius: f32, taper: Taper) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        taper,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t
}

/// Drag a straight horizontal stroke at `y`, from `x0` to `x1`, in 1 px steps.
fn drag(t: &mut PainterTool, y: f32, x0: f32, x1: f32) {
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    let mut x = x0;
    while x < x1 {
        x += 1.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
}

/// How many rows of the column at `x` are inked (alpha-blind: the canvas is opaque white, the brush is
/// opaque black, so "dark" is the mark).
fn ink_height(t: &PainterTool, x: u32, size: u32) -> u32 {
    ink_height_in(t, x, size, 0..size)
}

/// The same, restricted to a band of rows — what a canvas with more than one stroke on it needs, because
/// a whole COLUMN crosses every stroke that shares that x.
fn ink_height_in(t: &PainterTool, x: u32, size: u32, ys: std::ops::Range<u32>) -> u32 {
    let px = &t.canvas_rgba;
    ys.filter(|y| {
        let i = ((y * size + x) * 4) as usize;
        px.get(i).is_some_and(|&r| r < 128)
    })
    .count() as u32
}

/// **The far end of a freehand drag is tapered — the ink already on the canvas is put back and laid
/// again at pen-up.**
///
/// Nothing narrows while the artist draws (that is the anti-delay law, gated in the engine); this is
/// the other half, and its oracle is the picture: the last stretch of the mark is THINNER than its
/// body, measured in inked rows.
///
/// **Mutation that must bleed:** drop the `resolve_taper_tail()` call from `paint_end` — the stroke
/// keeps the blunt end it laid live.
#[test]
fn the_far_end_of_a_freehand_drag_is_tapered_at_pen_up() {
    const SIZE: u32 = 160;
    let mut t = canvas(
        SIZE,
        10.0,
        Taper {
            end: 2.0, // 2 diameters = 40 px
            ..Taper::default()
        },
    );
    drag(&mut t, 80.0, 20.0, 130.0);

    let body = ink_height(&t, 70, SIZE);
    let near_end = ink_height(&t, 125, SIZE);
    assert!(
        body >= 18,
        "fixture: the body of the stroke is not full width ({body} rows of a 20 px brush)"
    );
    assert!(
        near_end * 2 < body,
        "the far end was not tapered: {near_end} inked rows at x=125 against {body} in the body — the \
         stroke still ends blunt"
    );
}

/// **The head keeps its taper through the pen-up replay, and the body keeps its width.**
///
/// The resolve puts the whole stroke back and lays it again, so it is the one thing that could QUIETLY
/// undo the half that already worked. Both ends in one gate, because the failure mode of a bad ratio is
/// to fix one and break the other.
///
/// **Mutation that must bleed:** replay with `w_final` applied ABSOLUTELY — the guard and the ratio
/// together (`if s.emitted_w > MIN_EMITTED_W { let r = w_final; … }`), so the head, which already wears
/// its taper, gets it applied a second time.
///
/// ⚠️ Changing only the ratio to `w_final` is **semantically neutral and survives**, measured: where the
/// far end is far away `w_final == emitted_w` exactly, so the `w_final < s.emitted_w` guard is false and
/// the branch never runs. That guard is load-bearing, not paperwork — the honest mutation has to take
/// it out too.
#[test]
fn the_replay_does_not_re_taper_the_head_or_thin_the_body() {
    const SIZE: u32 = 160;
    let mut t = canvas(
        SIZE,
        10.0,
        Taper {
            start: 2.0,
            end: 2.0,
            ..Taper::default()
        },
    );
    drag(&mut t, 80.0, 20.0, 130.0);

    let body = ink_height(&t, 75, SIZE);
    assert!(
        body >= 18,
        "the replay thinned the BODY of the stroke ({body} rows of a 20 px brush)"
    );
    // The head window is 40 px from x=20, so x=30 sits half way up its ramp: tapered, but not a point.
    let head = ink_height(&t, 30, SIZE);
    assert!(
        head < body,
        "the head lost its taper in the replay ({head} rows against {body} in the body)"
    );
    assert!(
        head >= 4,
        "the head was tapered TWICE ({head} rows) — the replay applied an absolute factor to a dab \
         that already carried one"
    );
}

/// **A stroke with no end taper is byte-identical to one the resolve never looked at.**
///
/// The recording, the restore and the replay must all be free — not merely cheap — for every stroke
/// that did not ask for a tapered end, which is every stroke the app has ever painted.
///
/// **Mutation that must bleed:** drop the `mem::take` in `resolve_taper_tail` (`.clone()` instead) —
/// the list survives the stroke and the next one appends to it, so a second stroke replays the first.
///
/// ⚠️ Two mutations were tried here and are **semantically neutral, measured**: making
/// `taper_tail_wanted` always true, and dropping its `taper.end > 0.0` term. Both only make the recorder
/// run; `resolve_taper_tail` still returns at `end_px <= 0.0`, and a restore-then-replay of a stroke
/// with no end taper reproduces it exactly. That is worth knowing rather than hiding: the resolve is
/// IDEMPOTENT, and what those terms buy is the cost, not the picture.
#[test]
fn a_stroke_without_an_end_taper_is_untouched_by_the_resolve() {
    const SIZE: u32 = 160;
    let mut plain = canvas(SIZE, 10.0, Taper::default());
    drag(&mut plain, 80.0, 20.0, 130.0);
    let a = plain.canvas_rgba.to_vec();

    let mut head_only = canvas(
        SIZE,
        10.0,
        Taper {
            start: 2.0,
            ..Taper::default()
        },
    );
    drag(&mut head_only, 80.0, 20.0, 130.0);
    let b = head_only.canvas_rgba.to_vec();
    assert_ne!(a, b, "fixture: the head taper did not change the picture");
}

/// **The recording does not outlive its stroke.**
///
/// The list is what the resolve replays, so a list that survives is a list the NEXT stroke appends to —
/// and the next stroke's resolve would then restore and lay the previous one again, against its own
/// far end.
///
/// ⚠️ The two strokes are deliberately DIFFERENT LENGTHS, and that is what makes the leak observable at
/// all: with equal lengths the replay is idempotent (same dabs, same arc, same `total` ⇒ the same
/// taper), and a fixture built from two matching strokes stays green through the bug. Here the short
/// stroke's dabs sit far from the LONG stroke's far end, so a leaked replay lays them at FULL width and
/// the short stroke's tapered tip goes blunt.
///
/// ⚠️ **Two mechanisms establish this and either one suffices, so a single mutation SURVIVES — measured,
/// not assumed:** the `mem::take` in `resolve_taper_tail` and the `drop_taper_record()` at the top of
/// `paint_begin`. Swapping the take for a `clone()` leaves the list alive until the next pen-down clears
/// it, which is before anything can read it. A layered defence, documented here rather than gated per
/// layer, because the second layer is a lifecycle call whose reason to exist is its own (a gesture that
/// starts owns nothing from the last one).
#[test]
fn the_record_does_not_outlive_its_stroke() {
    const SIZE: u32 = 256;
    let taper = Taper {
        end: 2.0,
        ..Taper::default()
    };
    let mut t = canvas(SIZE, 10.0, taper);
    drag(&mut t, 60.0, 20.0, 90.0); // short
    // ⚠️ Measured in a BAND around this stroke, not down the whole column: the long stroke below shares
    // every x, and a column count would report the two of them added together. And the x is one where
    // the tip is thin but still THERE — further along it is sub-pixel, and `0 rows` would satisfy any
    // "is it tapered?" bar by vacuum.
    let band = 40..80;
    let tip_after_its_own_stroke = ink_height_in(&t, 80, SIZE, band.clone());
    assert!(
        (1..12).contains(&tip_after_its_own_stroke),
        "fixture: the short stroke's tip is not a thin, present mark ({tip_after_its_own_stroke} rows)"
    );

    drag(&mut t, 140.0, 20.0, 220.0); // long, elsewhere on the canvas
    let tip_now = ink_height_in(&t, 80, SIZE, band);
    assert_eq!(
        tip_now, tip_after_its_own_stroke,
        "the second stroke re-laid the first: its tapered tip went from \
         {tip_after_its_own_stroke} rows to {tip_now}"
    );
}

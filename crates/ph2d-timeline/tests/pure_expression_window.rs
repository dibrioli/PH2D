//! **A PURE per-clip expression obeys its AUTHORED duration; with none it is INFINITE**
//! (Enio, 2026-07-28: *"se o usuário coloca zero na duração … ele é infinito"* — `0` = ∞).
//!
//! A per-clip expression has no last key to end it, so a clip that keys NOTHING but drives a
//! channel by formula has no content length. Two questions, two answers:
//!
//! - **The CUT (`clip_cut`, the solo/Keys clock):** an authored `length_override` clamps the
//!   playhead at the composition end (freeze); NO override is UNBOUNDED — the formula runs
//!   forever (`x(6) = 600` for `time*100`). That is what `0` = infinite means at playback, and
//!   it matches a keyed clip (whose track holds its last key past the end) and the scene and
//!   container, which never clamp an un-authored clock either.
//! - **The DERIVED length (`clip_end_seconds`, strip-sizing + go-to-end):** an unbounded
//!   pure-expression clip still needs a finite window to place in a strip and a ruler extent,
//!   so it falls back to [`DEFAULT_DURATION_SECONDS`]. This is the length only — NOT a cut.
//!
//! The PRODUCT never hits the unbounded cut by accident: every authored clip carries 4 s
//! (`with_default_duration`, `AddClip`), so a pure expression is windowed by that override
//! (freeze + veil). Clearing the Dur to `0` is the artist asking for infinite — no veil, ∞ in
//! the box, formula forever. The rules are inert for a formula-free clip (`!expr.is_empty()`
//! never fires ⇒ the fade fingerprint is byte-identical) and for a keyed clip (its keyed end
//! wins).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    DEFAULT_DURATION_SECONDS, PropKind, StackHost, StripSource, TimelineDoc, apply_from_doc,
    apply_scene,
};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn x_of(w: &World, e: Entity) -> f32 {
    w.get::<Transform>(e).unwrap().translation.x
}

/// The value `time*100` freezes at, past the composition default (E at the 4 s cut).
fn frozen_at_cut() -> f32 {
    #[expect(clippy::cast_possible_truncation, reason = "test value")]
    let v = DEFAULT_DURATION_SECONDS as f32 * 100.0;
    v
}

/// **The CUT: an authored 4 s duration freezes a pure expression at the composition end.**
///
/// The product default (`with_default_duration`/`AddClip` stamp 4 s) windows the formula
/// exactly as it windows a keyed track: inside the composition it runs, past the authored end
/// it FREEZES at `E(4)`. (Mutation: make `clip_cut` ignore `length_override` ⇒ `x(6) = 600`,
/// running past the authored end, RED.)
#[test]
fn a_pure_expression_with_an_authored_duration_is_cut_at_the_end() {
    let mut w = World::new();
    let e = w.spawn(Transform::default()).id();
    let mut doc = TimelineDoc::new();
    let tgt = doc.bind(e.to_bits(), PropKind::TranslationX);
    doc.set_clip_expr(0, tgt, Some("time*100".into()));
    // The product default: an AUTHORED composition duration (what boot/AddClip stamp).
    doc.set_clip_length_override(0, Some(DEFAULT_DURATION_SECONDS));

    apply_from_doc(&mut w, &mut doc, 1.0);
    assert!(
        (x_of(&w, e) - 100.0).abs() < 1e-2,
        "inside the composition the pure expression runs (t=1 -> 100), got {}",
        x_of(&w, e)
    );
    apply_from_doc(&mut w, &mut doc, 6.0);
    assert!(
        (x_of(&w, e) - frozen_at_cut()).abs() < 1e-2,
        "past the AUTHORED end (4 s) the pure expression FREEZES at E(4)={}, never E(6)=600; got {}",
        frozen_at_cut(),
        x_of(&w, e)
    );
}

/// **No authored duration is UNBOUNDED — the formula runs forever (`0` = infinite).**
///
/// A cleared Dur (`length_override == None`) is the artist asking for no time limit, so
/// `clip_cut` does NOT clamp: `time*100` reaches `E(6) = 600`. The Dur box reads ∞ and the veil
/// is absent (`view_authored_end` is `None`). (Mutation: re-add the pure-expression clamp
/// [`clip_cut` cutting an un-authored formula clip at `clip_end_seconds`] ⇒ `x(6) = 400`, the
/// old 4 s freeze, RED — the exact behaviour "0 = infinite" reverses.)
#[test]
fn a_pure_expression_with_no_duration_runs_forever() {
    let mut w = World::new();
    let e = w.spawn(Transform::default()).id();
    let mut doc = TimelineDoc::new();
    let tgt = doc.bind(e.to_bits(), PropKind::TranslationX);
    doc.set_clip_expr(0, tgt, Some("time*100".into())); // PURE, and NO duration → infinite

    assert_eq!(
        doc.view_authored_end(None, true),
        None,
        "no override → the view is unbounded (no veil, the box reads infinity)"
    );
    apply_from_doc(&mut w, &mut doc, 6.0);
    assert!(
        (x_of(&w, e) - 600.0).abs() < 1e-2,
        "an unbounded pure expression runs forever (t=6 -> 600), never frozen at 4 s; got {}",
        x_of(&w, e)
    );
}

/// **Arrange: a pure-expression clip plays WINDOWED inside a strip, 1:1.**
///
/// A strip sizes its source slice from `clip_end_seconds` (`source_length`); with the
/// composition default the slice is 4 s, so the expression advances with the strip instead
/// of collapsing to `E(0)`. (Mutation: revert `clip_end_seconds` to the keyed-only end ⇒ the
/// slice is 0 and the strip reads clip-time 0 ⇒ `x(1)=0`, RED.)
#[test]
fn a_pure_expression_plays_windowed_inside_an_arrange_strip() {
    let mut w = World::new();
    let e = w.spawn(Transform::default()).id();
    let mut doc = TimelineDoc::new();
    let tgt = doc.bind(e.to_bits(), PropKind::TranslationX);
    doc.set_clip_expr(0, tgt, Some("time*100".into()));
    // The panel sizes a new strip's span from `source_length` (== `clip_end_seconds`); a
    // strip spanning the composition default plays the clip 1:1.
    let lane = doc.add_lane("L".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Clip(0),
        0.0,
        DEFAULT_DURATION_SECONDS,
    )
    .unwrap();

    apply_scene(&mut w, &mut doc, 1.0, |_| false);
    assert!(
        (x_of(&w, e) - 100.0).abs() < 1e-2,
        "inside the strip the pure expr runs 1:1 (t=1 -> local 1 -> 100), got {} \
         (0 = the zero-slice collapse this gate exists to catch)",
        x_of(&w, e)
    );
    apply_scene(&mut w, &mut doc, 3.0, |_| false);
    assert!(
        (x_of(&w, e) - 300.0).abs() < 1e-2,
        "and advances with the strip (t=3 -> 300), got {}",
        x_of(&w, e)
    );
    // Outside the strip [0,4]: the pure expr is quiet (nothing plays), so a sentinel holds —
    // it does NOT keep running to E(6)=600.
    w.get_mut::<Transform>(e).unwrap().translation.x = 42.0;
    apply_scene(&mut w, &mut doc, 6.0, |_| false);
    assert!(
        (x_of(&w, e) - 42.0).abs() < 1e-2,
        "past the strip the pure expr is quiet (sentinel 42 holds, not E), got {}",
        x_of(&w, e)
    );
}

/// **`0` = infinite is the SAME law for a clip, a container, and the scene** (Enio,
/// 2026-07-28: *"a regra do infinito também deve se aplicar a strips e containers"*). With no
/// authored duration each scope is UNBOUNDED — [`TimelineDoc::view_authored_end`] is `None` (no
/// veil, the box reads ∞) and the cut leaves the clock untouched; author one and it clamps; a
/// typed `0` clears back to infinite. (Mutation: make any `*_cut` clamp an un-authored clock ⇒
/// the "runs on" assert RED; or make `set_clip_length_override` keep `Some(0.0)` ⇒ the `0`
/// clears back to infinite assert RED.)
#[test]
fn zero_is_infinite_for_clip_container_and_scene() {
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("C".into());

    // No override anywhere → every scope is unbounded: no authored end, and no cut.
    assert_eq!(doc.view_authored_end(None, true), None, "clip: unbounded (no veil, box reads infinity)");
    assert_eq!(doc.view_authored_end(Some(c), false), None, "container: unbounded");
    assert_eq!(doc.scene_length, None, "scene: unbounded");
    assert_eq!(doc.clip_cut(0, 9.0), 9.0, "clip: the clock runs on (infinite)");
    assert_eq!(doc.container_cut(c, 9.0), 9.0, "container: the clock runs on");
    assert_eq!(doc.cut_scene(9.0), 9.0, "scene: the clock runs on");

    // Author 4 s on each → each becomes finite: an authored end AND a cut at 4 s.
    doc.set_clip_length_override(0, Some(4.0));
    doc.set_container_length_override(c, Some(4.0));
    doc.set_scene_length(Some(4.0));
    assert_eq!(doc.view_authored_end(None, true), Some(4.0), "clip: finite (veil at 4)");
    assert_eq!(doc.view_authored_end(Some(c), false), Some(4.0), "container: finite");
    assert_eq!(doc.scene_length, Some(4.0), "scene: finite");
    assert_eq!(doc.clip_cut(0, 9.0), 4.0, "clip: cut at the authored end");
    assert_eq!(doc.container_cut(c, 9.0), 4.0, "container: cut at the authored end");
    assert_eq!(doc.cut_scene(9.0), 4.0, "scene: cut at the authored end");

    // And a typed `0` clears back to infinite (the numeric-box gesture).
    doc.set_clip_length_override(0, Some(0.0));
    assert_eq!(doc.clip_length_override(0), None, "0 clears the override -> infinite again");
    assert_eq!(doc.clip_cut(0, 9.0), 9.0, "and the clock runs on again");
}

/// **A KEYED clip keeps its keyed end — the pure-expression default fires only with no
/// keyed content.** A clip with keyed content bounds itself (the track holds at its last
/// key), and its expression channels ride that same window. (Mutation: drop the
/// `keyed_end <= 0` guard ⇒ a keyed clip's end jumps to the 4 s default, retiming every
/// strip placed on it, RED.)
#[test]
fn a_keyed_clip_keeps_its_keyed_end_even_with_an_expression() {
    let mut w = World::new();
    let e = w.spawn(Transform::default()).id();
    let mut doc = TimelineDoc::new();
    // Keyed X: content ends at 2 s.
    doc.insert_key(
        e.to_bits(),
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        e.to_bits(),
        PropKind::TranslationX,
        s(2.0),
        AnimValue::Float(20.0),
        Interp::Linear,
    );
    // + a pure expression on a DIFFERENT channel (Y).
    let ytgt = doc.bind(e.to_bits(), PropKind::TranslationY);
    doc.set_clip_expr(0, ytgt, Some("time*10".into()));

    assert!(
        (doc.clip_end_seconds(0) - 2.0).abs() < 1e-9,
        "a clip with keyed content (end 2 s) keeps its keyed end, not the 4 s expr default; got {}",
        doc.clip_end_seconds(0)
    );
    // And the cut agrees: past 2 s the keyed X holds (track extrapolation), never cut early.
    assert!(
        (doc.clip_cut(0, 3.0) - 3.0).abs() < 1e-9,
        "a keyed clip stays open-ended at the cut (holds at its last key), got {}",
        doc.clip_cut(0, 3.0)
    );
}

/// **The rule is inert for a formula-free clip** — the guard is `!expr.is_empty()`, so a
/// clip that keys nothing AND drives nothing has a derived end of `0`, exactly as before
/// (sparsity: no content, no window to invent). This is why the fade fingerprint corpus
/// (formula-free) is byte-identical. (Mutation: fire the default for any empty clip ⇒ this
/// `0` becomes 4, RED.)
#[test]
fn an_empty_formula_free_clip_still_has_a_zero_derived_end() {
    let doc = TimelineDoc::new();
    assert!(
        doc.clip_end_seconds(0).abs() < 1e-9,
        "a clip with no keys and no expression has a derived end of 0 (unchanged), got {}",
        doc.clip_end_seconds(0)
    );
    assert!(
        (doc.clip_cut(0, 5.0) - 5.0).abs() < 1e-9,
        "and its cut is open-ended (returns t untouched), got {}",
        doc.clip_cut(0, 5.0)
    );
}

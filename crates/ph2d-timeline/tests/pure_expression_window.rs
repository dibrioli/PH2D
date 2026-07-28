//! **A PURE per-clip expression obeys the composition window** (ADR-0145, the "pure
//! expression rides its strip" close — Enio's "vincular a expressão pura").
//!
//! A per-clip expression has no last key to end it, so a clip that keys NOTHING but drives
//! a channel by formula had a derived content-end of `0`. Two symptoms, one root (measured
//! headless, 2026-07-28):
//!
//! - **Keys/solo:** `clip_cut` used only `length_override`, so a pure-expression clip with
//!   no explicit duration ran the formula on the RAW playhead forever (`x(6) = 600` for
//!   `time*100`).
//! - **Arrange:** `clip_end_seconds` (== `source_length`) was `0`, so a strip placed on the
//!   clip got a zero-length source slice and read clip-time `0` forever (`E(0) = 0`).
//!
//! The composition-duration model already gives every AUTHORED clip a 4 s default
//! (`with_default_duration`), which windows a pure expression in both views — so the product
//! was correct. This makes it correct BY CONSTRUCTION: a clip that carries a per-clip
//! expression and keys nothing gets [`DEFAULT_DURATION_SECONDS`] as its composition end even
//! without an explicit override, so clearing the Dur (or a legacy save) can never reopen the
//! hole. The change is inert for a formula-free clip (`!expr.is_empty()` never fires ⇒ the
//! fade fingerprint is byte-identical) and for a keyed clip (its keyed end wins).

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

/// **Keys/solo: a pure expression is bounded by the composition default, not extrapolated.**
///
/// Inside the composition the formula runs; past its end it FREEZES at `E(end)` — exactly
/// as a keyed track holds at its last key. (Mutation: revert `clip_cut` to the
/// `length_override`-only cut ⇒ `x(6) = 600`, extrapolating past the composition end, RED.)
#[test]
fn a_pure_expression_is_windowed_by_the_solo_composition_default() {
    let mut w = World::new();
    let e = w.spawn(Transform::default()).id();
    let mut doc = TimelineDoc::new();
    let tgt = doc.bind(e.to_bits(), PropKind::TranslationX);
    doc.set_clip_expr(0, tgt, Some("time*100".into())); // PURE: no keyed track, no duration

    apply_from_doc(&mut w, &mut doc, 1.0);
    assert!(
        (x_of(&w, e) - 100.0).abs() < 1e-2,
        "inside the composition the pure expression runs (t=1 -> 100), got {}",
        x_of(&w, e)
    );
    apply_from_doc(&mut w, &mut doc, 6.0);
    assert!(
        (x_of(&w, e) - frozen_at_cut()).abs() < 1e-2,
        "past the composition end (4 s) the pure expression FREEZES at E(4)={}, \
         never extrapolating to E(6)=600; got {}",
        frozen_at_cut(),
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

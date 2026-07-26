//! Gates for the timeline-signal crossing law ([ADR-0143] §2). Each is red-first: the
//! mutation that would reinstate the Godot "frame equality" bug — or drop the wrap, or
//! open the loop-start end — is named next to the gate it kills.
//!
//! [ADR-0143]: ../../../docs/architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md

use ph2d_anim::RationalTime;
use ph2d_timeline::{TimelineDoc, signals_crossed};

/// A doc whose markers sit at the given `(seconds, signal)` — a `None` signal is a
/// pure annotation. Returned by value so its `markers()` slice outlives the borrow.
fn doc_with(markers: &[(f64, Option<&str>)]) -> TimelineDoc {
    let mut doc = TimelineDoc::new();
    for (t, sig) in markers {
        let i = doc.add_marker(RationalTime::from_seconds(*t), "m");
        if let Some(s) = sig {
            doc.set_marker_signal(i, Some((*s).to_string()));
        }
    }
    doc
}

/// The heart of the law: it fires on the tick whose advance CROSSES the marker, once,
/// and it is a fact of the interval — not of frame equality (the Godot bug). The
/// marker sits STRICTLY inside `(0.5, 1.5]`, so a `t == now` mutation misses it.
#[test]
fn a_signal_fires_once_when_the_play_crosses_it() {
    let doc = doc_with(&[(1.0, Some("foo"))]);
    assert_eq!(signals_crossed(doc.markers(), 0.5, 1.5, None), ["foo"]);
    // The next tick does not re-fire it (t is now <= prev).
    assert!(signals_crossed(doc.markers(), 1.5, 2.5, None).is_empty());
    // A tick that stops short of it fires nothing.
    assert!(signals_crossed(doc.markers(), 0.0, 0.9, None).is_empty());
}

/// Catch-up: one big advance crosses several markers; all fire, in TIME order (they
/// are stored out of order on purpose — the emission sorts).
#[test]
fn catch_up_fires_every_crossed_marker_in_chronological_order() {
    let doc = doc_with(&[(3.0, Some("c")), (1.0, Some("a")), (2.0, Some("b"))]);
    assert_eq!(signals_crossed(doc.markers(), 0.0, 3.5, None), ["a", "b", "c"]);
}

/// A marker with no signal is a pure annotation and never emits — the v12 behaviour.
#[test]
fn a_marker_without_a_signal_never_fires() {
    let doc = doc_with(&[(1.0, None), (1.2, Some("only_this"))]);
    assert_eq!(signals_crossed(doc.markers(), 0.0, 2.0, None), ["only_this"]);
}

/// Half-open `(prev, now]`: a marker exactly at `prev` already fired last tick and
/// must not re-fire; one exactly at `now` fires. This pins the boundary convention.
#[test]
fn a_marker_at_prev_does_not_refire_but_one_at_now_does() {
    let doc = doc_with(&[(1.0, Some("at_prev")), (2.0, Some("at_now"))]);
    assert_eq!(signals_crossed(doc.markers(), 1.0, 2.0, None), ["at_now"]);
}

/// A loop wrap `[0, 4)`: the sweep goes `3.8 -> 4 -> 0 -> 0.7`, so the crossed set is
/// `(3.8, 4] ∪ [0, 0.7]`, in that order — late, then the loop-start (CLOSED low), then
/// early. The marker at 2.0 is in neither segment. Dropping the wrap or opening the
/// low end each changes the answer.
#[test]
fn a_looped_play_fires_the_wrapped_markers_in_time_order() {
    let doc = doc_with(&[
        (3.9, Some("late")),
        (0.0, Some("start")),
        (0.5, Some("early")),
        (2.0, Some("middle_untouched")),
    ]);
    assert_eq!(
        signals_crossed(doc.markers(), 3.8, 0.7, Some((0.0, 4.0))),
        ["late", "start", "early"],
    );
}

/// `now < prev` with no loop is a backward jump (scrub/reverse), not a forward
/// crossing — nothing fires. (The bridge also gates on play-only; this is the pure
/// law's half.)
#[test]
fn a_backward_jump_without_a_loop_emits_nothing() {
    let doc = doc_with(&[(1.0, Some("foo"))]);
    assert!(signals_crossed(doc.markers(), 2.0, 0.0, None).is_empty());
}

/// An empty or whitespace-only name is not a contract anyone can match — the setter
/// clears it, so it reads as "no signal", never as a nameless emitter.
#[test]
fn a_blank_signal_name_clears_the_signal() {
    let mut doc = TimelineDoc::new();
    let i = doc.add_marker(RationalTime::from_seconds(1.0), "m");
    assert!(doc.set_marker_signal(i, Some("   ".to_string())));
    assert!(doc.markers()[i].signal.is_none());
    assert!(signals_crossed(doc.markers(), 0.0, 2.0, None).is_empty());
}

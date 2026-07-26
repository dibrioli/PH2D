//! What the signal smoke AUTHORS is testable without a window: the Mover's track and
//! the marker/signal pattern. (The fire itself is gated in
//! `render_loop::timeline_bridge::signal_tests`; the crossing law in `ph2d-timeline`.)

use super::author_mover;
use ph2d_anim::RationalTime;
use ph2d_timeline::{PropKind, TimelineDoc, signals_crossed};

#[test]
fn the_mover_is_keyed_on_x() {
    let mut doc = TimelineDoc::new();
    author_mover(&mut doc, 7);
    let n = doc
        .binding_for(7, PropKind::TranslationX)
        .and_then(|b| doc.active_clip().track(b.target))
        .map_or(0, |t| t.keys().len());
    assert_eq!(n, 2, "X keyed at both ends so there is motion to sync to");
}

#[test]
fn the_scene_fires_exactly_the_two_signals_over_one_pass() {
    // Mirror the smoke's markers: two that emit, one pure annotation.
    let mut doc = TimelineDoc::new();
    let a = doc.add_marker(RationalTime::from_seconds(1.0), "step");
    doc.set_marker_signal(a, Some("footstep".to_string()));
    let b = doc.add_marker(RationalTime::from_seconds(2.5), "beat");
    doc.set_marker_signal(b, Some("beat".to_string()));
    doc.add_marker(RationalTime::from_seconds(3.5), "chapter"); // no signal
    // One forward pass over the loop crosses both signals, in time order, and never
    // the plain annotation — the smoke's whole premise, made executable.
    assert_eq!(
        signals_crossed(doc.markers(), 0.0, 4.0, Some((0.0, 4.0))),
        ["footstep", "beat"],
        "the pass fires the two signals, not the annotation"
    );
}

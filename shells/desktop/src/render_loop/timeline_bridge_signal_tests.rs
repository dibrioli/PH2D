//! Gates for the timeline-signal outbox (ADR-0143 W1): the bridge EMITS the crossing
//! law's result as a decoupled event, forward SCENE play only. The crossing law itself
//! is gated in `ph2d-timeline/tests/signals_crossed.rs`; here we pin the BRIDGE half —
//! play-only, a seek re-baselines instead of bursting, reverse is silent — which the
//! pure law cannot see. Sibling `#[path]` module (`use super::*`) so `timeline_bridge.rs`
//! stays under the shell LOC cap.

use super::*;

/// A doc with one marker carrying signal `"foo"` at `t = 1.0 s`.
fn doc_with_signal() -> ph2d_timeline::TimelineDoc {
    let mut doc = ph2d_timeline::TimelineDoc::new();
    let i = doc.add_marker(ph2d_anim::RationalTime::from_seconds(1.0), "m");
    doc.set_marker_signal(i, Some("foo".to_string()));
    doc
}

/// A playhead playing FORWARD, positioned at `t`.
fn forward_at(t: f64) -> Playhead {
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.play();
    ph.seek(t);
    ph
}

#[test]
fn forward_play_across_a_marker_emits_the_signal() {
    let doc = doc_with_signal();
    let mut sig = SignalEmitter::default(); // last_time = 0
    sig.emit(&doc, &forward_at(1.5), false); // (0, 1.5] crosses 1.0
    assert_eq!(sig.out.len(), 1);
    assert_eq!(sig.out[0].name, "foo");
    assert!(
        (sig.out[0].t - 1.5).abs() < 1e-9,
        "stamped at the scene time"
    );
}

#[test]
fn a_paused_scrub_emits_nothing() {
    let doc = doc_with_signal();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.pause(); // a fresh Playhead plays by default — a scrub is paused
    ph.seek(1.5);
    let mut sig = SignalEmitter::default();
    sig.emit(&doc, &ph, false);
    assert!(sig.out.is_empty(), "a paused scrub is not a crossing");
}

#[test]
fn reverse_play_emits_nothing() {
    let doc = doc_with_signal();
    let mut ph = forward_at(1.5);
    ph.set_rate(-1.0); // playing, but backwards
    let mut sig = SignalEmitter::default();
    sig.emit(&doc, &ph, false);
    assert!(sig.out.is_empty(), "a footstep is a forward-time event");
}

#[test]
fn a_seek_while_playing_re_baselines_instead_of_firing_the_skipped_span() {
    let doc = doc_with_signal();
    let mut sig = SignalEmitter::default();
    // jumped = true: the seek jumped the playhead past the marker — a discontinuity.
    sig.emit(&doc, &forward_at(1.5), true);
    assert!(sig.out.is_empty(), "a seek is not a crossing");
    // And it re-baselined last_time to 1.5: the next forward tick does not re-fire it.
    sig.emit(&doc, &forward_at(1.6), false);
    assert!(
        sig.out.is_empty(),
        "the marker is behind the re-baselined last_time"
    );
}

#[test]
fn a_scrub_past_the_marker_does_not_arm_the_next_play_to_burst() {
    let doc = doc_with_signal();
    let mut sig = SignalEmitter::default();
    let mut paused = Playhead::new(1.0 / 60.0);
    paused.pause(); // a fresh Playhead plays by default
    paused.seek(2.0); // scrubbed past 1.0, paused → re-baselines last_time = 2.0
    sig.emit(&doc, &paused, false);
    assert!(sig.out.is_empty());
    sig.emit(&doc, &forward_at(2.5), false); // playing forward from 2.0
    assert!(
        sig.out.is_empty(),
        "the marker at 1.0 is behind us, not re-fired"
    );
}

#[test]
fn a_marker_without_a_signal_never_reaches_the_outbox() {
    let mut doc = ph2d_timeline::TimelineDoc::new();
    doc.add_marker(ph2d_anim::RationalTime::from_seconds(1.0), "annotation");
    let mut sig = SignalEmitter::default();
    sig.emit(&doc, &forward_at(2.0), false);
    assert!(sig.out.is_empty(), "a pure annotation emits nothing");
}

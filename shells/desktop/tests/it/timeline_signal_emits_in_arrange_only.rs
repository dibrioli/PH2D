//! Arch-gate (ADR-0143 W1): the SCENE signal is emitted ONLY in the Arrange branch of
//! `timeline_bridge::run`. Arrange is the one view where the caller passes the SCENE
//! clock and where the document's markers live; the container and clip-solo views
//! freeze the scene clock, so emitting there would fire scene signals off a frozen (or
//! interior) clock. This reads the source because no unit test drives the full
//! three-way `run` (it needs a `World`, the autokey state, the two-way `TimelineState`).
//!
//! Mutation it kills: moving `signals.emit(` up into the container or clip-solo branch
//! puts it before one of the interior applies, so the position check fails.

use std::fs;

#[test]
fn the_scene_signal_is_emitted_once_in_the_arrange_branch() {
    let src = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/render_loop/timeline_bridge.rs"
    ))
    .expect("read timeline_bridge.rs");

    // Exactly one emit site (a second would be a second, undocumented, emitter).
    assert_eq!(
        src.matches("signals.emit(").count(),
        1,
        "the scene signal has exactly one emit site"
    );

    // It sits in the Arrange branch: after BOTH interior applies (container / clip
    // solo), which is what "the else branch" means positionally.
    let emit = src.find("signals.emit(").expect("emit site");
    let container = src
        .find("apply_container(world")
        .expect("container branch apply");
    let solo = src
        .find("apply_active_clip(world")
        .expect("clip-solo branch apply");
    assert!(
        emit > container && emit > solo,
        "the sole emit is in the Arrange branch (after both interior applies)"
    );
}

//! **The shell hands the open Expression card's formula to the scene** — and does it
//! on a WALL clock, with the undo queue standing down while it drives.
//!
//! ⚠️ Why these live in an arch-gate instead of a unit test: both sites are inside
//! `render_loop::render_frame` and `App::post_frame_undo`, which need a window and a
//! GPU device. Nothing headless reaches them, so mutating either one leaves the whole
//! workspace green — the behaviour is gated in `ph2d-timeline` (the pass) and in
//! `ph2d-panel-timeline` (the publish), and this file gates the WIRE between them.
//!
//! ⚠️ Each assertion states a PROPERTY, never a byte distance: a proxy anchored on how
//! far apart two things sit in the file expires the first time somebody inserts a line
//! between them ([[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]]).

const RENDER_LOOP: &str = include_str!("../src/render_loop/mod.rs");
const UNDO: &str = include_str!("../src/undo.rs");

/// **The preview is installed BEFORE the apply that consumes it.**
///
/// The expression pass reads the live channel while it runs, so installing after
/// `timeline_bridge::run` would make every previewed frame one frame stale — the artist
/// would be tuning against the number they typed a frame ago, which reads as lag.
#[test]
fn the_card_is_installed_before_the_apply_that_consumes_it() {
    let install = RENDER_LOOP
        .find("expr_live::set_live_expr")
        .expect("the shell installs the live expression somewhere");
    // ⚠️ `timeline_bridge::run(` WITH the paren: a doc-comment 250 lines above mentions
    // `timeline_bridge::run` in prose, and the first draft of this gate found THAT and
    // failed on correct code — a gate that anchors on a name finds the name wherever it
    // is written, including where somebody is only talking about it.
    let apply = RENDER_LOOP
        .find("timeline_bridge::run(")
        .expect("the shell applies the timeline somewhere");
    assert!(
        install < apply,
        "the live expression must be installed before the apply reads it \
         (install at {install}, apply at {apply})"
    );
}

/// **The preview runs on the WALL clock, not on the playhead.**
///
/// ⚠️ This is the whole request — *"em tempo real mesmo que o clip esteja pausado"*. A
/// preview riding `active_playhead` would sit perfectly still in exactly the case the
/// feature exists for, and a preview riding a frame COUNT would make the wobble's speed
/// a function of the frame rate.
#[test]
fn the_preview_clock_is_wall_time() {
    let install = RENDER_LOOP
        .find("expr_live::set_live_expr")
        .expect("install site");
    let close = RENDER_LOOP[install..]
        .find(");")
        .expect("the install call closes");
    let call = &RENDER_LOOP[install..install + close];
    assert!(
        call.contains("time: self.expr_preview_clock"),
        "the previewed time must be the wall clock; got:\n{call}"
    );
    // …and that clock is fed from the same frame delta everything else reads.
    assert!(
        RENDER_LOOP.contains("self.expr_preview_clock += wall_dt;"),
        "the preview clock must advance by the real frame delta"
    );
}

/// **The formula comes from the PANEL, every frame, `Option` and all.**
///
/// ⚠️ The `None` is load-bearing: it is what makes closing the card stop driving the
/// scene. An install that only ran on the frames a card exists would leave the last
/// formula running forever, and the property would never come back.
#[test]
fn the_shell_passes_the_panels_answer_through_including_the_none() {
    let install = RENDER_LOOP
        .find("expr_live::set_live_expr")
        .expect("install site");
    let call = &RENDER_LOOP[install..install + 400];
    assert!(
        call.contains("expr_live_target()"),
        "the shell must ask the panel what is open; got:\n{call}"
    );
    assert!(
        call.contains(".map("),
        "and pass its Option through — a `Some(..)` built unconditionally would never \
         stop driving the scene; got:\n{call}"
    );
}

/// **The undo queue stands down while a preview drives a property.**
///
/// A world being driven by a live preview is not a world anybody authored, so the
/// by-diff recorder would log a step per frame for a pose the artist never made — the
/// same reason `flip_colorize.live_busy` sits in this very condition.
#[test]
fn the_undo_diff_is_suppressed_while_a_preview_drives_the_scene() {
    let f = UNDO
        .find("fn post_frame_undo")
        .expect("the by-diff recorder exists");
    let guard_start = UNDO[f..]
        .find("if !had_input")
        .expect("the early-out guard")
        + f;
    let guard_end = UNDO[guard_start..]
        .find('{')
        .expect("the guard opens a block")
        + guard_start;
    let guard = &UNDO[guard_start..guard_end];
    assert!(
        guard.contains("expr_live::is_previewing()"),
        "the guard must skip the diff while previewing; got:\n{guard}"
    );
}

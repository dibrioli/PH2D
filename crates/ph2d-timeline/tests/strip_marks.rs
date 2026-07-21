//! **The change bars**: what each of a strip's four corners last DID, kept on the
//! strip so it stays visible after the pointer lets go (Enio, 2026-07-16).
//!
//! The mark is `edge_before_the_gesture - edge_now`, in seconds. Everything here is
//! asserted on that number's **sign and size**, because the sign is what the panel
//! turns into a side: a start edge pulled outward gained time that is now inside the
//! strip, and pushed inward lost time that is now outside. One expression, both
//! edges, both operations — so the gates below have to hold the sign, not just the
//! magnitude.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_timeline::{
    PropKind, StripId, TimelineIntent as I, TimelineState, apply_intent, mark_index,
};

/// A 2 s clip on a lane, placed over `[2, 4)` with 2 s of empty lane in front of it —
/// room for a start edge to be pulled outward without hitting zero.
fn app_with_a_strip() -> (TimelineState, Playhead, StripId) {
    let (mut st, mut ph) = (TimelineState::default(), Playhead::default());
    st.doc.insert_key(
        7,
        PropKind::TranslationX,
        RationalTime::from_seconds(2.0),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    apply_intent(&mut st, &mut ph, I::AddLane);
    apply_intent(
        &mut st,
        &mut ph,
        I::AddStrip {
            lane: 0,
            source: ph2d_timeline::StripSource::Clip(0),
            t_start: 2.0,
            t_end: 4.0,
        },
    );
    let id = st.doc.stack()[0].strips[0].id;
    (st, ph, id)
}

fn mark(st: &TimelineState, id: StripId, stretch: bool, edge: u8) -> f64 {
    st.doc.strip(0, id).unwrap().marks[mark_index(stretch, edge)]
}

/// **A fresh strip has nothing to say.** Zero is "not edited", and the panel draws
/// nothing for it — so a strip nobody has touched must not be born wearing a bar.
#[test]
fn a_strip_nobody_has_edited_carries_no_marks() {
    let (st, _ph, id) = app_with_a_strip();
    assert_eq!(st.doc.strip(0, id).unwrap().marks, [0.0; 4]);
}

/// **Growing points one way, shrinking the other.** This is the whole visual rule,
/// stated on the number the panel reads: pull the start edge OUT (earlier) and the
/// mark is positive, so the bar covers `[t_start, t_start + mark]` — inside. Push it
/// IN (later) and the mark is negative, so the bar covers `[t_start + mark, t_start]`
/// — outside. Get the sign backwards and every bar in the panel points at the wrong
/// side of its own edge.
#[test]
fn the_marks_sign_says_which_side_of_the_edge_the_change_is_on() {
    let (mut st, mut ph, id) = app_with_a_strip();

    // Pull the start edge OUT, 2.0 -> 1.5: the strip GREW by half a second.
    apply_intent(
        &mut st,
        &mut ph,
        I::TrimStrip {
            lane: 0,
            id,
            edge: 0,
            t: 1.5,
            from: 2.0,
        },
    );
    assert!(
        (mark(&st, id, false, 0) - 0.5).abs() < 1e-12,
        "grew: positive, so the bar lies INSIDE the strip"
    );

    // Push the END edge in, 4.0 -> 3.0: the strip SHRANK by a second.
    apply_intent(
        &mut st,
        &mut ph,
        I::TrimStrip {
            lane: 0,
            id,
            edge: 1,
            t: 3.0,
            from: 4.0,
        },
    );
    assert!(
        (mark(&st, id, false, 1) - 1.0).abs() < 1e-12,
        "shrank at the end: positive too, and the bar lies OUTSIDE — the sign is \
         read against the edge, so the same number means opposite sides at the two ends"
    );
    assert!(
        (mark(&st, id, false, 0) - 0.5).abs() < 1e-12,
        "and editing one corner leaves the other corner's bar alone"
    );
}

/// **Every frame of a drag reports the same total.** The apply is handed the anchor,
/// not a per-frame delta, so a gesture that crawls the edge across in forty frames
/// leaves exactly the mark of a gesture that jumped there in one. An accumulating
/// implementation passes the one-frame case and silently multiplies the real one.
#[test]
fn a_drag_leaves_the_same_mark_however_many_frames_it_took() {
    let (mut st, mut ph, id) = app_with_a_strip();
    let mut crawl = |t: f64| {
        apply_intent(
            &mut st,
            &mut ph,
            I::TrimStrip {
                lane: 0,
                id,
                edge: 1,
                t,
                from: 4.0, // the anchor never moves: it is where the GESTURE began
            },
        );
    };
    for step in 1..=40 {
        crawl(4.0 + f64::from(step) * 0.025);
    }
    assert!(
        (mark(&st, id, false, 1) + 1.0).abs() < 1e-12,
        "one second of travel, forty frames, one second of mark"
    );
}

/// **The bar describes where the edge ENDED UP, not where the pointer asked it to
/// go.** Drag a start edge past its own end and the trim clamps; a mark taken from
/// the request would draw a bar out into the timeline describing a strip that does
/// not exist.
#[test]
fn a_corner_that_hit_its_limit_marks_the_change_it_actually_made() {
    let (mut st, mut ph, id) = app_with_a_strip();
    apply_intent(
        &mut st,
        &mut ph,
        I::TrimStrip {
            lane: 0,
            id,
            edge: 0,
            t: 99.0, // way past the end — the trim clamps to a minimum span
            from: 2.0,
        },
    );
    let s = st.doc.strip(0, id).unwrap();
    assert!(s.t_start < s.t_end, "the clamp held");
    assert!(
        (mark(&st, id, false, 0) - (2.0 - s.t_start)).abs() < 1e-12,
        "the mark measures the edge that landed, not the pointer that asked"
    );
}

/// **A trim and a stretch mark DIFFERENT corners.** They are two operations on one
/// edge, drawn on two bands (red bottom, green top), and a shared slot would make
/// each erase the other's history.
#[test]
fn a_trim_and_a_stretch_at_the_same_edge_keep_separate_marks() {
    let (mut st, mut ph, id) = app_with_a_strip();
    apply_intent(
        &mut st,
        &mut ph,
        I::TrimStrip {
            lane: 0,
            id,
            edge: 1,
            t: 3.0,
            from: 4.0,
        },
    );
    apply_intent(
        &mut st,
        &mut ph,
        I::StretchStrip {
            lane: 0,
            id,
            edge: 1,
            t: 5.0,
            from: 3.0,
        },
    );
    assert!(
        (mark(&st, id, false, 1) - 1.0).abs() < 1e-12,
        "the trim's bar is still there"
    );
    assert!(
        (mark(&st, id, true, 1) + 2.0).abs() < 1e-12,
        "and the stretch wrote its own, on the other band"
    );
}

/// **Undo takes the bar back with the edit.** The mark lives in the document, so it
/// is part of the state a step restores — a bar left behind by an undone trim would
/// describe an edit the strip no longer carries.
#[test]
fn undoing_the_edit_undoes_its_change_bar() {
    let (mut st, mut ph, id) = app_with_a_strip();
    apply_intent(
        &mut st,
        &mut ph,
        I::TrimStrip {
            lane: 0,
            id,
            edge: 1,
            t: 3.0,
            from: 4.0,
        },
    );
    assert!(mark(&st, id, false, 1).abs() > 0.5);
    apply_intent(&mut st, &mut ph, I::Undo);
    assert!(
        mark(&st, id, false, 1).abs() < 1e-12,
        "the bar went back with the edit it described"
    );
}

/// **Typing a rate marks the same corner a stretch drag would.** `SetStripSpeed` is
/// the retime stated as a number instead of felt as a drag (its own doc says so), and
/// a change bar that appeared for one and not the other would make one edit look like
/// two.
#[test]
fn setting_the_speed_marks_the_end_corner_like_a_stretch_does() {
    let (mut st, mut ph, id) = app_with_a_strip();
    apply_intent(
        &mut st,
        &mut ph,
        I::SetStripSpeed {
            lane: 0,
            id,
            speed: 0.5, // half rate: 2 s of slice now takes 4 s of timeline
        },
    );
    let s = st.doc.strip(0, id).unwrap();
    assert!((s.t_end - 6.0).abs() < 1e-12, "the span doubled");
    assert!(
        (mark(&st, id, true, 1) + 2.0).abs() < 1e-12,
        "and the green end corner wears the two seconds it grew"
    );
}

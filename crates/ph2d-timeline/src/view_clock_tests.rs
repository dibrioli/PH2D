//! Gates of the ONE clock (ADR-0115 R8 emenda 3).

use super::*;
use crate::{KeyRefusal, PropKind, StackHost, StripSource};
use ph2d_anim::{AnimValue, Interp, RationalTime};

const DT: f64 = 1.0 / 60.0;
const BITS: u64 = 7;

/// A document whose clip 0 is 3 s long (keys at 0 and 3).
fn three_second_clip() -> TimelineState {
    let mut st = TimelineState::new();
    for t in [0.0, 3.0] {
        st.doc.upsert_key(
            BITS,
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(1.0),
            Interp::Linear,
        );
    }
    st
}

/// The same, with clip 0 placed once in the scene at `[2, 5)`.
fn placed_once() -> TimelineState {
    let mut st = three_second_clip();
    let lane = st.doc.add_lane("L".into()).expect("a lane");
    st.doc.add_strip(lane, 0, 2.0, 5.0).expect("a strip");
    st
}

/// Two clocks at rest (a `Playhead` is born PLAYING).
fn paused() -> (Playhead, Playhead) {
    let (mut editor, mut clip) = (Playhead::new(DT), Playhead::new(DT));
    editor.pause();
    clip.pause();
    (editor, clip)
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// ⭐ **(iv) The clip clock exists ONLY under a stack** — the gate ADR-0115 emenda 2 cited
/// (`keys_mode_is_published_only…`) and nobody wrote. `keys_mode` stayed the TAB (the scope,
/// Enio 2026-07-27); the CLOCK is this law. Without a stack the Keys tab rules the editor's.
#[test]
fn the_clip_clock_exists_only_under_a_stack() {
    assert_eq!(ViewClock::of(true, None, false), ViewClock::Editor);
    assert_eq!(ViewClock::of(true, None, true), ViewClock::Clip);
    assert_eq!(ViewClock::of(false, None, true), ViewClock::Editor);
    assert_eq!(ViewClock::of(false, Some(0), false), ViewClock::Container);
    let mut st = three_second_clip();
    st.keys_mode = true;
    assert_eq!(st.view_clock(), ViewClock::Editor, "Keys, no stack");
    st.doc.add_lane("L".into());
    assert_eq!(st.view_clock(), ViewClock::Clip, "Keys under a stack");
    st.runtime = true;
    assert_eq!(st.view_clock(), ViewClock::Editor, "the scene RUNS");
}

/// Without a stack the ONE clock carries the loop of the tab on screen: the Keys loop on Keys,
/// the Arrange loop on Arrange — each view keeps its own, as it always did.
#[test]
fn the_one_clock_carries_the_loop_of_the_tab_on_screen() {
    let mut st = three_second_clip();
    st.doc.set_active_loop_for(true, Some((1.0, 2.0)));
    st.doc.set_active_loop_for(false, Some((0.0, 4.0)));
    let (mut editor, mut clip) = (Playhead::new(DT), Playhead::new(DT));
    st.keys_mode = true;
    arm_view_loop(&mut st, &mut editor, &mut clip);
    assert_eq!(editor.loop_range(), Some((1.0, 2.0)), "Keys tab");
    assert_eq!(clip.loop_range(), None, "the clip clock is not in play");
    st.keys_mode = false;
    arm_view_loop(&mut st, &mut editor, &mut clip);
    assert_eq!(editor.loop_range(), Some((0.0, 4.0)), "Arrange tab");
    // A stack appears under the Keys tab: the clip clock takes over, seeded where it stood.
    st.keys_mode = true;
    arm_view_loop(&mut st, &mut editor, &mut clip);
    editor.seek(1.5);
    st.doc.add_lane("L".into());
    arm_view_loop(&mut st, &mut editor, &mut clip);
    assert_eq!(
        clip.loop_range(),
        Some((1.0, 2.0)),
        "the clip clock, Keys loop"
    );
    assert_eq!(
        editor.loop_range(),
        Some((0.0, 4.0)),
        "the scene clock, Arrange loop"
    );
    assert!(
        near(clip.time(), 1.5),
        "seeded: the Keys playhead does not jump"
    );
}

/// ⭐ **(ii) Under a stack, a clip that plays ONCE carries the editor's clock** to the scene
/// second where it reads the clip clock's second — and only when the clip clock moved.
#[test]
fn a_clip_that_plays_once_carries_the_editor_clock() {
    let st = placed_once();
    let (mut editor, mut clip) = paused();
    clip.seek(1.25);
    follow_clip_clock(&st.doc, &clip, &mut editor, false);
    assert!(
        near(editor.time(), 0.0),
        "the clip clock did not move: a tab switch stays put"
    );
    follow_clip_clock(&st.doc, &clip, &mut editor, true);
    assert!(
        near(editor.time(), 3.25),
        "clip 1.25 s plays at scene 3.25 s"
    );
    clip.play();
    clip.seek(2.0);
    follow_clip_clock(&st.doc, &clip, &mut editor, false);
    assert!(near(editor.time(), 4.0), "playing follows too");
}

/// ⭐ **(iii) Zero or two placements: the editor stays**, and the reason is the same `K` speaks.
#[test]
fn a_clip_that_plays_twice_or_nowhere_leaves_the_editor_clock() {
    let mut st = three_second_clip();
    st.doc.add_lane("L".into());
    assert_eq!(
        crate::clip_scene_map(&st.doc, 0),
        Err(KeyRefusal::NotPlaying)
    );
    let mut st = placed_once();
    st.doc.add_strip(0, 0, 6.0, 9.0).expect("a second strip");
    assert_eq!(
        crate::clip_scene_map(&st.doc, 0),
        Err(KeyRefusal::PlaysTwice)
    );
    let (mut editor, mut clip) = (Playhead::new(DT), Playhead::new(DT));
    editor.seek(0.5);
    clip.seek(1.0);
    follow_clip_clock(&st.doc, &clip, &mut editor, true);
    assert!(
        near(editor.time(), 0.5),
        "no inverse: the editor does not move"
    );
}

/// The walk goes down through a container: a clip at `[1, 4)` inside one instanced at scene
/// 10 plays at scene 11 — and a container instanced twice makes its clip play twice.
#[test]
fn a_clip_inside_a_container_is_mapped_through_the_instance() {
    let mut st = three_second_clip();
    let c = st.doc.add_container("C".into());
    let inner = StackHost::Container(c);
    let l = st
        .doc
        .add_lane_in(inner, "in".into())
        .expect("a lane inside");
    st.doc
        .add_strip_to(inner, l, StripSource::Clip(0), 1.0, 4.0)
        .expect("the clip inside");
    let top = st.doc.add_lane("L".into()).expect("a scene lane");
    let src = StripSource::Container(u16::try_from(c).expect("small"));
    let len = st.doc.container_length_seconds(c);
    assert!(len >= 4.0, "the interior holds the whole strip: {len}");
    st.doc
        .add_strip_to(StackHost::Document, top, src, 10.0, 10.0 + len)
        .expect("the instance");
    let map = crate::clip_scene_map(&st.doc, 0).expect("plays once");
    assert!(near(map.host_time(1.5), 12.5), "{map:?}");
    st.doc
        .add_strip_to(StackHost::Document, top, src, 30.0, 30.0 + len)
        .expect("a second instance");
    assert_eq!(
        crate::clip_scene_map(&st.doc, 0),
        Err(KeyRefusal::PlaysTwice)
    );
}

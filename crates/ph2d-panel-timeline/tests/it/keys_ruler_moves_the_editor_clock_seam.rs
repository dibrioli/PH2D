//! ⭐ **The Keys ruler moves the editor's clock** (ADR-0115 R8 emenda 3; Enio, 2026-10-13: *«na
//! aba Keys arrastar a régua e dar play tem de mexer a simulação»*) — by the panel's REAL event:
//! the ruler drag (`WidgetEvent::ValueChanged(TIMELINE_RULER)` → `PanelEvent::SetValue`, which
//! the shell turns into `TimelineIntent::Scrub`), applied to the clock the shell picks through
//! the same door (`TimelineState::view_clock`), then the shell's follow step.

use ph2d_core::Playhead;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_panel_timeline::{TimelinePanel, ids::TIMELINE_RULER, tab::Tab};
use ph2d_timeline::{
    AnimValue, Interp, KeyRefusal, PropKind, TimelineIntent, TimelineState, TimelineViewSnapshot,
};

const DT: f64 = 1.0 / 60.0;
const SPAN_S: f64 = 10.0;

/// Clip 0, 3 s long; with `strips`, placed at each `[t, t + 3)` of the scene.
fn doc(strips: &[f64]) -> TimelineState {
    let mut st = TimelineState::new();
    for t in [0.0, 3.0] {
        st.doc.upsert_key(
            7,
            PropKind::TranslationX,
            ph2d_anim::RationalTime::from_seconds(t),
            AnimValue::Float(1.0),
            Interp::Linear,
        );
    }
    if !strips.is_empty() {
        let lane = st.doc.add_lane("L".into()).expect("a lane");
        for &t in strips {
            st.doc.add_strip(lane, 0, t, t + 3.0).expect("a strip");
        }
    }
    st.keys_mode = true; // the shell stamps the Keys tab the panel published
    st
}

/// Drag the Keys ruler to `frac` of a `[0, SPAN_S)` view; the second the panel asks for.
fn drag_keys_ruler(frac: f32) -> f64 {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<TimelinePanel>();
    let mut panel = ph2d_panel_timeline::state::TimelinePanelState {
        view_start_s: 0.0,
        view_span_s: SPAN_S,
        tab: Tab::Keys,
        ..ph2d_panel_timeline::state::TimelinePanelState::default()
    };
    host.set_slider_value(TIMELINE_RULER, frac);
    host.apply_panel_event::<TimelinePanel>(&mut panel, WidgetEvent::ValueChanged(TIMELINE_RULER));
    host.drained_actions()
        .into_iter()
        .find_map(|a| match a {
            EditorAction::TimelinePanelEvent(PanelEvent::SetValue(id, t))
                if id == TIMELINE_RULER =>
            {
                Some(t)
            }
            _ => None,
        })
        .expect("the ruler asks for a Scrub")
}

/// The shell's frame, cut to the clock: scrub the view's clock, then the follow step.
fn scrub(st: &mut TimelineState, clocks: &mut [Playhead; 3], t: f64) {
    let [editor, clip, cont] = clocks;
    let vista = st.view_clock();
    ph2d_timeline::apply_intent(
        st,
        vista.pick(&mut *editor, &mut *clip, &mut *cont),
        TimelineIntent::Scrub(t),
    );
    if vista == ph2d_timeline::ViewClock::Clip {
        ph2d_timeline::follow_clip_clock(&st.doc, clip, editor, true);
    }
}

fn clocks() -> [Playhead; 3] {
    let mut c = [Playhead::new(DT), Playhead::new(DT), Playhead::new(DT)];
    for p in &mut c {
        p.pause();
    }
    c
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// (i) No stack: the drag moves the EDITOR's clock — the one the Motion cooks on.
#[test]
fn without_a_stack_the_keys_ruler_moves_the_editor_clock() {
    let mut st = doc(&[]);
    let mut c = clocks();
    let t = drag_keys_ruler(0.25);
    scrub(&mut st, &mut c, t);
    assert!(
        near(c[0].time(), t),
        "the editor's clock: {} vs {t}",
        c[0].time()
    );
    assert!(near(c[1].time(), 0.0), "no second clock moved");
}

/// (ii) Under a stack whose clip plays ONCE (at scene 2 s): the drag moves the clip clock and
/// the editor follows to the scene second where the clip reads it.
#[test]
fn a_clip_that_plays_once_carries_the_editor_to_its_scene_second() {
    let mut st = doc(&[2.0]);
    let mut c = clocks();
    let t = drag_keys_ruler(0.15);
    scrub(&mut st, &mut c, t);
    assert!(near(c[1].time(), t), "the Keys ruler scrubs the CLIP clock");
    assert!(
        near(c[0].time(), 2.0 + t),
        "the editor at scene {}: {}",
        2.0 + t,
        c[0].time()
    );
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &c[1], true);
    assert!(
        ph2d_panel_timeline::ruler_clock_for_tests(Tab::Keys, &snap)
            .history
            .is_some(),
        "followed: the history band rides this ruler"
    );
    assert_eq!(
        ph2d_panel_timeline::breadcrumb_width_for_tests(&snap),
        0.0,
        "nothing to excuse"
    );
}

/// (iii) Under a stack whose clip plays TWICE: the editor stays, and the bar SAYS why — the
/// readout takes room in the transport, and the history band is not painted (it would lie).
#[test]
fn a_clip_that_plays_twice_leaves_the_editor_and_says_why() {
    let mut st = doc(&[2.0, 6.0]);
    let mut c = clocks();
    c[0].seek(0.5);
    let t = drag_keys_ruler(0.15);
    scrub(&mut st, &mut c, t);
    assert!(near(c[1].time(), t), "the clip clock still scrubs");
    assert!(
        near(c[0].time(), 0.5),
        "the editor does not move: {}",
        c[0].time()
    );
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &c[1], true);
    assert_eq!(snap.keys_scene, Some(Err(KeyRefusal::PlaysTwice)));
    assert!(
        ph2d_panel_timeline::breadcrumb_width_for_tests(&snap) > 0.0,
        "the bar speaks"
    );
    assert_eq!(
        ph2d_panel_timeline::ruler_clock_for_tests(Tab::Keys, &snap).history,
        None
    );
}

//! **The two view tabs exist ON SCREEN, CLICK, and change what the ruler means.**
//!
//! This gate paints the panel for real (`MockPanelHost::paint` returns the hit index the
//! paint registered) and drives the pointer through `dispatch_pointer` — the same entry
//! the app uses. It is the seam the pure `tab.rs` tests cannot reach: they prove a table
//! is ordered, and "somebody paints that table, registers it, and routes it" is exactly
//! what can fail to happen ([[feedback_widget_is_done_when_a_test_clicks_it]]).
//!
//! The bug the tabs exist to kill is at the bottom of this file: under a stack the KEYS
//! are ruled by the clip's clock and the STRIPS by the timeline's, so one ruler was two
//! rulers wearing the same ticks.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_panel_timeline::tab::{TABS, Tab};
use ph2d_timeline::{
    AnimTarget, Interp, KeyId, KeyView, LaneMode, LaneView, PropKind, StripId, StripLoop,
    StripView, TimelineViewSnapshot, TrackView,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

/// The `PH2D_STACK_SMOKE` screen: one bound track of keys (clip time 0..3) AND one lane
/// whose strip plays that clip at timeline 2..5. Both halves are present — which is the
/// only way to tell whether a tab is hiding one of them.
fn keys_and_a_stack(clip_time: Option<f64>, playhead: f64) -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        time_seconds: playhead,
        clip_time,
        clip_length_seconds: 3.0,
        tracks: vec![TrackView {
            target: AnimTarget::new(7),
            prop: PropKind::TranslationX,
            entity: 1,
            missing: false,
            keys: vec![
                KeyView {
                    id: KeyId::new(1),
                    t_seconds: 0.0,
                    value: -3.0,
                    interp: Interp::Linear,
                    selected: false,
                    roving: false,
                },
                KeyView {
                    id: KeyId::new(2),
                    t_seconds: 3.0,
                    value: 3.0,
                    interp: Interp::Linear,
                    selected: false,
                    roving: false,
                },
            ],
        }],
        lanes: vec![LaneView {
            name: "Lane 1".into(),
            muted: false,
            weight: 1.0,
            mode: LaneMode::Override,
            strips: vec![StripView {
                id: StripId(1),
                clip_name: "Right".into(),
                t_start: 2.0,
                t_end: 5.0,
                blend_in: 0.0,
                blend_out: 0.0,
                ease_locked_in: false,
                ease_locked_out: false,
                loop_mode: StripLoop::Once,
                speed: 1.0,
            }],
        }],
        clips: vec!["Left".into(), "Right".into()],
        active_clip: 1,
        ..TimelineViewSnapshot::default()
    }
}

/// Paint once and hand back the registrations.
fn paint(
    host: &mut MockPanelHost,
    state: &mut TimelinePanelState,
    snap: TimelineViewSnapshot,
) -> Vec<(ph2d_editor_core::NodeId, Rect)> {
    set_current_timeline(Some(snap));
    host.paint::<TimelinePanel>(state, VIEWPORT)
}

fn rect_of(
    regs: &[(ph2d_editor_core::NodeId, Rect)],
    id: ph2d_editor_core::NodeId,
) -> Option<Rect> {
    regs.iter().find(|(w, _)| *w == id).map(|(_, r)| *r)
}

/// **A tab is done when a test clicks it.** Paint, find the rect the paint registered,
/// put the pointer in it, and require the panel to actually switch.
#[test]
fn clicking_a_tab_switches_the_view() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    assert_eq!(state.tab, Tab::Keys, "a fresh panel opens on Keys");

    let regs = paint(&mut host, &mut state, keys_and_a_stack(Some(1.0), 3.0));
    for (tab, (id, _)) in [Tab::Keys, Tab::Arrange].into_iter().zip(TABS) {
        let r = rect_of(&regs, id).unwrap_or_else(|| {
            panic!("the {tab:?} tab was painted but never hit-registered: it clicks into nothing")
        });
        assert!(
            r.w > 0.0 && r.h > 0.0,
            "{tab:?} has no area to click: {r:?}"
        );
    }

    // Click Arrange — through the real dispatcher, and then through the real router.
    let arrange = rect_of(&regs, ids::TIMELINE_TAB_ARRANGE).expect("Arrange");
    let evs = host.click_at(arrange.x + arrange.w * 0.5, arrange.y + arrange.h * 0.5);
    assert!(
        evs.contains(&WidgetEvent::Click(ids::TIMELINE_TAB_ARRANGE)),
        "the pointer landed on {:?}, not on the Arrange tab — got {evs:?}",
        host.hit_at(arrange.x + arrange.w * 0.5, arrange.y + arrange.h * 0.5)
    );
    for ev in evs {
        host.apply_panel_event::<TimelinePanel>(&mut state, ev);
    }
    assert_eq!(
        state.tab,
        Tab::Arrange,
        "the click was routed but changed nothing"
    );

    // And back — a tab that only goes one way is a trapdoor.
    let regs = paint(&mut host, &mut state, keys_and_a_stack(Some(1.0), 3.0));
    let keys = rect_of(&regs, ids::TIMELINE_TAB_KEYS).expect("Keys");
    for ev in host.click_at(keys.x + keys.w * 0.5, keys.y + keys.h * 0.5) {
        host.apply_panel_event::<TimelinePanel>(&mut state, ev);
    }
    assert_eq!(state.tab, Tab::Keys);
}

/// Each tab puts ONLY its own half on screen — measured on what the paint registered,
/// not on what the layout said it would.
#[test]
fn each_tab_registers_only_its_own_half() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    let strip = ids::timeline_strip_hit_id(0, 1, 2);
    let row = ids::timeline_row_id(7);

    let regs = paint(&mut host, &mut state, keys_and_a_stack(Some(1.0), 3.0));
    assert!(
        rect_of(&regs, row).is_some(),
        "the Keys tab shows its track row"
    );
    assert!(
        rect_of(&regs, strip).is_none(),
        "the Keys tab registered a STRIP: the two halves are back on one ruler"
    );
    assert!(
        rect_of(&regs, ids::TIMELINE_ADD_TRACK).is_some(),
        "the Keys tab offers +Track"
    );
    assert!(
        rect_of(&regs, ids::TIMELINE_ADD_LANE).is_none(),
        "…and never +Lane, which would add a lane this tab cannot show"
    );

    state.tab = Tab::Arrange;
    let regs = paint(&mut host, &mut state, keys_and_a_stack(Some(1.0), 3.0));
    assert!(
        rect_of(&regs, strip).is_some(),
        "the Arrange tab shows its strip"
    );
    assert!(
        rect_of(&regs, row).is_none(),
        "the Arrange tab registered a track ROW"
    );
    assert!(rect_of(&regs, ids::TIMELINE_ADD_LANE).is_some());
    assert!(rect_of(&regs, ids::TIMELINE_ADD_TRACK).is_none());
}

/// **Under a stack the Keys ruler SCRUBS the clip clock AND draws its OWN loop — only the
/// timeline MARKERS are withheld** (Enio 2026-07-16). The loop is per-view now: the
/// snapshot's `loop_range` is the clip's own, in the clip clock this ruler scrubs, so its
/// braces belong here. Markers, however, are timeline-time and would sit at the wrong
/// second on the clip's ruler, so they stay on Arrange.
#[test]
fn the_clip_ruler_under_a_stack_scrubs_and_draws_its_own_loop_but_no_timeline_markers() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let mut snap = keys_and_a_stack(Some(2.0), 4.0);
    snap.loop_range = Some((0.0, 5.0)); // the clip's own loop (rebuild fills the keys pair)
    snap.markers = vec![(1.0, "M1".into())];

    let regs = paint(&mut host, &mut state, snap.clone());
    assert!(
        rect_of(&regs, ids::TIMELINE_RULER).is_some(),
        "the Keys ruler under a stack must scrub the clip clock — that is how you author keys"
    );
    assert!(
        rect_of(&regs, ids::timeline_loop_brace_id(0)).is_some(),
        "the Keys view draws its OWN clip loop — independent of Arrange (Enio, 2026-07-16)"
    );
    assert!(
        rect_of(&regs, ids::timeline_marker_hit_id(0)).is_none(),
        "the timeline's marker was drawn on the clip's ruler — at the wrong second"
    );

    // The Arrange tab rules the timeline, so it scrubs AND carries its loop + markers.
    state.tab = Tab::Arrange;
    let regs = paint(&mut host, &mut state, snap);
    assert!(
        rect_of(&regs, ids::TIMELINE_RULER).is_some(),
        "Arrange scrubs"
    );
    assert!(rect_of(&regs, ids::timeline_loop_brace_id(0)).is_some());
    assert!(rect_of(&regs, ids::timeline_marker_hit_id(0)).is_some());
}

/// **Nothing changes for a document with no stack** — the case every animator is in almost
/// all of the time. The clip IS the timeline, so the Keys tab is the panel it has always
/// been: it scrubs, it carries the loop and the markers.
#[test]
fn without_a_stack_the_keys_tab_is_the_panel_it_has_always_been() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let mut snap = keys_and_a_stack(Some(1.5), 1.5);
    snap.lanes.clear(); // no stack
    snap.loop_range = Some((0.0, 3.0));
    snap.markers = vec![(1.0, "M1".into())];

    let regs = paint(&mut host, &mut state, snap);
    assert!(
        rect_of(&regs, ids::TIMELINE_RULER).is_some(),
        "a timeline that never touched the stack lost its scrub"
    );
    assert!(rect_of(&regs, ids::timeline_loop_brace_id(0)).is_some());
    assert!(rect_of(&regs, ids::timeline_marker_hit_id(0)).is_some());
    assert!(rect_of(&regs, ids::timeline_row_id(7)).is_some());
}

/// **The clip buttons Enio asked for exist on screen and CLICK** (2026-07-16).
///
/// Duplicate and `I` both push their intent from inside the panel, like `+` does — so
/// there are three ways each could be dead: unpainted, unregistered in `populate` (the
/// Down never makes it active), or unclaimed by `transport_clips::owns` (the router
/// hands it to nobody). This drives the real pointer through all three.
#[test]
fn the_duplicate_and_reverse_buttons_are_painted_and_click() {
    for (id, what) in [
        (ids::TIMELINE_DUP_CLIP, "Duplicate"),
        (ids::TIMELINE_REVERSE_CLIP, "I (reverse)"),
    ] {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        let regs = paint(&mut host, &mut state, keys_and_a_stack(Some(1.0), 3.0));
        let r = rect_of(&regs, id)
            .unwrap_or_else(|| panic!("{what} was painted but never hit-registered"));

        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        let evs = host.click_at(cx, cy);
        assert!(
            evs.contains(&WidgetEvent::Click(id)),
            "{what}: the pointer landed on {:?}, not on it — got {evs:?}",
            host.hit_at(cx, cy)
        );
        // Only the CLICK must be consumed: a press/focus event alongside it is the
        // dispatcher's business and the panel is right to let it pass.
        assert_eq!(
            host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(id)),
            ph2d_editor_core::panel::EventOutcome::Consumed,
            "{what}: the panel ignored its own button"
        );
    }
}

/// …and each raises the intent its glyph promises. A button that consumes the click and
/// pushes nothing is the same dead button, one layer in.
#[test]
fn the_clip_buttons_raise_the_intents_their_glyphs_promise() {
    use ph2d_timeline::TimelineIntent;
    for (id, want) in [
        (
            ids::TIMELINE_DUP_CLIP,
            TimelineIntent::DuplicateClip { index: 1 },
        ),
        (
            ids::TIMELINE_REVERSE_CLIP,
            TimelineIntent::ReverseClip { index: 1 },
        ),
    ] {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        // `active_clip` is 1 ("Right") in this fixture — the intent must name the clip
        // the animator is LOOKING at, not clip zero.
        set_current_timeline(Some(keys_and_a_stack(Some(1.0), 3.0)));
        let _ = ph2d_panel_timeline::state::drain_intents();
        host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(id));
        assert!(
            ph2d_panel_timeline::state::drain_intents().contains(&want),
            "{id:?} consumed its click and raised no {want:?}"
        );
    }
}

/// **The rename field opens OVER the chip it renames** (Enio, 2026-07-16). It used to
/// paint at the corner of the panel body, with nothing to say what it was for.
#[test]
fn the_clip_rename_field_opens_over_the_dropdown_it_renames() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let regs = paint(&mut host, &mut state, keys_and_a_stack(Some(1.0), 3.0));
    let chip = rect_of(&regs, ids::TIMELINE_CLIP_DD).expect("the clip chip");

    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::TIMELINE_RENAME_CLIP),
    );
    let regs = paint(&mut host, &mut state, keys_and_a_stack(Some(1.0), 3.0));
    let field = rect_of(&regs, ids::TIMELINE_CLIP_RENAME_INPUT).expect("the rename field");

    assert_eq!(field.x, chip.x, "same left edge as the chip");
    assert_eq!(field.y, chip.y, "same row as the chip");
    assert!(
        field.w >= chip.w,
        "it must COVER the chip, not sit beside it: {} vs {}",
        field.w,
        chip.w
    );
}

/// **The panel publishes `keys_mode` = Keys tab AND a stack exists.** This is the
/// shell's trigger to solo the active clip on its own clock. Without a stack there is
/// nothing to solo (the clip IS the timeline), so a fresh document on the Keys tab —
/// the DEFAULT — must NOT flip the shell into solo, or a document that never touched
/// the feature would drive a separate clip playhead and drift from Motion (Enio,
/// 2026-07-16).
#[test]
fn keys_mode_is_published_only_on_the_keys_tab_and_only_under_a_stack() {
    use ph2d_panel_timeline::state::keys_mode;
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    // Keys tab (default) WITHOUT a stack: not solo.
    let mut no_stack = keys_and_a_stack(Some(1.5), 1.5);
    no_stack.lanes.clear();
    paint(&mut host, &mut state, no_stack.clone());
    assert!(
        !keys_mode(),
        "a fresh document on the Keys tab must not solo — there is no stack"
    );

    // Keys tab WITH a stack: solo.
    paint(&mut host, &mut state, keys_and_a_stack(Some(1.5), 1.5));
    assert!(keys_mode(), "Keys tab + a stack = solo the active clip");

    // Arrange tab WITH a stack: not solo (you are arranging strips, not editing keys).
    state.tab = Tab::Arrange;
    paint(&mut host, &mut state, keys_and_a_stack(Some(1.5), 1.5));
    assert!(!keys_mode(), "the Arrange tab is never solo");

    // (The hidden-panel case — keys_mode false when the panel is not visible — is
    // covered by `paint`'s early `publish_keys_mode(false)` on the not-visible branch;
    // MockPanelHost forces the panel visible, so it cannot be exercised here.)
}

//! **The Dur(s) chip authors what the artist TYPES — driven by the real gesture.**
//!
//! Every prior gate on this chain was green while the product failed (Enio,
//! 2026-07-23): the ruler gate built the snapshot by hand, the clamp gate called
//! `set_clip_length_override` directly, the seam gate injected a synthetic
//! `ValueChanged`, and the commit-always gate began pre-focused. None contained
//! the GESTURE — click the chip, type, Enter — which is where the bug lived:
//! focus did not select the buffer, so typing "2" into a chip showing "2"
//! parsed 22 and authored a 22 s duration (veil off-screen, clamp pinning
//! nothing, box "not matching the real duration": all three symptoms at once).
//!
//! These gates paint the REAL panel (`with_panel` runs `populate`, which
//! registers the chip and sets its commit-always flag), put a real pointer on
//! the painted rect, type through `dispatch_text_input`, press Enter through
//! `dispatch_key`, route the resulting events through the panel, and read the
//! intent the shell would drain.

use bumpalo::Bump;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::interaction::dispatch::keymap::KEY_ENTER;
use ph2d_editor_core::interaction::{dispatch_key, dispatch_text_input};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{KeyEvent, KeyKind, Modifiers};
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

fn enter() -> KeyEvent {
    KeyEvent {
        keycode: KEY_ENTER,
        modifiers: Modifiers {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        },
        kind: KeyKind::Down,
        timestamp_ns: 0,
    }
}

/// Paint the Keys tab over a document showing `secs` and return the chip's RECT.
/// `explicit=false` mirrors the report's derived-end screenshot.
fn painted_dur_rect(host: &mut MockPanelHost, state: &mut TimelinePanelState, secs: f64) -> Rect {
    set_current_timeline(Some(TimelineViewSnapshot {
        fps: 60.0,
        view_length_seconds: secs,
        view_length_explicit: false,
        ..TimelineViewSnapshot::default()
    }));
    let regs = host.paint::<TimelinePanel>(state, VIEWPORT);
    regs.iter()
        .find(|(w, _)| *w == ph2d_panel_timeline::ids::TIMELINE_LENGTH_NUM)
        .map(|(_, r)| *r)
        .expect("the Dur(s) chip was painted but never hit-registered")
}

/// Paint the Keys tab over a 2 s derived-end doc and return a click point on the
/// chip's LEFT body (the right column is the stepper zone, a value bump — not a
/// focus gesture).
fn painted_dur_chip(host: &mut MockPanelHost, state: &mut TimelinePanelState) -> (f32, f32) {
    let r = painted_dur_rect(host, state, 2.0);
    (r.x + r.w * 0.3, r.y + r.h * 0.5)
}

fn route_events(
    host: &mut MockPanelHost,
    state: &mut TimelinePanelState,
    evs: impl IntoIterator<Item = WidgetEvent>,
) {
    for ev in evs {
        host.apply_panel_event::<TimelinePanel>(state, ev);
    }
}

/// The reported gesture end to end: the chip shows the derived 2.00, the artist
/// clicks it, types "2", presses Enter — the intent must carry 2.0. Before the
/// select-all-on-focus fix this authored `Some(22.0)`.
#[test]
fn typing_the_shown_duration_authors_that_duration() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default(); // tab: Keys
    let (cx, cy) = painted_dur_chip(&mut host, &mut state);

    let clicked = host.click_at(cx, cy);
    route_events(&mut host, &mut state, clicked);
    assert_eq!(
        drain_intents(),
        vec![],
        "the focusing click alone must author NOTHING (a stepper hit here \
         would silently author derived ± one frame)"
    );

    let arena = Bump::new();
    let _ = dispatch_text_input(host.store_mut(), '2', &arena);
    let evs: Vec<WidgetEvent> = dispatch_key(host.store_mut(), enter(), &arena).to_vec();
    route_events(&mut host, &mut state, evs);

    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SetClipLength { len: Some(2.0) }],
        "typing '2' into the chip showing 2.00 must author 2.0 — not 22 \
         (append), not nothing"
    );
    set_current_timeline(None);
}

/// The no-typing half (the commit-always chain, through the REAL populate):
/// click the chip, change nothing, Enter — the shown derived value becomes the
/// authored one. This is the derived→authored transition the chip exists for.
#[test]
fn enter_on_the_untouched_chip_authors_the_shown_value() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let (cx, cy) = painted_dur_chip(&mut host, &mut state);

    let clicked = host.click_at(cx, cy);
    route_events(&mut host, &mut state, clicked);
    let _ = drain_intents();

    let arena = Bump::new();
    let evs: Vec<WidgetEvent> = dispatch_key(host.store_mut(), enter(), &arena).to_vec();
    route_events(&mut host, &mut state, evs);

    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SetClipLength { len: Some(2.0) }],
        "Enter on the untouched chip must author the shown value \
         (`set_number_commit_always`, wired by the real populate)"
    );
    set_current_timeline(None);
}

/// Typing a DIFFERENT value replaces the readout (the general case): the chip
/// shows 2.00, the artist types 5, Enter — the intent carries 5.0, never 25.
#[test]
fn typing_a_new_duration_replaces_the_readout() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let (cx, cy) = painted_dur_chip(&mut host, &mut state);

    let clicked = host.click_at(cx, cy);
    route_events(&mut host, &mut state, clicked);
    let _ = drain_intents();

    let arena = Bump::new();
    let _ = dispatch_text_input(host.store_mut(), '5', &arena);
    let evs: Vec<WidgetEvent> = dispatch_key(host.store_mut(), enter(), &arena).to_vec();
    route_events(&mut host, &mut state, evs);

    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SetClipLength { len: Some(5.0) }],
        "typing '5' must author 5.0 — the focus click selects the readout"
    );
    set_current_timeline(None);
}

// ── the stepper arrows: +/- 0.2 s per click (Enio, 2026-07-23) ───────────────
//
// The Dur chip's right column is a stepper. It stepped by `1/fps` (~0.04 s), so a
// click produced the fiddly "wrong" values the report named (2.04, 2.08, …). A
// duration is coarser: each click is 0.2 s. `stepper_up_rect`/`_down_rect` mark the
// upper/lower half of the stepper column (`stepper_width` = clamp(h*0.6, 16, 22)).

use ph2d_editor_core::widget::NumberInput;

/// A click on the UP arrow raises the authored duration by exactly 0.2 s.
#[test]
fn the_up_stepper_adds_a_fifth_of_a_second() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let r = painted_dur_rect(&mut host, &mut state, 2.0);
    let up = NumberInput::new(ph2d_panel_timeline::ids::TIMELINE_LENGTH_NUM, "", 2.0).up_rect(r);
    let evs = host.click_at(up.x + up.w * 0.5, up.y + up.h * 0.5);
    route_events(&mut host, &mut state, evs);
    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SetClipLength { len: Some(2.2) }],
        "one up-click authors +0.2 s (was 1/fps ~0.04)"
    );
    set_current_timeline(None);
}

/// A click on the DOWN arrow lowers it by exactly 0.2 s.
#[test]
fn the_down_stepper_subtracts_a_fifth_of_a_second() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let r = painted_dur_rect(&mut host, &mut state, 2.0);
    let down =
        NumberInput::new(ph2d_panel_timeline::ids::TIMELINE_LENGTH_NUM, "", 2.0).down_rect(r);
    let evs = host.click_at(down.x + down.w * 0.5, down.y + down.h * 0.5);
    route_events(&mut host, &mut state, evs);
    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SetClipLength { len: Some(1.8) }],
        "one down-click authors -0.2 s"
    );
    set_current_timeline(None);
}

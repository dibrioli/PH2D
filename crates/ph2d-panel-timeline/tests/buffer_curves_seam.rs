//! **The Buffer-Curves chips (§5) are painted, hit-registered, and a REAL click
//! on them raises Store/Swap — Swap only on the track that owns the buffer.**
//!
//! These are the anti-dead-button gates for the A/B toggle. A chip is a
//! `TimelineSurface`, so a click travels the gesture channel (not `WidgetEvent`):
//! the drive is a real pointer Down+Up on the painted rect, then a second paint
//! (which runs `interact::process`, draining the gesture into an intent) — the
//! same path the app runs. Painting + routing proven SEPARATELY can each be green
//! while the connector between them is broken, so every gate here clicks the pixel
//! and reads the intent the shell would drain.
//!
//! The chips live in the graph band, so the track must be EXPANDED. The Swap
//! chip's existence is keyed on `TrackView::buffer_ghost.is_some()` — the one fact
//! that also draws the ghost — so its presence AND absence are both pinned (a Swap
//! on a track with no buffer would be a no-op under the mouse).

use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::ids::timeline_buffer_button_id;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_timeline::{
    AnimTarget, Interp, KeyId, KeyView, PropKind, TimelineIntent, TimelineViewSnapshot, TrackView,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);
const TGT: u64 = 7;

fn kv(id: u64, t: f64, v: f32) -> KeyView {
    KeyView {
        id: KeyId::new(id),
        t_seconds: t,
        value: v,
        interp: Interp::Linear,
        selected: false,
        roving: false,
    }
}

/// A snapshot with one expanded-able track; `ghost` is its buffered curve (`Some`
/// == this track owns the A/B buffer, which shows the Swap chip and the ghost).
fn snap(ghost: Option<Vec<KeyView>>) -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        tracks: vec![TrackView {
            target: AnimTarget::new(TGT),
            prop: PropKind::TranslationX,
            entity: 1,
            missing: false,
            keys: vec![kv(1, 0.0, 0.0), kv(2, 1.0, 10.0)],
            buffer_ghost: ghost,
        }],
        ..TimelineViewSnapshot::default()
    }
}

/// Paint the panel with the track expanded (the graph band is where the chips are).
fn paint_expanded(
    host: &mut MockPanelHost,
    state: &mut TimelinePanelState,
    ghost: Option<Vec<KeyView>>,
) -> Vec<(ph2d_editor_core::ids::NodeId, Rect)> {
    set_current_timeline(Some(snap(ghost)));
    state.expanded = vec![TGT];
    host.paint::<TimelinePanel>(state, VIEWPORT)
}

fn rect_of(
    regs: &[(ph2d_editor_core::ids::NodeId, Rect)],
    id: ph2d_editor_core::ids::NodeId,
) -> Option<Rect> {
    regs.iter().find(|(w, _)| *w == id).map(|(_, r)| *r)
}

/// A real Primary Down+Up on `(x, y)` — the surface capture pushes a Begin then a
/// Click gesture; the following paint drains it.
fn tap(host: &mut MockPanelHost, x: f32, y: f32) {
    let ev = |kind, t| PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: t,
    };
    let _ = host.dispatch_pointer_event(ev(PointerKind::Down, 0));
    let _ = host.dispatch_pointer_event(ev(PointerKind::Up, 1));
}

/// Store is painted, hit-registered, and a click on it raises `StoreTrackBuffer`
/// for the track — end to end. (Mutation: drop the `register`/`hit_index` in
/// `paint_buffer_chip` -> the chip is invisible to the hit index -> the `expect`
/// fires. Drop the `GraphBufferButton` route arm -> the intent list is empty.)
#[test]
fn the_store_chip_is_painted_and_clicking_it_stores() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let regs = paint_expanded(&mut host, &mut state, None);
    let r = rect_of(&regs, timeline_buffer_button_id(TGT, 0))
        .expect("the Store chip was painted but never hit-registered (dead under the mouse)");
    let _ = drain_intents();

    tap(&mut host, r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = host.paint::<TimelinePanel>(&mut state, VIEWPORT); // drains the gesture

    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::StoreTrackBuffer {
            target: AnimTarget::new(TGT)
        }],
        "a click on Store must raise StoreTrackBuffer for this track"
    );
    set_current_timeline(None);
}

/// Swap exists ONLY on the track that owns the buffer — presence AND absence, the
/// two halves in one gate. (Mutation: paint Swap unconditionally -> the absence
/// half fails; gate Store on `buffer_ghost` -> the presence baseline breaks.)
#[test]
fn the_swap_chip_appears_only_when_the_track_owns_the_buffer() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let swap = timeline_buffer_button_id(TGT, 1);

    let regs = paint_expanded(&mut host, &mut state, None);
    assert!(
        rect_of(&regs, swap).is_none(),
        "no buffer -> no Swap chip (a Swap here would be a no-op under the mouse)"
    );

    let regs = paint_expanded(
        &mut host,
        &mut state,
        Some(vec![kv(1, 0.0, 5.0), kv(2, 1.0, 5.0)]),
    );
    assert!(
        rect_of(&regs, swap).is_some(),
        "the buffer-owning track must show the Swap chip"
    );
    set_current_timeline(None);
}

/// A click on Swap raises `SwapTrackBuffer` — the A/B flip, end to end. (Mutation:
/// route Swap to `StoreTrackBuffer` -> the intent is wrong -> RED.)
#[test]
fn clicking_the_swap_chip_swaps() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let regs = paint_expanded(
        &mut host,
        &mut state,
        Some(vec![kv(1, 0.0, 5.0), kv(2, 1.0, 5.0)]),
    );
    let r = rect_of(&regs, timeline_buffer_button_id(TGT, 1))
        .expect("the Swap chip was painted but never hit-registered");
    let _ = drain_intents();

    tap(&mut host, r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = host.paint::<TimelinePanel>(&mut state, VIEWPORT);

    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SwapTrackBuffer {
            target: AnimTarget::new(TGT)
        }],
        "a click on Swap must raise SwapTrackBuffer (the A/B toggle)"
    );
    set_current_timeline(None);
}

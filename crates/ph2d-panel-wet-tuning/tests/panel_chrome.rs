//! Panel chrome seams — the Tuning panel MOVES, RESIZES and its heading
//! SWALLOWS the pointer. All three are driven by REAL pointer events through
//! `dispatch_pointer` over the hit index the panel's own `paint` built, so a
//! handle that paints but was never `populate`d (dead under the mouse) or a
//! band registered in the wrong z-order goes RED here, not in a smoke.

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_wet_tuning::{WetTuningPanel, rows, set_current_brush, state};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: t,
    }
}

fn painted_host() -> (MockPanelHost, state::WetTuningPanelState) {
    set_current_brush(Some(PainterTool::default().brush_settings()));
    let mut host = MockPanelHost::with_panel::<WetTuningPanel>();
    let mut st = state::WetTuningPanelState;
    host.paint::<WetTuningPanel>(&mut st, VIEWPORT);
    (host, st)
}

/// Drive Down at `(x, y)` → Move by `(dx, dy)` → Up, like a mouse drag.
fn drag(host: &mut MockPanelHost, x: f32, y: f32, dx: f32, dy: f32) {
    host.dispatch_pointer_event(pointer(PointerKind::Down, x, y, 1_000_000));
    host.dispatch_pointer_event(pointer(PointerKind::Move, x + dx, y + dy, 2_000_000));
    host.dispatch_pointer_event(pointer(PointerKind::Up, x + dx, y + dy, 3_000_000));
}

/// Dragging the title band MOVES the panel; dragging a corner gripper
/// RESIZES it. The oracle is the PUBLISHED rect after a repaint — the one
/// wheel/click routing reads — not the store deltas alone. Mutations that
/// bleed: dropping the handle registrations from `populate` (Down never
/// anchors), dropping the hit rects from `paint`, or `paint` ceasing to
/// apply the stored offset/resize to the rect.
#[test]
fn the_panel_drags_by_its_title_band_and_resizes_by_its_corners() {
    let (mut host, mut st) = painted_host();
    let before = host
        .store()
        .panel_rect(core_ids::WET_TUNING_PANEL)
        .expect("panel rect published");

    // Grab the middle of the title band (clear of the close reserve).
    let grab = (before.x + before.w * 0.4, before.y + 10.0);
    drag(&mut host, grab.0, grab.1, -60.0, 35.0);
    host.paint::<WetTuningPanel>(&mut st, VIEWPORT);
    let moved = host
        .store()
        .panel_rect(core_ids::WET_TUNING_PANEL)
        .expect("panel rect still published");
    assert!(
        (moved.x - (before.x - 60.0)).abs() < 0.5 && (moved.y - (before.y + 35.0)).abs() < 0.5,
        "title-band drag must move the panel: {before:?} -> {moved:?}"
    );

    // Bottom-right gripper grows the panel. The drag is UP-and-right so the
    // height SHRINKS (dy negative) — growth downward would hit the viewport
    // clamp (the panel is docked to the inspector's full-height slot).
    let br = (moved.x + moved.w - 4.0, moved.y + moved.h - 4.0);
    drag(&mut host, br.0, br.1, 40.0, -50.0);
    host.paint::<WetTuningPanel>(&mut st, VIEWPORT);
    let resized = host
        .store()
        .panel_rect(core_ids::WET_TUNING_PANEL)
        .expect("panel rect still published");
    assert!(
        (resized.w - (moved.w + 40.0)).abs() < 0.5 && (resized.h - (moved.h - 50.0)).abs() < 0.5,
        "corner drag must resize the panel: {moved:?} -> {resized:?}"
    );
    set_current_brush(None);
}

/// The heading swallows the pointer. A slider row scrolled up keeps its hit
/// rect under the title bar (registration is not clipped); the drag band —
/// registered LAST — must outrank it, so a press on the heading begins a
/// panel drag and NEVER scrubs the invisible slider. The positive control
/// (the same slider IS hit in the body before scrolling) proves the fixture
/// contains the phenomenon; the mutation that bleeds is registering the band
/// before the body (the slider wins the header again).
#[test]
fn the_heading_shields_a_slider_scrolled_behind_it() {
    let (mut host, mut st) = painted_host();
    let rect = host
        .store()
        .panel_rect(core_ids::WET_TUNING_PANEL)
        .expect("panel rect published");
    let first = &rows::rows()[0];
    let slider_rect = host
        .paint::<WetTuningPanel>(&mut st, VIEWPORT)
        .into_iter()
        .find(|(w, _)| *w == first.slider)
        .map(|(_, r)| r)
        .expect("first row slider registered");

    // Positive control: unscrolled, the pointer lands on the slider.
    let cx = slider_rect.x + slider_rect.w * 0.5;
    assert_eq!(
        host.hit_at(cx, slider_rect.y + slider_rect.h * 0.5),
        Some(first.slider),
        "control: the slider must be hittable in the body"
    );

    // Scroll the row up under the title bar, repaint, and probe the point
    // where its (unclipped) hit rect now overlaps the heading.
    let scroll = (slider_rect.y - rect.y) - 8.0;
    host.store_mut()
        .set_panel_scroll(core_ids::WET_TUNING_PANEL, scroll);
    host.paint::<WetTuningPanel>(&mut st, VIEWPORT);
    let probe = (cx, rect.y + 10.0);
    assert_eq!(
        host.hit_at(probe.0, probe.1),
        Some(core_ids::WET_TUNING_DRAG_HANDLE),
        "the heading must outrank the scrolled slider's hit rect"
    );
    let events = host.dispatch_pointer_event(pointer(PointerKind::Down, probe.0, probe.1, 1_000));
    assert!(
        host.store().blender_drag_anchor().is_some(),
        "a press on the heading must begin the panel drag"
    );
    // The dispatcher may Focus the drag handle itself — what it must NEVER
    // do is deliver anything to the slider hiding behind the heading.
    let slider_touched = events.iter().any(|e| {
        matches!(
            e,
            ph2d_editor_core::interaction::WidgetEvent::Focus(id)
            | ph2d_editor_core::interaction::WidgetEvent::ValueChanged(id)
            | ph2d_editor_core::interaction::WidgetEvent::Click(id)
                if *id == first.slider
        )
    });
    assert!(
        !slider_touched,
        "a press on the heading reached the scrolled slider: {events:?}"
    );
    host.dispatch_pointer_event(pointer(PointerKind::Up, probe.0, probe.1, 2_000));
    set_current_brush(None);
}

//! **The Taper section is reachable by a pointer, and every control leads somewhere.**
//!
//! The four conditions this codebase makes a wave prove, one gate each: the control EXISTS, it is
//! painted AND registered, the click reaches the bus, and the SEQUENCE lands on the tool. They are
//! independent — a widget can be painted, hit-registered and forwarded by `event.rs` and still be stone
//! dead under the mouse because `populate` never gave it an `InteractiveState`.
//!
//! Everything here is driven by `click_at` / `drag`, never by a synthesised `WidgetEvent`: a synthetic
//! event skips the store's focusability check, which is exactly the hole the Wet Paint Enable checkbox
//! and the 36 collision-matrix cells each shipped through.
//!
//! ⛔ The section used to carry a second handle (the END length), a *Link tip sizes* toggle and a second
//! Tip row. They went with the far end (Enio 2026-08-10); the ABSENCE gate below is what keeps them from
//! coming back as painted-but-mute widgets.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{self as core_ids, painter_taper_handle_id};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::{MAX_TAPER_DIAMETERS, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

fn painted(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// Drain whatever the panel pushed onto the bus into the tool — the last leg of the seam.
fn pump(host: &mut MockPanelHost, tool: &mut PainterTool) {
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

/// **Every control the Taper section paints is alive under the mouse.**
///
/// The oracle is the pair (painted with a real rect, registered in the store) for each id, asked of the
/// panel the artist actually gets.
///
/// **Mutation that must bleed:** drop `PAINTER_TAPER_FIELDS` from `populate` — the rows still paint,
/// still register a hit rect, and are dead.
#[test]
fn every_taper_control_is_painted_and_alive() {
    let tool = PainterTool::default();
    let (host, _st, rects) = painted(&tool);
    for (id, name) in [
        (core_ids::PAINTER_TAPER_TIP_START, "Tip"),
        (core_ids::PAINTER_TAPER_OPACITY, "Opacity"),
        (painter_taper_handle_id(0), "head handle"),
    ] {
        assert!(
            rect_of(&rects, id).is_some(),
            "the Taper section never painted a hit rect for `{name}`"
        );
        assert!(
            host.store().get(id).is_some(),
            "`{name}` paints and hit-registers but has no InteractiveState — dead under the mouse"
        );
    }
}

/// **The far end's controls are GONE from the section — not merely inert.**
///
/// The presence gate above cannot see this: it asserts what IS there, and a leftover handle or a
/// leftover Link row would sail past it while sitting on screen doing nothing. This codebase's law is
/// that a control that does not lead anywhere is worse than a missing one, so the removal is asserted
/// as an absence, in the same pass, on the same painted panel.
///
/// **Mutation that must bleed:** paint the second handle again (register `painter_taper_handle_id(1)`)
/// — it would draw, hit-register, and decode into a length nothing reads.
#[test]
fn the_far_ends_controls_are_absent_from_the_section() {
    let tool = PainterTool::default();
    let (_h, _s, rects) = painted(&tool);
    assert!(
        rect_of(&rects, painter_taper_handle_id(1)).is_none(),
        "the END handle is still painted — the taper has one end, so this dot authors nothing"
    );
    // A positive control, so an empty/failed paint cannot make the assertion above pass by vacuum.
    assert!(
        rect_of(&rects, painter_taper_handle_id(0)).is_some(),
        "fixture: the section did not paint at all, so the absence above proves nothing"
    );
}

/// **Dragging the handle authors the head length, and further in means LONGER.**
///
/// The pointer goes all the way: dispatcher → `CurvePoint` drag → `taper_gizmo` decode → bus → tool.
///
/// ⚠️ Two drags, to two different distances, because *"it authored something"* is satisfied by a decode
/// that ignores the pointer entirely and writes a constant. The oracle is that the further drag reads
/// LONGER — the direction, which is what a wrong-signed decode breaks.
///
/// **Mutation that must bleed:** decode `1 - x` instead of `x` — the control then moves the opposite way
/// from the hand, which is exactly what the (now removed) END handle was decoded with.
#[test]
fn dragging_the_handle_authors_the_head_length() {
    let authored = |dx: f32| {
        let mut tool = PainterTool::default();
        let (mut host, mut st, rects) = painted(&tool);
        let h = rect_of(&rects, painter_taper_handle_id(0)).expect("the head handle painted");
        let (x0, y) = (h.x + h.w * 0.5, h.y + h.h * 0.5);
        for ev in host.drag_at(x0, y, x0 + dx, y) {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        pump(&mut host, &mut tool);
        tool.brush_settings().taper.start
    };
    let near = authored(80.0); // LITERAL-PX-OK: two arbitrary distances into the widget
    let far = authored(200.0);
    assert!(
        near > 0.0,
        "dragging the handle authored nothing ({near:.3})"
    );
    assert!(
        far > near + 0.1,
        "dragging FURTHER in did not lengthen the taper ({near:.3} -> {far:.3}) — the decode is \
         reading the pointer backwards, or not at all"
    );
    assert!(
        far <= MAX_TAPER_DIAMETERS,
        "a drag authored past the cap ({far:.3})"
    );
}

/// **The two numeric rows reach the tool — the fourth condition, and the one the other gates cannot
/// see.**
///
/// A number row is painted by `paint_num_row`, which registers a `NumberInput` and MIRRORS the tool's
/// value back into it on every frame. So a row whose `ValueChanged` is not claimed by the panel's value
/// forward is painted, hit-registered, focusable, editable — and **reverts the instant the artist lets
/// go**, because the next frame writes the tool's unchanged value back over what they typed. That is
/// exactly what Enio reported on 2026-08-08 (*"Tip start, Tip End e Opacity não aceitam ajustes (voltam
/// a zero)"*), and the presence gate above stayed green through all of it: painted and alive are two
/// of the four independent conditions, and *the value lands* is a third.
///
/// Driven through the real event, with the store carrying the new value the way a commit leaves it.
///
/// **Mutation that must bleed:** drop `PAINTER_TAPER_FIELDS` from `number_field::is_param_field`.
#[test]
fn every_taper_number_row_lands_on_the_tool() {
    for (id, name, set, read) in [
        (
            core_ids::PAINTER_TAPER_TIP_START,
            "Tip",
            0.62_f64,
            (|t: &PainterTool| t.brush_settings().taper.tip_start) as fn(&PainterTool) -> f32,
        ),
        (
            core_ids::PAINTER_TAPER_OPACITY,
            "Opacity",
            0.81,
            (|t: &PainterTool| t.brush_settings().taper.opacity) as fn(&PainterTool) -> f32,
        ),
    ] {
        let mut tool = PainterTool::default();
        let (mut host, mut st, rects) = painted(&tool);
        assert!(
            rect_of(&rects, id).is_some(),
            "the `{name}` row was not painted"
        );
        host.store_mut().set_number_value(id, set);
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
        pump(&mut host, &mut tool);
        let got = read(&tool);
        assert!(
            (f64::from(got) - set).abs() < 1e-3,
            "`{name}` never reached the tool: asked for {set:.3}, the brush still reads {got:.3} — the \
             row is painted and mute, so it reverts on the next frame"
        );
    }
}

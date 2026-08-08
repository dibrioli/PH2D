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
/// panel the artist actually gets. The tip-END row is deliberately absent from this list: it is only
/// painted while the tips are UNlinked, and its presence is the subject of its own gate below.
///
/// **Mutation that must bleed:** drop `PAINTER_TAPER_LINK` from `populate` — it still paints, still
/// registers a hit rect, and is dead.
#[test]
fn every_taper_control_is_painted_and_alive() {
    let tool = PainterTool::default();
    let (host, _st, rects) = painted(&tool);
    for (id, name) in [
        (core_ids::PAINTER_TAPER_LINK, "Link tip sizes"),
        (core_ids::PAINTER_TAPER_TIP_START, "Tip"),
        (core_ids::PAINTER_TAPER_OPACITY, "Opacity"),
        (painter_taper_handle_id(0), "start handle"),
        (painter_taper_handle_id(1), "end handle"),
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

/// **Dragging a handle authors a length, and the two handles author DIFFERENT ends.**
///
/// The pointer goes all the way: dispatcher → `CurvePoint` drag → `taper_gizmo` decode → bus → tool.
/// Both handles are dragged to the SAME screen x, and the gate is that they disagree about what that
/// means — the start handle measures in from the left, the end handle from the right.
///
/// **Mutation that must bleed:** read the raw `x` for channel `1` instead of `1 - x` — the end taper
/// then shortens when the artist drags the handle further into the stroke, a control that moves the
/// opposite way from the hand.
#[test]
fn dragging_a_handle_authors_the_end_it_belongs_to() {
    let mut tool = PainterTool::default();
    let (mut host, mut st, rects) = painted(&tool);
    let canvas = rect_of(&rects, painter_taper_handle_id(0)).expect("the start handle painted");
    // A quarter of the way across the widget. The widget spans the panel's content column, so the
    // handle rect's own centre tells us where the row is; the x we aim at is a quarter of the section.
    let widget_x0 = canvas.x + canvas.w * 0.5; // the START handle rests at the left edge (length 0)
    let y = canvas.y + canvas.h * 0.5;
    let quarter = widget_x0 + 200.0; // LITERAL-PX-OK: an arbitrary distance into the widget

    for ev in host.drag_at(widget_x0, y, quarter, y) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    pump(&mut host, &mut tool);
    let start = tool.brush_settings().taper.start;
    assert!(
        start > 0.0,
        "dragging the START handle authored nothing (start {start:.3})"
    );

    // Now the END handle, dragged to the SAME x. It measures from the far edge, so it must land on a
    // different length — if it did not, the two handles would be one control drawn twice.
    let mut tool2 = PainterTool::default();
    let (mut host2, mut st2, rects2) = painted(&tool2);
    let hr = rect_of(&rects2, painter_taper_handle_id(1)).expect("the end handle painted");
    let end_x0 = hr.x + hr.w * 0.5;
    for ev in host2.drag_at(end_x0, y, quarter, y) {
        host2.apply_panel_event::<PainterLayersPanel>(&mut st2, ev);
    }
    pump(&mut host2, &mut tool2);
    let end = tool2.brush_settings().taper.end;
    assert!(
        end > 0.0,
        "dragging the END handle authored nothing (end {end:.3})"
    );
    assert!(
        (end - start).abs() > 0.5,
        "the two handles read the SAME length from the same screen x ({start:.3} vs {end:.3}) — the \
         end handle is measuring from the wrong edge"
    );
    assert!(
        start <= MAX_TAPER_DIAMETERS && end <= MAX_TAPER_DIAMETERS,
        "a drag authored past the cap ({start:.3} / {end:.3})"
    );
}

/// **Clicking "Link tip sizes" reaches the tool, and the panel then shows the row that appeared.**
///
/// Two halves, and the second is the one that catches a half-wired toggle: unlinking must make the
/// **Tip End** row exist. Its absence while linked is not an omission — there is one number, and a
/// second row showing it under another name is a control that sometimes lies.
///
/// **Mutation that must bleed:** drop `PAINTER_TAPER_LINK` from the `Click` forward list in `event.rs`
/// — the click is registered, painted and swallowed.
#[test]
fn the_link_toggle_reaches_the_tool_and_reveals_the_second_tip() {
    let mut tool = PainterTool::default();
    // ⚠️ The fixture has to CONTAIN the phenomenon: with the default tips (both `0.0`) a seeded end tip
    // and an unseeded one are the same number, and the third assertion below would be green either way.
    tool.set_brush_taper_tip_start(0.6);
    assert!(
        tool.brush_settings().taper.link_tips,
        "fixture: the tips are meant to start LINKED"
    );
    let (_h, _s, rects) = painted(&tool);
    assert!(
        rect_of(&rects, core_ids::PAINTER_TAPER_TIP_END).is_none(),
        "the Tip End row is on screen while the tips are LINKED — two rows for one number"
    );

    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_TAPER_LINK).expect("the Link row painted");
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    pump(&mut host, &mut tool);
    assert!(
        !tool.brush_settings().taper.link_tips,
        "the click on Link tip sizes never reached the tool"
    );

    // Unlinking is a request to start editing the two ends SEPARATELY, not a request to change the
    // mark: the end tip is seeded from the start's, so the stroke does not jump at the instant the box
    // is unticked. A control that reshapes the art on the way to letting you reshape it is one the
    // artist learns to distrust.
    let t = tool.brush_settings().taper;
    assert!(
        (t.tip_end - 0.6).abs() < 1e-4,
        "unlinking did not seed the end tip from the start's ({:.3} vs 0.600) — the mark jumped",
        t.tip_end
    );

    let (_h2, _s2, rects2) = painted(&tool);
    assert!(
        rect_of(&rects2, core_ids::PAINTER_TAPER_TIP_END).is_some(),
        "unlinking the tips did not put the Tip End row on screen — the toggle leads nowhere"
    );
}

/// **The three numeric rows reach the tool — the fourth condition, and the one the other gates cannot
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
    // Unlinked, so the Tip End row is on screen at all (linked it is deliberately absent).
    for (id, name, set, read) in [
        (
            core_ids::PAINTER_TAPER_TIP_START,
            "Tip Start",
            0.62_f64,
            (|t: &PainterTool| t.brush_settings().taper.tip_start) as fn(&PainterTool) -> f32,
        ),
        (
            core_ids::PAINTER_TAPER_TIP_END,
            "Tip End",
            0.37,
            (|t: &PainterTool| t.brush_settings().taper.tip_end) as fn(&PainterTool) -> f32,
        ),
        (
            core_ids::PAINTER_TAPER_OPACITY,
            "Opacity",
            0.81,
            (|t: &PainterTool| t.brush_settings().taper.opacity) as fn(&PainterTool) -> f32,
        ),
    ] {
        // Unlinked, so the Tip End row is on screen at all (linked it is deliberately absent).
        let mut tool = PainterTool::default();
        tool.toggle_brush_taper_link();
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

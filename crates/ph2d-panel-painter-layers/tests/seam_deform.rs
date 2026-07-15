//! **The Deform panel's Affect Relief toggle, driven by a POINTER** — W4's seam gate.
//!
//! Same law as `seam_sculpt.rs`, and the same scar behind it: a widget that paints, hit-indexes and even
//! forwards is still stone dead unless `populate` gave it an `InteractiveState` — and one registered as a
//! `Checkbox` emits `Toggled`, which `event.rs` does not forward: registered, and STILL dead. So this
//! starts at the pixel the artist aims at and ends at the tool's state, through every real link.
//!
//! **A widget is not done when it PAINTS. It is done when a test CLICKS it.**

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A `PainterTool` whose active layer carries REAL relief (one impasto stroke through the public API),
/// parked in Deform · Reshape — the only state in which the Affect Relief row exists. The row is gated on
/// `deform_layer_has_relief`, and that gate is honest ("a toggle over a plane that does not exist is a
/// control that silently does nothing"), so the fixture must EARN the row the way the artist does.
fn tool_in_deform_with_relief() -> PainterTool {
    let size = 128u32;
    let mut tool = PainterTool::default();
    tool.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    tool.toggle_brush_impasto(); // impasto ON for the deposit brush
    tool.set_brush_impasto_depth(1.0);
    tool.on_canvas_pointer(cp([40.0, 64.0], PointerPhase::Down));
    tool.on_canvas_pointer(cp([80.0, 64.0], PointerPhase::Move));
    tool.on_canvas_pointer(cp([80.0, 64.0], PointerPhase::Up));

    tool.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "deform".to_string(),
    ));
    tool.set_deform_temperament(1); // Reshape — the brush temperament, where the row lives

    let bs = tool.brush_settings();
    assert!(bs.is_deform, "fixture: the tool is not in Deform");
    assert!(
        bs.deform_layer_has_relief,
        "fixture: the deposit laid no relief, so the Affect Relief row is never painted and nothing \
         below could mean anything"
    );
    set_current_brush(Some(bs));
    tool
}

fn click_through(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    x: f32,
    y: f32,
) {
    for ev in host.click_at(x, y) {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

/// **Clicking Affect Relief flips it — and clicking again flips it back.**
///
/// **Mutations that must bleed** (each a different dead link): drop `PAINTER_DEFORM_RELIEF` from
/// `populate.rs` (painted, hit-indexed, and stone dead — `is_focusable` answers `None => false`); drop it
/// from `is_deform_click` (activated but never forwarded); drop the `route_deform_event` arm (forwarded
/// and never applied).
#[test]
fn clicking_affect_relief_flips_the_toggle() {
    let mut tool = tool_in_deform_with_relief();
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;

    assert!(
        tool.brush_settings().deform_affect_relief,
        "fixture: Affect Relief defaults ON (paint is a substance — the body rides by default)"
    );

    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let Some((_, rect)) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_DEFORM_RELIEF && r.w > 0.0 && r.h > 0.0)
        .copied()
    else {
        panic!(
            "the Affect Relief row is not painted with a clickable rect — the Deform card never drew it \
             (is `deform_layer_has_relief` reaching the panel snapshot?)"
        );
    };

    let (x, y) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert_eq!(
        host.hit_at(x, y),
        Some(core_ids::PAINTER_DEFORM_RELIEF),
        "the pixel at the centre of the Affect Relief row does not resolve to it — something painted \
         after it covers its rect"
    );

    click_through(&mut host, &mut st, &mut tool, x, y);
    assert!(
        !tool.brush_settings().deform_affect_relief,
        "the click never reached the tool — the row paints but the seam has a dead link (populate / \
         forward / route)"
    );

    // The row must still be there (state repaints), and a second click restores the default.
    set_current_brush(Some(tool.brush_settings()));
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let (_, rect) = painted
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_DEFORM_RELIEF && r.w > 0.0 && r.h > 0.0)
        .copied()
        .expect("the row vanished after one click");
    let (x, y) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    click_through(&mut host, &mut st, &mut tool, x, y);
    assert!(
        tool.brush_settings().deform_affect_relief,
        "the second click did not flip it back — the toggle is a one-way street"
    );
}

/// **On a relief-bare layer the row does not exist at all** — the honest absence (never a control that
/// silently does nothing), and the presence sibling of the gate above.
///
/// **Mutation that must bleed:** paint the row unconditionally (drop the `deform_layer_has_relief` guard
/// in `paint_deform.rs`) — a bare document shows a toggle over a plane that does not exist.
#[test]
fn the_row_does_not_exist_on_a_layer_with_no_relief() {
    let size = 96u32;
    let mut tool = PainterTool::default();
    tool.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    tool.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "deform".to_string(),
    ));
    tool.set_deform_temperament(1);
    let bs = tool.brush_settings();
    assert!(bs.is_deform && !bs.deform_layer_has_relief);
    set_current_brush(Some(bs));

    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());
    assert!(
        !painted
            .iter()
            .any(|(w, r)| *w == core_ids::PAINTER_DEFORM_RELIEF && r.w > 0.0 && r.h > 0.0),
        "the Affect Relief row is painted on a layer with NO relief — a toggle over a plane that does \
         not exist, i.e. a control that silently does nothing"
    );
}

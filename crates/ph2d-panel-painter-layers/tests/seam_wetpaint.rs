//! **Wet Paint presence seams** — what the panel OFFERS in and around the wet mode.
//!
//! Two facts the 2026-07-21 smoke tripped on: the Enable checkbox has to be painted BOTH for the
//! plain brush (to arm) and inside the wet mode (to disarm — without it the artist cannot leave);
//! and the **Paper** section has to be offered in wet mode (W2.7 seeds the engine's tooth from the
//! Paper slot — with the section hidden there is no door to arm a paper, and the seam is
//! unreachable by hand). Presence AND absence are asserted: the plain brush shows no Paper (it
//! reads no substrate — Enio: "deve ser assim mesmo").

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

fn painted(tool: &PainterTool) -> Vec<(NodeId, Rect)> {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    host.paint::<PainterLayersPanel>(&mut st, viewport())
}

fn has(rects: &[(NodeId, Rect)], id: NodeId) -> bool {
    rects
        .iter()
        .any(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
}

/// The Enable checkbox exists on BOTH sides of the arm — offered to check
/// (plain brush) and to UNCHECK (wet mode; hiding it there is "não consigo
/// sair do modo wet" with different clothes). Mutation that bleeds it: the
/// section gated on `!brush.wetpaint` (or on the mode) instead of always.
#[test]
fn the_enable_checkbox_is_offered_to_arm_and_to_disarm() {
    let plain = PainterTool::default();
    assert!(
        has(&painted(&plain), core_ids::PAINTER_WETPAINT_ENABLE),
        "the plain brush has no Wet Paint checkbox to ARM"
    );
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    assert!(
        has(&painted(&wet), core_ids::PAINTER_WETPAINT_ENABLE),
        "the wet mode has no checkbox to DISARM — the artist cannot leave"
    );
}

/// W2.7's door: the **Paper** section is offered in wet mode (the slot seeds
/// the engine's tooth) and stays hidden for the plain brush (no substrate to
/// read). Mutation that bleeds it: `|| brush.wetpaint` dropped from the
/// Paper gate in `paint_brush_sections`.
#[test]
fn the_paper_section_is_offered_in_wet_mode_and_hidden_for_the_plain_brush() {
    let plain = PainterTool::default();
    assert!(
        !has(&painted(&plain), core_ids::PAINTER_WATERCOLOR_PAPER_SECTION),
        "the plain brush must NOT offer Paper (deve ser assim mesmo — Enio)"
    );
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    assert!(
        has(&painted(&wet), core_ids::PAINTER_WATERCOLOR_PAPER_SECTION),
        "wet mode offers no Paper section — the W2.7 seam is unreachable by hand"
    );
}

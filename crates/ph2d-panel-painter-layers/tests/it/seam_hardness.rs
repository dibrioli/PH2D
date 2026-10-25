//! **A Hardness pelo ponteiro** (dono 24/10): a linha pinta-se logo abaixo do Falloff e digitar nela muda o pincel do
//! tool (o Glaze Round de fábrica tinha a dureza 0,8 sem linha nenhuma no painel).

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::ids::{PAINTER_BRUSH_FALLOFF, PAINTER_BRUSH_HARDNESS};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

fn painted(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    host.open_all_sections();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, Rect::new(0.0, 0.0, 1600.0, 4000.0));
    (host, st, rects)
}

fn y(rects: &[(NodeId, Rect)], id: NodeId) -> Option<f32> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| r.y)
}

#[test]
fn a_hardness_fica_abaixo_do_falloff_e_muda_o_pincel() {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    let (mut host, mut st, rects) = painted(&t);
    let (falloff, dureza) = (
        y(&rects, PAINTER_BRUSH_FALLOFF).expect("o Falloff pintado"),
        y(&rects, PAINTER_BRUSH_HARDNESS).expect("a Hardness pintada"),
    );
    assert!(dureza > falloff, "a Hardness não está abaixo do Falloff");
    let entre = rects
        .iter()
        .filter(|(_, r)| r.h > 0.0 && r.y > falloff + 0.5 && r.y < dureza - 0.5)
        .count();
    assert_eq!(entre, 0, "linhas entre o Falloff e a Hardness");
    for ev in host.type_into_number(PAINTER_BRUSH_HARDNESS, "0.4") {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            t.handle_panel_event(pe);
        }
    }
    assert!(
        (t.brush_settings().hardness - 0.4).abs() < 1e-6,
        "digitar não mudou a dureza"
    );
}

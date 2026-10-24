//! **O Flow pelo ponteiro** (dono 24/10): a linha Flow logo abaixo da Strength, onde a ferramenta e o meio o lêem
//! (`BrushSettings::le_o_flow`); o arrasto do slider chega ao `BrushSettings::flow` do tool.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::{PaintMedia, PainterTool, ids};
use ph2d_ui_testkit::MockPanelHost;

fn pintado(
    tool: &PainterTool,
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
) -> Vec<(NodeId, Rect)> {
    set_current_brush(Some(tool.brush_settings()));
    host.paint::<PainterLayersPanel>(st, Rect::new(0.0, 0.0, 1600.0, 6000.0))
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

fn com(mode: &str, media: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_tool_mode(mode);
    t.set_paint_media(media);
    t
}

/// ⭐ Presença E ausência: o par é o que prova a guarda. A tabela medida (meio × ferramenta, Flow 1 → 0,05) é a de
/// `o_flow_a_vista_e_o_flow_que_age`; aqui, pela porta do painel, o lugar e um representante de cada lado.
#[test]
fn o_flow_pinta_onde_a_ferramenta_e_o_meio_o_leem() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let r = pintado(&com("brush", PaintMedia::Digital), &mut host, &mut st);
    let flow = rect_of(&r, ids::PAINTER_BRUSH_FLOW_SLIDER).expect("Flow no Pincel Digital");
    let forca =
        rect_of(&r, ids::PAINTER_BRUSH_STRENGTH_SLIDER).expect("Strength no Pincel Digital");
    assert!(flow.y > forca.y, "o Flow vem logo depois da Strength");
    for (mode, media, a_vista) in [
        ("eraser", PaintMedia::Digital, true),
        ("mask", PaintMedia::Digital, true),
        ("brush", PaintMedia::Impasto, true),
        ("blur", PaintMedia::Digital, false),
        ("smear", PaintMedia::Digital, false),
        ("clone", PaintMedia::Digital, false),
        ("brush", PaintMedia::Watercolor, false),
        ("brush", PaintMedia::WetPaint, false),
    ] {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let r = pintado(&com(mode, media), &mut host, &mut st);
        assert_eq!(
            rect_of(&r, ids::PAINTER_BRUSH_FLOW_SLIDER).is_some(),
            a_vista,
            "{mode}/{media:?}: o Flow à vista tem de ser {a_vista}"
        );
    }
}

#[test]
fn arrastar_o_flow_muda_o_flow_do_tool() {
    let mut tool = com("brush", PaintMedia::Digital);
    let antes = tool.brush_settings().flow;
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let r = pintado(&tool, &mut host, &mut st);
    let s = rect_of(&r, ids::PAINTER_BRUSH_FLOW_SLIDER).expect("Flow pintado");
    let y = s.y + s.h * 0.5;
    for ev in host.drag_at(s.x + s.w * 0.9, y, s.x + s.w * 0.2, y) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for a in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = a {
            tool.handle_panel_event(pe);
        }
    }
    let depois = tool.brush_settings().flow;
    assert!(
        depois < 0.5 && depois < antes,
        "o arrasto para a esquerda baixa o Flow: {antes} → {depois}"
    );
}

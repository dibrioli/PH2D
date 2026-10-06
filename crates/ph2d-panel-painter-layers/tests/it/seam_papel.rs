//! **O botão «Apply to Paper» nos quatro meios** (pedido do dono, 2026-10-05) — do píxel clicado no
//! painel ao papel do documento: o dispatcher, o painel, o barramento e a ferramenta.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_brush, set_current_dock_shows_layers,
};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::ids::{
    PAINTER_BRUSH_MEDIA, PAINTER_PAPER_APPLY, PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
};
use ph2d_ui_testkit::MockPanelHost;

#[test]
fn o_botao_aplicar_no_papel_aplica_o_papel_nos_quatro_meios() {
    for meio in ["0", "1", "2", "3"] {
        let mut tool = PainterTool::default();
        tool.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        tool.handle_panel_event(PanelEvent::SelectOption(PAINTER_BRUSH_MEDIA, meio.into()));
        tool.handle_panel_event(PanelEvent::SelectOption(
            PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
            "230,200,150".into(),
        ));
        set_current_brush(Some(tool.brush_settings()));
        set_current_dock_shows_layers(false);
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let vp = Rect::new(0.0, 0.0, 1600.0, 6000.0);
        let _ = host.paint::<PainterLayersPanel>(&mut st, vp);
        host.open_all_sections();
        let pintados = host.paint::<PainterLayersPanel>(&mut st, vp);
        let cor = pintados
            .iter()
            .any(|(id, r)| *id == PAINTER_WATERCOLOR_PAPER_COLOR_THUMB && r.w > 0.0);
        assert!(cor, "meio {meio}: a cor do papel não é pintada");
        let Some((_, r)) = pintados
            .iter()
            .find(|(id, r)| *id == PAINTER_PAPER_APPLY && r.w > 0.0 && r.h > 0.0)
            .copied()
        else {
            panic!("meio {meio}: o botão Apply to Paper não é pintado");
        };
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(
            host.hit_at(x, y),
            Some(PAINTER_PAPER_APPLY),
            "meio {meio}: outro widget cobre o botão"
        );
        for ev in host.click_at(x, y) {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        for action in host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = action {
                tool.handle_panel_event(pe);
            }
        }
        assert_eq!(
            tool.papel(),
            Some([230, 200, 150]),
            "meio {meio}: o clique real no botão não aplicou o papel"
        );
    }
}

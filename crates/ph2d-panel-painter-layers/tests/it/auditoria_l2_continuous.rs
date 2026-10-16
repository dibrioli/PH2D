//! AUDITORIA lente 2 (costura), 15/10 — o checkbox Continuous pelo PONTEIRO em cada meio: pintado,
//! registado, sem dica de inerte com Spacing 0,25, e o clique chega ao `continuous_spacing` do pincel vivo
//! (e volta pintado). Sonda, não portão: `#[ignore]`.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_brush, set_current_dock_shows_layers,
};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

const ID: NodeId = ph2d_tool_painter::ids::PAINTER_BRUSH_CONTINUOUS_SPACING;

fn pinta(t: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(t.brush_settings()));
    set_current_dock_shows_layers(false);
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let vp = Rect::new(0.0, 0.0, 1600.0, 6000.0);
    let _ = host.paint::<PainterLayersPanel>(&mut st, vp);
    host.open_all_sections();
    let rects = host.paint::<PainterLayersPanel>(&mut st, vp);
    (host, st, rects)
}

fn clica(t: &mut PainterTool) -> Vec<String> {
    let (mut host, mut st, rects) = pinta(t);
    let Some(r) = rects
        .iter()
        .find(|(w, r)| *w == ID && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
    else {
        return vec!["NÃO PINTADO".into()];
    };
    let dica = host.store().tooltip_for(ID).is_some();
    let mut out = vec![format!("rect {:.0}x{:.0} dica_inerte={dica}", r.w, r.h)];
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for a in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = a {
            out.push(format!("{pe:?}"));
            t.handle_panel_event(pe);
        }
    }
    out
}

#[test]
#[ignore = "sonda de auditoria (lente 2)"]
fn auditoria_l2_continuous_pelo_ponteiro_em_cada_meio() {
    let mut linhas = Vec::new();
    for media in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_media(media);
        t.handle_panel_event(PanelEvent::SetValue(
            ph2d_tool_painter::ids::PAINTER_BRUSH_SPACING,
            0.25,
        ));
        let antes = t.brush_settings().continuous_spacing;
        let ida = clica(&mut t);
        let meio = t.brush_settings().continuous_spacing;
        let volta = clica(&mut t);
        let fim = t.brush_settings().continuous_spacing;
        linhas.push(format!(
            "{media:?}: antes={antes} → clique → {meio} → clique → {fim} | ida {ida:?} | volta {volta:?}"
        ));
    }
    let s = linhas.join("\n");
    println!("{s}");
    if let Ok(f) = std::env::var("PH2D_AUD_OUT") {
        std::fs::write(f, &s).expect("grava");
    }
}

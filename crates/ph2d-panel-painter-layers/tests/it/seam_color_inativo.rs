//! **Bug #51 (c): em «Per-Layer Color» o Color fica INATIVO — visível, esmaecido e sem clique.** Cada
//! camada é dona da sua cor e nenhuma lê o Color; a ferramenta di-lo no instantâneo (`color_inactive`,
//! a lei e o seu portão moram no `textura_tinge` da `ph2d-tool-painter`) e o painel obedece.
//!
//! Dirigido por PONTEIRO: um controlo inativo não é o que não se PINTA, é o que um clique não alcança.

use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

const COLOR: NodeId = ph2d_editor_core::ids::PAINTER_COLOR_THUMB;

fn painted(inativo: bool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    let mut b = PainterTool::default().brush_settings();
    b.color_inactive = inativo;
    set_current_brush(Some(b));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    host.open_all_sections();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, Rect::new(0.0, 0.0, 1600.0, 4000.0));
    (host, st, rects)
}

#[test]
fn o_color_inativo_ocupa_a_linha_e_nao_aceita_clique() {
    let (mut host_vivo, mut st_vivo, vivo) = painted(false);
    let (mut host, mut st, inativo) = painted(true);
    let r = vivo
        .iter()
        .find(|(id, r)| *id == COLOR && r.w > 0.0)
        .map(|(_, r)| *r)
        .expect("CONTROLO: o Color activo é um alvo de clique");
    assert!(
        !inativo.iter().any(|(id, _)| *id == COLOR),
        "o Color inativo não se regista como alvo"
    );
    // A linha continua lá: tudo o que se pinta depois dela fica no mesmo sítio.
    let sem_color = |v: &[(NodeId, Rect)]| -> Vec<(NodeId, Rect)> {
        v.iter().filter(|(id, _)| *id != COLOR).copied().collect()
    };
    assert_eq!(
        sem_color(&vivo),
        sem_color(&inativo),
        "o layout não se mexe"
    );
    // CONTROLO: no Color activo o mesmo clique abre o selector.
    for ev in host_vivo.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host_vivo.apply_panel_event::<PainterLayersPanel>(&mut st_vivo, ev);
    }
    assert_eq!(
        host_vivo.store().picker_target(),
        Some(COLOR),
        "CONTROLO: o clique abre o selector"
    );
    // E o clique onde ela está não abre o selector.
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    assert_ne!(
        host.store().picker_target(),
        Some(COLOR),
        "o clique no Color inativo não abre o selector"
    );
}

/// O Randomize Color «a cada traço»: o segmento pinta-se na secção e o clique chega ao pincel.
#[test]
fn o_segmento_a_cada_traco_chega_ao_pincel() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::tool::Tool;
    let mut tool = PainterTool::default();
    assert_eq!(
        tool.brush_settings().color_jitter_scope,
        0,
        "nasce «a cada carimbo»"
    );
    let (mut host, mut st, rects) = painted(false);
    let r = rects
        .iter()
        .find(|(id, r)| {
            *id == ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_PER_STROKE && r.w > 0.0
        })
        .map(|(_, r)| *r)
        .expect("o segmento Every Stroke é pintado");
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    assert_eq!(
        tool.brush_settings().color_jitter_scope,
        1,
        "o clique liga «a cada traço»"
    );
}

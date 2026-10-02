//! **As linhas da forma da borda** (BUGS_painter #31) — o Flow, o Size e o Angle dele, e o Paper Edge:
//! pintadas, registadas, alcançáveis, e no LUGAR delas dentro do cartão Wash.
//!
//! ⚠️ O cartão Wash passou a DELEGAR estas linhas a `paint_flow_rows`, e o censo
//! `o_numero_de_linhas_que_um_cartao_declara_e_o_que_ele_pinta` salta os cartões que delegam — a
//! contagem dele mora aqui, pela tela, nos dois estados (Classic: 2 linhas; padrão: 4).

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::ids::{
    PAINTER_WATERCOLOR_FLOW_ANGLE, PAINTER_WATERCOLOR_FLOW_KIND, PAINTER_WATERCOLOR_FLOW_SIZE,
    PAINTER_WATERCOLOR_PAPER_EDGE, PAINTER_WATERCOLOR_SMOOTH_EDGES, PAINTER_WATERCOLOR_WARP,
    painter_flow_kind_option_id,
};
use ph2d_tool_painter::{PaintMedia, PainterTool, TextureKind};
use ph2d_ui_testkit::MockPanelHost;

fn aquarela(flow: TextureKind) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_edge_flow_kind(flow.to_u8());
    t
}

fn painted(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    // O preview publicado como a ponte o publica — a tela tem de o pintar sem empurrar nada.
    let (lum, w, h) = tool.edge_flow_preview();
    ph2d_panel_painter_layers::set_current_brush_flow_preview(Some((
        std::sync::Arc::new(lum),
        w,
        h,
    )));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, Rect::new(0.0, 0.0, 1600.0, 900.0));
    (host, st, rects)
}

fn rect_de(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(i, r)| *i == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// ⭐ Com Classic aparecem o Flow e o Paper Edge; com um padrão, também o Size e o Angle — e as linhas
/// descem em ORDEM entre o Ragged Edge e o Smooth Edges, sem se sobreporem (o `card_frame` conta-as pelo
/// `flow_row_count`; uma conta errada empurra o Smooth Edges para fora do cartão ou por cima de uma delas).
#[test]
fn as_linhas_do_flow_aparecem_em_ordem_dentro_do_cartao() {
    for (flow, esperadas) in [
        (
            TextureKind::None,
            vec![PAINTER_WATERCOLOR_FLOW_KIND, PAINTER_WATERCOLOR_PAPER_EDGE],
        ),
        (
            TextureKind::Clouds,
            vec![
                PAINTER_WATERCOLOR_FLOW_KIND,
                PAINTER_WATERCOLOR_FLOW_SIZE,
                PAINTER_WATERCOLOR_FLOW_ANGLE,
                PAINTER_WATERCOLOR_PAPER_EDGE,
            ],
        ),
    ] {
        let (_, _, rects) = painted(&aquarela(flow));
        let ragged = rect_de(&rects, PAINTER_WATERCOLOR_WARP).expect("Ragged Edge pintado");
        let suave = rect_de(&rects, PAINTER_WATERCOLOR_SMOOTH_EDGES).expect("Smooth Edges pintado");
        let mut y = ragged.y;
        for id in &esperadas {
            let r = rect_de(&rects, *id)
                .unwrap_or_else(|| panic!("{flow:?}: a linha {id:?} não foi pintada"));
            assert!(
                r.y > y + 0.5,
                "{flow:?}: a linha {id:?} não desce depois da anterior"
            );
            y = r.y;
        }
        assert!(
            suave.y > y + 0.5,
            "{flow:?}: o Smooth Edges ficou por cima das linhas do Flow"
        );
        // A pré-visualização mora entre o menu e a linha seguinte: 3 fileiras de espaço reservado.
        let menu = rect_de(&rects, PAINTER_WATERCOLOR_FLOW_KIND).expect("Flow pintado");
        let seguinte = rect_de(&rects, esperadas[1]).expect("a linha depois do preview");
        let passo = ph2d_tokens::row_pitch_px();
        assert!(
            seguinte.y - menu.y >= 4.0 * passo - 0.5,
            "{flow:?}: o preview do Flow não tem as 3 fileiras dele ({} px entre o menu e a linha seguinte)",
            seguinte.y - menu.y
        );
        if flow == TextureKind::None {
            assert!(
                rect_de(&rects, PAINTER_WATERCOLOR_FLOW_SIZE).is_none(),
                "Classic não tem Size"
            );
            assert!(
                rect_de(&rects, PAINTER_WATERCOLOR_FLOW_ANGLE).is_none(),
                "Classic não tem Angle"
            );
        }
    }
}

/// O clique numa opção do Flow chega à ferramenta como `SelectOption(FLOW_KIND, k)`, e só as opções
/// oferecidas são decodificadas.
#[test]
fn a_opcao_do_flow_chega_a_ferramenta() {
    for &k in &ph2d_tool_painter::FLOW_KINDS {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let opt = painter_flow_kind_option_id(k.to_u8());
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(opt));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "{k:?}: a opção do Flow foi ignorada"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::SelectOption(i, v))
                    if *i == PAINTER_WATERCOLOR_FLOW_KIND && *v == k.to_u8().to_string()
            )),
            "{k:?}: a escolha do Flow nunca chegou à ferramenta — drained = {actions:?}"
        );
    }
}

/// Os três campos novos chegam à ferramenta pelo `SetValue` (o arrasto não morre dentro do painel).
#[test]
fn os_campos_do_flow_chegam_a_ferramenta() {
    let (mut host, mut st, _) = painted(&aquarela(TextureKind::Wood));
    for id in [
        PAINTER_WATERCOLOR_FLOW_SIZE,
        PAINTER_WATERCOLOR_FLOW_ANGLE,
        PAINTER_WATERCOLOR_PAPER_EDGE,
    ] {
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
        let chegou = host.drained_actions().iter().any(
            |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id),
        );
        assert!(
            outcome == EventOutcome::Consumed && chegou,
            "o campo {id:?} é pintado e não chega à ferramenta"
        );
    }
}

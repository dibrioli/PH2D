//! **As opções da W1g do Wet Paint pelo clique REAL** — *Live Water* (a caixa) e *Heavy Edge* (a linha
//! numérica): pintadas só com o meio armado, vivas sob o ponteiro, e o evento chega à ferramenta. A dica
//! do custo da Live Water está registada; a Heavy Edge esmaece com a dica do Tilt quando a tela está
//! direita (sem `|g|` ela não age).
//!
//! Mutações que sangram: o id fora de `PAINTER_WETPAINT_CLICKS`/`PAINTER_WETPAINT_FIELDS` (o clique morre
//! no painel), o braço da ferramenta em falta (o evento chega e nada muda), a dica fora do `populate`.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::ids::{PAINTER_WETPAINT_HEAVY_EDGE, PAINTER_WETPAINT_LIVE_WATER};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 4000.0)
}

fn molhado() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_wetpaint_armed(true);
    t
}

fn pinta(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

fn rect_de(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// Os eventos do painel → o barramento → a ferramenta, pela porta do contrato.
fn entrega(host: &mut MockPanelHost, tool: &mut PainterTool) {
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

#[test]
fn as_opcoes_da_w1g_so_aparecem_com_o_wet_paint_armado() {
    let (_, _, plano) = pinta(&PainterTool::default());
    let (_, _, armado) = pinta(&molhado());
    for id in [PAINTER_WETPAINT_LIVE_WATER, PAINTER_WETPAINT_HEAVY_EDGE] {
        assert!(
            rect_de(&armado, id).is_some(),
            "{id:?} não é pintado com o Wet Paint armado"
        );
        assert!(
            rect_de(&plano, id).is_none(),
            "{id:?} pintado para o pincel simples"
        );
    }
}

/// O clique na caixa **Live Water**, no pixel do centro, liga a opção na ferramenta — e o segundo
/// desliga-a (o controlo: o primeiro clique não é um «liga sempre»).
#[test]
fn o_clique_na_live_water_liga_e_desliga_a_agua_ao_vivo() {
    let mut tool = molhado();
    assert!(!tool.wet_agua_viva(), "a Live Water nasce desligada");
    for esperado in [true, false] {
        let (mut host, mut st, rects) = pinta(&tool);
        let r = rect_de(&rects, PAINTER_WETPAINT_LIVE_WATER).expect("Live Water pintada");
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(
            host.hit_at(cx, cy),
            Some(PAINTER_WETPAINT_LIVE_WATER),
            "a caixa Live Water não responde ao ponteiro"
        );
        for ev in host.click_at(cx, cy) {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        entrega(&mut host, &mut tool);
        assert_eq!(
            tool.wet_agua_viva(),
            esperado,
            "o clique na Live Water não chegou à ferramenta"
        );
    }
}

/// A edição da **Heavy Edge** vira `SetValue` no painel e a ferramenta guarda o valor (preso a `0..1`).
#[test]
fn a_edicao_da_heavy_edge_chega_a_ferramenta() {
    let mut tool = molhado();
    let (mut host, mut st, rects) = pinta(&tool);
    let r = rect_de(&rects, PAINTER_WETPAINT_HEAVY_EDGE).expect("Heavy Edge pintada");
    assert_eq!(
        host.hit_at(r.x + r.w * 0.5, r.y + r.h * 0.5),
        Some(PAINTER_WETPAINT_HEAVY_EDGE),
        "a linha Heavy Edge não responde ao ponteiro"
    );
    let outcome = host.apply_panel_event::<PainterLayersPanel>(
        &mut st,
        WidgetEvent::ValueChanged(PAINTER_WETPAINT_HEAVY_EDGE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "a Heavy Edge morreu no painel"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == PAINTER_WETPAINT_HEAVY_EDGE
        )),
        "a Heavy Edge nunca virou SetValue. drained = {actions:?}"
    );
    tool.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_HEAVY_EDGE, 0.6));
    assert!((tool.wet_borda_que_pesa() - 0.6).abs() < 1e-12);
    tool.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_HEAVY_EDGE, 7.0));
    assert_eq!(
        tool.wet_borda_que_pesa(),
        1.0,
        "a Heavy Edge não se prende a 0..1"
    );
}

/// O **Smooth Edges** é uma linha numérica desde 10/10 (dono: «ajustar a intensidade»): viva sob o
/// ponteiro, a edição vira `SetValue`, e a ferramenta guarda a intensidade presa a `0..1`. Desde a W3b (D1) é da
/// TELA e mora no cartão Paper (`seam_papel`).
#[test]
fn o_smooth_edges_e_um_slider_que_chega_a_ferramenta() {
    use ph2d_tool_painter::ids::PAINTER_WETPAINT_SMOOTH_EDGES as ID;
    let mut tool = molhado();
    let (mut host, mut st, _) = pinta(&tool);
    host.open_all_sections();
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let r = rect_de(&rects, ID).expect("Smooth Edges pintado");
    assert_eq!(
        host.hit_at(r.x + r.w * 0.5, r.y + r.h * 0.5),
        Some(ID),
        "a linha Smooth Edges não responde ao ponteiro"
    );
    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(ID));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o Smooth Edges morreu no painel"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == ID
        )),
        "o Smooth Edges nunca virou SetValue. drained = {actions:?}"
    );
    tool.handle_panel_event(PanelEvent::SetValue(ID, 0.35));
    assert!((tool.wet_smooth_edges() - 0.35).abs() < 1e-6);
    tool.handle_panel_event(PanelEvent::SetValue(ID, -2.0));
    assert_eq!(
        tool.wet_smooth_edges(),
        0.0,
        "o Smooth Edges não se prende a 0..1"
    );
}

/// A dica da Live Water diz o custo; a Heavy Edge, com a tela direita, diz que precisa do Tilt — e com
/// o Tilt ligado não tem dica de inércia (o controlo).
#[test]
fn as_dicas_das_opcoes() {
    let (host, _, _) = pinta(&molhado());
    assert_eq!(
        host.store().tooltip_for(PAINTER_WETPAINT_LIVE_WATER),
        Some(ph2d_i18n::tr(
            "panel.painter_layers.wetpaint.live_water_tip"
        )),
        "a Live Water sem a dica do custo"
    );
    let inerte = ph2d_i18n::tr("panel.painter_layers.inerte.tilt_on");
    assert_ne!(
        host.store().tooltip_for(PAINTER_WETPAINT_HEAVY_EDGE),
        Some(inerte),
        "CONTROLO: com o Tilt ligado a Heavy Edge diz-se inerte"
    );
    let mut direita = molhado();
    direita.handle_panel_event(PanelEvent::Click(
        ph2d_tool_painter::ids::PAINTER_WETPAINT_TILT_TOGGLE,
    ));
    let (host, _, _) = pinta(&direita);
    assert_eq!(
        host.store().tooltip_for(PAINTER_WETPAINT_HEAVY_EDGE),
        Some(inerte),
        "com a tela direita a Heavy Edge não esmaece com a dica do Tilt"
    );
}

/// ⭐ **As dicas dos knobs do pincel no cartão** (dono 25/10): o Wet bloom e o Tip pickup dizem que só agem na tinta
/// MOLHADA de outra cor (R4: «sem efeito» num traço de uma cor só); o Diffusion leva o nome inteiro (R2).
#[test]
fn as_dicas_dos_knobs_do_pincel() {
    use ph2d_tool_painter::ids::{
        PAINTER_WETPAINT_DIFFUSION, PAINTER_WETPAINT_TIP_PICKUP, PAINTER_WETPAINT_WET_BLOOM,
    };
    let (host, _, _) = pinta(&molhado());
    for (id, chave) in [
        (
            PAINTER_WETPAINT_WET_BLOOM,
            "panel.painter_wet_cartao.wet_bloom_tip",
        ),
        (
            PAINTER_WETPAINT_TIP_PICKUP,
            "panel.painter_wet_cartao.tip_pickup_tip",
        ),
        (
            PAINTER_WETPAINT_DIFFUSION,
            "panel.painter_wet_cartao.diffusion_tip",
        ),
    ] {
        let dica = ph2d_i18n::tr(chave);
        assert_ne!(dica, chave, "{chave} sem tradução");
        assert_eq!(host.store().tooltip_for(id), Some(dica), "{chave}");
    }
}

/// O clique REAL no centro de `r` → o painel → o barramento → a ferramenta.
fn clica_em(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    r: Rect,
) {
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
    entrega(host, tool);
}

/// Abre o chip **Over Oil** e clica a opção `i` do popover, pelo ponteiro.
fn escolhe_sobre_oleo(tool: &mut PainterTool, i: u8) {
    use ph2d_tool_painter::ids::{
        PAINTER_WETPAINT_SOBRE_OLEO, painter_wetpaint_sobre_oleo_option_id,
    };
    let (mut host, mut st, rects) = pinta(tool);
    let chip = rect_de(&rects, PAINTER_WETPAINT_SOBRE_OLEO).expect("o chip Over Oil pintado");
    clica_em(&mut host, &mut st, tool, chip);
    set_current_brush(Some(tool.brush_settings()));
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    for k in 0..ph2d_tool_painter::ids::PAINTER_WETPAINT_SOBRE_OLEO_COUNT {
        assert!(
            rect_de(&rects, painter_wetpaint_sobre_oleo_option_id(k)).is_some(),
            "clicar o chip não abriu a opção {k}"
        );
    }
    let o = rect_de(&rects, painter_wetpaint_sobre_oleo_option_id(i)).expect("opção pintada");
    clica_em(&mut host, &mut st, tool, o);
}

/// ⭐ O **Over Oil** (dono 30/10): o chip mora na aguada (sem o Real Oil ligado), nasce em Mix (`1`), e
/// escolher Tint (`0`) e Blend (`2`) no popover pelo ponteiro chega a `wet_sobre_oleo` e volta ao snapshot
/// que o chip pinta. Mutações que sangram: o chip fora do populate (não abre), a rota da opção em falta, o
/// braço `SelectOption` da ferramenta em falta, o snapshot preso ao Mix.
#[test]
fn o_over_oil_escolhido_no_popover_chega_a_ferramenta_e_volta_ao_painel() {
    use ph2d_tool_painter::ids::PAINTER_WETPAINT_SOBRE_OLEO;
    let (_, _, plano) = pinta(&PainterTool::default());
    assert!(
        rect_de(&plano, PAINTER_WETPAINT_SOBRE_OLEO).is_none(),
        "o Over Oil pintado sem o Wet Paint"
    );
    let mut tool = molhado();
    assert!(
        tool.wet_oleo_real().is_none(),
        "o chip é da aguada, sem o Real Oil"
    );
    assert_eq!(tool.wet_sobre_oleo(), 1, "o de fábrica é o Mix");
    assert_eq!(tool.brush_settings().wet_sobre_oleo, 1);
    for i in [0, 2, 1] {
        escolhe_sobre_oleo(&mut tool, i);
        assert_eq!(tool.wet_sobre_oleo(), i);
        assert_eq!(tool.brush_settings().wet_sobre_oleo, i);
    }
    // Fora da faixa: a ferramenta recusa e guarda o que tinha.
    escolhe_sobre_oleo(&mut tool, 0);
    tool.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_WETPAINT_SOBRE_OLEO,
        "7".into(),
    ));
    assert_eq!(tool.wet_sobre_oleo(), 0);
}

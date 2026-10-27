//! **O Real Oil (Óleo Real, doc 49) pelo clique REAL** — a caixa no cartão do Wet Paint liga a óptica do
//! relevo na ferramenta com o look aprovado (`OleoReal::default()`); só com ela ligada aparecem as cinco
//! linhas (Depth, Shadow, Gloss, Light height, Light angle), cada edição vira `SetValue` e chega à
//! ferramenta (os ângulos em GRAUS no painel, em radianos na ferramenta); desligar dá `None` e guarda os
//! valores para a próxima vez.
//!
//! Mutações que sangram: o id fora de `PAINTER_WETPAINT_CLICKS`/`PAINTER_WETPAINT_OLEO_FIELDS` (o clique
//! ou a edição morrem no painel), o braço da ferramenta em falta, as linhas pintadas com a caixa desligada,
//! a conversão de graus em falta.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::ids::{
    PAINTER_WETPAINT_OLEO_ALCANCE_FENDA, PAINTER_WETPAINT_OLEO_ALTITUDE,
    PAINTER_WETPAINT_OLEO_AMBIENTE, PAINTER_WETPAINT_OLEO_AZIMUTE, PAINTER_WETPAINT_OLEO_BRILHO,
    PAINTER_WETPAINT_OLEO_FENDA, PAINTER_WETPAINT_OLEO_FORCA_AMBIENTE,
    PAINTER_WETPAINT_OLEO_PROFUNDIDADE, PAINTER_WETPAINT_OLEO_REAL,
    PAINTER_WETPAINT_OLEO_SOMBRA, painter_wetpaint_oleo_ambiente_option_id,
};
use ph2d_ui_testkit::MockPanelHost;

/// As oito linhas por LITERAL (um gate que iterasse a lista sob teste encolheria com ela).
const LINHAS: [NodeId; 8] = [
    PAINTER_WETPAINT_OLEO_PROFUNDIDADE,
    PAINTER_WETPAINT_OLEO_SOMBRA,
    PAINTER_WETPAINT_OLEO_BRILHO,
    PAINTER_WETPAINT_OLEO_ALTITUDE,
    PAINTER_WETPAINT_OLEO_AZIMUTE,
    PAINTER_WETPAINT_OLEO_FENDA,
    PAINTER_WETPAINT_OLEO_ALCANCE_FENDA,
    PAINTER_WETPAINT_OLEO_FORCA_AMBIENTE,
];

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

fn entrega(host: &mut MockPanelHost, tool: &mut PainterTool) {
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

/// O clique REAL na caixa (ponteiro no centro dela) → o barramento → a ferramenta.
fn clica_na_caixa(tool: &mut PainterTool) {
    let (mut host, mut st, rects) = pinta(tool);
    let r = rect_de(&rects, PAINTER_WETPAINT_OLEO_REAL).expect("a caixa Real Oil pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(
        host.hit_at(cx, cy),
        Some(PAINTER_WETPAINT_OLEO_REAL),
        "a caixa Real Oil não responde ao ponteiro"
    );
    for ev in host.click_at(cx, cy) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    entrega(&mut host, tool);
}

#[test]
fn a_caixa_real_oil_so_aparece_com_o_wet_paint_armado_e_as_linhas_so_com_ela_ligada() {
    let (_, _, plano) = pinta(&PainterTool::default());
    assert!(
        rect_de(&plano, PAINTER_WETPAINT_OLEO_REAL).is_none(),
        "o Real Oil pintado no pincel simples"
    );
    let mut tool = molhado();
    let (_, _, desligado) = pinta(&tool);
    assert!(rect_de(&desligado, PAINTER_WETPAINT_OLEO_REAL).is_some());
    for id in LINHAS {
        assert!(
            rect_de(&desligado, id).is_none(),
            "uma linha do Real Oil pintada com a caixa desligada ({id:?})"
        );
    }
    tool.set_wet_oleo_real_ligado(true);
    let (_, _, ligado) = pinta(&tool);
    for id in LINHAS {
        assert!(
            rect_de(&ligado, id).is_some(),
            "a linha {id:?} do Real Oil não aparece com a caixa ligada"
        );
    }
}

#[test]
fn o_real_oil_liga_com_o_look_aprovado_cada_linha_chega_a_ferramenta_e_desligar_guarda_os_valores()
{
    let mut tool = molhado();
    assert_eq!(tool.wet_oleo_real(), None, "o Real Oil nasce desligado");
    clica_na_caixa(&mut tool);
    // O look aprovado (`OleoReal::default()`), por número: profundidade 1, luz a 42° vinda de 225°, sombra e
    // brilho cheios.
    let o = tool.wet_oleo_real().expect("o clique não ligou o Real Oil");
    assert_eq!((o.profundidade, o.sombra, o.brilho), (1.0, 1.0, 1.0));
    assert!((o.altitude - 42f64.to_radians()).abs() < 1e-12);
    assert!((o.azimute - 225f64.to_radians()).abs() < 1e-12);
    // Cada linha: viva sob o ponteiro, a edição vira `SetValue` no painel.
    let (mut host, mut st, rects) = pinta(&tool);
    for id in LINHAS {
        let r = rect_de(&rects, id).expect("linha pintada");
        assert_eq!(
            host.hit_at(r.x + r.w * 0.5, r.y + r.h * 0.5),
            Some(id),
            "a linha {id:?} não responde ao ponteiro"
        );
        assert_eq!(
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id)),
            EventOutcome::Consumed,
            "a edição da linha {id:?} morreu no painel"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id
            )),
            "a linha {id:?} nunca virou SetValue. drained = {actions:?}"
        );
    }
    // O `SetValue` na ferramenta, nas unidades do painel.
    for (id, v) in [
        (PAINTER_WETPAINT_OLEO_PROFUNDIDADE, 2.5),
        (PAINTER_WETPAINT_OLEO_SOMBRA, 0.35),
        (PAINTER_WETPAINT_OLEO_BRILHO, 0.4),
        (PAINTER_WETPAINT_OLEO_ALTITUDE, 30.0),
        (PAINTER_WETPAINT_OLEO_AZIMUTE, 90.0),
    ] {
        tool.handle_panel_event(PanelEvent::SetValue(id, v));
    }
    let o = tool.wet_oleo_real().expect("continua ligado");
    assert!((o.profundidade - 2.5).abs() < 1e-12);
    assert!((o.sombra - 0.35).abs() < 1e-12);
    assert!((o.brilho - 0.4).abs() < 1e-12);
    assert!(
        (o.altitude - 30f64.to_radians()).abs() < 1e-12,
        "a altura da luz não chegou em radianos"
    );
    assert!((o.azimute - 90f64.to_radians()).abs() < 1e-12);
    // As faixas: a luz não desce de 5°, a direcção dá a volta, a sombra prende-se a 0..1.
    tool.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_OLEO_ALTITUDE, 1.0));
    tool.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_OLEO_AZIMUTE, 450.0));
    tool.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_OLEO_SOMBRA, 7.0));
    let o = tool.wet_oleo_real().expect("continua ligado");
    assert!((o.altitude - 5f64.to_radians()).abs() < 1e-12);
    assert!((o.azimute - 90f64.to_radians()).abs() < 1e-12);
    assert_eq!(o.sombra, 1.0);
    // O painel lê de volta em GRAUS.
    let b = tool.brush_settings();
    assert!(b.wet_oleo_real);
    assert!((b.wet_oleo_altitude_graus - 5.0).abs() < 1e-4);
    assert!((b.wet_oleo_azimute_graus - 90.0).abs() < 1e-4);
    // Desligar = `None`; voltar a ligar devolve os últimos valores.
    clica_na_caixa(&mut tool);
    assert_eq!(tool.wet_oleo_real(), None, "o segundo clique não desligou");
    assert!(!tool.brush_settings().wet_oleo_real);
    clica_na_caixa(&mut tool);
    assert_eq!(
        tool.wet_oleo_real(),
        Some(o),
        "religar não devolveu os últimos valores"
    );
}

/// Os índices em `Ambiente::TODOS` por LITERAL (`Classico, Estudio, Interior, Patio`): o wire do chip.
const ESTUDIO: u8 = 1;
const INTERIOR: u8 = 2;

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

/// ⭐ O **Environment** (W1c): o chip só aparece com a caixa ligada, nasce em Studio (índice 1, o
/// `Ambiente::default()`), e escolher «Interior» no popover pelo ponteiro chega a `OleoReal::ambiente`
/// e volta ao painel como índice 2. Mutações que sangram: o chip fora do populate (não abre), a rota da
/// opção em falta (não chega), o braço `SelectOption` da ferramenta em falta, o snapshot preso a um valor.
#[test]
fn o_environment_escolhido_no_popover_chega_ao_oleo_real_e_volta_ao_painel() {
    let mut tool = molhado();
    let (_, _, desligado) = pinta(&tool);
    assert!(
        rect_de(&desligado, PAINTER_WETPAINT_OLEO_AMBIENTE).is_none(),
        "o Environment pintado com a caixa desligada"
    );
    clica_na_caixa(&mut tool);
    assert_eq!(
        tool.wet_oleo_real().expect("ligado").ambiente.indice(),
        ESTUDIO,
        "o ambiente de fábrica é o Studio"
    );
    assert_eq!(tool.brush_settings().wet_oleo_ambiente, ESTUDIO);
    let (mut host, mut st, rects) = pinta(&tool);
    let chip = rect_de(&rects, PAINTER_WETPAINT_OLEO_AMBIENTE).expect("o chip Environment pintado");
    clica_em(&mut host, &mut st, &mut tool, chip);
    set_current_brush(Some(tool.brush_settings()));
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let interior = painter_wetpaint_oleo_ambiente_option_id(INTERIOR);
    let o = rect_de(&rects, interior).expect("clicar o chip não abriu a opção Interior");
    clica_em(&mut host, &mut st, &mut tool, o);
    assert_eq!(
        tool.wet_oleo_real()
            .expect("continua ligado")
            .ambiente
            .indice(),
        INTERIOR
    );
    assert_eq!(tool.wet_oleo_real_valores().ambiente.indice(), INTERIOR);
    assert_eq!(tool.brush_settings().wet_oleo_ambiente, INTERIOR);
    // Desligar e religar guarda o ambiente com os outros knobs.
    clica_na_caixa(&mut tool);
    clica_na_caixa(&mut tool);
    assert_eq!(
        tool.wet_oleo_real().expect("religado").ambiente.indice(),
        INTERIOR
    );
}

/// ⭐ A **fenda** (W2 E1): Crevice shadow e Crevice reach nascem no neutro (força `1`, alcance `24` = a
/// imagem de antes), o valor escrito no campo chega a `OleoReal::fenda`/`alcance_fenda`, prende-se às
/// faixas medidas (`0..3`, `1..24`), volta ao painel e sobrevive a desligar/religar. Mutações que sangram:
/// o braço `5`/`6` do `set_wet_oleo_knob` trocado, o snapshot preso ao neutro, a faixa em falta.
#[test]
fn a_forca_e_o_alcance_da_fenda_chegam_ao_oleo_real_e_sobrevivem_a_religar() {
    let mut tool = molhado();
    clica_na_caixa(&mut tool);
    let o = tool.wet_oleo_real().expect("ligado");
    assert_eq!(
        (o.fenda, o.alcance_fenda),
        (1.0, 24.0),
        "o neutro é a imagem de antes"
    );
    let b = tool.brush_settings();
    assert_eq!((b.wet_oleo_fenda, b.wet_oleo_alcance_fenda), (1.0, 24.0));
    // A edição pelo campo: o painel emite `SetValue` (gate acima); a ferramenta recebe-o.
    tool.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_OLEO_FENDA, 2.25));
    tool.handle_panel_event(PanelEvent::SetValue(
        PAINTER_WETPAINT_OLEO_ALCANCE_FENDA,
        8.0,
    ));
    let o = tool.wet_oleo_real().expect("continua ligado");
    assert_eq!((o.fenda, o.alcance_fenda), (2.25, 8.0));
    let b = tool.brush_settings();
    assert_eq!((b.wet_oleo_fenda, b.wet_oleo_alcance_fenda), (2.25, 8.0));
    // As faixas: a força não passa do tecto medido, o alcance não desce de uma célula nem passa de 24.
    tool.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_OLEO_FENDA, 9.0));
    tool.handle_panel_event(PanelEvent::SetValue(
        PAINTER_WETPAINT_OLEO_ALCANCE_FENDA,
        0.0,
    ));
    let o = tool.wet_oleo_real().expect("continua ligado");
    assert_eq!((o.fenda, o.alcance_fenda), (3.0, 1.0));
    tool.handle_panel_event(PanelEvent::SetValue(
        PAINTER_WETPAINT_OLEO_ALCANCE_FENDA,
        99.0,
    ));
    tool.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_OLEO_FENDA, 0.0));
    let o = tool.wet_oleo_real().expect("continua ligado");
    assert_eq!((o.fenda, o.alcance_fenda), (0.0, 24.0));
    tool.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_OLEO_FENDA, 1.75));
    tool.handle_panel_event(PanelEvent::SetValue(
        PAINTER_WETPAINT_OLEO_ALCANCE_FENDA,
        12.0,
    ));
    // Desligar e religar guarda os dois com os outros knobs.
    clica_na_caixa(&mut tool);
    assert_eq!(tool.wet_oleo_real(), None);
    assert_eq!(
        tool.brush_settings().wet_oleo_fenda,
        1.75,
        "desligado, o painel lê o último valor"
    );
    clica_na_caixa(&mut tool);
    let o = tool.wet_oleo_real().expect("religado");
    assert_eq!((o.fenda, o.alcance_fenda), (1.75, 12.0));
}

/// ⭐ A **força do ambiente** (W2 E2): Environment light nasce na mistura de fábrica (`0,6` = o Clássico de
/// antes, bit a bit), o valor do campo chega a `OleoReal::forca_ambiente`, prende-se a `0..1` (é uma
/// mistura), volta ao painel e sobrevive a desligar/religar. Mutações que sangram: o braço `7` do
/// `set_wet_oleo_knob` em falta, o snapshot preso ao neutro, a faixa aberta.
#[test]
fn a_forca_do_ambiente_chega_ao_oleo_real_e_sobrevive_a_religar() {
    let mut tool = molhado();
    clica_na_caixa(&mut tool);
    let o = tool.wet_oleo_real().expect("ligado");
    assert_eq!(o.forca_ambiente, 0.6, "o neutro é a mistura de antes");
    assert_eq!(tool.brush_settings().wet_oleo_forca_ambiente, 0.6);
    tool.handle_panel_event(PanelEvent::SetValue(
        PAINTER_WETPAINT_OLEO_FORCA_AMBIENTE,
        0.85,
    ));
    assert_eq!(tool.wet_oleo_real().expect("ligado").forca_ambiente, 0.85);
    assert_eq!(tool.brush_settings().wet_oleo_forca_ambiente, 0.85);
    tool.handle_panel_event(PanelEvent::SetValue(
        PAINTER_WETPAINT_OLEO_FORCA_AMBIENTE,
        4.0,
    ));
    assert_eq!(tool.wet_oleo_real().expect("ligado").forca_ambiente, 1.0);
    tool.handle_panel_event(PanelEvent::SetValue(
        PAINTER_WETPAINT_OLEO_FORCA_AMBIENTE,
        -1.0,
    ));
    assert_eq!(tool.wet_oleo_real().expect("ligado").forca_ambiente, 0.0);
    tool.handle_panel_event(PanelEvent::SetValue(
        PAINTER_WETPAINT_OLEO_FORCA_AMBIENTE,
        0.3,
    ));
    clica_na_caixa(&mut tool);
    assert_eq!(tool.wet_oleo_real(), None);
    assert_eq!(tool.brush_settings().wet_oleo_forca_ambiente, 0.3);
    clica_na_caixa(&mut tool);
    assert_eq!(tool.wet_oleo_real().expect("religado").forca_ambiente, 0.3);
}

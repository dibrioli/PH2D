//! **A busca do popover do Preset, pelo ponteiro e pelo teclado** (dono 08/11: «precisamos de um campo de
//! busca no dropdown»). Clique no chip, repintura, as letras pela porta real (`dispatch_text_input` com o
//! foco que o painel deu ao campo), os eventos pelo `apply_panel_event` do painel, repintura — e o que se lê
//! é o que o popover REGISTOU: só os pincéis que casam têm linha viva; sem achados, nenhum, e a linha
//! «No brushes found» está na lista sem se oferecer ao clique. O Enter escolhe o primeiro achado.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, dispatch_text_input};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_panel_painter_layers::{
    PainterLayersPanel, VistaPreset, VistaPresets, set_current_wet_presets,
};
use ph2d_tool_painter::ids;
use ph2d_tool_painter::{PaintMedia, PainterTool, PedidoPreset};
use ph2d_ui_testkit::MockPanelHost;
use std::sync::Arc;

const GOUACHE: &str = "WetPainter/Gouache/Gouache Flat.ph2d-brush.png";
const ROUND: &str = "WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png";
const WASH: &str = "WetPainter/Watercolor/Fluid/Wet Wash.ph2d-brush.png";

fn vista() -> Arc<VistaPresets> {
    let e = |rel: &str, nome: &str, categoria: &str| VistaPreset {
        relativo: rel.to_owned(),
        nome: nome.to_owned(),
        categoria: categoria.to_owned(),
        favorito: false,
        apagavel: false,
    };
    Arc::new(VistaPresets {
        entradas: vec![
            e(GOUACHE, "Gouache Flat", "Gouache"),
            e(ROUND, "Wet Round", "Watercolor"),
            e(WASH, "Wet Wash", "Watercolor"),
        ],
        ativo: Some(ROUND.to_owned()),
        politica: 0,
        ilegiveis: 0,
        ordem_manual: false,
    })
}

fn pinta(
    t: &PainterTool,
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
) -> Vec<(NodeId, Rect)> {
    set_current_brush(Some(t.brush_settings()));
    host.paint::<PainterLayersPanel>(st, Rect::new(0.0, 0.0, 1600.0, 1400.0))
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// As letras pela porta real do teclado, cada evento pelo painel.
fn escreve(host: &mut MockPanelHost, st: &mut PainterLayersPanelState, texto: &str) {
    let arena = Bump::new();
    for ch in texto.chars() {
        let evs = dispatch_text_input(host.store_mut(), ch, &arena).to_vec();
        for ev in evs {
            host.apply_panel_event::<PainterLayersPanel>(st, ev);
        }
    }
}

fn campo(host: &MockPanelHost) -> String {
    match host.store().get(ids::PAINTER_WETPAINT_PRESET_SEARCH) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        other => panic!("o campo da busca não é um TextInput: {other:?}"),
    }
}

/// Abre o Preset no painel de antes; devolve o host, o estado e os rects da lista aberta.
fn abre(t: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_wet_presets(Some(vista()));
    let mut host = crate::host_de_antes();
    let mut st = PainterLayersPanelState;
    let rects = pinta(t, &mut host, &mut st);
    let chip = rect_of(&rects, ids::PAINTER_WETPAINT_PRESET).expect("o chip do Preset");
    for ev in host.click_at(chip.x + chip.w * 0.5, chip.y + chip.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    let rects = pinta(t, &mut host, &mut st);
    (host, st, rects)
}

#[test]
fn escrever_no_campo_filtra_a_lista_e_sem_achados_diz_no_brushes_found() {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let (mut host, mut st, rects) = abre(&t);
    let opcao = |rects: &[(NodeId, Rect)], rel: &str| {
        rect_of(rects, ids::painter_wetpaint_preset_option_id(2, rel))
    };
    // Piso: aberta e vazia, a lista inteira e o campo à vista e com o foco.
    assert!(
        rect_of(&rects, ids::PAINTER_WETPAINT_PRESET_SEARCH).is_some(),
        "o campo não está pintado"
    );
    assert_eq!(
        host.store().focus_id(),
        Some(ids::PAINTER_WETPAINT_PRESET_SEARCH),
        "abrir não focou o campo"
    );
    for rel in [GOUACHE, ROUND, WASH] {
        assert!(
            opcao(&rects, rel).is_some(),
            "a lista aberta sem busca não tem {rel}"
        );
    }

    escreve(&mut host, &mut st, "gou");
    assert_eq!(campo(&host), "gou");
    let rects = pinta(&t, &mut host, &mut st);
    assert!(
        opcao(&rects, GOUACHE).is_some(),
        "«gou» escondeu o Gouache Flat"
    );
    for rel in [ROUND, WASH] {
        assert!(opcao(&rects, rel).is_none(), "«gou» deixou {rel} na lista");
    }

    escreve(&mut host, &mut st, "zzz");
    let rects = pinta(&t, &mut host, &mut st);
    for rel in [GOUACHE, ROUND, WASH] {
        assert!(
            opcao(&rects, rel).is_none(),
            "«gouzzz» deixou {rel} na lista"
        );
    }
    // «No brushes found» é a linha apagada (o id de título 0): está na lista e não se oferece ao clique.
    let aviso = ph2d_i18n::tr("panel.painter_layers.wetpaint.preset_no_match");
    assert_eq!(aviso, "No brushes found");
    assert!(
        rect_of(&rects, ids::painter_wetpaint_preset_title_id(0)).is_none(),
        "a linha «{aviso}» ofereceu-se ao clique"
    );
    assert!(
        rect_of(&rects, ids::PAINTER_WETPAINT_PRESET_SEARCH).is_some(),
        "sem achados o campo sumiu"
    );
}

#[test]
fn enter_no_campo_escolhe_o_primeiro_achado() {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let (mut host, mut st, _) = abre(&t);
    escreve(&mut host, &mut st, "wash");
    let _ = pinta(&t, &mut host, &mut st);
    for ev in host.type_keys_then_enter(ids::PAINTER_WETPAINT_PRESET_SEARCH, "") {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            t.handle_panel_event(pe);
        }
    }
    assert_eq!(
        t.take_wet_preset_request(),
        Some(PedidoPreset::Escolher(WASH.to_owned()))
    );
}

/// ⭐ **O campo fica no topo e a faixa dele tapa as linhas** (dono 08/11: o de antes «ficou feio» — rolava com a lista).
/// Com 80 pincéis a lista rola; rolada, o campo está no MESMO sítio, e um clique na faixa (por cima de uma linha
/// que passou por baixo) é do campo, nunca de um pincel escondido.
#[test]
fn o_campo_fica_no_topo_quando_a_lista_rola_e_a_faixa_tapa_as_linhas() {
    let entradas: Vec<VistaPreset> = (0..80)
        .map(|k| VistaPreset {
            relativo: format!("WetPainter/Gouache/G{k:02}.ph2d-brush.png"),
            nome: format!("Gouache {k:02}"),
            categoria: "Gouache".to_owned(),
            favorito: false,
            apagavel: false,
        })
        .collect();
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    set_current_wet_presets(Some(Arc::new(VistaPresets {
        entradas,
        ativo: None,
        politica: 0,
        ilegiveis: 0,
        ordem_manual: false,
    })));
    let mut host = crate::host_de_antes();
    let mut st = PainterLayersPanelState;
    let rects = pinta(&t, &mut host, &mut st);
    let chip = rect_of(&rects, ids::PAINTER_WETPAINT_PRESET).expect("o chip do Preset");
    for ev in host.click_at(chip.x + chip.w * 0.5, chip.y + chip.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    let rects = pinta(&t, &mut host, &mut st);
    let faixa = rect_of(&rects, ids::PAINTER_WETPAINT_PRESET_SEARCH).expect("a faixa do campo");
    host.store_mut()
        .set_panel_scroll(ids::PAINTER_WETPAINT_PRESET, 3.0 * chip.h);
    let rects = pinta(&t, &mut host, &mut st);
    assert_eq!(
        host.store().panel_scroll(ids::PAINTER_WETPAINT_PRESET),
        3.0 * chip.h,
        "a lista de 80 não rolou (a régua não mede nada)"
    );
    let rolada =
        rect_of(&rects, ids::PAINTER_WETPAINT_PRESET_SEARCH).expect("o campo sumiu ao rolar");
    assert_eq!(rolada, faixa, "o campo rolou com a lista");
    let por_baixo = rects
        .iter()
        .filter(|(id, r)| {
            *id != ids::PAINTER_WETPAINT_PRESET_SEARCH
                && r.y < faixa.y + faixa.h
                && r.y + r.h > faixa.y
                && r.x < faixa.x + faixa.w
                && r.x + r.w > faixa.x
        })
        .count();
    assert!(
        por_baixo > 0,
        "nenhuma linha passou por baixo da faixa (a régua não mede nada)"
    );
    for fy in [
        faixa.y + 2.0,
        faixa.y + faixa.h * 0.5,
        faixa.y + faixa.h - 2.0,
    ] {
        assert_eq!(
            host.hit_at(faixa.x + faixa.w * 0.5, fy),
            Some(ids::PAINTER_WETPAINT_PRESET_SEARCH),
            "um clique na faixa (y {fy}) foi para uma linha escondida"
        );
    }
}

//! ⭐⭐⭐ **ARRASTAR UM PARAM NO CARTÃO** — os gates do gesto (ciclo 1, doc 103).
//!
//! ⚠️ Irmão de `interact_tests` por RESPONSABILIDADE: aquele mede os gestos do GRAFO (mover
//! um nó, puxar um fio, laçar), este o gesto que o cartão passou a ter dentro de si.

use super::*;
use crate::snapshot::{CardParam, GraphNodeView, GraphViewSnapshot, NodeViewKind, PortView, drain_intents};
use super::tests::{CENTER, RECT, gesture};
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory, ParamUiHint, ParamWidget};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

fn card(params: Vec<CardParam>) -> GraphViewSnapshot {
    GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![GraphNodeView {
            kind: NodeViewKind::Node,
            id: 7,
            display_name: "Grid".into(),
            category: NodeUiCategory::Source,
            silhouette: NodeSilhouette::Rect,
            x: 40.0,
            y: 40.0,
            inputs: vec![PortView {
                name: "in",
                domain: Domain::Instances,
                dim: Dim::Vec2,
                clock: Clock::Frame,
            }],
            outputs: vec![],
            readout: None,
            count: None,
            hot: false,
            is_sink: false,
            preview: None,
            bypassed: false,
            inert: false,
            thumbnail: None,
            params,
            sections: Vec::new(),
        }],
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }
}

fn param(name: &'static str, value: f32, step: f32, driven: bool) -> CardParam {
    CardParam {
        hint: ParamUiHint {
            param: name,
            label: "Rows",
            min: 0.0,
            max: 10.0,
            step,
            widget: ParamWidget::Slider,
        },
        value,
        driven,
        swatch: None,
    }
}

/// Arrasta a row `row` do cartão por `dx` px e devolve o que saiu na fila de intenções.
fn drag(snap: &GraphViewSnapshot, row: u16, dx: f32) -> Vec<GraphIntent> {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    st.fitted = true; // senão o painel enquadra e o zoom do teste evapora
    let kind = GraphHitKind::ParamRow { node: 7, row };
    let x0 = 100.0;
    super::apply_gesture(
        &mut st,
        gesture(kind, GesturePhase::Begin, x0, 60.0),
        RECT,
        CENTER,
        snap,
    );
    super::apply_gesture(
        &mut st,
        gesture(kind, GesturePhase::Update, x0 + dx, 60.0),
        RECT,
        CENTER,
        snap,
    );
    drain_intents()
}

/// ⭐⭐ **O ARRASTO CHEGA AO DOCUMENTO** — e é o gesto REAL (Begin + Update), não um `Click`
/// sintético: o chip do Vector passou num clique sintético e estava **morto sob o dedo**.
/// FALSIFICADO por o handler não emitir nada, ou emitir para o param errado.
#[test]
fn dragging_a_param_row_writes_the_document() {
    let snap = card(vec![param("rows", 5.0, 0.1, false)]);
    let saiu = drag(&snap, 0, 50.0);
    let Some(GraphIntent::SetParam { node, param, value }) = saiu.first() else {
        panic!("o arrasto tem de emitir um SetParam, e emitiu {saiu:?}");
    };
    assert_eq!(*node, 7);
    assert_eq!(*param, "rows");
    assert!(*value > 5.0, "arrastar para a direita SOBE o valor ({value})");
}

/// ⭐⭐ **ATRAVESSAR A LARGURA DO CARTÃO VARRE A FAIXA INTEIRA** — a lei que faz o dedo e a
/// barra desenhada concordarem por construção (nenhum segundo número de sensibilidade).
/// FALSIFICADO por qualquer outra escala: o valor deixa de bater com o preenchimento.
#[test]
fn dragging_the_card_width_spans_the_whole_range() {
    let snap = card(vec![param("rows", 0.0, 0.1, false)]);
    // A vista de omissão tem zoom 1, então a largura em ecrã é a `CARD_W`.
    let saiu = drag(&snap, 0, crate::geom::CARD_W);
    let Some(GraphIntent::SetParam { value, .. }) = saiu.first() else {
        panic!("sem intencao: {saiu:?}");
    };
    assert!(
        (*value - 10.0).abs() < 1e-3,
        "de `min` a `max` numa largura de cartao ({value} contra 10)"
    );
}

/// **UM PARAM INTEIRO NÃO PÁRA ENTRE DOIS NÚMEROS** — o `step >= 1` do hint decide, a mesma
/// leitura que a row usa para o escrever. FALSIFICADO por tirar o `round`.
#[test]
fn an_integer_param_lands_on_a_whole_number() {
    let snap = card(vec![param("rows", 3.0, 1.0, false)]);
    let saiu = drag(&snap, 0, 37.0);
    let Some(GraphIntent::SetParam { value, .. }) = saiu.first() else {
        panic!("sem intencao: {saiu:?}");
    };
    assert_eq!(*value, value.round(), "um inteiro cai em inteiro ({value})");
}

/// ⚠️ **UM PARAM DIRIGIDO POR FIO NÃO SE ARRASTA** — o número vem de fora, e um alvo que não
/// obedece ao dedo é a mentira que a row dirigida do painel já aprendeu a não contar. O alvo
/// nem chega a ser REGISTADO. FALSIFICADO por tirar o `continue` do `push_param_row_hits`.
#[test]
fn a_driven_param_row_registers_no_target() {
    let snap = card(vec![param("rows", 5.0, 0.1, true)]);
    let view = crate::geom::View::new(RECT, crate::state::ViewState::default());
    let mut hits = Vec::new();
    crate::hits::push_param_row_hits(&mut hits, &snap.nodes[0], &view, RECT);
    assert!(
        hits.is_empty(),
        "uma row dirigida nao regista alvo, e registou {hits:?}"
    );
}

/// ⭐ **AS ROWS GANHAM O GESTO AO CORPO DO CARTÃO** — `register_hits` é *«a última ganha»*, e
/// sem isto arrastar sobre uma row moveria o NÓ em vez de mexer no número. FALSIFICADO por
/// empurrar as rows ANTES do corpo.
#[test]
fn a_param_row_is_registered_after_the_card_body() {
    let snap = card(vec![param("rows", 5.0, 0.1, false)]);
    let view = crate::geom::View::new(RECT, crate::state::ViewState::default());
    let n = &snap.nodes[0];
    let mut hits = Vec::new();
    let body = crate::geom::card_rect(n, &view);
    crate::hits::push_card_hit(&mut hits, n, body, RECT);
    let corpo = hits.len();
    crate::hits::push_param_row_hits(&mut hits, n, &view, RECT);
    assert!(hits.len() > corpo, "a row entrou na lista");
    assert!(
        matches!(hits[corpo].1, GraphHitKind::ParamRow { .. }),
        "e entrou DEPOIS do corpo, que e' o que a faz ganhar o gesto"
    );
}

fn enum_param(value: f32, labels: &'static [&'static str]) -> CardParam {
    CardParam {
        hint: ParamUiHint {
            param: "mode",
            label: "Mode",
            min: 0.0,
            max: (labels.len().saturating_sub(1)) as f32,
            step: 1.0,
            widget: ParamWidget::Enum { labels },
        },
        value,
        driven: false,
        swatch: None,
    }
}

fn toggle_param(value: f32) -> CardParam {
    CardParam {
        hint: ParamUiHint {
            param: "invert",
            label: "Invert",
            min: 0.0,
            max: 1.0,
            step: 1.0,
            widget: ParamWidget::Toggle,
        },
        value,
        driven: false,
        swatch: None,
    }
}

/// Um CLIQUE (pressão e largada sem varrer) na row `row`.
fn click(snap: &GraphViewSnapshot, row: u16) -> Vec<GraphIntent> {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    st.fitted = true;
    super::apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::ParamRow { node: 7, row },
            GesturePhase::Click,
            100.0,
            60.0,
        ),
        RECT,
        CENTER,
        snap,
    );
    drain_intents()
}

/// ⭐⭐ **UM CLIQUE AVANÇA O ENUM, COM VOLTA AO PRINCÍPIO** — é o gesto de quem quer *a
/// seguinte*, e o `Enum` é **20 % de todas as rows do catálogo, em 85 nós** (censo de
/// 2026-09-05). Arrastar continua a varrer, que é como se atravessa um enum de 48 opções.
/// FALSIFICADO por o clique não emitir nada (o artista fica com um rótulo que não muda) ou por
/// não dar a volta (a última opção prende).
#[test]
fn a_click_advances_an_enum_and_wraps() {
    let snap = card(vec![enum_param(0.0, &["Sine", "Triangle", "Square"])]);
    let saiu = click(&snap, 0);
    let Some(GraphIntent::SetParam { value, param, .. }) = saiu.first() else {
        panic!("o clique tem de avancar o enum, e saiu {saiu:?}");
    };
    assert_eq!(*param, "mode");
    assert_eq!(*value, 1.0, "de Sine para Triangle");

    let snap = card(vec![enum_param(2.0, &["Sine", "Triangle", "Square"])]);
    let saiu = click(&snap, 0);
    let Some(GraphIntent::SetParam { value, .. }) = saiu.first() else {
        panic!("sem intencao na ultima opcao");
    };
    assert_eq!(*value, 0.0, "da ultima volta ao principio");
}

/// **UM CLIQUE VIRA O INTERRUPTOR** — nos dois sentidos. FALSIFICADO por o clique escrever
/// sempre o mesmo valor (o interruptor liga e nunca desliga).
#[test]
fn a_click_flips_a_toggle_both_ways() {
    for (antes, depois) in [(0.0_f32, 1.0_f32), (1.0, 0.0)] {
        let snap = card(vec![toggle_param(antes)]);
        let saiu = click(&snap, 0);
        let Some(GraphIntent::SetParam { value, .. }) = saiu.first() else {
            panic!("o clique tem de virar o interruptor");
        };
        assert_eq!(*value, depois, "de {antes} para {depois}");
    }
}

/// ⚠️ **LARGAR DEPOIS DE VARRER NÃO DÁ MAIS UM PASSO** — se o `End` de um arrasto também
/// avançasse, o valor saltaria por cima do que o artista acabou de escolher. FALSIFICADO por
/// juntar `End` ao braço do `Click`.
#[test]
fn releasing_after_a_scrub_does_not_advance_the_enum() {
    let snap = card(vec![enum_param(1.0, &["a", "b", "c"])]);
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    st.fitted = true;
    let kind = GraphHitKind::ParamRow { node: 7, row: 0 };
    for fase in [GesturePhase::Begin, GesturePhase::End] {
        super::apply_gesture(&mut st, gesture(kind, fase, 100.0, 60.0), RECT, CENTER, &snap);
    }
    assert!(
        drain_intents().is_empty(),
        "pressionar e largar sem varrer nao escreve — quem escreve e' o Click"
    );
}

/// **UM PARAM CONTÍNUO IGNORA O CLIQUE** — clicar num slider não deve mexer no número (só o
/// arrasto o move). FALSIFICADO por o braço do clique cair no `_ => {}` errado.
#[test]
fn a_click_on_a_slider_changes_nothing() {
    let snap = card(vec![param("rows", 5.0, 0.1, false)]);
    assert!(
        click(&snap, 0).is_empty(),
        "um clique num slider nao escreve nada"
    );
}

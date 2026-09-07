//! ⭐⭐⭐ **ARRASTAR UM PARAM NO CARTÃO** — os gates do gesto (ciclo 1, doc 103).
//!
//! ⚠️ Irmão de `interact_tests` por RESPONSABILIDADE: aquele mede os gestos do GRAFO (mover
//! um nó, puxar um fio, laçar), este o gesto que o cartão passou a ter dentro de si.

use super::tests::{CENTER, RECT, gesture};
use super::*;
use crate::snapshot::{
    CardParam, GraphNodeView, GraphViewSnapshot, NodeViewKind, PortView, drain_intents,
};
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
    let p = CardParam::from_hint(
        ParamUiHint {
            param: name,
            label: "Rows",
            min: 0.0,
            max: 10.0,
            step,
            widget: ParamWidget::Slider,
        },
        value,
    );
    if driven { p.driven_by_wire() } else { p }
}

/// Arrasta a row `row` do cartão por `dx` px e devolve o que saiu na fila de intenções.
fn drag(snap: &GraphViewSnapshot, row: u16, dx: f32) -> Vec<GraphIntent> {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState {
        fitted: true,
        ..MotionGraphPanelState::default()
    }; // senão o painel enquadra e o zoom do teste evapora
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
    assert!(
        *value > 5.0,
        "arrastar para a direita SOBE o valor ({value})"
    );
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
    CardParam::from_hint(
        ParamUiHint {
            param: "mode",
            label: "Mode",
            min: 0.0,
            max: (labels.len().saturating_sub(1)) as f32,
            step: 1.0,
            widget: ParamWidget::Enum { labels },
        },
        value,
    )
}

fn toggle_param(value: f32) -> CardParam {
    CardParam::from_hint(
        ParamUiHint {
            param: "invert",
            label: "Invert",
            min: 0.0,
            max: 1.0,
            step: 1.0,
            widget: ParamWidget::Toggle,
        },
        value,
    )
}

fn file_param() -> CardParam {
    CardParam::from_hint(
        ParamUiHint {
            param: "file",
            label: "File",
            min: 0.0,
            max: 0.0,
            step: 0.0,
            widget: ParamWidget::File {
                kind: ph2d_node_registry::FileKind::Table,
            },
        },
        0.0,
    )
}

fn source_param() -> CardParam {
    CardParam::from_hint(
        ParamUiHint {
            param: "path",
            label: "Shape",
            min: 0.0,
            max: 0.0,
            step: 0.0,
            widget: ParamWidget::Source,
        },
        0.0,
    )
}

/// Um CLIQUE (pressão e largada sem varrer) na row `row` — o **estado** que ele deixa e o que
/// saiu na fila. As duas coisas, porque desde 2026-09-05 um clique num número não escreve: ele
/// abre uma caixa, e isso só se vê no estado.
fn click_state(snap: &GraphViewSnapshot, row: u16) -> (MotionGraphPanelState, Vec<GraphIntent>) {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState {
        fitted: true,
        ..MotionGraphPanelState::default()
    };
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
    (st, drain_intents())
}

/// Só o que saiu na fila — o caso comum.
fn click(snap: &GraphViewSnapshot, row: u16) -> Vec<GraphIntent> {
    click_state(snap, row).1
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
    let mut st = MotionGraphPanelState {
        fitted: true,
        ..MotionGraphPanelState::default()
    };
    let kind = GraphHitKind::ParamRow { node: 7, row: 0 };
    for fase in [GesturePhase::Begin, GesturePhase::End] {
        super::apply_gesture(
            &mut st,
            gesture(kind, fase, 100.0, 60.0),
            RECT,
            CENTER,
            &snap,
        );
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

/// ⭐⭐⭐ **UM CLIQUE NUM NÚMERO ABRE A CAIXA DE ESCRITA — e NÃO mexe no valor** (report do Enio,
/// 2026-09-05: *«vários nós não permitem clicar no número para usar o teclado para escrever»*).
///
/// As duas metades: uma caixa que não abre é o report, e uma que abre **escrevendo** destruiria o
/// número em quem só quis olhar. FALSIFICADO por o braço do clique cair no `match` dos estados
/// (nada abre) ou por ele emitir um `SetParam`.
#[test]
fn clicking_a_number_row_opens_the_typing_box_and_writes_nothing() {
    let snap = card(vec![param("rows", 5.0, 0.1, false)]);
    let (st, saiu) = click_state(&snap, 0);
    let e = st
        .param_edit
        .expect("o clique num numero tem de abrir a caixa");
    assert_eq!((e.node, e.param), (7, "rows"));
    assert!(
        saiu.is_empty(),
        "abrir a caixa nao escreve nada, e saiu {saiu:?}"
    );
}

/// ⭐⭐ **UM ENUM CONTINUA A AVANÇAR, e NÃO abre caixa** — não há número para escrever ali, e o
/// clique já era o gesto de *«a opção seguinte»*. FALSIFICADO por o braço novo engolir o clique de
/// todas as espécies (o enum deixaria de avançar) ou por abrir uma caixa sobre um enum.
#[test]
fn clicking_an_enum_row_still_advances_it_and_opens_no_box() {
    let snap = card(vec![enum_param(0.0, &["Off", "Cycle", "Random"])]);
    let (st, saiu) = click_state(&snap, 0);
    assert!(st.param_edit.is_none(), "um enum nao abre caixa de numero");
    let Some(GraphIntent::SetParam { value, .. }) = saiu.first() else {
        panic!("o enum tem de avancar, e saiu {saiu:?}");
    };
    assert!((*value - 1.0).abs() < 1e-6, "avancou para {value}");
}

/// ⭐⭐⭐ **O ARRASTO TAMBÉM VOLTA À UNIDADE DO DOCUMENTO** — a barra, o número e o dedo trabalham
/// na face do artista (`px`), e a escrita converte. FALSIFICADO por o arrasto emitir o número
/// mostrado: com a face de `100 px/unidade` o objecto andaria cem vezes mais.
#[test]
fn dragging_a_faced_param_writes_the_document_unit() {
    let mut p = param("dx", 0.0, 0.1, false);
    // A face que o `motion.move::dx` tem no projecto de omissão: metros guardados, px mostrados.
    p.face_scale = 100.0;
    p.min = -1000.0;
    p.max = 1000.0;
    let snap = card(vec![p]);
    // Atravessar meia largura de cartão varre METADE da faixa mostrada (1000 px).
    let saiu = drag(&snap, 0, crate::geom::CARD_W * 0.5);
    let Some(GraphIntent::SetParam { value, .. }) = saiu.first() else {
        panic!("sem intencao: {saiu:?}");
    };
    // 1000 px de excursão ⇒ 10 unidades de mundo.
    assert!(
        (*value - 10.0).abs() < 1e-2,
        "meia largura varre 1000 px = 10 unidades, e escreveu {value}"
    );
}

/// ⭐⭐ **UM CLIQUE NUM CONTROLO DE FICHEIRO PEDE O DIÁLOGO** — o cartão **pede**, nunca abre.
///
/// ⚠️ **Antes disto ele não fazia NADA**, e essa é a espécie de defeito que só se vê medindo:
/// a row era pintada, o alvo estava registado, o dedo acertava — e o braço do `match` caía no
/// `_ => {}`. O censo do catálogo (`what_the_card_still_cannot_reach`) contava **26** controlos
/// assim; este é um dos três que saem.
///
/// FALSIFICADO por o braço voltar ao `Nothing`: a fila sai vazia e o diálogo nunca abre.
#[test]
fn a_click_on_a_file_row_asks_the_shell_for_the_dialog() {
    let snap = card(vec![file_param()]);
    let saiu = click(&snap, 0);
    let Some(GraphIntent::PickFile { param, .. }) = saiu.first() else {
        panic!("o clique tem de PEDIR o dialogo, e saiu {saiu:?}");
    };
    assert_eq!(*param, "file");
    // ⚠️ E o cartão **não escreve o caminho** — quem o escreve é a shell, depois de o artista
    // escolher. Uma intenção de escrita aqui seria o cartão a inventar um valor.
    assert!(
        !saiu
            .iter()
            .any(|i| matches!(i, GraphIntent::SetParam { .. })),
        "o cartao nao pode escrever nada por si: {saiu:?}"
    );
}

/// ⭐⭐ **UM CLIQUE NUMA ESCOLHA DE FONTE PEDE A SEGUINTE** — e o cartão **não diz qual**.
///
/// ⚠️ **A lista é VIVA** (ela muda quando o artista desenha outra forma), e mandá-la para dentro
/// do cartão poria uma `Vec<String>` por row num snapshot que a medição do ciclo 1 manteve sem
/// **uma única** alocação. O cartão pede *«a seguinte»*; quem sabe quais há é a shell.
///
/// FALSIFICADO por o braço voltar ao `Nothing`, ou por o cartão tentar escrever um nome (que
/// seria inventar um valor que ele não pode conhecer).
#[test]
fn a_click_on_a_source_row_asks_for_the_next_published_name() {
    let snap = card(vec![source_param()]);
    let saiu = click(&snap, 0);
    let Some(GraphIntent::CycleSource { param, .. }) = saiu.first() else {
        panic!("o clique tem de pedir a fonte seguinte, e saiu {saiu:?}");
    };
    assert_eq!(*param, "path");
    assert!(
        !saiu
            .iter()
            .any(|i| matches!(i, GraphIntent::SetParam { .. })),
        "o cartao nao conhece os nomes publicados: {saiu:?}"
    );
}

/// ⛔ **E o veredito do clique é a PORTA ÚNICA, lida também pelo censo.** Este gate prende as
/// duas leituras: se alguém acrescentar uma espécie ao gesto sem ela aparecer no `click_does`,
/// o censo do catálogo continua a contá-la como inalcançável e a conta mente para o lado
/// perigoso — *o painel lateral sairia com um controlo morto atrás*.
#[test]
fn the_click_law_answers_for_every_species_the_gesture_handles() {
    use crate::{ClickDoes, click_does};
    assert_eq!(click_does(&file_param()), ClickDoes::PickFile);
    assert_eq!(click_does(&source_param()), ClickDoes::CycleSource);
    assert_eq!(
        click_does(&enum_param(0.0, &["A", "B"])),
        ClickDoes::Cycle(2)
    );
    assert_eq!(click_does(&toggle_param(0.0)), ClickDoes::Toggle);
    assert_eq!(click_does(&param("size", 1.0, 0.1, false)), ClickDoes::Type);
}

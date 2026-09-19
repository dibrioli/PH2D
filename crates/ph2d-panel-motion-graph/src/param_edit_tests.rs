//! ⭐⭐⭐ **ESCREVER UM NÚMERO NO CARTÃO** — os gates do report do Enio de 2026-09-05
//! (*«vários nós não permitem clicar no número para usar o teclado para escrever»*).
//!
//! ⚠️ **Quatro perguntas independentes**, e é preciso uma por cada: a caixa ABRE ao clique · ela
//! abre com a faixa e o número CERTOS · ela CHEGA A PIXEL · e o que se escreve nela CHEGA AO
//! DOCUMENTO. Um gate só sobre a primeira deixa passar uma caixa que abre vazia, ou invisível, ou
//! muda — e as três já aconteceram neste módulo.

use crate::snapshot::{
    CardParam, GraphIntent, GraphNodeView, GraphViewSnapshot, NodeViewKind, PortView, drain_intents,
};
use crate::state::{MotionGraphPanelState, ParamEdit, ViewState};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::TextInputState;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory, ParamUiHint, ParamWidget};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

/// A tela dos gates que precisam de geometria.
const RECT: ph2d_editor_core::zones::Rect = ph2d_editor_core::zones::Rect {
    x: 0.0,
    y: 0.0,
    w: 800.0, // LITERAL-PX-OK: tela de teste
    h: 600.0, // LITERAL-PX-OK: tela de teste
};

fn hint(widget: ParamWidget) -> ParamUiHint {
    ParamUiHint {
        param: "rows",
        label: "Rows",
        min: 1.0,
        max: 20.0,
        step: 0.1,
        widget,
    }
}

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

/// Arma a caixa sobre a row 0 de `snap` — o que o clique faz, sem o dispatch pelo meio (esse é
/// gateado do lado do GESTO, em `interact_param_row_tests`).
fn arm_row0(snap: &GraphViewSnapshot) -> MotionGraphPanelState {
    let mut st = MotionGraphPanelState::default();
    crate::param_edit::arm(&mut st, 7, 0, &snap.nodes[0].params[0]);
    st
}

/// ⭐⭐⭐ **A CAIXA ABRE COM A FAIXA DIGITÁVEL, não com a do arrasto** — a lei do doc 88, que o
/// painel pagou com um report (*«Scale não aceita mais que 4 na caixa de texto e 4 não é quase
/// nada para rot»*). FALSIFICADO por semear a caixa com `hint.min`/`hint.max`: o teclado passaria
/// a recusar exactamente os números por que a faixa larga existe.
#[test]
fn the_box_offers_the_typed_range_and_not_the_drag_one() {
    let mut p = CardParam::from_hint(hint(ParamWidget::Slider), 5.0);
    p.hard_max = 1_000_000.0;
    let snap = card(vec![p]);
    let e = arm_row0(&snap).param_edit.expect("caixa");
    assert!(
        (e.max - 1_000_000.0).abs() < 1e-3,
        "a caixa aceita ate' o tecto digitavel, e aceitou {}",
        e.max
    );
}

/// ⭐⭐ **A CAIXA ABRE COM O NÚMERO INTEIRO, não com as duas casas que a faixa mostra.**
///
/// A row escreve `0.50` porque a faixa do cartão é estreita; comitar essa LEITURA destruiria um
/// `0.503` em quem só abriu a caixa e carregou `Enter`. FALSIFICADO por semear com o texto da
/// row: a semente lê-se `0.50` e o valor autorado perde-se ao primeiro `Enter`.
#[test]
fn the_seed_is_the_number_and_not_the_two_decimals_the_row_shows() {
    let snap = card(vec![CardParam::from_hint(hint(ParamWidget::Slider), 0.503)]);
    let e = arm_row0(&snap).param_edit.expect("caixa");
    assert_ne!(e.seed, "0.50", "a semente nao e' o texto da row");
    assert!(
        e.seed.starts_with("0.503"),
        "a semente e' o numero vivo, e foi {}",
        e.seed
    );
}

/// ⭐⭐ **A CAIXA MORRE COM O SUJEITO** — se a faixa `row` já não é aquele param (a secção dobrou,
/// o nó mudou, o artista entrou noutro nível), ela não tem onde se desenhar.
///
/// FALSIFICADO por `box_rect` confiar no índice de faixa: a caixa continuaria aberta, a escrever
/// noutro param, e ainda com o teclado — que é a forma de um defeito que come todos os atalhos.
#[test]
fn the_box_dies_when_that_row_no_longer_holds_that_param() {
    let snap = card(vec![CardParam::from_hint(
        ParamUiHint {
            param: "cols",
            ..hint(ParamWidget::Slider)
        },
        5.0,
    )]);
    let edit = ParamEdit {
        node: 7,
        row: 0,
        param: "rows", // outro param na mesma faixa
        seed: "5".into(),
        min: 1.0,
        max: 20.0,
        step: 0.1,
        face_scale: 1.0,
        text: false,
        opened: true,
    };
    let view = crate::geom::View::new(RECT, ViewState::default());
    assert!(
        crate::param_edit::box_rect(&edit, &snap, &view).is_none(),
        "a faixa 0 ja' nao e' o `rows`: a caixa tem de fechar"
    );
    // E o controlo: com o param certo ela TEM sítio (senão o gate acima passaria por vácuo).
    let edit_certo = ParamEdit {
        param: "cols",
        ..edit
    };
    assert!(
        crate::param_edit::box_rect(&edit_certo, &snap, &view).is_some(),
        "com o param certo a caixa tem de ter sitio"
    );
}

/// Um `WidgetStore` com a caixa registada a valer `v`.
fn store_com(v: f64) -> WidgetStore {
    let mut store = WidgetStore::default();
    store.register(
        crate::hits::param_edit_id(),
        InteractiveState::NumberInput {
            state: TextInputState::Focused,
            value: v,
            buffer: format!("{v}"),
            caret: 0,
            last_committed: v,
            selection_anchor: None,
        },
    );
    store
}

/// ⭐⭐⭐ **O NÚMERO ESCRITO CHEGA AO DOCUMENTO** — pela MESMA porta do arrasto e da row do painel.
/// FALSIFICADO por o `commit` não emitir, ou emitir para o param errado.
#[test]
fn the_typed_number_reaches_the_document() {
    let _ = drain_intents();
    let st = MotionGraphPanelState {
        param_edit: Some(ParamEdit {
            node: 7,
            row: 0,
            param: "rows",
            seed: "5".into(),
            min: 1.0,
            max: 1_000_000.0,
            step: 0.1,
            face_scale: 1.0,
            text: false,
            opened: true,
        }),
        ..MotionGraphPanelState::default()
    };
    crate::param_edit::commit(&st, &store_com(1250.0));
    let saiu = drain_intents();
    let Some(GraphIntent::SetParam { node, param, value }) = saiu.first() else {
        panic!("o commit tem de emitir um SetParam, e saiu {saiu:?}");
    };
    assert_eq!((*node, *param), (7, "rows"));
    assert!((*value - 1250.0).abs() < 1e-3, "escreveu {value}");
}

/// ⭐ **UM PARAM INTEIRO COMITA UM INTEIRO** — a mesma leitura do passo que o arrasto faz.
/// FALSIFICADO por o commit não arredondar: um `Rows` ficaria a `3,7`.
#[test]
fn an_integer_param_commits_a_whole_number() {
    let _ = drain_intents();
    let st = MotionGraphPanelState {
        param_edit: Some(ParamEdit {
            node: 7,
            row: 0,
            param: "rows",
            seed: "3".into(),
            min: 1.0,
            max: 20.0,
            step: 1.0,
            face_scale: 1.0,
            text: false,
            opened: true,
        }),
        ..MotionGraphPanelState::default()
    };
    crate::param_edit::commit(&st, &store_com(3.7));
    let saiu = drain_intents();
    let Some(GraphIntent::SetParam { value, .. }) = saiu.first() else {
        panic!("sem intencao: {saiu:?}");
    };
    assert!((*value - 4.0).abs() < 1e-6, "arredondou para {value}");
}

/// A cena pintada com e sem a caixa aberta: `(glifos, segmentos)`.
fn painted(com_caixa: bool) -> (u32, u32) {
    use crate::MotionGraphPanel;
    use crate::snapshot::set_current_motion_graph;
    let snap = card(vec![CardParam::from_hint(hint(ParamWidget::Slider), 5.0)]);
    set_current_motion_graph(Some(snap.clone()));
    let viewport = ph2d_editor_core::zones::Rect::new(0.0, 0.0, RECT.w, RECT.h);
    let mut layout = ph2d_editor_core::screens::layout::HeroLayout::for_viewport(viewport);
    // Sem isto o painel do split recebe área ZERO e o gate fica vácuo.
    layout.motion_graph = viewport;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState {
        // Senão o painel ENQUADRA o grafo e o zoom deixa de ser o que o teste pediu.
        fitted: true,
        param_edit: com_caixa.then(|| ParamEdit {
            node: 7,
            row: 0,
            param: "rows",
            seed: "5".into(),
            min: 1.0,
            max: 20.0,
            step: 0.1,
            face_scale: 1.0,
            text: false,
            opened: true,
        }),
        ..MotionGraphPanelState::default()
    };
    let out =
        host.paint_and_count_geometry_with_layout::<MotionGraphPanel>(&mut state, layout, viewport);
    set_current_motion_graph(None);
    out
}

/// ⭐⭐⭐ **A CAIXA CHEGA A PIXEL** — e é o gate que este módulo já teve de aprender **duas** vezes:
/// uma faixa RESERVADA não é uma faixa PINTADA, e seis gates de geometria ficaram verdes sobre um
/// ecrã em branco. O oráculo é a cena do Vello: **glifos** (o número dentro da caixa) e
/// **segmentos** (a moldura dela).
///
/// FALSIFICADO por não chamar o pintor da caixa no `paint`, ou por chamá-lo antes do registo dos
/// hits (aí ela seria desenhada e o clique dentro dela iria para a row por baixo).
#[test]
fn the_typing_box_reaches_pixel_over_the_card() {
    let (g_sem, s_sem) = painted(false);
    let (g_com, s_com) = painted(true);
    assert!(
        s_com > s_sem,
        "a moldura da caixa tem de emitir geometria ({s_com} contra {s_sem})"
    );
    assert!(
        g_com > g_sem,
        "o numero dentro da caixa tem de emitir glifos ({g_com} contra {g_sem})"
    );
}

/// ⭐⭐⭐ **O QUE SE ESCREVE ESTÁ NA UNIDADE DO ARTISTA; O QUE CHEGA AO DOCUMENTO, NA DELE.**
///
/// Um comprimento do mundo guarda-se em **metros** e o artista trabalha em **pixels** — o painel
/// mostra `94 px` onde o documento tem `0,94`, e **109 de 454** rows escalares do catálogo são
/// assim. FALSIFICADO por comitar o número cru: escrever `94` num `Move X` mandaria o objecto
/// cem vezes mais longe do que o painel manda, e nada no ecrã diria porquê.
#[test]
fn a_typed_number_in_the_artists_unit_reaches_the_document_in_the_documents_unit() {
    let _ = drain_intents();
    let st = MotionGraphPanelState {
        param_edit: Some(ParamEdit {
            node: 7,
            row: 0,
            param: "dx",
            seed: "0".into(),
            min: -1000.0,
            max: 1000.0,
            step: 0.1,
            face_scale: 100.0, // 100 px por unidade de mundo
            text: false,
            opened: true,
        }),
        ..MotionGraphPanelState::default()
    };
    crate::param_edit::commit(&st, &store_com(94.0));
    let saiu = drain_intents();
    let Some(GraphIntent::SetParam { value, .. }) = saiu.first() else {
        panic!("sem intencao: {saiu:?}");
    };
    assert!(
        (*value - 0.94).abs() < 1e-5,
        "94 px tem de virar 0,94 no documento, e virou {value}"
    );
}

/// Uma fórmula mais comprida do que a row consegue mostrar — é ela que faz a diferença entre
/// **semear com o inteiro** e semear com o que se lê.
const LONGO: &str = "clamp(sin(t * 2.0) * amplitude + offset, min, max)";

/// ⭐⭐⭐ **A CAIXA DE TEXTO ABRE COM O VALOR INTEIRO, NUNCA COM O QUE A ROW MOSTRA.**
///
/// ⛔⛔ **É a armadilha do número uma letra acima.** A row do cartão carrega um
/// [`crate::RowText`] — o que **cabe** na largura —, e semear a caixa com ele faria um `Enter`
/// distraído gravar meia fórmula. Ali perdiam-se casas decimais; aqui perde-se metade do valor.
///
/// ⚠️ **O controlo é a primeira asserção**: sem ela o gate seria vácuo no dia em que o texto
/// coubesse inteiro na row, e passaria a não medir nada.
///
/// FALSIFICADO por `arm_text` semear a partir do [`CardParam`] em vez do canal lateral.
#[test]
fn the_text_box_opens_with_the_whole_value_never_with_the_row_text() {
    assert_ne!(
        crate::RowText::new(LONGO).as_str(),
        LONGO,
        "controle: este valor TEM de ser truncado pela row, senao o gate nao mede nada"
    );
    crate::snapshot::set_card_texts(vec![(7, "expr", LONGO.to_string())]);
    let mut state = MotionGraphPanelState::default();
    crate::param_edit::arm_text(&mut state, 7, 0, "expr");
    let e = state.param_edit.as_ref().expect("a caixa abre");
    assert!(e.text, "e' uma caixa de TEXTO");
    assert_eq!(e.seed, LONGO, "a semente e' o valor INTEIRO");
    crate::snapshot::set_card_texts(Vec::new());
}

/// ⛔ **SEM SEMENTE PUBLICADA A CAIXA NÃO ABRE** — abrir vazia sobre um valor que existe
/// apagá-lo-ia com um `Enter`. ⚠️ E um param de texto **vazio** continua a poder receber o
/// primeiro caractere: o vazio publicado É uma semente.
///
/// FALSIFICADO por `arm_text` cair num `unwrap_or_default()` quando o canal não tem o par.
#[test]
fn no_published_text_no_box_but_an_empty_one_still_opens() {
    crate::snapshot::set_card_texts(Vec::new());
    let mut state = MotionGraphPanelState::default();
    crate::param_edit::arm_text(&mut state, 7, 0, "expr");
    assert!(
        state.param_edit.is_none(),
        "sem valor publicado a caixa nao pode abrir"
    );
    crate::snapshot::set_card_texts(vec![(7, "expr", String::new())]);
    crate::param_edit::arm_text(&mut state, 7, 0, "expr");
    assert!(
        state.param_edit.is_some(),
        "um param de texto VAZIO ainda tem de aceitar o primeiro caractere"
    );
    crate::snapshot::set_card_texts(Vec::new());
}

/// ⭐⭐⭐ **O QUE SE ESCREVE CHEGA AO DOCUMENTO — pelo EVENTO, não pela função.**
///
/// ⛔⛔ **A 1.ª versão deste gate chamava `param_edit::commit` directamente, e passou verde
/// sobre o produto quebrado** (report do Enio, 2026-09-07: *«não funcionou. Tempo permaneceu»*).
/// O commit estava ligado a `WidgetEvent::ValueChanged`, que é o que um **`NumberInput`** produz
/// depois de analisar o buffer como número — uma caixa de TEXTO comita por **`Submit`**, e esse
/// braço não existia: o `Enter` caía no `_ => Ignored`.
///
/// *O cabeçalho deste ficheiro já avisava — «um teste que empurra o gesto já assumiu a
/// resposta» — e eu escrevi o gate que ele proíbe.*
///
/// FALSIFICADO por apagar o braço `WidgetEvent::Submit(param_edit_id())` do `apply_event`.
#[test]
fn what_is_typed_leaves_by_the_text_door() {
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::PanelHostInternal;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::MotionGraphPanel>();
    crate::snapshot::set_card_texts(vec![(7, "expr", LONGO.to_string())]);
    let mut state = MotionGraphPanelState::default();
    crate::param_edit::arm_text(&mut state, 7, 0, "expr");
    crate::snapshot::set_card_texts(Vec::new());
    host.store_mut().register(
        crate::hits::param_edit_id(),
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: "vendas".to_string(),
            caret: 6,
            selection_anchor: None,
        },
    );
    let _ = drain_intents();
    let saida = host.apply_panel_event::<crate::MotionGraphPanel>(
        &mut state,
        WidgetEvent::Submit(crate::hits::param_edit_id()),
    );
    assert_eq!(
        saida,
        ph2d_editor_core::panel::EventOutcome::Consumed,
        "o painel tem de CONSUMIR o Submit da sua propria caixa"
    );
    let saiu = drain_intents();
    let Some(GraphIntent::SetTextParam { param, value, .. }) = saiu.first() else {
        panic!("o texto tem de sair pela porta de TEXTO, e saiu {saiu:?}");
    };
    assert_eq!(*param, "expr");
    assert_eq!(value, "vendas");
}

/// ⚠️ **E o `Esc` continua a fechar pelo `Blur`** — a lei que o `apply_event` escreve ao lado:
/// *o `Blur` é o ÚNICO fecho, e tem de ser*, porque é o que os três caminhos têm em comum.
///
/// ⛔ A 1.ª versão da caixa de texto chamava `mark_cancel_on_escape`, que manda o `Esc` para um
/// braço `Cancel` **que não existe** — a caixa ficava no ecrã a comer o teclado.
///
/// FALSIFICADO por voltar a marcar o Esc: o `Blur` deixa de ser o que a fecha.
#[test]
fn the_text_box_closes_by_blur_like_the_number_one() {
    use ph2d_editor_core::interaction::WidgetEvent;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::MotionGraphPanel>();
    crate::snapshot::set_card_texts(vec![(7, "expr", "sin(t)".to_string())]);
    let mut state = MotionGraphPanelState::default();
    crate::param_edit::arm_text(&mut state, 7, 0, "expr");
    crate::snapshot::set_card_texts(Vec::new());
    assert!(state.param_edit.is_some(), "a caixa abriu");
    let saida = host.apply_panel_event::<crate::MotionGraphPanel>(
        &mut state,
        WidgetEvent::Blur(crate::hits::param_edit_id()),
    );
    assert_eq!(saida, ph2d_editor_core::panel::EventOutcome::Consumed);
    assert!(state.param_edit.is_none(), "o Blur FECHA a caixa de texto");
}

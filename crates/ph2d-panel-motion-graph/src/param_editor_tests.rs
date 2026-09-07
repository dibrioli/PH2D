//! ⭐⭐⭐ **OS GATES DA JANELA DO EDITOR RICO** — a curva que o cartão passou a alcançar.
//!
//! ⚠️ Irmão de [`super`] por responsabilidade: lá vive a janela, aqui as três perguntas que só o
//! gesto responde — *abre?*, *o que se arrasta lá dentro CHEGA ao documento?*, *fecha?*

use super::{EditorKind, Open, key_of, on_click, on_drag, window};
use crate::snapshot::{CardParam, GraphIntent, drain_intents, set_card_texts};
use crate::state::MotionGraphPanelState;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::zones::Rect;
use ph2d_node_registry::{ParamUiHint, ParamWidget};
use ph2d_param_editors::EditorKey;

const NODE: u32 = 7;
const PARAM: &str = "curve";
/// A identidade, serializada — o que um param de curva ainda por autorar publica não é isto
/// (é `""`), e os dois abrem na mesma diagonal.
const DIAGONAL: &str = "c1 0:0:L 1:1:L";

fn curve_param() -> CardParam {
    CardParam::from_hint(
        ParamUiHint {
            param: "curve",
            label: "Curve",
            min: 0.0,
            max: 0.0,
            step: 0.0,
            widget: ParamWidget::Curve,
        },
        0.0,
    )
}

fn aberto() -> MotionGraphPanelState {
    MotionGraphPanelState {
        editor: Some(Open {
            node: NODE,
            param: PARAM,
            title: "Curve",
            kind: EditorKind::Curve,
            screen: (100.0, 100.0),
        }),
        ..MotionGraphPanelState::default()
    }
}

fn chave() -> (String, String) {
    key_of(NODE, PARAM)
}

/// ⭐⭐⭐ **UM CLIQUE NUMA ROW DE CURVA ABRE A JANELA — e não escreve nada.**
///
/// ⚠️ **Sem valor publicado ela NÃO abre**, e é a mesma lei da caixa de texto: um editor semeado
/// com o vazio sobre uma curva que existe apagá-la-ia no primeiro arrasto. ⛔ Um param ainda por
/// autorar publica `""` — que É uma semente, e abre na identidade.
///
/// FALSIFICADO por o braço `OpensEditor` do gesto voltar ao `Nothing` (a row volta a ser um selo
/// que não abre — os dois controlos que este censo contava).
#[test]
fn a_click_on_a_curve_row_opens_the_window_and_writes_nothing() {
    let mut st = MotionGraphPanelState::default();
    let p = curve_param();
    let _ = drain_intents();

    // (a) Sem semente publicada, nada abre.
    set_card_texts(Vec::new());
    super::arm(&mut st, NODE, &p, Rect::new(10.0, 10.0, 190.0, 22.0));
    assert!(
        st.editor.is_none(),
        "sem valor publicado a janela nao pode abrir"
    );

    // (b) Com semente, abre — e o documento fica intacto.
    set_card_texts(vec![(NODE, "curve", DIAGONAL.to_string())]);
    super::arm(&mut st, NODE, &p, Rect::new(10.0, 10.0, 190.0, 22.0));
    let aberto = st.editor.as_ref().expect("a janela abre");
    assert_eq!((aberto.node, aberto.param), (NODE, PARAM));
    assert!(
        drain_intents().is_empty(),
        "abrir uma janela nao e' uma edicao"
    );
    set_card_texts(Vec::new());
}

/// ⭐⭐⭐ **O QUE SE ARRASTA LÁ DENTRO CHEGA AO DOCUMENTO** — a metade que a abertura não prova.
///
/// O despacho do `editor-core` guarda o ponto normalizado num canal global e emite
/// `ValueChanged(raiz)`; este gate percorre esse caminho e mede a intenção que sai.
///
/// ⚠️⚠️ **Ele percorre o `apply_event` REAL do painel, e não chama o `on_drag`**: um gate que
/// empurrasse a função directamente já teria assumido que alguém a chama — e foi exactamente
/// esse o defeito que o `Enter` da caixa de texto pagou (o braço não existia e o gate estava
/// verde).
///
/// FALSIFICADO por apagar o braço `WidgetEvent::ValueChanged` que chama o editor (a alça move-se
/// no ecrã, o desenho volta ao que era no quadro seguinte, e nada diz porquê).
#[test]
fn dragging_a_handle_writes_the_curve_through_the_text_door() {
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::PanelHostInternal;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::MotionGraphPanel>();
    let mut st = aberto();
    set_card_texts(vec![(
        NODE,
        "curve",
        "c1 0:0:L 0.5:0.5:L 1:1:L".to_string(),
    )]);
    let _ = drain_intents();
    let (own, swatch) = chave();
    let key = EditorKey {
        own: &own,
        swatch: &swatch,
    };
    // O ponto do MEIO, arrastado para cima — como o despacho o deixa.
    host.store_mut()
        .set_curve_point_drag(key.root(), 0, 1, 0.5, 0.9);
    let saida = host.apply_panel_event::<crate::MotionGraphPanel>(
        &mut st,
        WidgetEvent::ValueChanged(key.root()),
    );
    assert_eq!(
        saida,
        ph2d_editor_core::panel::EventOutcome::Consumed,
        "o painel tem de CONSUMIR o arrasto da sua propria janela"
    );
    let saiu = drain_intents();
    let Some(GraphIntent::SetTextParam { node, param, value }) = saiu.first() else {
        panic!("o arrasto tem de escrever a curva, e saiu {saiu:?}");
    };
    assert_eq!((*node, *param), (NODE, PARAM));
    let c = ph2d_curve::parse(value).expect("a curva sai bem formada");
    assert!(
        (c.points[1].y - 0.9).abs() < 1e-6,
        "o y arrastado tem de chegar: {:?}",
        c.points[1]
    );
    set_card_texts(Vec::new());
}

/// ⭐⭐ **UM ARRASTO DE OUTRO EDITOR NÃO É DESTE** — a metade que impede um editor de roubar o
/// arrasto do vizinho.
///
/// ⚠️ O canal do arrasto é **global** (o painel lateral usa o mesmo), então a pergunta *«este
/// arrasto é meu?»* faz parte da chamada. Um `take` incondicional seria irreversível.
#[test]
fn a_drag_that_belongs_to_another_editor_is_left_alone() {
    let st = aberto();
    set_card_texts(vec![(NODE, "curve", DIAGONAL.to_string())]);
    let mut store = WidgetStore::with_capacity(4);
    let outro = ph2d_param_editors::fnv_id("motion_param/curve/0");
    store.set_curve_point_drag(outro, 0, 1, 0.5, 0.9);
    assert!(
        !on_drag(&st, &mut store, outro),
        "o arrasto do painel lateral nao e' deste editor"
    );
    assert!(
        store.take_curve_point_drag_if(|p| p == outro).is_some(),
        "e fica LA' para o dono dele o drenar"
    );
    set_card_texts(Vec::new());
}

/// ⭐⭐ **OS BOTÕES DA JANELA EDITAM A CURVA** — `+` acrescenta um ponto, `−` tira-o, o interp
/// cicla. Todos pela mesma porta de texto.
///
/// FALSIFICADO por o braço de `Click` do painel não chamar o editor (os três botões ficam
/// pintados e mudos — a espécie de controlo morto que este módulo existe para não ter).
#[test]
fn the_windows_buttons_edit_the_curve() {
    let st = aberto();
    set_card_texts(vec![(NODE, "curve", DIAGONAL.to_string())]);
    let (own, swatch) = chave();
    let key = EditorKey {
        own: &own,
        swatch: &swatch,
    };
    let escreve = |id| {
        let _ = drain_intents();
        assert!(on_click(&st, id), "o botao e' deste editor");
        drain_intents().into_iter().find_map(|i| match i {
            GraphIntent::SetTextParam { value, .. } => Some(value),
            _ => None,
        })
    };
    let mais = escreve(key.sub("add")).expect("o `+` escreve");
    assert_eq!(
        ph2d_curve::parse(&mais).expect("bem formada").points.len(),
        3,
        "o `+` acrescenta um ponto a` diagonal"
    );
    let interp = escreve(key.sub("interp")).expect("o interp escreve");
    assert_ne!(interp, DIAGONAL, "o interp muda alguma coisa");
    // ⛔ Um id que não é de nenhum botão desta janela não é consumido — senão o editor comia
    // cliques do grafo por baixo dele.
    let _ = drain_intents();
    assert!(
        !on_click(&st, ph2d_param_editors::fnv_id("um/id/qualquer")),
        "um id alheio nao e' deste editor"
    );
    set_card_texts(Vec::new());
}

/// ⭐⭐ **A JANELA FICA DENTRO DO CANVAS** — aberta junto à borda, ela encosta para dentro.
///
/// ⚠️ *Uma janela meio fora do canvas tem metade dos controlos inalcançáveis*, e o `+` e o `−`
/// vivem exactamente na borda direita dela.
#[test]
fn the_window_is_pushed_back_inside_the_canvas() {
    let canvas = Rect::new(0.0, 0.0, 400.0, 300.0);
    let mut st = aberto();
    if let Some(o) = st.editor.as_mut() {
        o.screen = (10_000.0, 10_000.0);
    }
    let w = window(&st, canvas).expect("aberta");
    assert!(
        w.x >= canvas.x && w.x + w.w <= canvas.x + canvas.w,
        "cabe em x: {w:?}"
    );
    assert!(
        w.y >= canvas.y && w.y + w.h <= canvas.y + canvas.h,
        "cabe em y: {w:?}"
    );
}

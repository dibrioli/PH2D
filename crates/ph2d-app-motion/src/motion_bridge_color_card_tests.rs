//! ⭐⭐⭐ **A COR ESCOLHIDA NUMA JANELA DE CARTÃO CHEGA À STRING** — a metade que o censo do
//! `panel_exit_probe` **não vê**.
//!
//! ⚠️⚠️ Aquele censo mede o que um clique numa row FAZ, então ele conta um gradiente como
//! alcançável no dia em que a janela abre. *Um editor que abre, aceita o gesto e não guarda é a
//! espécie de controlo morto que uma sonda de alcance lê como vivo* — e a distância entre as
//! duas coisas, aqui, é a leitura de volta do selector, que vive na shell.

use crate::motion_state::MotionState;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ChannelMode, Harmony, InterpolationMode};
use ph2d_panel_motion_graph::card_editor_swatch_id;
use ph2d_tokens::ColorValue;

/// Um selector ABERTO sobre a amostra `i` de `(nó, param)`, já com a cor `rgba` escolhida.
fn picker_on(node: u32, param: &str, i: usize, rgba: [u8; 4]) -> WidgetStore {
    let mut store = WidgetStore::default();
    store.register(
        ph2d_editor_core::ids::INSP_BLENDER_PICKER,
        InteractiveState::BlenderPicker {
            value: ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
            channel_mode: ChannelMode::Rgb,
            interpolation: InterpolationMode::Linear,
            active_palette: 0,
            hsv_h: 0.0,
            hsv_s: 0.0,
            harmony: Harmony::None,
        },
    );
    store.set_picker_target(Some(card_editor_swatch_id(node, param, i)));
    store
}

fn texto(motion: &MotionState, nid: ph2d_nodegraph::graph::NodeId, param: &str) -> String {
    motion
        .doc
        .graph
        .node_text_param_overrides(nid)
        .and_then(|m| m.get(param))
        .cloned()
        .unwrap_or_default()
}

/// ⭐⭐⭐ **A PARADA ESCOLHIDA MUDA DE COR, e o resto da rampa fica.**
///
/// FALSIFICADO por a leitura de volta não consultar o cartão (a janela abre, o selector abre, o
/// artista escolhe — e a barra continua igual, sem uma palavra).
#[test]
fn a_pick_on_a_cards_gradient_stop_reaches_the_string() {
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("motion.color_ramp");
    m.doc
        .graph
        .set_text_param(id, "ramp", "g1 2 0:1,0,0 1:0,0,1".to_string());
    let store = picker_on(id.0, "ramp", 0, [0, 255, 0, 255]);
    super::apply_picker_readback(&mut m, &store);
    let saiu = texto(&m, id, "ramp");
    let rampa = ph2d_color::parse_gradient(&saiu).expect("a rampa sai bem formada");
    assert!(
        rampa.stops()[0].color[1] > 0.9 && rampa.stops()[0].color[0] < 0.1,
        "a 1.a parada tem de ficar VERDE: {saiu}"
    );
    assert!(
        rampa.stops()[1].color[2] > 0.9,
        "e a 2.a fica azul, como estava: {saiu}"
    );
}

/// ⭐⭐⭐ **DOIS CARTÕES DO MESMO TIPO NÃO PARTILHAM A AMOSTRA** — o defeito que a amostra de cor
/// simples já pagou uma vez, um nível acima.
///
/// ⚠️ O id de uma parada no PAINEL é função só do nome do param (*«unique within a node»*, e é
/// verdade lá, onde há um nó seleccionado). No cartão há vinte, e dois `motion.color_ramp` na
/// tela pediriam o mesmo widget: escolher a cor de um escreveria no outro, **em silêncio**.
///
/// FALSIFICADO por o `card_editor_swatch_id` deixar de levar o nó.
#[test]
fn two_cards_of_the_same_type_never_share_a_stop_swatch() {
    let mut m = MotionState::new();
    let a = m.doc.graph.add_node("motion.color_ramp");
    let b = m.doc.graph.add_node("motion.color_ramp");
    for n in [a, b] {
        m.doc
            .graph
            .set_text_param(n, "ramp", "g1 2 0:1,0,0 1:0,0,1".to_string());
    }
    assert_ne!(
        card_editor_swatch_id(a.0, "ramp", 0),
        card_editor_swatch_id(b.0, "ramp", 0),
        "duas rampas pediriam o MESMO selector"
    );
    // O selector aponta ao cartão de B — a cor tem de ir a B, com A por seleccionar.
    let store = picker_on(b.0, "ramp", 0, [0, 255, 0, 255]);
    super::apply_picker_readback(&mut m, &store);
    let (ta, tb) = (texto(&m, a, "ramp"), texto(&m, b, "ramp"));
    assert!(
        tb.contains("0,1,0")
            || ph2d_color::parse_gradient(&tb).expect("ok").stops()[0].color[1] > 0.9,
        "B tem de ficar verde: {tb}"
    );
    assert_eq!(ta, "g1 2 0:1,0,0 1:0,0,1", "e A tem de ficar INTACTO: {ta}");
}

/// ⭐⭐ **E a COR de uma paleta também** — a irmã, que escreve outra serialização.
///
/// ⚠️ **Uma paleta por autorar tem as cores DE FÁBRICA**, e elas são escolhíveis: contar `0`
/// amostras deixaria a primeira sem dono, e a escolha do artista evaporava no primeiro clique
/// que ele dá num nó acabado de criar.
#[test]
fn a_pick_on_a_cards_palette_colour_reaches_the_string_even_unauthored() {
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("motion.color_array");
    assert_eq!(texto(&m, id, "palette"), "", "a fixtura nasce por autorar");
    let store = picker_on(id.0, "palette", 0, [0, 255, 0, 255]);
    super::apply_picker_readback(&mut m, &store);
    let saiu = texto(&m, id, "palette");
    let cores = ph2d_color::parse_palette(&saiu).expect("a paleta sai bem formada");
    assert!(
        cores[0][1] > 0.9 && cores[0][0] < 0.1,
        "a 1.a cor tem de ficar VERDE: {saiu}"
    );
    assert_eq!(
        cores.len(),
        ph2d_color::DEFAULT_PALETTE_FALLBACK.len(),
        "e as outras de fabrica ficam: {saiu}"
    );
}

/// ⛔ **UM SELECTOR ABERTO NOUTRA COISA NÃO É DESTE CAMINHO** — sem isto a varredura escreveria
/// numa rampa qualquer sempre que o artista abrisse o selector do pincel.
#[test]
fn a_picker_open_on_something_else_writes_no_gradient() {
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("motion.color_ramp");
    m.doc
        .graph
        .set_text_param(id, "ramp", "g1 2 0:1,0,0 1:0,0,1".to_string());
    let mut store = picker_on(id.0, "ramp", 0, [0, 255, 0, 255]);
    store.set_picker_target(Some(ph2d_editor_core::ids::PAINTER_COLOR_THUMB));
    super::apply_picker_readback(&mut m, &store);
    assert_eq!(
        texto(&m, id, "ramp"),
        "g1 2 0:1,0,0 1:0,0,1",
        "a rampa nao pode mudar por causa de um selector alheio"
    );
}

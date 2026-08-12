//! Gates da porta única do scrub — ver o doc-header de [`super`].
//!
//! ⚠️ Estes são gates de MODELO (a lei, sem ponteiro). A metade que prova que o `pointer_move` de
//! facto *consulta* a porta vive em `dispatch::tests::number_drag`, porque **um gate de unidade é
//! cego à fiação**: dá para ter esta lei perfeita e um dispatch que continua a calcular a taxa por
//! conta própria, com esta suíte toda verde.

use super::*;
use crate::interaction::InteractiveState;
use crate::widget::{SliderOrientation, SliderState, TextInputState};

/// Uma caixa registada, com o buffer que o painel de facto escreveria.
fn chip(store: &mut WidgetStore, id: NodeId, value: f64, buffer: &str) {
    store.register(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value,
            buffer: buffer.into(),
            caret: 0,
            last_committed: value,
            selection_anchor: None,
        },
    );
}

fn slider(store: &mut WidgetStore, id: NodeId, value: f32) {
    store.register(
        id,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value,
            orientation: SliderOrientation::Horizontal,
        },
    );
}

/// ⭐ **A propriedade central: a taxa e o clamp saem do MESMO intervalo.**
///
/// Enunciada sobre o RESULTADO e não sobre a travessia — `rate_x · DRAG_RANGE_PX_H` tem de ser a
/// largura que o clamp usa, seja qual for a fonte. É isto que torna impossível uma quinta fonte
/// nascer conhecida por um consumidor e não pelo outro.
///
/// *Mutação que sangra:* devolver `DRAG_RATE_X · step` no arm do slider/canal ⇒ a igualdade parte
/// nas duas últimas fixtures.
#[test]
fn the_rate_and_the_clamp_read_the_same_interval() {
    // (nome, construtor) — uma por FONTE de intervalo.
    type Fixture = (&'static str, fn(&mut WidgetStore));
    let build: [Fixture; 3] = [
        ("range", |s| {
            chip(s, NodeId(1), 0.0, "0.0");
            s.set_number_range(NodeId(1), -3.0, 3.0, 0.01);
        }),
        ("slider", |s| {
            chip(s, NodeId(1), 2.0, "2");
            slider(s, NodeId(2), 0.1);
            s.link_slider_number_mapped(NodeId(2), NodeId(1), 15.0, 1.0);
        }),
        ("channel", |s| {
            chip(s, NodeId(1), 0.5, "0.50");
            s.register(
                NodeId(3),
                InteractiveState::Button {
                    state: crate::widget::ButtonState::Normal,
                },
            );
            s.link_blender_channel(NodeId(3), NodeId(1), 0);
        }),
    ];
    for (name, make) in build {
        let mut store = WidgetStore::with_capacity(8);
        make(&mut store);
        let law = store.number_scrub_law(NodeId(1), 1.0);
        let (lo, hi) = law.bounds.unwrap_or_else(|| panic!("{name}: sem bounds"));
        let width = hi - lo;
        assert!(
            (law.rate_x * drag::DRAG_RANGE_PX_H - width).abs() < 1e-9,
            "{name}: a taxa horizontal nao mede o intervalo do clamp \
             (rate_x {} x {} = {}, largura {width})",
            law.rate_x,
            drag::DRAG_RANGE_PX_H,
            law.rate_x * drag::DRAG_RANGE_PX_H
        );
        assert!(
            (law.rate_y * drag::DRAG_RANGE_PX_V - width).abs() < 1e-9,
            "{name}: a taxa vertical nao mede o intervalo do clamp"
        );
    }
}

/// ⭐ **O número do produto: a faixa inteira atravessa-se na distância de desenho.**
///
/// A fixture é o campo REAL medido pela sonda como o pior do `upscale` — factor `[1, 16]`, que
/// cruzava em **0,30 px** (um pixel saturava-o 53 vezes). RED-first: antes da porta, `rate_x` era
/// `DRAG_RATE_X · step` = 50.
#[test]
fn a_slider_linked_chip_crosses_its_whole_interval_in_the_designed_distance() {
    let mut store = WidgetStore::with_capacity(8);
    chip(&mut store, NodeId(1), 2.0, "2");
    slider(&mut store, NodeId(2), 0.0667);
    store.link_slider_number_mapped(NodeId(2), NodeId(1), 15.0, 1.0);

    let law = store.number_scrub_law(NodeId(1), 1.0);
    let px = 15.0 / law.rate_x;
    assert!(
        (px - drag::DRAG_RANGE_PX_H).abs() < 1e-6,
        "o factor [1,16] tem de cruzar em {} px, cruza em {px:.2}",
        drag::DRAG_RANGE_PX_H
    );
    assert_eq!(law.bounds, Some((1.0, 16.0)), "e continua clampado");
}

/// O chip de canal do picker (`0..1`) — a 4a fonte, e a que o `bounds` já conhecia sozinho.
#[test]
fn a_channel_chip_crosses_its_whole_interval_in_the_designed_distance() {
    let mut store = WidgetStore::with_capacity(8);
    chip(&mut store, NodeId(1), 0.5, "0.50");
    store.register(
        NodeId(3),
        InteractiveState::Button {
            state: crate::widget::ButtonState::Normal,
        },
    );
    store.link_blender_channel(NodeId(3), NodeId(1), 0);

    let law = store.number_scrub_law(NodeId(1), 0.01);
    let px = 1.0 / law.rate_x;
    assert!(
        (px - drag::DRAG_RANGE_PX_H).abs() < 1e-6,
        "o canal 0..1 tem de cruzar em {} px, cruza em {px:.2}",
        drag::DRAG_RANGE_PX_H
    );
}

/// ⭐ **A metade que torna a mudança auditável: o que já estava certo NÃO se move.**
///
/// Os três controlos, cada um a pinar um arm que a wave promete deixar byte-idêntico. Sem eles a
/// wave seria *"mudei a lei do arrasto"* em vez de *"estendi-a a quem já era clampado"*.
#[test]
fn the_three_arms_that_already_worked_are_byte_identical() {
    // (1) faixa explícita: continua proporcional a [min,max].
    let mut store = WidgetStore::with_capacity(8);
    chip(&mut store, NodeId(1), 0.0, "0.0");
    store.set_number_range(NodeId(1), -1.0, 1.0, 0.01);
    let law = store.number_scrub_law(NodeId(1), 0.01);
    assert!((law.rate_x - 2.0 / drag::DRAG_RANGE_PX_H).abs() < 1e-12);
    assert_eq!(law.bounds, Some((-1.0, 1.0)));

    // (2) taxa registada: vence tudo e continua SEM limites.
    let mut store = WidgetStore::with_capacity(8);
    chip(&mut store, NodeId(1), 0.0, "0.0");
    slider(&mut store, NodeId(2), 0.5);
    store.link_slider_number_mapped(NodeId(2), NodeId(1), 4.0, 0.0);
    store.set_number_range(NodeId(1), -1.0, 1.0, 0.01);
    store.set_number_drag_rate(NodeId(1), 2.0);
    let law = store.number_scrub_law(NodeId(1), 1.0);
    assert!((law.rate_x - 2.0).abs() < 1e-12, "a taxa registada vence");
    assert!(
        (law.rate_y - 0.2).abs() < 1e-12,
        "e o vertical e 10x mais fino"
    );
    assert_eq!(law.bounds, None, "uma taxa registada declara ILIMITADO");

    // (3) sem intervalo nenhum (posicao em px): o atalho historico, verbatim.
    let mut store = WidgetStore::with_capacity(8);
    chip(&mut store, NodeId(1), 5.0, "5");
    let law = store.number_scrub_law(NodeId(1), 1.0);
    assert!((law.rate_x - drag::DRAG_RATE_X).abs() < 1e-12);
    assert!((law.rate_y - drag::DRAG_RATE_Y).abs() < 1e-12);
    assert_eq!(law.bounds, None);
}

/// ⚠️ **O `step` sai do BUFFER, então antes desta porta a mesma caixa arrastava 100x mais depressa
/// no dia em que o valor calhasse de renderizar sem casa decimal.** Uma caixa com intervalo deixa
/// de o consultar; o gate afirma-o comparando as duas grafias do MESMO campo.
///
/// *Mutação que sangra:* fazer o arm do intervalo multiplicar por `step`.
#[test]
fn a_bounded_box_no_longer_lets_the_buffer_decide_its_rate() {
    let mut with_dot = WidgetStore::with_capacity(8);
    chip(&mut with_dot, NodeId(1), 2.0, "2.00");
    slider(&mut with_dot, NodeId(2), 0.1);
    with_dot.link_slider_number_mapped(NodeId(2), NodeId(1), 15.0, 1.0);

    let mut plain = WidgetStore::with_capacity(8);
    chip(&mut plain, NodeId(1), 2.0, "2");
    slider(&mut plain, NodeId(2), 0.1);
    plain.link_slider_number_mapped(NodeId(2), NodeId(1), 15.0, 1.0);

    // O `step` que o Down escolheria para cada grafia: 0.01 com ponto, 1.0 sem.
    let a = with_dot.number_scrub_law(NodeId(1), 0.01);
    let b = plain.number_scrub_law(NodeId(1), 1.0);
    assert!(
        (a.rate_x - b.rate_x).abs() < 1e-12,
        "a grafia do buffer nao pode mudar a taxa de uma caixa com intervalo: {} vs {}",
        a.rate_x,
        b.rate_x
    );
}

/// A ordem dos arms É a lei: a faixa explícita vence o slider ligado.
///
/// Sem isto, um painel que registe os dois (o padrão dos chips do transporte da timeline) veria a
/// taxa a sair de uma fonte e o clamp da outra — a doença que a porta existe para fechar, de
/// volta pelo lado de dentro.
#[test]
fn an_explicit_range_outranks_the_linked_sliders_projection() {
    let mut store = WidgetStore::with_capacity(8);
    chip(&mut store, NodeId(1), 0.0, "0.0");
    slider(&mut store, NodeId(2), 0.5);
    store.link_slider_number_mapped(NodeId(2), NodeId(1), 100.0, 0.0); // projeta [0,100]
    store.set_number_range(NodeId(1), 0.0, 10.0, 0.1); // mas a faixa diz [0,10]
    let law = store.number_scrub_law(NodeId(1), 1.0);
    assert_eq!(law.bounds, Some((0.0, 10.0)));
    assert!((law.rate_x * drag::DRAG_RANGE_PX_H - 10.0).abs() < 1e-9);
}

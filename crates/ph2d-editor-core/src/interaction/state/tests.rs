use super::*;
// ⚠️ O que só o TESTE usa importa-se AQUI. Deixá-los no pai fá-los "não usados" na build de lib —
//    e foi assim que o `cargo fix --lib`, que não enxerga este filho, os podou e partiu a suíte.
use crate::widget::{SliderOrientation, SliderState, TextInputState};

fn store_with(states: &[(u64, InteractiveState)]) -> WidgetStore {
    let mut s = WidgetStore::with_capacity(64);
    for (id, st) in states {
        s.register(NodeId(*id), st.clone());
    }
    s
}

#[test]
fn blank_number_input_clears_buffer_but_keeps_value_and_respects_focus() {
    let ni = |v: f64| InteractiveState::NumberInput {
        state: TextInputState::Normal,
        value: v,
        buffer: format!("{v}"),
        caret: 0,
        last_committed: v,
        selection_anchor: None,
    };
    let mut store = store_with(&[(1, ni(42.0)), (2, ni(7.0))]);

    // BulkSelect "Mixed": blank the display, preserve value/committed.
    store.blank_number_input(NodeId(1));
    match store.get(NodeId(1)) {
        Some(InteractiveState::NumberInput {
            buffer,
            value,
            last_committed,
            ..
        }) => {
            assert!(buffer.is_empty(), "buffer not blanked: {buffer:?}");
            assert_eq!(*value, 42.0, "value must survive (clean blur revert)");
            assert_eq!(*last_committed, 42.0);
        }
        _ => panic!("not a NumberInput"),
    }

    // No-op while the field is focused (must not fight live typing).
    store.set_focus(Some(NodeId(2)));
    store.blank_number_input(NodeId(2));
    match store.get(NodeId(2)) {
        Some(InteractiveState::NumberInput { buffer, .. }) => {
            assert_eq!(buffer, "7", "focused field must not be blanked");
        }
        _ => panic!("not a NumberInput"),
    }
}

#[test]
fn register_grows_focus_order_to_match() {
    let mut store = WidgetStore::with_capacity(16);
    for i in 0..16 {
        store.register(NodeId(i as u64), InteractiveState::Plain);
    }
    assert_eq!(store.focus_order().len(), 16);
    assert_eq!(store.len(), 16);
}

#[test]
fn focus_order_matches_registration_order() {
    let store = store_with(&[
        (1, InteractiveState::Plain),
        (5, InteractiveState::Plain),
        (3, InteractiveState::Plain),
    ]);
    assert_eq!(store.focus_order(), &[NodeId(1), NodeId(5), NodeId(3)]);
}

#[test]
fn re_register_overwrites_without_growing_focus_order() {
    let mut store = WidgetStore::with_capacity(8);
    store.register(NodeId(1), InteractiveState::Plain);
    store.register(
        NodeId(1),
        InteractiveState::Button {
            state: ButtonState::Hovered,
        },
    );
    assert_eq!(store.focus_order().len(), 1);
    assert_eq!(store.button_state(NodeId(1)), Some(ButtonState::Hovered));
}

#[test]
fn collapsed_defaults_to_false() {
    let store = WidgetStore::with_capacity(4);
    assert!(!store.is_collapsed(NodeId(99)));
}

#[test]
fn collapsed_set_and_toggle() {
    let mut store = WidgetStore::with_capacity(4);
    store.set_collapsed(NodeId(7), true);
    assert!(store.is_collapsed(NodeId(7)));
    store.toggle_collapsed(NodeId(7));
    assert!(!store.is_collapsed(NodeId(7)));
    store.toggle_collapsed(NodeId(8));
    assert!(store.is_collapsed(NodeId(8)));
}

#[test]
fn convenience_getters_return_none_for_wrong_kind() {
    let store = store_with(&[(1, InteractiveState::Plain)]);
    assert!(store.button_state(NodeId(1)).is_none());
    assert!(store.slider(NodeId(1)).is_none());
}

#[test]
fn hot_active_focus_round_trip() {
    let mut store = WidgetStore::with_capacity(4);
    store.set_hot(Some(NodeId(2)));
    store.set_active(Some(NodeId(3)));
    store.set_focus(Some(NodeId(4)));
    assert_eq!(store.hot_id(), Some(NodeId(2)));
    assert_eq!(store.active_id(), Some(NodeId(3)));
    assert_eq!(store.focus_id(), Some(NodeId(4)));
}

#[test]
fn slider_convenience_round_trip() {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(1),
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.42,
            orientation: SliderOrientation::Horizontal,
        },
    );
    let (st, v) = store.slider(NodeId(1)).unwrap();
    assert_eq!(st, SliderState::Normal);
    assert!((v - 0.42).abs() < f32::EPSILON);
}

/// Re-assenta o estado da trilha — o que o `hover.rs` faz ao escrever no `InteractiveState`.
fn reseat(store: &mut WidgetStore, id: NodeId, state: SliderState) {
    store.register(
        id,
        InteractiveState::Slider {
            state,
            value: 0.5,
            orientation: SliderOrientation::Horizontal,
        },
    );
}

/// **O TIQUE alcança um slider — e o par visual sai por UMA pergunta.**
///
/// ⚠️ **É a metade que faltava do outro lado do defeito.** O `paint_slider` deitava fora o estado
/// (gate `the_track_reacts_to_the_pointer`); aqui prova-se que a informação CHEGA: o
/// `hover_targets` publica o alvo `1.0` para uma trilha acesa, o relógio anima esse id, e o
/// `slider_visual` devolve estado e `t` juntos. Duas perguntas separadas (um `slider()` aqui, um
/// `hover_live()` ali) é a forma que apodrece — o segundo é o que se esquece.
///
/// ⚠️ **`Dragging` conta como aceso, e é o que separa uma trilha de um botão:** um botão é premido
/// e solto; uma trilha é AGARRADA e o dedo fica lá. Se o arrasto não fosse alvo, a superfície
/// apagaria debaixo da mão que a comanda.
///
/// **Mutação que deve sangrar:** tirar o braço `Slider` do `hover_targets` (o alvo desaparece e o
/// `t` congela), ou tirar `Dragging` da lista de acesos.
#[test]
fn the_tick_reaches_a_slider_and_the_visual_pair_comes_out_of_one_question() {
    let id = NodeId(7);
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        id,
        InteractiveState::Slider {
            state: SliderState::Hovered,
            value: 0.5,
            orientation: SliderOrientation::Horizontal,
        },
    );
    assert_eq!(
        store.hover_targets().collect::<Vec<_>>(),
        vec![(id, 1.0)],
        "a trilha acesa nao e alvo do relogio: o `t` dela nunca sobe"
    );
    assert_eq!(
        store.slider_visual(id),
        (SliderState::Hovered, crate::motion::SETTLED)
    );

    reseat(&mut store, id, SliderState::Dragging);
    assert_eq!(
        store.hover_targets().collect::<Vec<_>>(),
        vec![(id, 1.0)],
        "a trilha AGARRADA deixou de ser alvo: ela apaga debaixo da mao que a comanda"
    );

    reseat(&mut store, id, SliderState::Normal);
    assert_eq!(
        store.hover_targets().collect::<Vec<_>>(),
        vec![(id, 0.0)],
        "a trilha em repouso tem de ser alvo ZERO — e o que faz a SAIDA animar em vez de cortar"
    );
}

/// ⭐ **O TIQUE alcança uma TAG** — sem isto o eixo dela compila, pinta e nunca se move.
///
/// ⚠️ **É o degrau que torna a wave real, e o mais fácil de esquecer:** o campo `hover_t`, a lei do
/// anel e o arm da pele podem estar os três certos e a tag continua a SALTAR, porque ninguém
/// publica o alvo — o `t` fica no neutro para sempre. O doc do `hover_targets` já o diz pelo outro
/// lado (*"um tipo que não aparece aqui não ganha entrada nenhuma"*).
///
/// **Mutação que deve sangrar:** tirar o braço `Tag` do `hover_targets`.
#[test]
fn the_tick_reaches_a_tag() {
    use crate::widget::TagState;
    let id = NodeId(11);
    let mut store = WidgetStore::with_capacity(2);
    store.register(
        id,
        InteractiveState::Tag {
            state: TagState::Hovered,
        },
    );
    assert_eq!(
        store.hover_targets().collect::<Vec<_>>(),
        vec![(id, 1.0)],
        "a tag acesa nao e alvo do relogio: o `t` dela nunca sobe"
    );

    if let Some(InteractiveState::Tag { state }) = store.get_mut(id) {
        *state = TagState::Normal;
    }
    assert_eq!(
        store.hover_targets().collect::<Vec<_>>(),
        vec![(id, 0.0)],
        "a tag em repouso tem de ser alvo ZERO — e o que faz a SAIDA desvanecer em vez de cortar"
    );
}

/// **`set_slider_value` recentra o slider E o chip ligado.** É o que devolve o Offset ao
/// "sem offset" após um commit: sem a segunda metade (o número), o chip mostraria o valor
/// velho ao ser aberto para edição.
#[test]
fn set_slider_value_recenters_the_slider_and_its_linked_chip() {
    let (slider, number) = (NodeId(1), NodeId(2));
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        slider,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.9,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        number,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 3.2,
            buffer: format!("{}", 3.2),
            caret: 0,
            last_committed: 3.2,
            selection_anchor: None,
        },
    );
    // Mapa afim `display = track * 8 - 4` (a faixa bipolar do Offset: track 0.5 ⇒ 0).
    store.link_slider_number_mapped(slider, number, 8.0, -4.0);

    store.set_slider_value(slider, 0.5);
    let (_, v) = store.slider(slider).unwrap();
    assert!((v - 0.5).abs() < f32::EPSILON, "o track foi para {v}");
    assert!(
        (store.number_value(number).unwrap() - 0.0).abs() < 1e-6,
        "o chip devia mostrar 0 (0.5·8−4), não {:?}",
        store.number_value(number)
    );

    // Num id que não é slider, é no-op (não entra em pânico).
    store.set_slider_value(NodeId(99), 0.5);
}

#[test]
fn hierarchy_parent_round_trip_and_depth() {
    let mut store = WidgetStore::with_capacity(4);
    assert_eq!(store.hierarchy_depth_of(NodeId(10)), 0);
    assert!(store.hierarchy_set_parent(NodeId(11), Some(NodeId(10))));
    assert!(store.hierarchy_set_parent(NodeId(12), Some(NodeId(11))));
    assert_eq!(store.hierarchy_parent_of(NodeId(11)), Some(NodeId(10)));
    assert_eq!(store.hierarchy_parent_of(NodeId(12)), Some(NodeId(11)));
    assert_eq!(store.hierarchy_depth_of(NodeId(10)), 0);
    assert_eq!(store.hierarchy_depth_of(NodeId(11)), 1);
    assert_eq!(store.hierarchy_depth_of(NodeId(12)), 2);
}

#[test]
fn hierarchy_set_parent_rejects_cycles() {
    let mut store = WidgetStore::with_capacity(4);
    // Build: 12 → 11 → 10 (12 is grandchild of 10)
    store.hierarchy_set_parent(NodeId(11), Some(NodeId(10)));
    store.hierarchy_set_parent(NodeId(12), Some(NodeId(11)));
    // Attempt to parent 10 under 12 (a descendant) → rejected.
    assert!(!store.hierarchy_set_parent(NodeId(10), Some(NodeId(12))));
    assert_eq!(store.hierarchy_parent_of(NodeId(10)), None);
    // Self-parent is also rejected.
    assert!(!store.hierarchy_set_parent(NodeId(11), Some(NodeId(11))));
}

#[test]
fn hierarchy_set_parent_none_detaches() {
    let mut store = WidgetStore::with_capacity(4);
    store.hierarchy_set_parent(NodeId(11), Some(NodeId(10)));
    assert_eq!(store.hierarchy_depth_of(NodeId(11)), 1);
    assert!(store.hierarchy_set_parent(NodeId(11), None));
    assert_eq!(store.hierarchy_parent_of(NodeId(11)), None);
    assert_eq!(store.hierarchy_depth_of(NodeId(11)), 0);
}

/// **A drenagem do arrasto de `CurvePoint` PERGUNTA de quem é o gesto — e recusar deixa o stash
/// INTACTO.**
///
/// O stash é um canal GLOBAL (um `Option` só) com muitos donos possíveis, e o
/// [`WidgetStore::take_curve_point_drag_if`] é a única porta — não existe forma de tomar o gesto
/// sem responder à pergunta. A metade que importa é a **recusa não destrutiva**: um `take`
/// incondicional é irreversível, então um painel que drena antes de perguntar rouba o arrasto de
/// outro e o dono não tem o que drenar (medido 2026-07-29 no trilho de rampa do painel de vetor).
#[test]
fn a_curve_point_drag_is_only_taken_by_the_editor_it_belongs_to() {
    let mut store = WidgetStore::with_capacity(2);
    let mine = NodeId(700);
    let theirs = NodeId(701);
    store.set_curve_point_drag(mine, 1, 2, 0.25, 0.75);

    // Um estranho pergunta e leva NADA — e o stash sobrevive para o dono.
    assert!(store.take_curve_point_drag_if(|p| p == theirs).is_none());
    assert_eq!(
        store.take_curve_point_drag_if(|p| p == mine),
        Some((mine, 1, 2, 0.25, 0.75)),
        "a recusa tem de ser NAO-DESTRUTIVA: o dono ainda tem de encontrar o gesto dele"
    );
    // Drenado uma vez.
    assert!(store.take_curve_point_drag_if(|p| p == mine).is_none());
}

/// **O par visual cai no PONTEIRO quando o widget não está registado** — a mesma fallback do
/// [`WidgetStore::button_visual`], e a razão é idêntica: um checkbox de modal que o `populate` não
/// registou não tem `InteractiveState::Checkbox`, e sem a fallback ficaria `Normal` para sempre.
///
/// **Mutação que deve sangrar:** devolver `Normal` em vez de consultar `hot_id`/`active_id`.
#[test]
fn an_unregistered_checkbox_still_follows_the_pointer() {
    use crate::widget::CheckboxState;
    let mut store = WidgetStore::with_capacity(8);
    let id = NodeId(4242);
    assert_eq!(store.checkbox_visual(id).0, CheckboxState::Normal);
    store.set_hot(Some(id));
    assert_eq!(store.checkbox_visual(id).0, CheckboxState::Hovered);
    store.set_active(Some(id));
    assert_eq!(store.checkbox_visual(id).0, CheckboxState::Pressed);
    // E o `t` neutro mantém o mundo pré-substrato byte a byte.
    assert_eq!(store.checkbox_visual(id).1, crate::motion::SETTLED);
}

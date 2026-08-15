//! **As três famílias que o pintor sabia desenhar e ninguém acendia.**
//!
//! Medido em 2026-08-15, antes de uma linha: `border_token(Hovered)` devolve `BorderEmph` desde
//! sempre, e **nada neste repositório produzia `Hovered`** para um `TextInput`, um `NumberInput`,
//! um `Dropdown` ou uma `Tag` — todo campo numérico e de texto do editor, os dezoito chips e as
//! pílulas eram inertes sob o ponteiro. O `hover.rs` promovia as QUATRO primitivas
//! (Button/Toggle/Slider/Checkbox) e **declarava** a exclusão dos campos.
//!
//! ⚠️ **Estes gates correm pelo DESPACHANTE real** (`dispatch_pointer`), não escrevendo no store:
//! é o caminho que o rato de facto percorre, e é onde o defeito vivia.

use super::*;

fn num(state: TextInputState) -> InteractiveState {
    InteractiveState::NumberInput {
        state,
        value: 1.0,
        buffer: String::from("1"),
        caret: 0,
        last_committed: 1.0,
        selection_anchor: None,
    }
}

fn one_field(seed: InteractiveState) -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(NodeId(9), seed);
    let mut hits = HitIndex::new();
    hits.register(NodeId(9), Rect::new(0.0, 0.0, 100.0, 24.0));
    (store, hits)
}

fn field_state(store: &WidgetStore) -> TextInputState {
    match store.get(NodeId(9)) {
        Some(
            InteractiveState::NumberInput { state, .. } | InteractiveState::TextInput { state, .. },
        ) => *state,
        other => panic!("a fixture nao contem um campo: {other:?}"),
    }
}

fn move_to(store: &mut WidgetStore, hits: &HitIndex, x: f32, y: f32) {
    let arena = Bump::new();
    let _ = dispatch_pointer(store, hits, pointer(PointerKind::Move, x, y), &arena);
}

/// **Um campo acende sob o ponteiro — e apaga-se ao sair.**
///
/// *Mutação que deve sangrar:* apagar o braço `TextInput | NumberInput` do
/// [`super::super::hover::enter_hover`] (ou do `leave_hover`).
#[test]
fn a_field_lights_up_under_the_pointer_and_goes_out_when_it_leaves() {
    let (mut store, hits) = one_field(num(TextInputState::Normal));
    move_to(&mut store, &hits, 50.0, 12.0);
    assert_eq!(
        field_state(&store),
        TextInputState::Hovered,
        "o campo continua inerte sob o ponteiro"
    );
    move_to(&mut store, &hits, 900.0, 900.0);
    assert_eq!(
        field_state(&store),
        TextInputState::Normal,
        "o campo ficou aceso depois de o rato sair"
    );
}

/// **O CONTROLO: o estado SEMÂNTICO vence o ponteiro.**
///
/// Sem esta metade, «promover sempre» satisfaria o gate acima — e um campo focado perderia o anel
/// de `Accent` (que é o que diz onde a digitação cai) no instante em que o rato passasse por cima,
/// enquanto um `Disabled` acenderia a prometer o que recusa.
#[test]
fn a_focused_or_disabled_field_does_not_answer_the_pointer() {
    for semantic in [
        TextInputState::Focused,
        TextInputState::Disabled,
        TextInputState::Error,
    ] {
        let (mut store, hits) = one_field(num(semantic));
        move_to(&mut store, &hits, 50.0, 12.0);
        assert_eq!(
            field_state(&store),
            semantic,
            "{semantic:?} foi atropelado pelo hover"
        );
    }
}

/// **Sair do FOCO com o rato em cima deixa o campo aceso, não escuro.**
///
/// ⚠️ É a metade que a exclusão do `hover.rs` de facto protegia: sete sítios do despacho escreviam
/// `Normal` ao desfocar **sem saber onde o ponteiro estava**, então o campo apagava-se sob o rato
/// e só reacendia ao primeiro movimento.
///
/// *Mutação que deve sangrar:* `blurred_field_state` a devolver `Normal` sempre.
#[test]
fn a_field_that_loses_focus_under_the_pointer_stays_lit() {
    let (mut store, hits) = one_field(num(TextInputState::Normal));
    move_to(&mut store, &hits, 50.0, 12.0);
    store.register(NodeId(9), num(TextInputState::Focused));
    super::super::reset_focused_visual_state(&mut store, NodeId(9));
    assert_eq!(
        field_state(&store),
        TextInputState::Hovered,
        "o campo apagou-se debaixo do rato ao perder o foco"
    );
}

/// **E o CONTROLO dela:** desfocar com o rato LONGE devolve `Normal`.
#[test]
fn a_field_that_loses_focus_away_from_the_pointer_goes_dark() {
    let (mut store, hits) = one_field(num(TextInputState::Focused));
    move_to(&mut store, &hits, 900.0, 900.0);
    super::super::reset_focused_visual_state(&mut store, NodeId(9));
    assert_eq!(field_state(&store), TextInputState::Normal);
}

/// **Um chip de dropdown ABERTO continua aberto sob o ponteiro; fechado, acende.**
///
/// ⚠️ A porta `dropdown_visual` **subsume** a derivação `if open { Focused } else { Normal }` que
/// estava escrita à mão em três painéis e **cravada em `Normal`** noutros doze.
#[test]
fn a_dropdown_chip_answers_the_pointer_but_open_still_wins() {
    use crate::widget::DropdownState;
    let seed = |open| InteractiveState::Dropdown {
        state: DropdownState::Normal,
        open,
        selected_index: Some(0),
    };

    let (mut store, hits) = one_field(seed(false));
    move_to(&mut store, &hits, 50.0, 12.0);
    assert_eq!(store.dropdown_visual(NodeId(9)).0, DropdownState::Hovered);

    let (mut store, hits) = one_field(seed(true));
    move_to(&mut store, &hits, 50.0, 12.0);
    assert_eq!(
        store.dropdown_visual(NodeId(9)).0,
        DropdownState::Focused,
        "o chip aberto perdeu o anel de Accent por estar sob o rato"
    );
}

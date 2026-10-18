//! ⭐ **O «misto» do BulkSelect chega à TINTA** (plano 31 da UI nova) — integração: o store e os dois
//! pintores do número (caixa e chip). Mora em `tests/it` e não em `widget/`, porque lê o store
//! (`interaction`) e a aresta `widget → interaction` tem catraca.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{
    NumberInput, TextInputState, paint_number_chip, paint_number_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// ⭐ **O «misto» do BulkSelect chega à TINTA.** O `blank_number_input` só esvaziava o buffer, e
/// todo pintor sem foco formata o `value` — o «misto» nunca aparecia (Inspector e painel do vetor).
/// A régua é a cena emitida pelas duas portas (caixa e chip), lidas pelos acessores do store.
/// (Mutação: apagar o braço do «misto» num dos pintores ⇒ o campo misto pinta o número, RED.)
#[test]
fn o_misto_pinta_em_branco_sem_foco_e_volta_com_o_valor() {
    let id = NodeId(9);
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 3.0,
            buffer: "3".into(),
            caret: 0,
            last_committed: 3.0,
            selection_anchor: None,
        },
    );
    let glifos = |store: &WidgetStore| {
        let (state, buffer, caret, anchor) = store.number_chip_view(id);
        let mixed = store.number_input_mixed(id);
        let value = store.number_value(id).unwrap_or(0.0);
        let mut text = TextSystem::without_system_fonts();
        let mut caixa = VectorScene::new();
        paint_number_input_with_buffer(
            &NumberInput::new(id, "", value)
                .suffix(Some("px"))
                .state(state),
            buffer,
            mixed,
            caret,
            anchor,
            Rect::new(0.0, 0.0, 160.0, 22.0),
            &mut caixa,
            &mut text,
            Theme::Dark,
        );
        let mut chip = VectorScene::new();
        paint_number_chip(
            Rect::new(0.0, 0.0, 80.0, 22.0),
            state,
            value,
            Some("3 px"),
            buffer,
            mixed,
            caret,
            anchor,
            &mut chip,
            &mut text,
            Theme::Dark,
        );
        (
            caixa.inner().encoding().resources.glyph_runs.len(),
            chip.inner().encoding().resources.glyph_runs.len(),
        )
    };
    let (caixa, chip) = glifos(&store);
    assert!(
        caixa > 0 && chip > 0,
        "controlo: o campo concordante pinta o número"
    );

    store.blank_number_input(id);
    assert!(store.number_input_mixed(id));
    assert_eq!(
        glifos(&store),
        (0, 0),
        "o campo misto ainda pinta um número"
    );

    // Com foco o campo misto pinta o que o artista escreve.
    if let Some(InteractiveState::NumberInput { state, buffer, .. }) = store.get_mut(id) {
        *state = TextInputState::Focused;
        buffer.push_str("12");
    }
    let (caixa, chip) = glifos(&store);
    assert!(
        caixa > 0 && chip > 0,
        "o campo misto com foco não mostra o que se escreve"
    );
    if let Some(InteractiveState::NumberInput { state, .. }) = store.get_mut(id) {
        *state = TextInputState::Normal;
    }

    store.set_number_value(id, 5.0);
    assert!(
        !store.number_input_mixed(id),
        "um valor escrito não apagou o «misto»"
    );
    let (caixa, chip) = glifos(&store);
    assert!(caixa > 0 && chip > 0, "o número não voltou à caixa");

    // Com foco, o `blank_number_input` é no-op (não briga com a digitação).
    store.set_focus(Some(id));
    store.blank_number_input(id);
    assert!(
        !store.number_input_mixed(id),
        "o campo focado ficou «misto»"
    );
}

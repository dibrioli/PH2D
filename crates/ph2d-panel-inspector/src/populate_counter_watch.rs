//! **O registo dos widgets da secção COUNTER WATCH.**
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato.** O despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique **em silêncio** — é o
//! defeito que a caça aos knobs mortos de 30/08 mediu em 34 controlos, e que esta crate pagou
//! **sete** vezes. O gate de costura desta secção carrega com `Down`+`Up` REAIS.
//!
//! ⚠️ **Nenhum valor é semeado aqui** — quem sabe o contador, o limiar e o sinal de cada regra é o
//! **snapshot**. Semear números aqui faria as regras de um objecto mostrar as do anterior até ao
//! primeiro sync.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, DropdownState, TextInputState};

use super::populate::register_button_ids;

pub(crate) fn populate_counter_watch(store: &mut WidgetStore) {
    // As linhas da lista, os dois botões E **as três opções da comparação** — as opções de um
    // dropdown são BOTÕES, e é o `is_focusable` delas que decide se a escolha chega.
    register_button_ids(store, &crate::ids::INSP_WATCH_ROW);
    register_button_ids(store, &crate::ids::INSP_WATCH_CMP_OPT);
    register_button_ids(
        store,
        &[crate::ids::INSP_WATCH_ADD, crate::ids::INSP_WATCH_REMOVE],
    );

    // ⚠️ **O CHIP é o único `Dropdown` no store** — ele guarda só o `open`; a escolha vem do
    // snapshot a cada quadro.
    store.register(
        crate::ids::INSP_WATCH_CMP_PICK,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );

    store.register(
        crate::ids::INSP_WATCH_ONCE,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    for id in [
        crate::ids::INSP_WATCH_COUNTER,
        crate::ids::INSP_WATCH_SIGNAL,
    ] {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
    // ⚠️ **Zero, e não um** — ao contrário do relógio: aqui `0` é o limiar mais comum de todos
    // (*«as vidas chegaram a zero»*) e não é o valor que torna a regra inerte.
    let value = 0.0_f64;
    store.register(
        crate::ids::INSP_WATCH_VALUE,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value,
            buffer: format_number(value),
            caret: 0,
            last_committed: value,
            selection_anchor: None,
        },
    );
}

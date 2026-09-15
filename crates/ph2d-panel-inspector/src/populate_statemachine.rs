//! **O registo dos widgets da secção STATE MACHINE** (TOP-20 #15, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — mesmo corte das irmãs.
//!
//! ⚠️ **Nenhum valor é semeado aqui** — quem sabe o que cada linha diz é o snapshot. E ⚠️⚠️ **um id
//! que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante decide pelo
//! `is_focusable`, e o ramo `None => false` engole o clique em silêncio.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::TextInputState;

use super::populate::register_button_ids;

pub(crate) fn populate_statemachine(store: &mut WidgetStore) {
    register_button_ids(store, &crate::ids::INSP_SM_STATE_ROW);
    register_button_ids(store, &crate::ids::INSP_SM_TRANS_ROW);
    register_button_ids(
        store,
        &[
            crate::ids::INSP_SM_STATE_ADD,
            crate::ids::INSP_SM_STATE_REMOVE,
            crate::ids::INSP_SM_TRANS_ADD,
            crate::ids::INSP_SM_TRANS_REMOVE,
        ],
    );
    // ⚠️ **Os três índices são NÚMEROS e não caixas de escolha**, e é uma decisão: um selector por
    // estado precisaria de N ids de opção por linha, e as opções mudam com a lista. O número é
    // alcançável, cravado pela shell ao alcance da lista, e o painel escreve o NOME do estado ao
    // lado dele — que é o que o artista lê.
    for id in [
        crate::ids::INSP_SM_STATE_NAME,
        crate::ids::INSP_SM_STATE_ON_ENTER,
        crate::ids::INSP_SM_STATE_ON_EXIT,
        crate::ids::INSP_SM_TRANS_ON,
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
    // ⚠️⚠️ **A faixa sai do PRÓPRIO array de linhas**, e não de um literal nem da constante da lei:
    // *o número sai do painel* (a lei do `TIMERS_MAX`), e derivá-lo aqui faz a faixa e a lista
    // moverem-se juntas por construção. ⭐ Que ele e o `STATES_MAX` da lei são o MESMO facto é
    // afirmado por um gate na shell, que vê as duas crates.
    #[allow(clippy::cast_precision_loss)]
    let topo = (crate::ids::INSP_SM_STATE_ROW.len() - 1) as f64;
    for id in [
        crate::ids::INSP_SM_TRANS_FROM,
        crate::ids::INSP_SM_TRANS_TO,
        crate::ids::INSP_SM_INITIAL,
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format_number(0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, 0.0, topo, 1.0); // LITERAL-PX-OK: índice de estado
    }
}

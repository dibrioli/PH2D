//! **O registo dos widgets da §11 Animation** (spec Sprite 08 §8.7).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte que a §12 fez.
//!
//! ⚠️ **Nenhum valor é semeado aqui.** Quem sabe o que está a tocar, a que velocidade e com que
//! intervalo é o **snapshot**; o store só guarda o visual e o texto que se está a escrever. Semear
//! números aqui faria a §11 de uma sprite mostrar os da sprite anterior até ao primeiro sync —
//! *o seed é dono do VALOR, o dispatch é dono do ESTADO*, e aqui o valor não é do seed.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState};

use super::populate::register_button_ids;

pub(crate) fn populate_anim(store: &mut WidgetStore) {
    // A lista, os dois botões da biblioteca, o botão que anexa o tocador, o rebobinar, e os
    // três segmentados — todos BOTÕES, porque é o `is_focusable` que decide se o clique chega.
    register_button_ids(store, &ids::INSP_ANIM_ROW);
    register_button_ids(
        store,
        &[
            ids::INSP_ANIM_ADD,
            ids::INSP_ANIM_REMOVE,
            ids::INSP_ANIM_ADD_PLAYER,
            ids::INSP_ANIM_REWIND,
        ],
    );
    register_button_ids(store, &ids::INSP_ANIM_DIR);
    register_button_ids(store, &ids::INSP_ANIM_DIR_OVERRIDE);
    register_button_ids(store, &ids::INSP_ANIM_LOOP_OVERRIDE);

    for id in [ids::INSP_ANIM_PLAYING, ids::INSP_ANIM_AUTOPLAY] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
    store.register(
        ids::INSP_ANIM_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    for (id, value) in [
        (ids::INSP_ANIM_FROM, 0.0_f64),
        (ids::INSP_ANIM_TO, 0.0),
        (ids::INSP_ANIM_FRAME_MS, 100.0), // LITERAL-PX-OK: ms por frame (10 fps), valor de domínio
        (ids::INSP_ANIM_HOLD_MS, 0.0),
        (ids::INSP_ANIM_DELAY_MS, 0.0),
        (ids::INSP_ANIM_REPEAT, 0.0),
        (ids::INSP_ANIM_SPEED, 1.0),
    ] {
        store.register(
            id,
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
    // ⚠️ **A velocidade tem faixa**, e ela é a do motor (`ph2d_ecs::SPEED_MAX_Q16`, ±100×). Sem
    // isto o scrub de arrasto teria passo livre sobre um valor que o commit depois trunca — o
    // knob prometeria uma excursão que a cena não tem.
    store.set_number_range(ids::INSP_ANIM_SPEED, -100.0, 100.0, 0.1); // LITERAL-PX-OK: faixa do motor
}

//! **O registo dos widgets da secção TIMERS** (TOP-20 #2, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte que a §11 e a §12
//! fizeram.
//!
//! ⚠️ **Nenhum valor é semeado aqui.** Quem sabe a duração, o nome e o sinal de cada timer é o
//! **snapshot**; o store só guarda o visual e o texto que se está a escrever. Semear números aqui
//! faria os timers de um objecto mostrar os do objecto anterior até ao primeiro sync — *o seed é
//! dono do VALOR, o dispatch é dono do ESTADO*, e aqui o valor não é do seed.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato.** O
//! despachante decide pelo `is_focusable`, e o ramo `None => false` engole o clique **em
//! silêncio** — é o defeito que a caça aos knobs mortos de 30/08 mediu em 34 controlos.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState};

use super::populate::register_button_ids;

pub(crate) fn populate_timer(store: &mut WidgetStore) {
    // As linhas da lista e os dois botões — todos BOTÕES, porque é o `is_focusable` que decide se
    // o clique chega.
    register_button_ids(store, &crate::ids::INSP_TIMER_ROW);
    register_button_ids(
        store,
        &[crate::ids::INSP_TIMER_ADD, crate::ids::INSP_TIMER_REMOVE],
    );

    for id in [
        crate::ids::INSP_TIMER_REPEAT,
        crate::ids::INSP_TIMER_AUTOSTART,
    ] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
    for id in [crate::ids::INSP_TIMER_NAME, crate::ids::INSP_TIMER_SIGNAL] {
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
    // ⚠️ **Um segundo, e não zero** — é o default do [`ph2d_ecs::Timer`], e pela mesma razão
    // escrita lá: `0` é o valor que NÃO dispara, e um campo que nasce mudo lê-se como partido.
    let value = 1.0_f64;
    store.register(
        crate::ids::INSP_TIMER_DURATION,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value,
            buffer: format_number(value),
            caret: 0,
            last_committed: value,
            selection_anchor: None,
        },
    );
    // ⚠️ **A faixa é a do MOTOR** — `ph2d_ecs::TIMER_MAX_US` é uma HORA, e aqui está em segundos.
    // Sem ela o scrub de arrasto teria passo livre sobre um valor que o commit depois satura, e o
    // artista veria o número a andar com o efeito parado.
    //
    // ⛔ **O número é literal e não importado, de propósito:** este painel não depende do
    // `ph2d-ecs` (ADR-0029 — uma crate de painel fala com a shell por snapshot), e puxar a crate
    // do motor para cá por uma constante seria pagar a camada inteira por um `3600`. Há gate na
    // shell a prender os dois (`the_timer_duration_range_is_the_engines`).
    store.set_number_range(crate::ids::INSP_TIMER_DURATION, 0.0, 3600.0, 0.1); // LITERAL-PX-OK: faixa do motor, em segundos
}

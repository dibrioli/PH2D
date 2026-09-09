//! **O registo dos widgets da secção AUDIO** (TOP-20 #4, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️ **Nenhum valor é semeado aqui** — quem sabe o volume, o alcance e o ficheiro é o snapshot.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio.
//!
//! # ⚠️ As faixas são as do MOTOR, e o número é literal de propósito
//!
//! Este painel não depende do `ph2d-ecs` (ADR-0029), então as constantes do motor não são
//! importáveis — puxar a crate inteira por um `1000` seria pagar a camada por um número. ⚠️ Sem as
//! faixas, o arrasto teria passo livre sobre um valor que o commit satura, e o artista veria o
//! número a andar com o som parado. Há gate na shell a prender os dois lados.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, DropdownState, TextInputState};

use super::populate::register_button_ids;

/// `(id, valor de partida, mínimo, máximo, passo)` — os sete campos numéricos.
///
/// ⚠️ **Os valores de partida são os do `AudioSource2D::default()`**, e não zeros: um `Pitch` que
/// nasce a `0` lê-se como um campo partido, e é o mesmo argumento que pôs a duração do timer a um
/// segundo em vez de zero.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 7] = [
    (ids::INSP_AUDIO_VOLUME, 0.0, -80.0, 24.0, 1.0), // LITERAL-PX-OK: decibéis, não pixels
    (ids::INSP_AUDIO_PITCH, 1.0, 0.05, 8.0, 0.05),   // LITERAL-PX-OK: factor de tom, não pixels
    (ids::INSP_AUDIO_MAX_DIST, 10.0, 0.0, 1000.0, 0.5), // LITERAL-PX-OK: metros, não pixels
    (ids::INSP_AUDIO_ATTENUATION, 1.0, 0.0, 8.0, 0.1), // LITERAL-PX-OK: expoente, adimensional
    (ids::INSP_AUDIO_RADIUS, 0.0, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: metros, não pixels
    (ids::INSP_AUDIO_PANNING, 1.0, 0.0, 1.0, 0.05),  // LITERAL-PX-OK: fracção, adimensional
    (ids::INSP_AUDIO_POLYPHONY, 1.0, 1.0, 16.0, 1.0), // LITERAL-PX-OK: contagem de vozes
];

pub(crate) fn populate_audio(store: &mut WidgetStore) {
    register_button_ids(
        store,
        &[
            ids::INSP_AUDIO_BROWSE,
            ids::INSP_AUDIO_PREVIEW,
            ids::INSP_AUDIO_STOP,
        ],
    );
    // ⚠️ **As entradas do seletor continuam BOTÕES** — elas são as linhas do popover. Só o CHIP é
    // um `Dropdown`, e o que ele guarda é o `open`, nunca a escolha.
    register_button_ids(store, &ids::INSP_AUDIO_BUS_OPT);
    store.register(
        ids::INSP_AUDIO_BUS_PICK,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    store.register(
        ids::INSP_AUDIO_SOUND,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    for id in [ids::INSP_AUDIO_LOOP, ids::INSP_AUDIO_AUTOPLAY] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
    for (id, value, lo, hi, step) in NUMEROS {
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
        store.set_number_range(id, lo, hi, step);
    }
}

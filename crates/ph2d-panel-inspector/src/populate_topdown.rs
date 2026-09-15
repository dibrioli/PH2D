//! **O registo dos widgets da secção TOP-DOWN PLAYER** (TOP-20 #13, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️ **Nenhum valor é semeado aqui** — quem sabe a velocidade e os modos é o snapshot.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio.
//!
//! # ⚠️ As faixas são as do MOTOR, e os números são literais de propósito
//!
//! Este painel não depende do `ph2d-topdown` (ADR-0029). Os tectos:
//!
//! - **velocidade** `0..200 m/s` — o carro mais rápido de um jogo 2D anda a `~50`; `200` é folga
//!   larga sem deixar o arrasto sair da escala;
//! - **ângulo mínimo de deslize** `0..89°` — a `90` toda incidência seria frontal e o corpo nunca
//!   deslizaria, que é um estado que o painel não deve conseguir produzir;
//! - **deslizes por tique** `1..8` — o oráculo usa `4`; acima de `8` cada passo custa um cast e não
//!   compra caso nenhum (⚠️ o custo medido é `~45 µs` por mover por tique com `4`).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState};

/// `(id, valor de partida, mínimo, máximo, passo)`.
///
/// ⚠️ **Os valores de partida são os do `TopDownLaw::default()`**, e não zeros: uma velocidade que
/// nasce a `0` lê-se como um campo partido — o mesmo argumento que pôs a duração do timer a um
/// segundo.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 7] = [
    (ids::INSP_TD_SPEED, 4.0, 0.0, 200.0, 0.1), // LITERAL-PX-OK: m/s
    (ids::INSP_TD_ACCEL, 0.0, 0.0, 1000.0, 0.5), // LITERAL-PX-OK: m/s²
    (ids::INSP_TD_DECEL, 0.0, 0.0, 1000.0, 0.5), // LITERAL-PX-OK: m/s²
    (ids::INSP_TD_VIEW_ANGLE, 26.565, 1.0, 89.0, 0.5), // LITERAL-PX-OK: graus
    (ids::INSP_TD_TURN_SPEED, 720.0, 0.0, 3600.0, 10.0), // LITERAL-PX-OK: graus/s
    (ids::INSP_TD_MIN_SLIDE, 15.0, 0.0, 89.0, 1.0), // LITERAL-PX-OK: graus
    (ids::INSP_TD_MAX_SLIDES, 4.0, 1.0, 8.0, 1.0), // LITERAL-PX-OK: contagem
];

pub(crate) fn populate_topdown(store: &mut WidgetStore) {
    store.register(
        ids::INSP_TD_DEFAULT_CONTROLS,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
    // ⚠️ **Os botões dos TRÊS segmentados são BOTÕES** — sem registo eles pintam e morrem sob o dedo.
    for id in ids::INSP_TD_DIRECTIONS {
        store.register(
            id,
            InteractiveState::Button {
                state: Default::default(),
            },
        );
    }
    for id in ids::INSP_TD_VIEWPOINT {
        store.register(
            id,
            InteractiveState::Button {
                state: Default::default(),
            },
        );
    }
    for id in ids::INSP_TD_FACING {
        store.register(
            id,
            InteractiveState::Button {
                state: Default::default(),
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

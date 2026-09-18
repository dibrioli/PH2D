//! **O registo dos widgets das secções FACTORY e LIFECYCLE** (TOP-20 #11 e #12, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️ **Nenhum valor é semeado aqui** — quem sabe a receita, a rajada e a vida é o snapshot.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio.
//!
//! # ⚠️ As faixas são as do MOTOR, e o número é literal de propósito
//!
//! Este painel não depende do `ph2d-ecs` (ADR-0029). ⚠️ O tecto da rajada é o
//! `ph2d_ecs::BURST_MAX = 1024`, **medido** (1 024 cópias custam 17 % de um quadro; 4 096 custam
//! 64 %) — sem ele aqui, o arrasto teria passo livre sobre um valor que o motor satura, e o artista
//! veria o número a andar com a cena parada. Há gate na shell a prender os dois lados.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState};

/// `(id, valor de partida, mínimo, máximo, passo)`.
///
/// ⚠️ **Os valores de partida são os do `Factory::default()`/`Lifetime::default()`**, e não zeros:
/// uma rajada que nasce a `0` lê-se como um campo partido — o mesmo argumento que pôs a duração do
/// timer a um segundo.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 8] = [
    (ids::INSP_FACTORY_AREA_W, 0.0, 0.0, 10_000.0, 0.1), // LITERAL-PX-OK: metros
    (ids::INSP_FACTORY_AREA_H, 0.0, 0.0, 10_000.0, 0.1), // LITERAL-PX-OK: metros
    // ⚠️ O tecto é o `BURST_MAX` do motor, medido — ver o cabeçalho.
    (ids::INSP_FACTORY_BURST, 1.0, 0.0, 1024.0, 1.0), // LITERAL-PX-OK: contagem
    (ids::INSP_FACTORY_ALIVE_MAX, 0.0, 0.0, 100_000.0, 1.0), // LITERAL-PX-OK: contagem
    (ids::INSP_FACTORY_TOTAL_MAX, 0.0, 0.0, 1_000_000.0, 1.0), // LITERAL-PX-OK: contagem
    // ⚠️ A semente é um NÚMERO e não um texto: ela sorteia, e o artista muda-a para ver outra chuva.
    (ids::INSP_FACTORY_SEED, 1.0, 0.0, 4_294_967_295.0, 1.0), // LITERAL-PX-OK: contagem
    // A vida, em segundos. ⚠️ O tecto é o do timer (uma hora): o mesmo recurso, o mesmo painel.
    (ids::INSP_LIFE_SECONDS, 2.0, 0.0, 3600.0, 0.1), // LITERAL-PX-OK: segundos
    (ids::INSP_LIFE_OUTSIDE_MARGIN, 0.5, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: metros
];

pub(crate) fn populate_factory(store: &mut WidgetStore) {
    for id in [
        ids::INSP_FACTORY_RECIPE,
        ids::INSP_FACTORY_ON_SIGNAL,
        ids::INSP_FACTORY_TAG,
        ids::INSP_FACTORY_ON_SPAWNED,
        ids::INSP_FACTORY_ON_EXHAUSTED,
        ids::INSP_LIFE_ON_DEATH,
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
    for id in [ids::INSP_FACTORY_PICK_RANDOM, ids::INSP_FACTORY_AIM] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
    // ⚠️ **Os três botões do segmentado são BOTÕES** — sem registo eles pintam e morrem sob o dedo.
    for id in ids::INSP_FACTORY_WHERE {
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

//! **O registo dos widgets da secção LUZ** (Luz 2D, W4). Um id que não passa por aqui é pintado,
//! hit-registado e MORTO sob o rato (o despacho decide pelo `is_focusable`).
//!
//! # As faixas, e de que recurso cada uma é
//!
//! - **intensidade** `0..65 504` — o maior finito do `f16`: a fluência vive numa grade
//!   `Rgba16Float`, e uma luz acima disso não cabe nela nem a 1 m;
//! - **tamanho** `0..200 m` — a escala de uma cena deste app (a do `lado` do seguidor e dos raios
//!   do abanão); `0` é o ponto exato, que a secção avisa que ainda não desenha (W5);
//! - **alcance** `0,1..200 m` — `0` não é um alcance (a lei recusa-o); o passo é o piso. Sem
//!   alcance (`∞`) é a caixa «Limit Range» desligada, não um número.
//! - **opacidade** `0..1` — a fração bloqueada por entrada do raio (`BloqueiaLuz`): é uma fração.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState, format_number};

/// `(id, valor de partida, mínimo, máximo, passo)` — a partida é a `Luz::default()`.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 4] = [
    (
        ids::INSP_LUZ_INTENSIDADE,
        std::f64::consts::TAU,
        0.0,
        65504.0, // LITERAL-PX-OK: o maior finito do f16 (a grade da fluência)
        0.1,     // LITERAL-PX-OK: passo da fluência
    ),
    (ids::INSP_LUZ_TAMANHO, 0.25, 0.0, 200.0, 0.01), // LITERAL-PX-OK: metros
    (ids::INSP_LUZ_ALCANCE, 10.0, 0.1, 200.0, 0.1),  // LITERAL-PX-OK: metros
    // A partida é a `BloqueiaLuz::default()`.
    (ids::INSP_LUZ_OPACIDADE, 1.0, 0.0, 1.0, 0.01), // LITERAL-PX-OK: fração
];

pub(crate) fn populate_luz(store: &mut WidgetStore) {
    super::populate::register_button_ids(store, &ids::INSP_LUZ_TIPO);
    super::populate::register_button_ids(
        store,
        &[ids::INSP_LUZ_RETIRAR_OCLUSOR, ids::INSP_LUZ_RETIRAR_SEM_LUZ],
    );
    // A amostra: `Plain` + `register_picker_swatch` — o despacho abre o seletor sozinho; a escolha
    // volta pelo `sync_luz`.
    store.register(ids::INSP_LUZ_COR, InteractiveState::Plain);
    store.register_picker_swatch(ids::INSP_LUZ_COR);
    store.register(
        ids::INSP_LUZ_ALCANCE_LIGADO,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
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

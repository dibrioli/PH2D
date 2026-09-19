//! **O registo dos widgets da secção PATH FOLLOW** (suplente #23).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio. *Esta crate já
//! o pagou nove vezes, e as três últimas foram nesta linha.*
//!
//! # ⚠️ As faixas, e de que recurso cada uma é
//!
//! - **relógio** `0..15` — o índice na lista de timers, e o tecto é o `ph2d_ecs::TIMERS_MAX − 1`.
//!   ⛔ O número é literal pela razão escrita nas irmãs (este painel não depende do `ph2d-ecs`), e
//!   o gate da shell prende os dois;
//! - **entrada na pista** `0..1` — é uma **fracção** do percurso, e acima de `1` ela dá a volta na
//!   lei; deixar o painel produzi-lo entregaria dois pontos do slider com a mesma saída;
//! - **ângulo** `−180..180` — GRAUS, a unidade autorada do app;
//! - **lado** `−50..50 m` — a escala de uma cena deste app, a mesma da origem do raio.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState, format_number};

/// `(id, valor de partida, mínimo, máximo, passo)`.
///
/// ⚠️ **Os valores de partida são os do `PathFollow::default()`**, e não zeros — com uma excepção
/// escrita: a duração nasce a **um segundo**, que é o default do `Timer` e não deste componente.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 5] = [
    (ids::INSP_PF_RELOGIO, 0.0, 0.0, 15.0, 1.0), // LITERAL-PX-OK: índice, tecto = TIMERS_MAX − 1
    (ids::INSP_PF_DURACAO, 1.0, 0.0, 3600.0, 0.1), // LITERAL-PX-OK: segundos, a faixa do motor
    (ids::INSP_PF_DESLOCAMENTO, 0.0, 0.0, 1.0, 0.01), // LITERAL-PX-OK: fracção do percurso
    (ids::INSP_PF_ANGULO, 0.0, -180.0, 180.0, 5.0), // LITERAL-PX-OK: graus
    (ids::INSP_PF_LADO, 0.0, -50.0, 50.0, 0.1),  // LITERAL-PX-OK: metros
];

pub(crate) fn populate_path_follow(store: &mut WidgetStore) {
    // ⚠️ **O NOME da forma é TEXTO** — a referência durável desta casa, e quem a resolve é a ponte.
    store.register(
        ids::INSP_PF_CAMINHO,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    // ⚠️ **As três CAIXAS** — e a do alinhar nasce MARCADA, como o componente.
    for (id, ligado) in [
        (ids::INSP_PF_REPEAT, false),
        (ids::INSP_PF_AUTOSTART, false),
        (ids::INSP_PF_ALINHA, true),
    ] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: if ligado {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                },
            },
        );
    }
    // ⚠️ **Os chips são BOTÕES** — sem registo eles pintam e morrem sob o dedo.
    //
    // ⭐ **A lista sai dos arrays de ids, e eles saem dos `ALL` do motor** — uma família ou um modo
    // novo entra aqui **sem esta linha mudar**, porque o que se percorre é o array.
    for id in ids::INSP_PF_CICLO
        .iter()
        .chain(ids::INSP_PF_AO_ACABAR.iter())
        .chain(ids::INSP_PF_FAMILIA.iter())
        .chain(ids::INSP_PF_MODO.iter())
        .copied()
    {
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

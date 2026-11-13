//! **O registo dos widgets da secção PROJECTILE MOTION** (TOP-20 #14, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio.
//!
//! # ⚠️ As faixas são as do MOTOR, e os números são literais de propósito
//!
//! Este painel não depende do `ph2d-projectile` (ADR-0029). Os tectos, e de que recurso cada um é:
//!
//! - **rapidez** `0..500 m/s` — uma bala de arcade rápida anda a `~60`; `500` é folga larga sem
//!   deixar o arrasto sair da escala;
//! - **ricochete** `0..1` — é uma **fracção**, e acima de `1` a bala **ganharia energia** a cada
//!   parede. A lei prende-o na porta; o painel não o deve conseguir produzir;
//! - **saltos** `0..32` — cada salto custa um cast dentro do mesmo tique (o oráculo mediu DOIS a
//!   caberem num), e acima disto não há caso de jogo que o compre;
//! - **alcance** `0..10 000 m` — o `0` é sem limite, e o tecto é a escala de uma cena grande.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState};

/// `(id, valor de partida, mínimo, máximo, passo)`.
///
/// ⚠️ **Os valores de partida são os do `ProjectileLaw::default()`**, e não zeros: uma rapidez que
/// nasce a `0` lê-se como um campo partido.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 7] = [
    (ids::INSP_PJ_SPEED, 12.0, 0.0, 500.0, 0.5), // LITERAL-PX-OK: m/s
    (ids::INSP_PJ_ACCEL, 0.0, -500.0, 500.0, 0.5), // LITERAL-PX-OK: m/s²
    (ids::INSP_PJ_MAX_SPEED, 0.0, 0.0, 500.0, 0.5), // LITERAL-PX-OK: m/s
    (ids::INSP_PJ_GRAVITY, 0.0, 0.0, 200.0, 0.5), // LITERAL-PX-OK: m/s²
    (ids::INSP_PJ_BOUNCINESS, 1.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fracção
    // ⭐ GEF 06: o campo só existe com a caixa Bounce ligada — zero é desmarcar a caixa, nunca o campo
    // sumir debaixo do arrasto.
    (ids::INSP_PJ_MAX_BOUNCES, 3.0, 1.0, 32.0, 1.0), // LITERAL-PX-OK: contagem
    (ids::INSP_PJ_RANGE, 0.0, 0.0, 10_000.0, 1.0),   // LITERAL-PX-OK: metros
];

/// As dicas de hover (Plain English) das linhas do cartão GEF 06; Gravity, Range, Bounciness, Pull e Max Speed têm a
/// sua no `populate_dicas`.
const DICAS: [(ph2d_a11y::NodeId, &str); 9] = [
    (ids::INSP_PJ_SPEED, "panel.inspector.projectile.tip_speed"),
    (
        ids::INSP_PJ_FACE_VELOCITY,
        "panel.inspector.projectile.tip_faces_path",
    ),
    (ids::INSP_PJ_BOUNCE, "panel.inspector.projectile.tip_bounce"),
    (
        ids::INSP_PJ_MAX_BOUNCES,
        "panel.inspector.projectile.tip_max_bounces",
    ),
    (ids::INSP_PJ_HOMING, "panel.inspector.projectile.tip_homing"),
    (
        ids::INSP_PJ_HOMING_TARGET,
        "panel.inspector.projectile.tip_target",
    ),
    (
        ids::INSP_PJ_TARGET_PICK,
        "panel.inspector.projectile.tip_target_pick",
    ),
    (
        ids::INSP_PJ_ACCEL,
        "panel.inspector.projectile.tip_acceleration",
    ),
    (
        ids::INSP_PJ_FOLD_SPEED_CHANGE,
        "panel.inspector.projectile.tip_speed_change",
    ),
];

pub(crate) fn populate_projectile(store: &mut WidgetStore) {
    for (id, chave) in DICAS {
        store.set_tooltip(id, ph2d_i18n::tr(chave));
    }
    // ⭐ A dobra Speed change nasce FECHADA; depois manda o que o artista abriu.
    store.mark_collapsible_section(ids::INSP_PJ_FOLD_SPEED_CHANGE);
    store.set_collapsed_if_unchosen(ids::INSP_PJ_FOLD_SPEED_CHANGE, true);
    crate::populate::register_button_ids(store, &[ids::INSP_PJ_TARGET_PICK]);
    for id in [
        ids::INSP_PJ_FACE_VELOCITY,
        ids::INSP_PJ_BOUNCE,
        ids::INSP_PJ_HOMING,
    ] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
    // ⚠️ A perseguição é um número como os outros, mas a faixa é MUITO maior: um míssil precisa de
    // acelerar bem mais do que anda para virar depressa.
    store.register(
        ids::INSP_PJ_HOMING_ACCEL,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.0,
            buffer: format_number(0.0),
            caret: 0,
            last_committed: 0.0,
            selection_anchor: None,
        },
    );
    // ⭐ GEF 06: como o Max Bounces, só existe com a caixa ligada — o piso é 1, não 0.
    store.set_number_range(ids::INSP_PJ_HOMING_ACCEL, 1.0, 5000.0, 1.0); // LITERAL-PX-OK: m/s²
    // ⚠️ **O ALVO é TEXTO** — o componente guarda o `stable_name_id`, e quem traduz é a shell.
    store.register(
        ids::INSP_PJ_HOMING_TARGET,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
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

//! Registro dos widgets FIXOS da tira (1× na instalação do painel).
//!
//! As células são dinâmicas (o número de chaves só se sabe em runtime) e se
//! registram no paint (`register_if_absent`), como as linhas do painel de camadas.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

// As FAIXAS das caixas numéricas. Não são medidas de design (px/espaçamento) — são
// os limites do DOMÍNIO (quadros, fps, inbetweens), e o `set_number_range` os usa
// para escalar o drag-scrub. O shell re-clampa nos mesmos valores ao aplicar: a
// caixa é a UI do limite, não a sua fonte de verdade.
/// FPS do objeto: cinema (24) por padrão, de 1 a 120.
const FPS_DEFAULT: f64 = 24.0; // LITERAL-PX-OK: domínio (quadros por segundo), não medida de design
const FPS_MIN: f64 = 1.0; // LITERAL-PX-OK: domínio
const FPS_MAX: f64 = 120.0; // LITERAL-PX-OK: domínio
/// Fantasmas por lado (0 = nenhum daquele lado).
const GHOST_DEFAULT: f64 = 1.0; // LITERAL-PX-OK: domínio (nº de desenhos)
const GHOST_MIN: f64 = 0.0; // LITERAL-PX-OK: domínio
const GHOST_MAX: f64 = 8.0; // LITERAL-PX-OK: domínio
/// Exposição de uma chave, em quadros (≥ 1 — uma chave sempre ocupa um quadro).
const HOLD_DEFAULT: f64 = 1.0; // LITERAL-PX-OK: domínio (quadros)
const HOLD_MIN: f64 = 1.0; // LITERAL-PX-OK: domínio
const HOLD_MAX: f64 = 999.0; // LITERAL-PX-OK: domínio
/// Inbetweens que o tween gera de uma vez.
const TWEEN_DEFAULT: f64 = 1.0; // LITERAL-PX-OK: domínio (nº de inbetweens)
const TWEEN_MIN: f64 = 1.0; // LITERAL-PX-OK: domínio
const TWEEN_MAX: f64 = 32.0; // LITERAL-PX-OK: domínio
/// Passo do drag/stepper: um quadro de cada vez (nada aqui é fracionário).
const STEP: f64 = 1.0; // LITERAL-PX-OK: domínio (o incremento é 1 quadro)

/// Um botão de ação simples.
fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// Uma caixa numérica LIMITADA: registra o valor E o range (`set_number_range`),
/// senão o drag-scrub escala errado (a caixa sem range assume amplitude 1).
fn number(store: &mut WidgetStore, id: ph2d_a11y::NodeId, value: f64, min: f64, max: f64) {
    store.register(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value,
            buffer: format!("{value}"),
            caret: 0,
            last_committed: value,
            selection_anchor: None,
        },
    );
    store.set_number_range(id, min, max, STEP);
}

/// Registra os widgets fixos da tira de frames.
pub fn populate(store: &mut WidgetStore) {
    // Transporte.
    button(store, ids::FLIP_PREV_DRAWING);
    button(store, ids::FLIP_PLAY);
    button(store, ids::FLIP_NEXT_DRAWING);
    number(store, ids::FLIP_FPS_NUM, FPS_DEFAULT, FPS_MIN, FPS_MAX);

    // Ghost Frames.
    button(store, ids::FLIP_GHOST);
    number(
        store,
        ids::FLIP_GHOST_BEFORE_NUM,
        GHOST_DEFAULT,
        GHOST_MIN,
        GHOST_MAX,
    );
    number(
        store,
        ids::FLIP_GHOST_AFTER_NUM,
        GHOST_DEFAULT,
        GHOST_MIN,
        GHOST_MAX,
    );

    // Autoria.
    button(store, ids::FLIP_AUTOKEY);
    button(store, ids::FLIP_ADDITIVE);

    // Ops de chave.
    button(store, ids::FLIP_KEY_ADD);
    button(store, ids::FLIP_KEY_DUP);
    button(store, ids::FLIP_KEY_DELETE);
    button(store, ids::FLIP_KEY_LEFT);
    button(store, ids::FLIP_KEY_RIGHT);
    number(store, ids::FLIP_HOLD_NUM, HOLD_DEFAULT, HOLD_MIN, HOLD_MAX);

    // Tween.
    number(
        store,
        ids::FLIP_TWEEN_NUM,
        TWEEN_DEFAULT,
        TWEEN_MIN,
        TWEEN_MAX,
    );
    button(store, ids::FLIP_TWEEN_ADD);

    // Chrome.
    button(store, ids::FLIP_STRIP_CLOSE);
}

//! **Os controles do BINDING DE TOKEN** (plano UI/UX §4/W4) — irmão do [`super::populate`] pelo
//! teto de 600 LOC do painel.
//!
//! ⚠️ Sem este registro os dois chips ficariam pintados, com hit-rect, e **MORTOS sob o mouse** —
//! a checagem de focabilidade mora no store. É o defeito que este painel já pagou com os pills de
//! modo (duas vezes), com o Cut e com a moldura.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, DropdownState};
use ph2d_tokens::ColorToken;

use crate::ids;

/// Os dois chips e TODAS as opções dos dois popovers.
///
/// ⚠️ **A contagem vem da tabela** (`ColorToken::ALL`), não de um literal: um token novo entra na
/// macro do `ph2d-tokens` e nasce registado, pintado e clicável. Um número escrito aqui deixaria
/// o último token da lista pintado e morto no dia em que a tabela crescesse — e ninguém olharia
/// para este arquivo.
pub(super) fn token_controls(store: &mut WidgetStore) {
    for (chip, prop) in [
        (ids::VECTOR_TOKEN_FILL, 0_u16),
        (ids::VECTOR_TOKEN_STROKE, 1),
    ] {
        store.register(
            chip,
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
        // `+ 1`: a linha de SOLTAR (índice 0) vem antes dos tokens.
        for i in 0..=ColorToken::ALL.len() {
            store.register(
                ids::vector_token_option_id(prop, i),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
}

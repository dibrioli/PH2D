//! **Os controles do BINDING DE TOKEN** (plano UI/UX §4/W4) — irmão do [`super::populate`] pelo
//! teto de 600 LOC do painel.
//!
//! ⚠️ Sem este registro os chips ficariam pintados, com hit-rect, e **MORTOS sob o mouse** — a
//! checagem de focabilidade mora no store. É o defeito que este painel já pagou com os pills de
//! modo (duas vezes), com o Cut e com a moldura.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, DropdownState};

use crate::ids;

/// Cada chip, e TODAS as opções de cada popover.
///
/// ⚠️ **A lista de slots e a contagem de cada tabela vêm de `ids::TOKEN_SLOTS`**, não de literais:
/// um slot novo (ou um token novo) nasce registado, pintado e clicável. Um número escrito aqui
/// deixaria o último item pintado e morto no dia em que a tabela crescesse — e ninguém olharia
/// para este arquivo.
pub(super) fn token_controls(store: &mut WidgetStore) {
    for slot in ids::TOKEN_SLOTS {
        store.register(
            slot.chip,
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
        // `+ 1`: a linha de SOLTAR (índice 0) vem antes dos tokens.
        for i in 0..=slot.table.len() {
            store.register(
                ids::vector_token_option_id(slot.code, i),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
}

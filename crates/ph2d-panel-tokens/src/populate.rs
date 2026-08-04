//! O registro dos widgets — percorre a MESMA lista que o `paint`.
//!
//! ⚠️ Um widget que o `paint` põe no índice de hit e que ninguém regista tem
//! `is_focusable() == false`, e o clique dele é descartado **em silêncio**: sem erro de
//! compilação, sem warning, só um controlo que não faz nada. É a classe que o
//! `architecture_panel_wiring_parity` existe para pegar, e derivar esta lista da tabela que o
//! `paint` percorre é o que faz dela algo que ninguém pode esquecer.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;
use ph2d_tokens::ColorToken;

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub fn populate(store: &mut WidgetStore) {
    button(store, ids::TOKENS_CLOSE);
    button(store, ids::TOKENS_RESET_ALL);
    for row in 0..ColorToken::ALL.len() {
        // ⚠️ A swatch é alvo de PICKER, não botão: registá-la como botão faria o clique acender o
        // widget e **nunca abrir o picker** — a cor ficaria ineditável com todos os gates verdes.
        store.register_picker_swatch(ids::tokens_swatch_id(row));
        button(store, ids::tokens_reset_id(row));
    }
}

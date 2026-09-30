//! **As NOTAS do painel no `WidgetStore`** — a ranhura, a PEGA, o título e o corpo de cada uma.
//!
//! Irmão do [`super::pre_populate`] por assunto e pelo tecto de 700 LOC (2026-09-30, quando a
//! pega das notas o levou a 703): lá fica *o que o app regista ao abrir*, aqui *o que uma nota
//! é para o despacho*. ⚠️ A ordem das quatro famílias não importa, mas as quatro têm de estar
//! aqui — uma pega que não se regista é **pintada e morta sob o dedo**, e o gate de costura
//! `as_notas_tem_pega_e_menu` é quem o diz.

use crate::interaction::{InteractiveState, WidgetStore};
use crate::widget::TextInputState;
use crate::widget::showcase::{NOTE_BODY_IDS, NOTE_GRIP_IDS, NOTE_SLOT_IDS, NOTE_TITLE_IDS};
use ph2d_i18n::tr;

/// Regista as quatro famílias de ids de cada nota.
pub(super) fn populate_notes(store: &mut WidgetStore) {
    for id in NOTE_SLOT_IDS {
        store.register(id, InteractiveState::Plain);
    }
    // ⭐ A PEGA de cada nota (2026-09-30) — a dica ensina os dois gestos da nota: arrastar pela
    //    pega, e o botão direito para a cor, duplicar e apagar.
    for id in NOTE_GRIP_IDS {
        store.register(id, InteractiveState::Plain);
        store.set_tooltip(id, tr("chrome.note.grip_hint"));
    }
    for id in NOTE_TITLE_IDS {
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
    for id in NOTE_BODY_IDS {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
        store.mark_multiline_text(id);
    }
}

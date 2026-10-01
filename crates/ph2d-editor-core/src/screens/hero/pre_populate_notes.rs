//! **As NOTAS do painel no `WidgetStore`** — a ranhura, a PEGA, o título e o corpo de cada uma.
//!
//! Irmão do [`super::pre_populate`] por assunto e pelo tecto de 700 LOC (2026-09-30, quando a
//! pega das notas o levou a 703): lá fica *o que o app regista ao abrir*, aqui *o que uma nota
//! é para o despacho*. ⚠️ A ordem das quatro famílias não importa, mas as quatro têm de estar
//! aqui — uma pega que não se regista é **pintada e morta sob o dedo**, e o gate de costura
//! `as_notas_tem_pega_e_menu` é quem o diz.

use crate::interaction::{InteractiveState, WidgetStore};
use crate::widget::TextInputState;
use ph2d_i18n::tr;

/// Regista as quatro famílias de ids de cada nota, das ranhuras de CADA painel que pinta notas
/// ([`crate::ids::NOTE_HOSTS`] — desde 2026-10-01 a Galeria e o Inspector têm caixas próprias).
pub(super) fn populate_notes(store: &mut WidgetStore) {
    for (_, n) in crate::ids::NOTE_HOSTS {
        populate_host(store, &n);
    }
}

fn populate_host(store: &mut WidgetStore, n: &crate::ids::NoteIds) {
    for id in n.slot {
        store.register(id, InteractiveState::Plain);
    }
    // ⭐ A PEGA de cada nota (2026-09-30) — a dica ensina os dois gestos da nota: arrastar pela
    //    pega, e o botão direito para a cor, duplicar e apagar.
    for id in n.grip {
        store.register(id, InteractiveState::Plain);
        store.set_tooltip(id, tr("chrome.note.grip_hint"));
    }
    for id in n.title {
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
    for id in n.body {
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

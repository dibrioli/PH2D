//! ⭐ **O registo dos widgets da secção TAGS** (TOP-20 #9, W3a) — irmão por CAP de ficheiro.
//!
//! ⚠️ **Um widget pintado e hit-registado que não esteja aqui morre SOB O DEDO**, em silêncio: o
//! despachante decide pelo `is_focusable`, e um id sem entrada no store não é focável. É o defeito
//! que os quatro chips da booleana pagaram, e o `architecture_panel_wiring_parity` é o gate que o
//! apanha — mas só para os ids passados como LITERAL ao `.register(`, e os desta secção vão por
//! ciclo sobre um array. ⇒ o que os prende é o `every_painted_id_is_reachable`, que deriva a lista
//! da SAÍDA do `paint`.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{DropdownState, TagState, TextInputState};

use super::populate::register_button_ids;

pub(crate) fn populate_tags(store: &mut WidgetStore) {
    // ⚠️ **Os CHIPS são `Tag`, não botões** — o widget existe desde sempre
    // (`ph2d_editor_core::widget::tag`) e até hoje só o showcase o usava. É ele que traz o `×` com
    // rect próprio, e é isso que separa *tirar esta tag* de *carregar no chip*.
    for &id in &crate::ids::INSP_TAGS_CHIP {
        store.register(
            id,
            InteractiveState::Tag {
                state: TagState::Normal,
            },
        );
    }
    // As opções da caixa continuam BOTÕES — elas são as linhas do popover. Só o CHIP da caixa é um
    // `Dropdown`, e o que ele guarda é o `open`, nunca a escolha (a escolha é do snapshot).
    register_button_ids(store, &crate::ids::INSP_TAGS_OPT);
    register_button_ids(store, &[crate::ids::INSP_TAGS_CREATE]);
    store.register(
        crate::ids::INSP_TAGS_PICK,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    // ⭐ O campo que é a BUSCA **e** o nome da tag nova — ver o doc do id.
    store.register(
        crate::ids::INSP_TAGS_NEW,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
}

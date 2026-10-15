//! **O registo dos widgets da secção MACRO** (ADR-0186, W5d). Um id que não passa por aqui é
//! pintado e MORTO sob o rato.

use ph2d_editor_core::interaction::WidgetStore;

use super::populate::register_button_ids;

pub(crate) fn populate_macro(store: &mut WidgetStore) {
    register_button_ids(store, &crate::ids::INSP_MACRO_MODO);
    register_button_ids(store, &[crate::ids::INSP_MACRO_EDITAR_ETAPAS]);
}

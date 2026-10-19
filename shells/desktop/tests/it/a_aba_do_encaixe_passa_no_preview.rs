//! ⭐ **F22 (plano 33 §6.7) — durante o Preview, a ABA do encaixe passa; o controlo de edição não.**
//!
//! A isenção (`ui_preview_lock::recusa` → `panel_for_tab`) resolve-se pelo registo de painéis, que é
//! do processo: o binário de teste do `app-vec` não o instala e o caso ficava sem gate. Aqui o
//! registo é o do app (`register_all_panels`).

use ph2d_app_vec::ui_preview_lock::recusa;
use ph2d_editor_core::ids::VECTOR_PANEL;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::screens::hero::slot_tabs::{panel_for_tab, tab_node_id};

#[test]
fn a_aba_do_encaixe_passa_no_preview() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let aba = tab_node_id(VECTOR_PANEL);
    // O controlo do vácuo: sem o registo a aba não resolve e o teste não provaria nada.
    assert_eq!(
        panel_for_tab(aba),
        Some(VECTOR_PANEL),
        "a aba do vetor não resolve"
    );
    let s = WidgetStore::default();
    assert!(
        !recusa(&s, Some(VECTOR_PANEL), Some(aba)),
        "a aba do encaixe foi recusada no Preview: o artista não troca de painel"
    );
    // O controlo do outro lado: um controlo de edição do mesmo painel continua trancado.
    assert!(recusa(
        &s,
        Some(VECTOR_PANEL),
        Some(ph2d_tool_vector::ids::VECTOR_WIDTH)
    ));
}

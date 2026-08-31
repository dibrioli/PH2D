//! ⭐⭐ **REPOR A ARRUMAÇÃO REPÕE AS TRÊS COISAS** — pedido do Enio, 2026-08-30:
//! *«Precisamos da opção de resetar. Coloque nas opções de Theme.»*
//!
//! ⛔ **Repor duas de três não é repor.** O artista clica, vê o ecrã mudar e conclui que funcionou;
//! o terço que ficou volta a mordê-lo mais tarde, sem ligação nenhuma com o gesto que o deixou. É
//! por isso que este gate afirma as três **na mesma corrida**, e não uma por teste.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{HeroScreen, slot_tabs};
use ph2d_editor_core::screens::layout::DockSide;
use ph2d_editor_core::screens::slot::Slot;

fn node_of(id: &str) -> ph2d_editor_core::NodeId {
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.id == id)
            .map(|p| p.manifest.panel_node_id)
            .unwrap_or_else(|| panic!("{id} não está registado"))
    })
}

/// Um app arrumado à mão: um painel movido, um aberto, um fechado, e uma coluna mais estreita.
fn arranged() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    h.store
        .set_panel_slot(node_of("audio_mixer"), Slot::LeftTop);
    h.panel_visibility.insert("audio_mixer", true);
    h.panel_visibility.insert("inspector", false);
    h.store.set_dock_width(DockSide::Left, 260.0);
    h
}

#[test]
fn resetting_puts_the_slot_the_open_panels_and_the_column_width_back() {
    let mut h = arranged();

    // ⭐ O controlo: sem ele, um `reset` que não fizesse nada passaria por «já estava reposto».
    assert_eq!(
        h.store.panel_slot(node_of("audio_mixer")),
        Some(Slot::LeftTop)
    );
    assert!(h.is_panel_visible("audio_mixer"));
    assert!(!h.is_panel_visible("inspector"));
    assert_eq!(h.store.dock_width_choice(DockSide::Left), Some(260.0));

    slot_tabs::reset(&mut h);

    assert_eq!(
        h.store.panel_slot(node_of("audio_mixer")),
        None,
        "o ENCAIXE não voltou — o painel fica onde o artista o pôs depois de pedir o reset"
    );
    assert!(
        !h.is_panel_visible("audio_mixer"),
        "o painel que o artista ABRIU continua aberto depois do reset"
    );
    assert!(
        h.is_panel_visible("inspector"),
        "o painel que o artista FECHOU continua fechado — a reposição só sabe fechar"
    );
    assert_eq!(
        h.store.dock_width_choice(DockSide::Left),
        None,
        "a LARGURA da coluna não voltou; é o terço que morde mais tarde"
    );
}

/// ⭐⭐ **E a linha do menu CHEGA lá** — o verbo, não só a lei.
///
/// ⚠️ Um `Click` no id tem de percorrer o despacho real do hero. Sem esta metade a lei podia estar
/// certa e o item de menu ser mudo, que é o defeito que este repo já pagou várias vezes.
#[test]
fn the_menu_row_reaches_the_reset() {
    let mut h = arranged();
    let consumed = h.apply_event(WidgetEvent::Click(
        ph2d_editor_core::ids::MENUBAR_VIEW_RESET_LAYOUT,
    ));
    assert!(
        consumed,
        "o clique em *Reset Panel Layout* não foi consumido por ninguém — a linha é muda"
    );
    assert_eq!(
        h.store.panel_slot(node_of("audio_mixer")),
        None,
        "o clique foi consumido e a arrumação não voltou"
    );
    assert!(!h.is_panel_visible("audio_mixer"));
    assert_eq!(h.store.dock_width_choice(DockSide::Left), None);
}

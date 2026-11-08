//! ⭐ **Settings ▸ Recovery** (P12 S): o submenu abre, cada linha escreve o intervalo e a MARCA
//! acende só a escolhida (de fábrica, 2 minutos).

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::recovery_pref::{DEFAULT_SECS, OPTIONS_SECS, secs, set_secs};
use crate::screens::hero::HeroScreen;
use ph2d_a11y::NodeId;

fn acesa(id: NodeId) -> bool {
    crate::screens::hero::menu_row_mark::id_is_currently_selected(
        id,
        ph2d_tokens::Theme::Forge,
        &crate::interaction::WidgetStore::default(),
        &crate::project::ProjectSettings::default(),
        &crate::motion::UiMotion::default(),
        &crate::luz_edits::PreferenciaDaLuz::default(),
    )
}

#[test]
fn o_submenu_da_recuperacao_escreve_o_intervalo_e_marca_a_linha_certa() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    assert_eq!(secs(), DEFAULT_SECS, "de fábrica, 2 minutos");
    assert!(acesa(ids::CTX_MENU_RECOVERY[4]));
    hero.store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::SettingsMenu,
    });
    assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_RECOVERY)));
    assert_eq!(
        hero.store.context_menu().map(|m| m.kind),
        Some(ContextMenuKind::SettingsRecoverySubmenu)
    );
    for (i, &s) in OPTIONS_SECS.iter().enumerate() {
        assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_RECOVERY[i])));
        assert_eq!(secs(), s);
        let marcadas: Vec<usize> = (0..OPTIONS_SECS.len())
            .filter(|&k| acesa(ids::CTX_MENU_RECOVERY[k]))
            .collect();
        assert_eq!(marcadas, vec![i], "só a escolhida acende");
    }
    set_secs(DEFAULT_SECS);
}

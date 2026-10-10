//! ⭐ **Settings ▸ 2D Light** (Luz 2D, W4; pedido do dono 2026-10-09): o submenu abre, cada nível
//! escreve no dono (`hero.view.luz2d`), o «Automatic» alterna nos dois sentidos, e a MARCA acende a
//! linha certa (um rádio de três + um toggle).

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::luz_edits::{NivelDaLuz, PreferenciaDaLuz};
use crate::screens::hero::HeroScreen;
use ph2d_a11y::NodeId;

fn abre(hero: &mut HeroScreen, kind: ContextMenuKind) {
    hero.store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind,
    });
}

fn acesa(id: NodeId, luz: &PreferenciaDaLuz) -> bool {
    crate::screens::hero::menu_row_mark::id_is_currently_selected(
        id,
        ph2d_tokens::Theme::Forge,
        &crate::interaction::WidgetStore::default(),
        &crate::project::ProjectSettings::default(),
        &crate::motion::UiMotion::default(),
        luz,
    )
}

#[test]
fn o_submenu_da_luz_escreve_no_dono_e_marca_a_linha_certa() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    assert_eq!(
        hero.view.luz2d,
        PreferenciaDaLuz::default(),
        "Alta + automático"
    );
    abre(&mut hero, ContextMenuKind::SettingsMenu);
    assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_LIGHT)));
    assert!(matches!(
        hero.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::SettingsLightSubmenu)
    ));
    // Cada nível, do NÃO-default para trás: escreve, fecha, e só a linha dele acende.
    for (i, nivel) in NivelDaLuz::ALL.into_iter().enumerate() {
        abre(&mut hero, ContextMenuKind::SettingsLightSubmenu);
        assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_LIGHT_NIVEL[i])));
        assert_eq!(hero.view.luz2d.nivel, nivel);
        assert!(
            hero.store.context_menu().is_none(),
            "a escolha fecha o menu"
        );
        let acesas: Vec<usize> = (0..NivelDaLuz::ALL.len())
            .filter(|&k| acesa(ids::CTX_MENU_LIGHT_NIVEL[k], &hero.view.luz2d))
            .collect();
        assert_eq!(acesas, vec![i], "o rádio acende só o nível em uso");
    }
    // O «Automatic» é um TOGGLE nos dois sentidos, e a marca segue-o.
    for esperado in [false, true] {
        abre(&mut hero, ContextMenuKind::SettingsLightSubmenu);
        assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_LIGHT_AUTO)));
        assert_eq!(hero.view.luz2d.automatica, esperado);
        assert_eq!(acesa(ids::CTX_MENU_LIGHT_AUTO, &hero.view.luz2d), esperado);
    }
}

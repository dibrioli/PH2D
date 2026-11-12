// ph2d-chrome-sync:z=130 (dispatch priority, ADR-0107; lower = earlier)
//! Settings → Text rendering cascade: um rádio sobre `TextRendering::ALL`.
//! Mirrors `settings_present.rs` 1:1.
//!
//! Picking a mode writes `HeroScreen.text_rendering`. The next frame's
//! `paint_hero_screen` calls `paint::set_text_rendering(hero.text_rendering)`,
//! which publishes the choice to the `paint_text*` thread-local. No
//! action bus push is needed — the strategy is core-side state, not
//! a shell-side resource (unlike `SetPresentMode`).

use crate::ids;
use crate::interaction::{ContextMenuKind, WidgetEvent};
use crate::screens::hero::HeroScreen;
use ph2d_tokens::TextRendering;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_TEXT {
        crate::screens::hero::menu_cascade::open_cascade(
            hero,
            id,
            ContextMenuKind::SettingsTextSubmenu,
        );
        return true;
    }
    let chosen = if id == ids::CTX_MENU_TEXT_DEFAULT {
        TextRendering::Default
    } else if id == ids::CTX_MENU_TEXT_CRISP_HEAVY {
        TextRendering::CrispHeavy
    } else if id == ids::CTX_MENU_TEXT_CRISP_HEAVY_PLUS {
        TextRendering::CrispHeavyPlus
    } else if id == ids::CTX_MENU_TEXT_CRISP_DARKENED {
        TextRendering::CrispPlusDarkened
    } else {
        return false;
    };
    hero.text_rendering = chosen;
    hero.store.close_context_menu();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::ContextMenuRequest;
    use crate::screens::hero::menu_rows::menu_rows;
    use ph2d_a11y::NodeId;

    /// ⭐ Cada linha do submenu escolhe o modo de MESMA ordem em `TextRendering::ALL` e fecha o
    /// menu — a quarta (dono 10/11) é «Crisp + stem darkening».
    #[test]
    fn every_text_rendering_row_picks_its_mode_and_closes_the_menu() {
        crate::test_support::ensure_panel_registry();
        let rows = menu_rows(ContextMenuKind::SettingsTextSubmenu);
        assert_eq!(rows.len(), TextRendering::ALL.len(), "uma linha por modo");
        assert_eq!(rows[3].0, ids::CTX_MENU_TEXT_CRISP_DARKENED);
        assert_eq!(rows[3].1.tr(), "Crisp + stem darkening");
        let mut hero = HeroScreen::new(NodeId(1));
        assert_eq!(
            hero.text_rendering,
            TextRendering::CrispHeavyPlus,
            "fábrica"
        );
        for (row, mode) in rows.iter().zip(TextRendering::ALL).rev() {
            hero.store.open_context_menu(ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind: ContextMenuKind::SettingsTextSubmenu,
            });
            assert!(apply(&mut hero, WidgetEvent::Click(row.0)));
            assert_eq!(hero.text_rendering, mode, "{}", row.1.tr());
            assert!(
                hero.store.context_menu().is_none(),
                "a escolha fecha o menu"
            );
        }
    }
}

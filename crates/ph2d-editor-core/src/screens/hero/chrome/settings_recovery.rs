// ph2d-chrome-sync:z=132 (dispatch priority, ADR-0107; lower = earlier)
//! Settings ▸ Recovery: o intervalo da cópia de recuperação (P12 S; molde do `settings_luz.rs`).
//! Escolher escreve em [`crate::recovery_pref`]; a shell aplica-o no quadro e persiste-o.

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::recovery_pref;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_RECOVERY {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::SettingsRecoverySubmenu,
        });
        return true;
    }
    let Some(i) = ids::CTX_MENU_RECOVERY.iter().position(|&n| n == id) else {
        return false;
    };
    recovery_pref::set_secs(recovery_pref::OPTIONS_SECS[i]);
    hero.store.close_context_menu();
    true
}

#[cfg(test)]
#[path = "../settings_recovery_tests.rs"]
mod tests;

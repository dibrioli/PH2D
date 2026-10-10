// ph2d-chrome-sync:z=131 (dispatch priority, ADR-0107; lower = earlier)
//! Settings → Light: a NITIDEZ da luz 2D no editor (Luz 2D, W4; pedido do dono, 2026-10-09).
//!
//! Molde do `settings_motion.rs`: os três níveis são um RÁDIO e o «Automatic» um TOGGLE — e são
//! perguntas independentes (*Alta + Automático* é o teto Alta com descida pela placa). Escolher
//! escreve em `HeroScreen.luz2d`, o dono do facto; a shell aplica-o à luz do quadro e persiste-o.

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::luz_edits::NivelDaLuz;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_LIGHT {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::SettingsLightSubmenu,
        });
        return true;
    }
    if id == ids::CTX_MENU_LIGHT_AUTO {
        // ⚠️ TOGGLE: lê o estado e inverte-o (um `set(true)` seria uma porta de sentido único).
        hero.view.luz2d.automatica = !hero.view.luz2d.automatica;
        hero.store.close_context_menu();
        return true;
    }
    let Some(i) = ids::CTX_MENU_LIGHT_NIVEL.iter().position(|&n| n == id) else {
        return false;
    };
    hero.view.luz2d.nivel = NivelDaLuz::ALL[i];
    hero.store.close_context_menu();
    true
}

#[cfg(test)]
#[path = "../settings_luz_tests.rs"]
mod tests;

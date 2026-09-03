// ph2d-chrome-sync:z=80 (dispatch priority, ADR-0107; lower = earlier)
//! **Os itens do menu Ficheiro** — cada um levanta uma bandeira que o shell drena.
//!
//! ⚠️ O `Save`, o `Save As…` e o `Open Project…` foram **placeholders até 2026-08-23**: eles
//! fechavam o menu, devolviam `true` (o gesto parecia consumido) e não faziam nada. Quem decide
//! o que acontece é o shell, porque é ele que tem o disco — aqui só se diz o que foi pedido.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_IMPORT {
        hero.import_requested = true;
        hero.store.close_context_menu();
        return true;
    }
    // ⚠️ **Os três deixaram de ser mudos** (2026-08-23). Até aqui eles fechavam o menu e
    // devolviam `true` — o gesto parecia ter acontecido, e nada acontecia. Cada um levanta a sua
    // bandeira; quem decide *o que* fazer é o shell, que é quem tem o disco.
    let flag = if id == ids::CTX_MENU_SAVE {
        &mut hero.file_menu.save
    } else if id == ids::CTX_MENU_SAVE_AS {
        &mut hero.file_menu.save_as
    } else if id == ids::CTX_MENU_OPEN_PROJECT {
        &mut hero.file_menu.open
    } else if id == ids::CTX_MENU_EXPORT_SVG {
        &mut hero.file_menu.export_svg
    } else {
        return false;
    };
    *flag = true;
    hero.store.close_context_menu();
    true
}

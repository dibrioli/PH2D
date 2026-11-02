// ph2d-chrome-sync:z=46 (dispatch priority, ADR-0107; lower = earlier)
//! ④ **Fixar** a barra de ações junto da seleção (D10 decisão 4) e o **«⋯»** das ações raras
//! (F50). Os outros botões dela são do painel dono e seguem pela porta dele; estes dois são do
//! chrome. ⛔ O desligar saiu a 15/10 (dono).

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::SELECTION_ACTIONS_PIN {
        let pinned = hero.store.selection_actions_pinned();
        hero.store.set_selection_actions_pinned(!pinned);
        true
    } else if id == ids::SELECTION_ACTIONS_MORE {
        let kind = ContextMenuKind::SelectionActionsMore;
        if hero.store.context_menu().is_some_and(|m| m.kind == kind) {
            hero.store.close_context_menu();
            return true;
        }
        // Por baixo do «⋯», onde ele caiu no quadro anterior (o mesmo dos menus da barra).
        let (x, y) = hero
            .hit_index
            .rect_for(id)
            .map_or((0.0, hero.last_viewport.y), |r| (r.x, r.y + r.h));
        hero.store
            .open_context_menu(ContextMenuRequest { x, y, kind });
        true
    } else {
        false
    }
}

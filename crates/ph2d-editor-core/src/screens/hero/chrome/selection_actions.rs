// ph2d-chrome-sync:z=46 (dispatch priority, ADR-0107; lower = earlier)
//! ④ **Fixar** a barra de ações junto da seleção (D10 decisão 4). Os outros botões dela são do
//! painel dono e seguem pela porta dele; este é do chrome. ⛔ O desligar saiu a 15/10 (dono).

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::SELECTION_ACTIONS_PIN {
        let pinned = hero.store.selection_actions_pinned();
        hero.store.set_selection_actions_pinned(!pinned);
        true
    } else {
        false
    }
}

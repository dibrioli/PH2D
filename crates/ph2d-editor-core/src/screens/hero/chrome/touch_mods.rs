// ph2d-chrome-sync:z=47 (dispatch priority, ADR-0107; lower = earlier)
//! F5: o widget dos **modificadores na tela** (um clique trava ou destrava — o rato também) e
//! *Ver ▸ Modo foco* (a porta visível do toque de 4 dedos).

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::interaction::touch::CHIPS;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if CHIPS.contains(&id) {
        hero.store.touch_mut().toggle_lock(id);
        true
    } else if id == ids::MENUBAR_VIEW_FOCUS {
        hero.bus.push(EditorAction::FocusMode);
        hero.store.close_context_menu();
        true
    } else {
        false
    }
}

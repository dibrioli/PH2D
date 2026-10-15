// ph2d-chrome-sync:z=73 (dispatch priority, ADR-0107; lower = earlier)
//! **A linha *Macros* do menu Window** (ADR-0186, W5d) — o abridor do painel MACROS, que nasce
//! FECHADO. A MESMA visibilidade que o painel escreve (`panel_visibility`, chave `"macros"`).

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::TOPBAR_MACROS {
        return false;
    }
    let visible = !hero.is_panel_visible("macros");
    hero.panel_visibility.insert("macros", visible);
    if let Some(InteractiveState::Button { state }) = hero.store.get_mut(ids::TOPBAR_MACROS) {
        *state = if visible {
            ButtonState::Pressed
        } else {
            ButtonState::Normal
        };
    }
    true
}

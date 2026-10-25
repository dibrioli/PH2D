// ph2d-chrome-sync:z=74 (dispatch priority, ADR-0107; lower = earlier)
//! **A linha *Pixel* do menu Window** (ADR-0192) — o abridor do painel PIXEL, que também abre sozinho
//! ao entrar em Pixel Sprite ▸ Edit. A MESMA visibilidade que o painel escreve (chave `"pixel"`).

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::TOPBAR_PIXEL {
        return false;
    }
    let visible = !hero.is_panel_visible("pixel");
    hero.panel_visibility.insert("pixel", visible);
    if let Some(InteractiveState::Button { state }) = hero.store.get_mut(ids::TOPBAR_PIXEL) {
        *state = if visible {
            ButtonState::Pressed
        } else {
            ButtonState::Normal
        };
    }
    true
}

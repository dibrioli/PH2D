// ph2d-chrome-sync:z=74 (dispatch priority, ADR-0107; lower = earlier)
//! **As linhas *Pixel* e *Pixel Preview* do menu Window** (ADR-0196) — o abridor do painel PIXEL,
//! que também abre sozinho ao entrar em Pixel Sprite ▸ Edit, e o da janela Preview (F7). A MESMA
//! visibilidade que o painel escreve (chaves [`PANEL_PIXEL`](crate::screens::hero::PANEL_PIXEL) e
//! [`PANEL_PIXEL_PREVIEW`](crate::screens::hero::PANEL_PIXEL_PREVIEW)).

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let panel = match id {
        x if x == ids::TOPBAR_PIXEL => crate::screens::hero::PANEL_PIXEL,
        x if x == ids::TOPBAR_PIXEL_PREVIEW => crate::screens::hero::PANEL_PIXEL_PREVIEW,
        _ => return false,
    };
    let visible = !hero.is_panel_visible(panel);
    hero.panel_visibility.insert(panel, visible);
    if let Some(InteractiveState::Button { state }) = hero.store.get_mut(id) {
        *state = if visible {
            ButtonState::Pressed
        } else {
            ButtonState::Normal
        };
    }
    true
}

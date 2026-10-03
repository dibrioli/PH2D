// ph2d-chrome-sync:z=46 (dispatch priority, ADR-0107; lower = earlier)
//! **O SELETOR DE MODO** (spec/06 F2) — uma linha do pulldown *Mode* vira um pedido no barramento.
//!
//! O CHIP é um pulldown da área como os outros (abre-o o [`super::tool_bar_overflow`]); as LINHAS
//! são deste. Quem decide o que o pedido faz é a composição, com
//! [`crate::object_mode::ModeState::resolve`] — o hero não alcança as ferramentas.

use crate::action_bus::EditorAction;
use crate::interaction::WidgetEvent;
use crate::object_mode::{ModeRequest, ObjectMode};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let Some(mode) = ObjectMode::of_row(id) else {
        return false;
    };
    hero.bus
        .push(EditorAction::ObjectMode(ModeRequest::Enter(mode)));
    hero.store.close_context_menu();
    true
}

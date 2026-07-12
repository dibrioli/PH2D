// ph2d-chrome-sync:z=271 (dispatch priority, ADR-0107; lower = earlier)
//! TopBar Flip pill — **toggle** para a ferramenta de desenho Flip (ADR-0114
//! W2). Clicar ativa `flip` quando inativa; clicar de novo desativa (volta pra
//! default). O drain do shell em `render_loop::mod` faz o `set_active` /
//! `activate_default` (o `flip_tools` é direct-activate, como o Vector).
//!
//! Fiação central: `ids::TOPBAR_FLIP` (ids/chrome/topbar.rs) + o pill em
//! `screens/hero/fixture.rs` (`IconId::Flip`) + registro em `topbar/mod.rs::populate`.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_FLIP {
        let currently_active = matches!(
            hero.store.get(ids::TOPBAR_FLIP),
            Some(InteractiveState::Button {
                state: ButtonState::Pressed,
            })
        );
        if currently_active {
            hero.bus.push(EditorAction::CancelActiveTool);
        } else {
            hero.bus
                .push(EditorAction::ActivateTool { tool_id: "flip" });
        }
        return true;
    }
    false
}

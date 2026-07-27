// ph2d-chrome-sync:z=71 (dispatch priority, ADR-0107; lower = earlier)
//! **O pill PHYS** — o abridor visível do painel de física (mundo).
//!
//! ⚠️ **Um painel de MUNDO não tem chip no rail**, que é de FERRAMENTAS, e por
//! isso o único abridor dele era a tecla `W`: uma feature que só existe para
//! quem já sabe que ela existe (pedido do Enio, 2026-07-27 — *"coloque um pill
//! ao lado de IMG para abrir o painel Physics"*).
//!
//! ⚠️ **É a MESMA visibilidade que a tecla escreve** (`panel_visibility` com a
//! chave `"physics"`), não um segundo estado que precisa concordar com ela — um
//! pill com bool próprio é como o botão passa a dizer *fechado* sobre um painel
//! aberto pelo atalho. O estado do botão é DERIVADO do mesmo bool, pelo mesmo
//! motivo (o irmão `rail_panels` faz igual com Inspector/Hierarchy).

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::TOPBAR_PHYSICS {
        return false;
    }
    let visible = !hero.is_panel_visible("physics");
    hero.panel_visibility.insert("physics", visible);
    if let Some(InteractiveState::Button { state }) = hero.store.get_mut(ids::TOPBAR_PHYSICS) {
        *state = if visible {
            ButtonState::Pressed
        } else {
            ButtonState::Normal
        };
    }
    true
}

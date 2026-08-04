// ph2d-chrome-sync:z=72 (dispatch priority, ADR-0107; lower = earlier)
//! **O pill TOK** — o abridor visível do painel de Tokens (plano UI/UX W6).
//!
//! ⚠️ Irmão exacto do [`super::physics_toggle`], e pela MESMA razão: um painel de MUNDO não tem
//! chip no rail (que é de FERRAMENTAS), então o único abridor dele era a tecla `T` — uma feature
//! que só existe para quem já sabe que ela existe. O Enio fez esta queixa sobre o painel de física
//! em 2026-07-27 e sobre este em 2026-08-04, e a resposta é a mesma.
//!
//! ⚠️ **É a MESMA visibilidade que a tecla escreve** (`panel_visibility` com a chave `"tokens"`),
//! não um segundo estado que precisa concordar com ela — um pill com bool próprio é como o botão
//! passa a dizer *fechado* sobre um painel aberto pelo atalho. O estado do botão é **DERIVADO** do
//! mesmo bool, pelo mesmo motivo.

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::TOPBAR_TOKENS {
        return false;
    }
    let visible = !hero.is_panel_visible("tokens");
    hero.panel_visibility.insert("tokens", visible);
    if let Some(InteractiveState::Button { state }) = hero.store.get_mut(ids::TOPBAR_TOKENS) {
        *state = if visible {
            ButtonState::Pressed
        } else {
            ButtonState::Normal
        };
    }
    true
}

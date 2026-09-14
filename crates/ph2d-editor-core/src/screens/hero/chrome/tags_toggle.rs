// ph2d-chrome-sync:z=72 (dispatch priority, ADR-0107; lower = earlier)
//! ⭐⭐⭐ **A linha *Tags* do menu Window** — o abridor do painel da taxonomia (TOP-20 #9, W4).
//!
//! ⚠️ **Ele existe porque o painel nasce FECHADO**, e é a **única** porta dele num projecto que
//! ainda não tem tag nenhuma — que é exactamente onde o artista carrega em *+ New* para fazer a
//! primeira. *Uma feature cuja única porta é já ter o que ela produz não tem porta* (a lei que o
//! [`super::skeleton_toggle`] pagou, e de que este é o irmão exacto).
//!
//! ⚠️ **É a MESMA visibilidade que o painel escreve** (`panel_visibility`, chave `"tags"`), nunca
//! um segundo bool — um abridor com estado próprio é como um botão passa a dizer *fechado* sobre um
//! painel aberto por outro caminho. O estado do botão é **derivado** do mesmo bool.

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::TOPBAR_TAGS {
        return false;
    }
    let visible = !hero.is_panel_visible("tags");
    hero.panel_visibility.insert("tags", visible);
    if let Some(InteractiveState::Button { state }) = hero.store.get_mut(ids::TOPBAR_TAGS) {
        *state = if visible {
            ButtonState::Pressed
        } else {
            ButtonState::Normal
        };
    }
    true
}

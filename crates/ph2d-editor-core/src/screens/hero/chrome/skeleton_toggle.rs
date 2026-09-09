// ph2d-chrome-sync:z=71 (dispatch priority, ADR-0107; lower = earlier)
//! ⭐⭐⭐ **A linha *Bones* do menu Window** — o abridor do painel de ossos.
//!
//! ⛔⛔ **Ordem do dono, 2026-09-09:** *«o Menu Windows deve receber a opção de Bones»*, e ela vem
//! junto com a mudança que a torna necessária. Enquanto a visibilidade do painel era **derivada da
//! cena** (*«há ossos?»*), não havia porta nenhuma numa cena **sem** ossos — e é exactamente aí que
//! o artista quer carregar em *Create* para fazer o primeiro. *Uma feature cuja única porta é já
//! ter o que ela produz não tem porta.*
//!
//! ⚠️ **É a MESMA visibilidade que a selecção de um osso escreve** (`panel_visibility`, chave
//! `"skeleton"`), nunca um segundo bool — um abridor com estado próprio é como um botão passa a
//! dizer *fechado* sobre um painel aberto por outro caminho. O estado do botão é **derivado** do
//! mesmo bool, pela mesma razão. É o irmão exacto do [`super::physics_toggle`].

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::TOPBAR_SKELETON {
        return false;
    }
    let visible = !hero.is_panel_visible("skeleton");
    hero.panel_visibility.insert("skeleton", visible);
    if let Some(InteractiveState::Button { state }) = hero.store.get_mut(ids::TOPBAR_SKELETON) {
        *state = if visible {
            ButtonState::Pressed
        } else {
            ButtonState::Normal
        };
    }
    true
}

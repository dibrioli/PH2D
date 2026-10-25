// ph2d-chrome-sync:z=176 (dispatch priority, ADR-0107; lower = earlier)
//! **O diálogo New Sprite** (ADR-0192): os modos e os fundos guardam a escolha; o OK arma o pedido e
//! fecha; o Cancel só fecha. A shell drena `take_new_sprite_request` e cria o Pixel Sprite.
//!
//! ⚠️ Irmão do [`super::sheet_size`] e logo a seguir na `z`: nenhum id partilhado, e ambos ANTES do
//! despacho genérico de linhas de menu.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::NEW_SPRITE_OK {
        hero.store.request_new_sprite();
        return true;
    }
    if id == ids::NEW_SPRITE_CANCEL {
        hero.store.close_context_menu();
        return true;
    }
    if let Some(&(m, ..)) = ids::NEW_SPRITE_MODES.iter().find(|(_, b, _)| *b == id) {
        hero.store.set_new_sprite_mode(m);
        return true;
    }
    if let Some(&(b, ..)) = ids::NEW_SPRITE_BGS.iter().find(|(_, b, _)| *b == id) {
        hero.store.set_new_sprite_background(b);
        return true;
    }
    false
}

// ph2d-chrome-sync:z=177 (dispatch priority, ADR-0107; lower = earlier)
//! **O diálogo Color Mode** do Pixel Art: o alvo, o dither e o cinza guardam a escolha (o modo actual
//! não se escolhe); o OK arma o pedido e fecha; o Cancel só fecha. A família drena
//! `take_color_mode_request` e converte o sprite como UM passo do desfazer.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::COLOR_MODE_OK {
        hero.store.request_color_mode();
        return true;
    }
    if id == ids::COLOR_MODE_CANCEL {
        hero.store.close_context_menu();
        return true;
    }
    if let Some(&(m, ..)) = ids::COLOR_MODE_TARGETS.iter().find(|(_, b, _)| *b == id) {
        let _ = hero.store.set_color_mode_target(m);
        return true;
    }
    if let Some(&(d, ..)) = ids::COLOR_MODE_DITHERS.iter().find(|(_, b, _)| *b == id) {
        hero.store.set_color_mode_dither(d);
        return true;
    }
    if let Some(&(g, ..)) = ids::COLOR_MODE_GRAYS.iter().find(|(_, b, _)| *b == id) {
        hero.store.set_color_mode_gray(g);
        return true;
    }
    false
}

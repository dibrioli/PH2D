//! New-image modal (Cmd/Ctrl+N): the Size / Background radio clicks update the store's remembered
//! selection; Create raises the `(size, bg)` request + closes the modal. The shell polls
//! `take_new_image_request` and spawns a blank canvas of that size + background.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_NEW_IMAGE_CREATE {
        hero.store.request_new_image();
        return true;
    }
    if let Some(&(px, _)) = ids::CTX_MENU_NEW_IMAGE_SIZES
        .iter()
        .find(|&&(_, b)| b == id)
    {
        hero.store.set_new_image_size(px);
        return true;
    }
    if let Some(&(bg, _)) = ids::CTX_MENU_NEW_IMAGE_BGS.iter().find(|&&(_, b)| b == id) {
        hero.store.set_new_image_bg(bg);
        return true;
    }
    false
}

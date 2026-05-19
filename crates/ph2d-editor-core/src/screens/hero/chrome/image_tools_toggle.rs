//! TopBar Image Tools mode toggle — flips `image_edit.mode_on`.
//!
//! Intercepted at Hero level (not the topbar stub) because
//! `image_edit` lives on `HeroScreen`, not on the WidgetStore.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_IMAGE_TOOLS {
        hero.image_edit.mode_on = !hero.image_edit.mode_on;
        return true;
    }
    false
}

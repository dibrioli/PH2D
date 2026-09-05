// ph2d-chrome-sync:z=10 (dispatch priority, ADR-0107; lower = earlier)
//! Theme menu — os temas das DUAS famílias (a clássica e a moderna, `ph2d_tokens::Theme`).
//!
//! ⚠️ A tabela `id ⇄ tema` vive em `theme_menu::THEME_MENU`; este handler só a consulta. Um
//! `match` aqui e outro na marca de estado do menu foi o par que a família moderna (2026-09-04)
//! teria feito envelhecer em separado.

use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;
use crate::screens::hero::theme_menu::theme_of_menu_id;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let Some(new_theme) = theme_of_menu_id(id) else {
        return false;
    };
    hero.theme = new_theme;
    hero.store.close_context_menu();
    true
}

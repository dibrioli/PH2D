// ph2d-chrome-sync:z=270 (dispatch priority, ADR-0107; lower = earlier)
//! TopBar Vector pill — **toggle** for the single Vector drawing tool
//! (ADR-0108 cutover). Clicking activates `vector` when inactive; clicking
//! again deactivates. The shell drain in `render_loop::mod` performs the actual
//! `ToolRegistry::set_active` / `default_tool_id` swap.
//!
//! Central wiring: `ids::TOPBAR_VECTOR` (ids/chrome/topbar.rs) + the pill in
//! `screens/hero/fixture.rs` (`IconId::Vector`) + registration in
//! `topbar/mod.rs::populate`.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_VECTOR {
        // ⛔⛔ **A pergunta é à VERDADE, não ao `ButtonState`.** Ela era
        // `store.get(ids::TOPBAR_VECTOR) == Pressed` — e **ninguém escrevia esse estado**: o laço de
        // reconciliação da shell só percorre os clusters do registry de ferramentas, e um pill de
        // módulo não está em cluster nenhum. ⇒ `currently_active` era **sempre falso** e o ramo
        // *cancelar* nunca corria: o segundo clique voltava a activar.
        // *Um estado que ninguém escreve e alguém lê é um `if` com um lado morto.*
        let currently_active =
            crate::screens::hero::menu_bar::module_is_on(hero, ids::TOPBAR_VECTOR).unwrap_or(false);
        if currently_active {
            hero.bus.push(EditorAction::CancelActiveTool);
        } else {
            hero.bus
                .push(EditorAction::ActivateTool { tool_id: "vector" });
        }
        return true;
    }
    false
}

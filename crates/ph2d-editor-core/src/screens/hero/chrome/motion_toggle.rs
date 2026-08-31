// ph2d-chrome-sync:z=280 (dispatch priority, ADR-0107; lower = earlier)
//! TopBar Motion pill — **toggle** for the Motion Nodes tool (Motion Nodes
//! M0.T9). Clicking activates `motion` when inactive; clicking again
//! deactivates. The shell drain in `render_loop::mod` performs the actual
//! `ToolRegistry::set_active` / `default_tool_id` swap (and the `motion_bridge`
//! shows/hides the docked graph + params panels + the center split).
//!
//! Central wiring: `ids::TOPBAR_MOTION` (ids/chrome/topbar.rs) + the pill in
//! `screens/hero/fixture.rs` (`IconId::MotionNodes`) + registration in
//! `topbar/mod.rs::populate`. Mirror of `chrome::vector_toggle`.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_MOTION {
        // ⛔⛔ **A pergunta é à VERDADE, não ao `ButtonState`.** Ela era
        // `store.get(ids::TOPBAR_MOTION) == Pressed` — e **ninguém escrevia esse estado**: o laço de
        // reconciliação da shell só percorre os clusters do registry de ferramentas, e um pill de
        // módulo não está em cluster nenhum. ⇒ `currently_active` era **sempre falso** e o ramo
        // *cancelar* nunca corria: o segundo clique voltava a activar.
        // *Um estado que ninguém escreve e alguém lê é um `if` com um lado morto.*
        let currently_active =
            crate::screens::hero::menu_bar::module_is_on(hero, ids::TOPBAR_MOTION).unwrap_or(false);
        if currently_active {
            hero.bus.push(EditorAction::CancelActiveTool);
        } else {
            hero.bus
                .push(EditorAction::ActivateTool { tool_id: "motion" });
        }
        return true;
    }
    false
}

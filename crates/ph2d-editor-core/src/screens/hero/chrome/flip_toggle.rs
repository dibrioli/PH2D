// ph2d-chrome-sync:z=271 (dispatch priority, ADR-0107; lower = earlier)
//! TopBar Flip pill — **toggle** para a ferramenta de desenho Flip (ADR-0114
//! W2). Clicar ativa `flip` quando inativa; clicar de novo desativa (volta pra
//! default). O drain do shell em `render_loop::mod` faz o `set_active` /
//! `activate_default` (o `flip_tools` é direct-activate, como o Vector).
//!
//! Fiação central: `ids::TOPBAR_FLIP` (ids/chrome/topbar.rs) + o pill em
//! `screens/hero/fixture.rs` (`IconId::Flip`) + registro em `topbar/mod.rs::populate`.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_FLIP {
        // ⛔⛔ **A pergunta é à VERDADE, não ao `ButtonState`.** Ela era
        // `store.get(ids::TOPBAR_FLIP) == Pressed` — e **ninguém escrevia esse estado**: o laço de
        // reconciliação da shell só percorre os clusters do registry de ferramentas, e um pill de
        // módulo não está em cluster nenhum. ⇒ `currently_active` era **sempre falso** e o ramo
        // *cancelar* nunca corria: o segundo clique voltava a activar.
        // *Um estado que ninguém escreve e alguém lê é um `if` com um lado morto.*
        let currently_active =
            crate::screens::hero::menu_bar::module_is_on(hero, ids::TOPBAR_FLIP).unwrap_or(false);
        if currently_active {
            hero.bus.push(EditorAction::CancelActiveTool);
        } else {
            hero.bus
                .push(EditorAction::ActivateTool { tool_id: "flip" });
        }
        return true;
    }
    false
}

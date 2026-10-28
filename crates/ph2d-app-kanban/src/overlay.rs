//! O que se pinta por cima do quadro: o menu aberto e o painel do arquivo. A disposição de cada um
//! sai de UMA função ([`menu_placed`], [`panel_placed`]), que o pintor e o clique chamam.

use crate::menu::{self, Menu};
use crate::tab::{Hit, Tab};
use ph2d_kanban_model::Kanban;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// O menu aberto, disposto sob o botão que o abriu.
pub(crate) fn menu_placed(tab: &Tab) -> Option<menu::Placed> {
    let mn = tab.menu?;
    let anchor = match mn {
        Menu::List(id) => tab.layout.list(id)?.menu,
        Menu::Board => tab.layout.board_menu,
    };
    Some(menu::place(mn, anchor, tab.layout.area))
}

pub(crate) fn panel_placed(
    tab: &Tab,
    k: &Kanban,
    ts: &mut TextSystem,
) -> Option<crate::archive::Placed> {
    let p = tab.panel?;
    Some(crate::archive::place(p, k, tab.layout.board, ts))
}

pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    tab: &Tab,
) {
    if let Some(p) = panel_placed(tab, k, ts) {
        let hover = match tab.hover {
            Some(Hit::Panel(r)) => Some(r),
            _ => None,
        };
        crate::archive::paint(scene, ts, theme, tab.panel.unwrap_or_default(), &p, hover);
    }
    if let (Some(mn), Some(p)) = (tab.menu, menu_placed(tab)) {
        let current = match mn {
            Menu::List(id) => k.lists.get(&id).and_then(|l| l.color),
            Menu::Board => None,
        };
        let hover = match tab.hover {
            Some(Hit::Menu(r)) => Some(r),
            _ => None,
        };
        menu::paint(scene, ts, theme, mn, &p, current, hover);
    }
}

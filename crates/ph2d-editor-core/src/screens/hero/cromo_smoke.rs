//! `PH2D_CROMO_SMOKE=<n>`: abre uma peça flutuante do cromo sem rato, para a foto
//! (`docs/CartoesUI/15_CHROME_CENSO.md`). O XTest não chega à Xwayland da sessão virtual.

use crate::ids;
use crate::interaction::{ContextMenuKind as K, ContextMenuRequest, WidgetStore};
use ph2d_a11y::NodeId;

/// Menu aberto e linha sob o rato de cada cena; `None` no menu = só a dica do id.
fn cena(n: u32) -> Option<(Option<K>, NodeId)> {
    Some(match n {
        1 => (
            Some(K::HierarchyRow { row: NodeId(1) }),
            ids::CTX_MENU_HIER_DUPLICATE,
        ),
        2 => (None, ids::HIERARCHY_ADD),
        3 => (Some(K::SceneList), ids::CTX_SCENE_ROWS[0]),
        4 => (Some(K::MenuBarFile), ids::CTX_MENU_SAVE),
        5 => (Some(K::SettingsMenu), ids::CTX_MENU_SETTINGS_UNIT),
        _ => return None,
    })
}

/// O ponto onde a cena abre o menu: à direita da barra vertical, abaixo do topo.
const ANCORA: (f32, f32) = (360.0, 120.0); // LITERAL-PX-OK: sítio do menu na foto, sem cliente real

/// Reabre a cena a cada quadro: a foto não tem rato que a feche nem que a mova.
pub(super) fn aplica(store: &mut WidgetStore) {
    let Some((menu, hot)) = std::env::var("PH2D_CROMO_SMOKE")
        .ok()
        .and_then(|v| v.parse().ok())
        .and_then(cena)
    else {
        return;
    };
    if let Some(kind) = menu {
        store.open_context_menu(ContextMenuRequest {
            x: ANCORA.0,
            y: ANCORA.1,
            kind,
        });
    }
    store.set_hot(Some(hot));
}

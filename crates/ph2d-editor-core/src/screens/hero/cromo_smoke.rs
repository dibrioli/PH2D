//! `PH2D_CROMO_SMOKE=<n>`: abre uma peça flutuante do cromo sem rato, para a foto
//! (`docs/CartoesUI/15_CHROME_CENSO.md`). O XTest não chega à Xwayland da sessão virtual.

use crate::ids;
use crate::interaction::{ContextMenuKind as K, ContextMenuRequest, WidgetStore};
use ph2d_a11y::NodeId;

/// Menu aberto e linha sob o rato de cada cena; sem menu, a linha é só a dica do id.
fn cena(n: u32) -> Option<(Option<K>, Option<NodeId>)> {
    Some(match n {
        1 => (
            Some(K::HierarchyRow { row: NodeId(1) }),
            Some(ids::CTX_MENU_HIER_DUPLICATE),
        ),
        2 => (None, Some(ids::HIERARCHY_ADD)),
        3 => (Some(K::SceneList), Some(ids::CTX_SCENE_ROWS[0])),
        4 => (Some(K::MenuBarFile), Some(ids::CTX_MENU_SAVE)),
        5 => (Some(K::SettingsMenu), Some(ids::CTX_MENU_SETTINGS_UNIT)),
        6 => (Some(K::NewImageDialog), None),
        7..=9 => (None, None),
        _ => return None,
    })
}

/// O ponto onde a cena abre o menu: à direita da barra vertical, abaixo do topo.
const ANCORA: (f32, f32) = (360.0, 120.0); // LITERAL-PX-OK: sítio do menu na foto, sem cliente real

/// Reabre a cena a cada quadro: a foto não tem rato que a feche nem que a mova.
pub(super) fn aplica(store: &mut WidgetStore) {
    let Some(n) = std::env::var("PH2D_CROMO_SMOKE")
        .ok()
        .and_then(|v| v.parse().ok())
    else {
        return;
    };
    let Some((menu, hot)) = cena(n) else {
        return;
    };
    if let Some(kind) = menu {
        store.open_context_menu(ContextMenuRequest {
            x: ANCORA.0,
            y: ANCORA.1,
            kind,
        });
    }
    match n {
        // Pela porta do produto: é ela que semeia Width e Height (a foto `m8_new_sprite` lia «0»).
        7 if store.context_menu().is_none() => store.open_new_sprite_dialog(),
        8 if store.fill_modal_pos().is_none() => store.open_fill_modal(ANCORA.0, ANCORA.1, 0.5),
        9 if store.onion_modal_pos().is_none() => {
            let (antes, depois) = ([230, 80, 80, 255], [80, 200, 120, 255]);
            let fantasmas = 0.3; // LITERAL-PX-OK: fração de fantasmas da cena de foto, não é geometria
            store.open_onion_modal(ANCORA.0, ANCORA.1, 0.5, fantasmas, fantasmas, antes, depois);
        }
        _ => {}
    }
    if hot.is_some() {
        store.set_hot(hot);
    }
}

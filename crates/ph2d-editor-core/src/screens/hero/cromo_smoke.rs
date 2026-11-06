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
        6..=10 | 12 | 13 | 16 | 18 => (None, None),
        11 => (Some(K::RenamePaletteDialog), None),
        14 => (Some(K::ConfirmDeleteBoard { board: 1 }), None),
        15 => (Some(K::SendToBoard { row: NodeId(1) }), None),
        17 => (Some(K::ToolBarOverflow), None),
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
        6 if store.context_menu().is_none() => store.open_new_image_dialog(),
        // O New Image com o Custom escolhido: Width e Height à vista.
        18 if store.context_menu().is_none() => {
            store.set_new_image_custom();
            store.open_new_image_dialog();
        }
        7 if store.context_menu().is_none() => store.open_new_sprite_dialog(),
        8 if store.fill_modal_pos().is_none() => store.open_fill_modal(ANCORA.0, ANCORA.1, 0.5),
        9 if store.onion_modal_pos().is_none() => {
            let (antes, depois) = ([230, 80, 80, 255], [80, 200, 120, 255]);
            let fantasmas = 0.3; // LITERAL-PX-OK: fração de fantasmas da cena de foto, não é geometria
            store.open_onion_modal(ANCORA.0, ANCORA.1, 0.5, fantasmas, fantasmas, antes, depois);
        }
        10 if store.context_menu().is_none() => store.open_sheet_size_dialog(1024),
        12 if store.context_menu().is_none() => {
            store.open_color_mode_dialog(crate::pixel_panel::NewSpriteMode::Rgba);
        }
        13 if store.context_menu().is_none() => {
            let lona = crate::pixel_panel::PixelDialogKind::CanvasSize;
            store.open_pixel_edit_dialog(lona, (32, 24));
        }
        16 if store.command_palette_model().is_none() => {
            let hero = super::HeroScreen::new(NodeId(1));
            store.open_command_palette(super::global_palette::build_global_model(&hero));
        }
        _ => {}
    }
    if hot.is_some() {
        store.set_hot(hot);
    }
}

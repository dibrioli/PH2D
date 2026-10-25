//! A costura do painel PIXEL — o registo dos widgets e o pedido que cada gesto faz.
//!
//! ⚠️ Os botões registam-se INCONDICIONALMENTE (quem decide se o clique é possível é a PINTURA:
//! sem hit-rect não há `Click`). As células da paleta registam-se por quadro, por quem as pinta.

use crate::ids;
use crate::state::with_current;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::pixel_panel::PixelRequest;
use ph2d_editor_core::widget::ButtonState;
use ph2d_i18n::{tr, tr_with};

pub(crate) fn populate(store: &mut WidgetStore) {
    let botoes = ids::TOOLS.iter().map(|(_, id)| *id);
    for id in botoes.chain([ids::PIXEL_CLOSE, ids::PIXEL_SWAP]) {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    // O tooltip diz o atalho: «Pencil (B)».
    for (t, id) in ids::TOOLS {
        let dica = tr_with(
            "pixel.tool_switched",
            &[("tool", &tr(t.label_key())), ("key", &t.shortcut())],
        );
        store.set_tooltip(id, dica);
    }
    store.set_tooltip(ids::PIXEL_SWAP, tr("panel.pixel.swap_tip"));
    // As amostras da coluna abrem o seletor partilhado num Down — marcadas AQUI, porque a coluna as
    // pinta com o painel fechado.
    store.register_picker_swatch(ids::PIXEL_FG);
    store.register_picker_swatch(ids::PIXEL_BG);
}

/// O registo POR QUADRO das células da paleta (os ids derivam do índice).
pub(crate) fn register_palette(store: &mut WidgetStore, n: usize) {
    for i in 0..n {
        store.register_if_absent(ids::palette_id(i), InteractiveState::Plain);
    }
}

/// O pedido de um clique, se o id é deste painel e há sprite em edição.
fn pedido(id: ph2d_a11y::NodeId) -> Option<PixelRequest> {
    with_current(|info| {
        if !info.editing {
            return None;
        }
        if let Some((t, _)) = ids::TOOLS.iter().find(|(_, b)| *b == id) {
            return Some(PixelRequest::Tool(*t));
        }
        if id == ids::PIXEL_SWAP {
            return Some(PixelRequest::Swap);
        }
        info.palette.iter().enumerate().find_map(|(i, c)| {
            if ids::palette_id(i) == id {
                Some(PixelRequest::Fg(*c))
            } else if ids::palette_bg_id(i) == id {
                Some(PixelRequest::Bg(*c))
            } else {
                None
            }
        })
    })
}

pub(crate) fn apply_event(
    _st: &mut crate::PixelToolsPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let WidgetEvent::Click(id) = ev else {
        return EventOutcome::Ignored;
    };
    if id == ids::PIXEL_CLOSE {
        host.set_panel_visible(crate::PixelToolsPanel::ID, false);
        return EventOutcome::Consumed;
    }
    match pedido(id) {
        Some(r) => {
            host.bus_mut().push(EditorAction::Pixel(r));
            EventOutcome::Consumed
        }
        None => EventOutcome::Ignored,
    }
}

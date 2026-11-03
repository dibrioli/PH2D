//! Widget Gallery `apply_event`: a pílula da TopBar / o fecho (X) alternam a visibilidade; os
//! cartões do vocabulário escrevem as escolhas no estado (o paint lê-as); a cor, o Fill, o Apply,
//! o Cancel e o ↺ moram em [`crate::valores`].

use crate::WidgetGalleryPanel;
use crate::ids as g;
use crate::state::{Choices, WidgetGalleryState};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

pub(crate) fn apply_event(
    state: &mut WidgetGalleryState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let WidgetEvent::Click(id) = ev else {
        return EventOutcome::Ignored;
    };
    if id == ids::TOPBAR_WIDGET_GALLERY || id == ids::GAL_CLOSE {
        let next = !host.panel_visible(WidgetGalleryPanel::ID);
        host.set_panel_visible(WidgetGalleryPanel::ID, next);
        return EventOutcome::Consumed;
    }
    if crate::valores::on_click(state, host, id) {
        return EventOutcome::Consumed;
    }
    if choose(&mut state.choices, id) {
        EventOutcome::Consumed
    } else {
        EventOutcome::Ignored
    }
}

/// Escreve a escolha do clique `id`; devolve se o id é da galeria.
fn choose(ch: &mut Choices, id: ph2d_a11y::NodeId) -> bool {
    let levels = [g::LEVEL_LOW, g::LEVEL_MID, g::LEVEL_HIGH];
    let caps = [g::CAP_BUTT, g::CAP_ROUND, g::CAP_SQUARE];
    let blends = [g::BLEND_NORMAL, g::BLEND_MULTIPLY, g::BLEND_SCREEN];
    let index = |t: &[ph2d_a11y::NodeId]| t.iter().position(|x| *x == id);
    if let Some(i) = index(&levels) {
        ch.level = i as u8;
    } else if let Some(i) = index(&caps) {
        ch.cap = i as u8;
    } else if let Some(i) = index(&blends) {
        ch.blend = i as u8;
    } else if id == g::BARE_VISIBLE {
        ch.visible = !ch.visible;
    } else if id == g::BARE_LOCK {
        ch.locked = !ch.locked;
    } else if id == g::SNAP {
        ch.snap = !ch.snap;
    } else if id == g::FILL_EYE {
        ch.fill_visible = !ch.fill_visible;
    } else if id == g::PHYSICS_ADD {
        ch.physics = true;
    } else if id == g::PHYSICS_REMOVE {
        ch.physics = false;
    } else {
        // A caixa da lista abre pelo despacho dos cartões; a galeria só consome o clique.
        return id == g::BLEND;
    }
    true
}

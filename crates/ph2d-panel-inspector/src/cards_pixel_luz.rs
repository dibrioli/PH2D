//! **A *Light* do cartão Pixel Sprite** (P13 L): *Smooth* (a luz tinge a cor, a lei de sempre) ·
//! *Palette* (a luz escolhe um degrau mais claro ou mais escuro da rampa da paleta). Escolher grava
//! `DocOp::SetRamps` no documento do objecto (um passo do Pixel).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget};
use ph2d_editor_core::pixel_panel::{PixelRequest, PixelSpriteCard};
use ph2d_i18n::tr;

use crate::ids::{INSP_PIXEL_LIGHT_PALETTE, INSP_PIXEL_LIGHT_SMOOTH};

/// A linha *Light*: os dois segmentos, aceso o do documento.
pub(crate) fn linha(c: &mut CardCtx<'_>, y: f32, card: &PixelSpriteCard) -> f32 {
    let segs = [
        Seg {
            id: INSP_PIXEL_LIGHT_SMOOTH,
            icon: None,
            text: tr("card.insp.pixel_light.smooth"),
            on: !card.luz_na_paleta,
        },
        Seg {
            id: INSP_PIXEL_LIGHT_PALETTE,
            icon: None,
            text: tr("card.insp.pixel_light.palette"),
            on: card.luz_na_paleta,
        },
    ];
    c.row(
        &[Cell::new(Widget::Choice(&segs)).label(tr("card.insp.pixel_light"))],
        y,
    )
}

/// Uma vez por quadro, antes da pintura.
pub(crate) fn prepara(store: &mut WidgetStore) {
    for id in [INSP_PIXEL_LIGHT_SMOOTH, INSP_PIXEL_LIGHT_PALETTE] {
        store.register_if_absent(
            id,
            InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Normal,
            },
        );
        store.set_tooltip(id, tr("card.insp.pixel_light_tip"));
    }
}

/// O segmento clicado vira o pedido ao documento do objecto (o do cartão preso, ou o selecionado).
pub(crate) fn evento(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    if id != INSP_PIXEL_LIGHT_SMOOTH && id != INSP_PIXEL_LIGHT_PALETTE {
        return false;
    }
    let bits = host
        .store()
        .card_event_subject(ev)
        .or_else(|| crate::state::current_inspector_transform().map(|t| t.entity_bits));
    let Some(card) = bits.and_then(|b| host.store().pixel_sprite_card(b)) else {
        return true;
    };
    let (doc, on) = (card.doc, id == INSP_PIXEL_LIGHT_PALETTE);
    host.bus_mut()
        .push(EditorAction::Pixel(PixelRequest::LuzNaPaleta { doc, on }));
    true
}

//! **O *Pivot* do cartão Pixel Sprite** (P14 §6): *Center* + as slices com pivô; escolher grava
//! `Op::PivotSlice` no documento do objecto (um passo do Pixel), e o objecto passa a ter a origem
//! nesse ponto (`ph2d_pixel_render::image_to_world`).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, dropdown};
use ph2d_editor_core::pixel_panel::{PixelRequest, PixelSpriteCard};
use ph2d_i18n::tr;

use crate::ids::{INSP_PIXEL_PIVOT_DD, INSP_PIXEL_PIVOT_OPT};

/// A linha *Pivot*: a lista com a escolha do documento.
pub(crate) fn linha(c: &mut CardCtx<'_>, y: f32, card: &PixelSpriteCard) -> f32 {
    let centro = tr("card.insp.pixel_pivot_center");
    let mut opcoes = vec![Seg {
        id: INSP_PIXEL_PIVOT_OPT[0],
        icon: None,
        text: centro,
        on: card.pivot.is_none(),
    }];
    for ((s, nome), &id) in card.pivots.iter().zip(&INSP_PIXEL_PIVOT_OPT[1..]) {
        opcoes.push(Seg {
            id,
            icon: None,
            text: nome,
            on: card.pivot == Some(*s),
        });
    }
    let texto = opcoes.iter().find(|o| o.on).map_or(centro, |o| o.text);
    c.row(
        &[Cell::new(Widget::Dropdown {
            id: INSP_PIXEL_PIVOT_DD,
            text: texto,
            options: &opcoes,
        })
        .label(tr("card.insp.pixel_pivot"))],
        y,
    )
}

/// Uma vez por quadro, antes da pintura.
pub(crate) fn prepara(store: &mut WidgetStore) {
    for id in std::iter::once(INSP_PIXEL_PIVOT_DD).chain(INSP_PIXEL_PIVOT_OPT) {
        store.register_if_absent(
            id,
            InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Normal,
            },
        );
    }
    store.set_tooltip(INSP_PIXEL_PIVOT_DD, tr("card.insp.pixel_pivot_tip"));
    dropdown::declare(INSP_PIXEL_PIVOT_DD);
}

/// Uma linha escolhida vira o pedido ao documento do objecto (o do cartão preso, ou o selecionado).
pub(crate) fn evento(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(i) = INSP_PIXEL_PIVOT_OPT.iter().position(|&o| o == id) else {
        return false;
    };
    let bits = host
        .store()
        .card_event_subject(ev)
        .or_else(|| crate::state::current_inspector_transform().map(|t| t.entity_bits));
    let Some(card) = bits.and_then(|b| host.store().pixel_sprite_card(b)) else {
        return true;
    };
    let slice = match i {
        0 => None,
        i => match card.pivots.get(i - 1) {
            Some((s, _)) => Some(*s),
            None => return true,
        },
    };
    let doc = card.doc;
    host.bus_mut()
        .push(EditorAction::Pixel(PixelRequest::PivotSlice { doc, slice }));
    true
}

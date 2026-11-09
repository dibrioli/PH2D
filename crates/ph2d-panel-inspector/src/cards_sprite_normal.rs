//! ⭐ **A linha Normal Map** do cartão Image (Sprite e Animated Sprite), P14: a ranhura abre o painel de Assets para
//! escolher a normal da IMAGEM mostrada (todo objecto com essa imagem acende por ela); **None** tira-a.

use crate::ids;
use crate::slot_image;
use ph2d_editor_core::ImageUseEdit;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget};
use ph2d_i18n::tr;

/// Os dois botões, uma vez por quadro.
pub(crate) fn prepara(store: &mut WidgetStore) {
    for (id, dica) in [
        (ids::INSP_IMAGE_NORMAL_SLOT, "card.insp.normal_map_tip"),
        (ids::INSP_IMAGE_NORMAL_NONE, "card.insp.normal_map_none_tip"),
    ] {
        store.register_if_absent(
            id,
            InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Normal,
            },
        );
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, tr(dica));
        }
    }
}

/// Pinta a linha: o nome da normal (ou «None») e, com normal, o **None** que a tira.
pub(crate) fn linha(c: &mut CardCtx<'_>, y: f32, bits: u64) -> f32 {
    let nome = slot_image::current_normal_map(bits);
    let texto = nome.as_deref().unwrap_or(tr("card.insp.normal_map_none"));
    let slot = Cell::new(Widget::Button {
        id: ids::INSP_IMAGE_NORMAL_SLOT,
        text: texto,
        primary: false,
    })
    .label(tr("card.insp.normal_map"));
    if nome.is_none() {
        return c.row(&[slot], y);
    }
    let none = Cell::new(Widget::Button {
        id: ids::INSP_IMAGE_NORMAL_NONE,
        text: tr("card.insp.normal_map_none"),
        primary: false,
    });
    c.row(&[slot, none], y)
}

/// O clique: a ranhura pede o seletor (`LoadNormalMap`), o **None** tira a normal (`SetNormalMap(None)`).
pub(crate) fn evento(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let edit = match ev {
        WidgetEvent::Click(id) if id == ids::INSP_IMAGE_NORMAL_SLOT => ImageUseEdit::LoadNormalMap,
        WidgetEvent::Click(id) if id == ids::INSP_IMAGE_NORMAL_NONE => {
            ImageUseEdit::SetNormalMap(None)
        }
        _ => return false,
    };
    let Some(sprite) = crate::state::current_inspector_sprite() else {
        return true;
    };
    host.bus_mut().push(EditorAction::ImageUseEdit {
        entity_bits: sprite.entity_bits,
        edit,
    });
    true
}

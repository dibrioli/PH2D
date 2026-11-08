//! Os cartões User Data do cel e da tag activos (P12 U; Aseprite R5) nas propriedades da linha do
//! tempo: os ids, o quadro e os gestos.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::cards::user_data::{self, UserDataIds, UserDataView};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::pixel_panel::{PixelOwner, PixelPanelInfo, PixelRequest};

use crate::state::with_current;

/// Os ids do cartão do cel.
#[must_use]
pub fn cel_ids() -> UserDataIds {
    UserDataIds::new("pixel_cel")
}

/// Os ids do cartão da tag.
#[must_use]
pub fn tag_ids() -> UserDataIds {
    UserDataIds::new("pixel_tag")
}

fn donos(i: &PixelPanelInfo) -> Vec<(PixelOwner, UserDataIds, UserDataView)> {
    let mut v = Vec::new();
    if let (Some(c), Some(u)) = (crate::paint_props::cel_activo(i), &i.user.cel) {
        let o = PixelOwner::Cel {
            layer: c.layer,
            frame: c.frame,
        };
        v.push((o, cel_ids(), u.clone()));
    }
    if let (Some(t), Some(u)) = (crate::event::tag_activa(i), &i.user.tag) {
        v.push((PixelOwner::Tag(t), tag_ids(), u.clone()));
    }
    v
}

/// Antes de pintar: regista e semeia; a cor do seletor vira pedido.
pub(crate) fn quadro(host: &mut dyn PanelHostInternal) {
    for (o, ids, v) in with_current(donos) {
        if let Some(e) = user_data::frame(host.store_mut(), &ids, &v) {
            host.bus_mut()
                .push(EditorAction::Pixel(PixelRequest::UserData(o, e)));
        }
    }
}

/// O gesto, se `ev` é de um dos cartões.
pub(crate) fn evento(host: &mut dyn PanelHostInternal, ev: &WidgetEvent) -> Option<EventOutcome> {
    with_current(donos).into_iter().find_map(|(o, ids, v)| {
        let e = user_data::on_event(host.store(), &ids, &v, ev)?;
        if let Some(e) = e {
            host.bus_mut()
                .push(EditorAction::Pixel(PixelRequest::UserData(o, e)));
        }
        Some(EventOutcome::Consumed)
    })
}

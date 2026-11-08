//! O cartão User Data da camada activa (P12 U, O17) no painel PIXEL: os ids, o quadro e os gestos.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::user_data::{self, UserDataIds};
use ph2d_editor_core::pixel_panel::{PixelOwner, PixelRequest};

use crate::state::with_current;

/// Os ids do cartão da camada.
#[must_use]
pub fn ids() -> UserDataIds {
    UserDataIds::new("pixel_layer")
}

fn dono() -> Option<(PixelOwner, user_data::UserDataView)> {
    with_current(|i| Some((PixelOwner::Layer(i.active_layer?), i.user.layer.clone()?)))
}

/// Antes de pintar: regista e semeia; a cor do seletor vira pedido.
pub(crate) fn quadro(host: &mut dyn PanelHostInternal) {
    let Some((o, v)) = dono() else {
        return;
    };
    if let Some(e) = user_data::frame(host.store_mut(), &ids(), &v) {
        let _ = crate::seam::empurra(host, Some(PixelRequest::UserData(o, e)));
    }
}

/// O gesto, se `ev` é do cartão.
pub(crate) fn evento(host: &mut dyn PanelHostInternal, ev: &WidgetEvent) -> Option<EventOutcome> {
    let (o, v) = dono()?;
    let e = user_data::on_event(host.store(), &ids(), &v, ev)?;
    let _ = crate::seam::empurra(host, e.map(|e| PixelRequest::UserData(o, e)));
    Some(EventOutcome::Consumed)
}

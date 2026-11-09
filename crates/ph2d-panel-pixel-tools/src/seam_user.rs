//! O cartão User Data da camada activa e o do sprite (Sprite ▸ Properties…, quando aberta) no
//! painel PIXEL (P12 U, O17): os ids, o quadro e os gestos.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::user_data::{self, UserDataIds};
use ph2d_editor_core::pixel_panel::user_data::UserDataView;
use ph2d_editor_core::pixel_panel::{PixelOwner, PixelRequest, sprite_props};

use crate::state::with_current;

/// Os ids do cartão da camada.
#[must_use]
pub fn ids() -> UserDataIds {
    UserDataIds::new("pixel_layer")
}

/// Os ids do cartão do sprite.
#[must_use]
pub fn ids_sprite() -> UserDataIds {
    UserDataIds::new("pixel_sprite")
}

/// Os donos com cartão neste quadro: a camada activa e, com a secção aberta, o sprite.
fn donos() -> Vec<(PixelOwner, UserDataIds, UserDataView)> {
    with_current(|i| {
        let camada = i
            .active_layer
            .zip(i.user.layer.clone())
            .map(|(l, v)| (PixelOwner::Layer(l), ids(), v));
        let sprite = i
            .user
            .sprite
            .clone()
            .filter(|_| sprite_props::is_open())
            .map(|v| (PixelOwner::Sprite, ids_sprite(), v));
        camada.into_iter().chain(sprite).collect()
    })
}

/// Antes de pintar: regista e semeia; a cor do seletor vira pedido.
pub(crate) fn quadro(host: &mut dyn PanelHostInternal) {
    for (o, ids, v) in donos() {
        if let Some(e) = user_data::frame(host.store_mut(), &ids, &v) {
            let _ = crate::seam::empurra(host, Some(PixelRequest::UserData(o, e)));
        }
    }
}

/// O gesto, se `ev` é de um dos cartões.
pub(crate) fn evento(host: &mut dyn PanelHostInternal, ev: &WidgetEvent) -> Option<EventOutcome> {
    let (o, e) = donos()
        .into_iter()
        .find_map(|(o, ids, v)| user_data::on_event(host.store(), &ids, &v, ev).map(|e| (o, e)))?;
    let _ = crate::seam::empurra(host, e.map(|e| PixelRequest::UserData(o, e)));
    Some(EventOutcome::Consumed)
}

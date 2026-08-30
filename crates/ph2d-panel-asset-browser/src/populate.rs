//! Registo dos widgets FIXOS (1× na instalação do painel).
//!
//! ⚠️ **As células NÃO se registam aqui**, e isso é deliberado: quantos cartões existem só se sabe
//! em runtime, e o paint regista o que pinta (é o mesmo idioma da tira do Flip e das linhas do
//! painel de camadas). ⛔ Um `for i in 0..MAX_ASSET_CELLS` aqui registaria 512 ids de que 20 são
//! pintados — os outros 492 seriam ids **órfãos**, que é a terceira espécie do §5.0 e cuja cura é
//! oposta à do knob morto.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

pub(crate) fn populate(store: &mut WidgetStore) {
    // ⚠️ **O pill do topo entra AQUI.** Ele já estava registado pela `topbar::populate` (é um dos
    // três chips do grupo da direita), então este registo é idempotente — o que faltava nunca foi
    // o registo: era o DESPACHO, que este painel passa a ter.
    for id in [
        ids::ASSET_CLOSE,
        ids::ASSET_DRAG_HANDLE,
        ids::ASSET_RESIZE_HANDLE_BL,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for id in ids::ASSET_KIND.iter().take(ids::ASSET_KIND_FILTERS) {
        store.register(
            *id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for id in ids::ASSET_SORT.iter().take(ids::ASSET_SORT_MODES) {
        store.register(
            *id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    store.register(
        ids::ASSET_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.register(
        ids::ASSET_SIZE,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: crate::state::AssetBrowserState::default().size_slider_value(),
            orientation: SliderOrientation::Horizontal,
        },
    );
    // ⛔ **A barra NÃO se regista**, e a ausência é a lei do substrato: um polegar não tem
    // `InteractiveState` nenhum — o arrasto vive no `scrollbar_drag()` (chaveado pelo PAINEL) e o
    // hover no `hot_id()` (chaveado pelo POLEGAR). O par visual sai do `scrollbar_visual_for`.
}

//! **O despacho da secção MACRO** (ADR-0186, W5d): o modo vai ao barramento; «Edit steps…» abre o
//! painel MACROS, que lista as etapas.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::macro_edits::MacroFieldEdit as E;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::ButtonState;

/// O id do painel MACROS (`ph2d_panel_macros::MacrosPanel::ID`; um painel não vê o outro).
pub(crate) const PAINEL_MACROS: &str = "macros";

/// Despacha um evento da secção. `true` = consumido.
pub(crate) fn apply_macro_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(info) = crate::state_components::current_inspector_macro() else {
        return false;
    };
    if let Some(i) = crate::ids::INSP_MACRO_MODO.iter().position(|&o| o == id) {
        if let Ok(modo) = u8::try_from(i) {
            host.bus_mut().push(EditorAction::InspectorComponentEdit {
                entity_bits: info.entity_bits,
                edit: ComponentEdit::Macro(E::Modo(modo)),
            });
        }
    } else if id == crate::ids::INSP_MACRO_EDITAR_ETAPAS {
        host.set_panel_visible(PAINEL_MACROS, true);
    } else {
        return false;
    }
    // Repõe o visual do botão momentâneo — senão ele fica `Pressed` depois do clique.
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
    true
}

//! §13 Pulley Wheel — os arms de evento do Inspector (W-Pulley W1).
//!
//! Módulo próprio pelo mesmo motivo do irmão `event_joint.rs`: este arquivo é a
//! resposta inteira a *"o que acontece quando o artista mexe num controle de
//! roldana"*.
//!
//! **Todo arm é gateado na seção estar VIVA.** O pintor só oferece estes widgets
//! para uma entidade que carrega uma roldana, mas uma recusa que mora no laço de
//! pintura não é recusa — os ids existem no store a sessão inteira, e um clique
//! roteado por outra coisa chegaria aqui sobre um objeto que não é roldana
//! nenhuma ([[feedback_disabled_button_still_dispatches]]).

use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::WheelFieldEdit;

pub(crate) fn apply_wheel_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = state::current_inspector_wheel() else {
        return false;
    };
    let edit = match ev {
        WidgetEvent::Click(id) => ids::INSP_WHEEL_WRAP
            .iter()
            .position(|&o| o == id)
            .map(|i| WheelFieldEdit::Wrap(i as u8)),
        WidgetEvent::ValueChanged(id) => {
            let v = host.store().number_value(id).unwrap_or(0.0);
            match id {
                ids::INSP_WHEEL_RADIUS => Some(WheelFieldEdit::Radius(v as f32)),
                // ⚠️ A caixa fala `f64` e a ordem é um ORDINAL: o arredondamento
                // mora aqui, na fronteira, como o `i8` da Dominance. E o piso é
                // 1 porque a row é 1-based — `0` chegaria à shell como "o nó
                // anterior ao primeiro", que não existe.
                ids::INSP_WHEEL_ORDER => Some(WheelFieldEdit::Order(v.round().max(1.0) as u32)),
                // Graus na row, radianos no componente — a shell converte.
                ids::INSP_WHEEL_MOTOR => Some(WheelFieldEdit::MotorDegPerS(v as f32)),
                _ => None,
            }
        }
        _ => None,
    };
    if let Some(edit) = edit {
        host.bus_mut().push(EditorAction::InspectorWheelEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    false
}

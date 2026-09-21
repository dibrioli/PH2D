//! **O despacho da secção LIVE MESH** — o catavento.
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão das irmãs.
//!
//! # ⚠️ O clique lê o SNAPSHOT e nunca o store
//!
//! O store guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o primeiro
//! clique depois de trocar de objecto mandar o valor do objecto **anterior**.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::mesh3d_edits::Mesh3dFieldEdit;
use ph2d_editor_core::panel::PanelHostInternal;

/// Despacha um evento da secção LIVE MESH. `true` = consumido.
pub(crate) fn apply_mesh3d_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_mesh3d() else {
        return false;
    };
    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        // ⚠️ **Um `match` e não um `if` por campo** — com `if`s, o terceiro acaba a escrever no
        // primeiro (a lição dos três campos de texto da tabela de acções).
        let edit = match id {
            crate::ids::INSP_MESH3D_YAW => Mesh3dFieldEdit::YawGraus(v),
            crate::ids::INSP_MESH3D_PITCH => Mesh3dFieldEdit::PitchGraus(v),
            crate::ids::INSP_MESH3D_SPIN => Mesh3dFieldEdit::Spin(v),
            _ => return false,
        };
        host.bus_mut().push(EditorAction::InspectorComponentEdit {
            entity_bits: info.entity_bits,
            edit: ComponentEdit::Mesh3d(edit),
        });
        return true;
    }
    false
}

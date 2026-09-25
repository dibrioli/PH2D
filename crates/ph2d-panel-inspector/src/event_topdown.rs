//! **O despacho da secção TOP-DOWN PLAYER** (TOP-20 #13, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão do [`crate::event_camera`] e
//! do [`crate::event_factory`].
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::topdown_edits::{
    InspectorFacing, InspectorMoveDirections, InspectorViewpoint, TopDownFieldEdit,
};

/// Despacha um evento da secção TOP-DOWN PLAYER. `true` = consumido.
pub(crate) fn apply_topdown_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_topdown() else {
        return false;
    };
    let bits = info.entity_bits;

    if let WidgetEvent::Click(id) = ev {
        // ⚠️ **Os TRÊS segmentados numa varredura só, pela posição** — a mesma lei de todos: a
        // posição no array é a tag do clique, e reordenar faria um clique escrever outro modo.
        if let Some(i) = crate::ids::INSP_TD_DIRECTIONS.iter().position(|&b| b == id) {
            push(
                host,
                bits,
                TopDownFieldEdit::Directions(InspectorMoveDirections::ALL[i]),
            );
            return true;
        }
        if let Some(i) = crate::ids::INSP_TD_VIEWPOINT.iter().position(|&b| b == id) {
            push(
                host,
                bits,
                TopDownFieldEdit::Viewpoint(InspectorViewpoint::ALL[i]),
            );
            return true;
        }
        if let Some(i) = crate::ids::INSP_TD_FACING.iter().position(|&b| b == id) {
            push(
                host,
                bits,
                TopDownFieldEdit::Facing(InspectorFacing::ALL[i]),
            );
            return true;
        }
    }

    if let WidgetEvent::Toggled(id) = ev
        && id == crate::ids::INSP_TD_DEFAULT_CONTROLS
    {
        // ⚠️ **O estado vem do SNAPSHOT**, e a edição é o INVERTIDO dele.
        push(
            host,
            bits,
            TopDownFieldEdit::DefaultControls(!info.default_controls),
        );
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation)]
        let f = v as f32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = v.max(0.0) as u32;
        // ⚠️ **Um `match` e não um `if` por campo** — com `if`s, o terceiro acaba a escrever no
        // primeiro (a lição dos três campos de texto da tabela de acções).
        let edit = match id {
            crate::ids::INSP_TD_SPEED => TopDownFieldEdit::Speed(f),
            crate::ids::INSP_TD_ACCEL => TopDownFieldEdit::Acceleration(f),
            crate::ids::INSP_TD_DECEL => TopDownFieldEdit::Deceleration(f),
            crate::ids::INSP_TD_KNOCKBACK_RECOVERY => TopDownFieldEdit::KnockbackRecovery(f),
            crate::ids::INSP_TD_VIEW_ANGLE => TopDownFieldEdit::ViewpointAngle(f),
            crate::ids::INSP_TD_TURN_SPEED => TopDownFieldEdit::TurnSpeed(f),
            crate::ids::INSP_TD_MIN_SLIDE => TopDownFieldEdit::MinSlideAngle(f),
            crate::ids::INSP_TD_MAX_SLIDES => TopDownFieldEdit::MaxSlides(n),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: TopDownFieldEdit) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::TopDown(edit),
    });
}

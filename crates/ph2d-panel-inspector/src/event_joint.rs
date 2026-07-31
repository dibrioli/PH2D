//! §12 Physics Joint — the Inspector event arms (W3).
//!
//! Its own module for the same reason `event_physics.rs` is one: this file is
//! the whole answer to "what happens when the artist clicks a joint control".
//!
//! **Every arm is gated on the section actually being live.** The painter only
//! offers these widgets for an entity that carries a joint, but a refusal that
//! lives in the paint loop is not a refusal — the ids exist in the store for
//! the whole session, and a click routed by anything else would arrive here on
//! an entity that is not a joint at all
//! ([[feedback_disabled_button_still_dispatches]]).

use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::JointFieldEdit;

pub(crate) fn apply_joint_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = state::current_inspector_joint() else {
        return false;
    };
    let edit = match ev {
        WidgetEvent::Click(id) => {
            if id == ids::INSP_JOINT_REMOVE {
                Some(JointFieldEdit::Remove)
            } else if id == ids::INSP_JOINT_ADD_WHEEL {
                Some(JointFieldEdit::AddWheel)
            } else if id == ids::INSP_JOINT_SWAP {
                Some(JointFieldEdit::Swap)
            } else if id == ids::INSP_JOINT_PICK_A {
                Some(JointFieldEdit::PickBodyA)
            } else if id == ids::INSP_JOINT_PICK_B {
                Some(JointFieldEdit::PickBodyB)
            } else if let Some(i) = ids::INSP_JOINT_KIND.iter().position(|&o| o == id) {
                Some(JointFieldEdit::Kind(i as u8))
            } else if let Some(i) = ids::INSP_JOINT_LIMITS.iter().position(|&o| o == id) {
                Some(JointFieldEdit::LimitsEnabled(i == 1))
            } else if let Some(i) = ids::INSP_JOINT_MOTOR.iter().position(|&o| o == id) {
                Some(JointFieldEdit::MotorEnabled(i == 1))
            } else if let Some(i) = ids::INSP_JOINT_BREAK.iter().position(|&o| o == id) {
                Some(JointFieldEdit::BreakEnabled(i == 1))
            } else if let Some(i) = ids::INSP_JOINT_ACTIVE.iter().position(|&o| o == id) {
                Some(JointFieldEdit::Active(i == 1))
            } else if let Some(i) = ids::INSP_JOINT_COLLIDE.iter().position(|&o| o == id) {
                Some(JointFieldEdit::CollideConnected(i == 1))
            } else if let Some(i) = ids::INSP_JOINT_ANCHOR_B.iter().position(|&o| o == id) {
                Some(JointFieldEdit::AnchorToWorld(i == 1))
            } else {
                ids::INSP_JOINT_MOTOR_MODE
                    .iter()
                    .position(|&o| o == id)
                    .map(|i| JointFieldEdit::MotorMode(i as u8))
            }
        }
        WidgetEvent::ValueChanged(id) => {
            let v = host.store().number_value(id).unwrap_or(0.0) as f32;
            match id {
                ids::INSP_JOINT_LIMIT_MIN => Some(JointFieldEdit::LimitMin(v)),
                ids::INSP_JOINT_LIMIT_MAX => Some(JointFieldEdit::LimitMax(v)),
                ids::INSP_JOINT_MOTOR_SPEED => Some(JointFieldEdit::MotorSpeed(v)),
                ids::INSP_JOINT_MOTOR_TARGET => Some(JointFieldEdit::MotorTarget(v)),
                ids::INSP_JOINT_MOTOR_FORCE => Some(JointFieldEdit::MotorMaxForce(v)),
                ids::INSP_JOINT_REST_LENGTH => Some(JointFieldEdit::RestLength(v)),
                ids::INSP_JOINT_STIFFNESS => Some(JointFieldEdit::Stiffness(v)),
                ids::INSP_JOINT_DAMPING => Some(JointFieldEdit::Damping(v)),
                ids::INSP_JOINT_MAX_LENGTH => Some(JointFieldEdit::MaxLength(v)),
                ids::INSP_JOINT_BREAK_FORCE => Some(JointFieldEdit::BreakForce(v)),
                ids::INSP_JOINT_BREAK_TORQUE => Some(JointFieldEdit::BreakTorque(v)),
                _ => None,
            }
        }
        _ => None,
    };
    if let Some(edit) = edit {
        host.bus_mut().push(EditorAction::InspectorJointEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    false
}

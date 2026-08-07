//! §14 Platform Player — os arms de evento do Inspector (W5).
//!
//! Módulo próprio pelo mesmo motivo dos irmãos `event_joint.rs`/`event_wheel.rs`:
//! este arquivo é a resposta inteira a *"o que acontece quando o artista mexe
//! num controle de player"*.
//!
//! **Todo arm é gateado na seção estar VIVA.** O pintor só oferece estes widgets
//! para um corpo Dynamic, mas uma recusa que mora no laço de pintura não é
//! recusa — os ids existem no store a sessão inteira, e um clique roteado por
//! outra coisa chegaria aqui sobre um objeto que não é player nenhum
//! ([[feedback_disabled_button_still_dispatches]]).

use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::PlayerFieldEdit;

pub(crate) fn apply_player_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = state::current_inspector_player() else {
        return false;
    };
    let edit = match ev {
        WidgetEvent::Click(id) if id == ids::INSP_PLAYER_ADD => Some(PlayerFieldEdit::Add),
        WidgetEvent::Click(id) if id == ids::INSP_PLAYER_REMOVE => Some(PlayerFieldEdit::Remove),
        WidgetEvent::Click(id) if id == ids::INSP_PLAYER_FIT => {
            Some(PlayerFieldEdit::FitFloatHeight)
        }
        WidgetEvent::Click(id) if id == ids::INSP_PLAYER_CLEAR_RUN => {
            Some(PlayerFieldEdit::ClearRun)
        }
        WidgetEvent::Click(id) if id == ids::INSP_PLAYER_FIT_CROUCH => {
            Some(PlayerFieldEdit::FitCrouchHeight)
        }
        WidgetEvent::ValueChanged(id) => {
            let v = host.store().number_value(id).unwrap_or(0.0) as f32;
            match id {
                ids::INSP_PLAYER_FLOAT => Some(PlayerFieldEdit::FloatHeight(v)),
                ids::INSP_PLAYER_CLING => Some(PlayerFieldEdit::ClingDistance(v)),
                ids::INSP_PLAYER_STIFFNESS => Some(PlayerFieldEdit::SpringStrength(v)),
                ids::INSP_PLAYER_DAMPING => Some(PlayerFieldEdit::SpringDamping(v)),
                ids::INSP_PLAYER_SPEED => Some(PlayerFieldEdit::Speed(v)),
                ids::INSP_PLAYER_ACCEL => Some(PlayerFieldEdit::Acceleration(v)),
                ids::INSP_PLAYER_AIR_ACCEL => Some(PlayerFieldEdit::AirAcceleration(v)),
                ids::INSP_PLAYER_JUMP_HEIGHT => Some(PlayerFieldEdit::JumpHeight(v)),
                ids::INSP_PLAYER_TAKEOFF_G => Some(PlayerFieldEdit::TakeoffGravity(v)),
                ids::INSP_PLAYER_TAKEOFF_SPEED => Some(PlayerFieldEdit::TakeoffSpeed(v)),
                ids::INSP_PLAYER_PEAK_G => Some(PlayerFieldEdit::PeakGravity(v)),
                ids::INSP_PLAYER_PEAK_SPEED => Some(PlayerFieldEdit::PeakSpeed(v)),
                ids::INSP_PLAYER_FALL_G => Some(PlayerFieldEdit::FallGravity(v)),
                ids::INSP_PLAYER_CUT_G => Some(PlayerFieldEdit::CutGravity(v)),
                ids::INSP_PLAYER_COYOTE => Some(PlayerFieldEdit::CoyoteTime(v)),
                ids::INSP_PLAYER_BUFFER => Some(PlayerFieldEdit::JumpBuffer(v)),
                ids::INSP_PLAYER_CORNER => Some(PlayerFieldEdit::CornerReach(v)),
                ids::INSP_PLAYER_LIFT => Some(PlayerFieldEdit::LiftMomentum(v)),
                ids::INSP_PLAYER_WALL_SLIDE => Some(PlayerFieldEdit::WallSlideSpeed(v)),
                ids::INSP_PLAYER_WALL_JUMP => Some(PlayerFieldEdit::WallJumpHeight(v)),
                ids::INSP_PLAYER_WALL_PUSH => Some(PlayerFieldEdit::WallJumpPush(v)),
                ids::INSP_PLAYER_WALL_LOCK => Some(PlayerFieldEdit::WallJumpLockout(v)),
                ids::INSP_PLAYER_WALL_REACH => Some(PlayerFieldEdit::WallReach(v)),
                ids::INSP_PLAYER_WALL_GRAB => Some(PlayerFieldEdit::WallGrabStamina(v)),
                ids::INSP_PLAYER_DASH_SPEED => Some(PlayerFieldEdit::DashSpeed(v)),
                ids::INSP_PLAYER_DASH_TIME => Some(PlayerFieldEdit::DashTime(v)),
                ids::INSP_PLAYER_DASH_COOL => Some(PlayerFieldEdit::DashCooldown(v)),
                ids::INSP_PLAYER_CROUCH_HEIGHT => Some(PlayerFieldEdit::CrouchHeight(v)),
                ids::INSP_PLAYER_CROUCH_SPEED => Some(PlayerFieldEdit::CrouchSpeed(v)),
                ids::INSP_PLAYER_REACT_SUPPORT => Some(PlayerFieldEdit::ReactionSupport(v)),
                ids::INSP_PLAYER_REACT_MOVEMENT => Some(PlayerFieldEdit::ReactionMovement(v)),
                // Graus na row, cosseno no motor — a conversão acontece UMA vez,
                // na `WalkConfig::max_slope_cos`.
                ids::INSP_PLAYER_MAX_SLOPE => Some(PlayerFieldEdit::MaxSlopeDeg(v)),
                _ => None,
            }
        }
        _ => None,
    };
    if let Some(edit) = edit {
        host.bus_mut().push(EditorAction::InspectorPlayerEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    false
}

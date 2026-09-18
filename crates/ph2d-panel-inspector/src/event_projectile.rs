//! **O despacho da secção PROJECTILE MOTION** (TOP-20 #14, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão das irmãs.
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::projectile_edits::ProjectileFieldEdit;

/// Despacha um evento da secção PROJECTILE MOTION. `true` = consumido.
pub(crate) fn apply_projectile_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_projectile() else {
        return false;
    };
    let bits = info.entity_bits;

    if let WidgetEvent::Toggled(id) = ev
        && id == crate::ids::INSP_PJ_FACE_VELOCITY
    {
        // ⚠️ **O estado vem do SNAPSHOT**, e a edição é o INVERTIDO dele.
        push(
            host,
            bits,
            ProjectileFieldEdit::FaceVelocity(!info.face_velocity),
        );
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && id == crate::ids::INSP_PJ_HOMING_TARGET
    {
        // ⚠️ **O NOME cru** — quem o converte em `stable_name_id` é a shell, que tem o mundo.
        let nome = host.store().text(id).unwrap_or("").to_string();
        push(host, bits, ProjectileFieldEdit::HomingTarget(nome));
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
            crate::ids::INSP_PJ_SPEED => ProjectileFieldEdit::InitialSpeed(f),
            crate::ids::INSP_PJ_ACCEL => ProjectileFieldEdit::Acceleration(f),
            crate::ids::INSP_PJ_MAX_SPEED => ProjectileFieldEdit::MaxSpeed(f),
            crate::ids::INSP_PJ_GRAVITY => ProjectileFieldEdit::Gravity(f),
            crate::ids::INSP_PJ_BOUNCINESS => ProjectileFieldEdit::Bounciness(f),
            crate::ids::INSP_PJ_MAX_BOUNCES => ProjectileFieldEdit::MaxBounces(n),
            crate::ids::INSP_PJ_RANGE => ProjectileFieldEdit::Range(f),
            crate::ids::INSP_PJ_HOMING_ACCEL => ProjectileFieldEdit::HomingAccel(f),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: ProjectileFieldEdit) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::Projectile(edit),
    });
}

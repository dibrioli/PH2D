//! **O despacho das secções HEALTH e DAMAGE** (plano 28, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão das irmãs.
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual — a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::vida_edits::{InspectorVidaInfo, VidaFieldEdit as E};

use crate::ids;

/// A caixa clicada, e a edição que ela pede — o INVERTIDO do que o snapshot diz.
fn caixa(id: ph2d_a11y::NodeId, i: &InspectorVidaInfo) -> Option<E> {
    let h = i.health.as_ref();
    let d = i.damage.as_ref();
    let b = i.bar.as_ref();
    Some(match id {
        ids::INSP_VIDA_OVERHEAL => E::Overheal(!h?.overheal),
        ids::INSP_VIDA_SHIELD_BLOCKS => E::ShieldBlocksExcess(!h?.shield_blocks_excess),
        ids::INSP_DANO_PER_SECOND => E::PerSecond(!d?.per_second),
        ids::INSP_DANO_IGNORES_SHIELD => E::IgnoresShield(!d?.ignores_shield),
        ids::INSP_DANO_IGNORES_ARMOR => E::IgnoresArmor(!d?.ignores_armor),
        ids::INSP_DANO_VANISH => E::Vanish(!d?.vanish),
        ids::INSP_BARRA_HIDE_FULL => E::BarHideWhenFull(!b?.hide_when_full),
        ids::INSP_VIDA_NUMBERS => E::Numbers(!h?.numbers),
        _ => return None,
    })
}

/// Um número editado. ⚠️ **Um `match` e não um `if` por campo** — com `if`s, o terceiro acaba a
/// escrever no primeiro (a lição dos três campos de texto da tabela de acções).
fn numero(id: ph2d_a11y::NodeId, v: f64) -> Option<E> {
    #[allow(clippy::cast_possible_truncation)]
    let f = v as f32;
    Some(match id {
        ids::INSP_VIDA_MAX => E::Max(f),
        ids::INSP_VIDA_START => E::Start(f),
        ids::INSP_VIDA_INVINCIBLE => E::InvincibleS(f),
        ids::INSP_VIDA_REGEN => E::Regen(f),
        ids::INSP_VIDA_REGEN_DELAY => E::RegenDelayS(f),
        ids::INSP_VIDA_SHIELD_START => E::ShieldStart(f),
        ids::INSP_VIDA_SHIELD_MAX => E::ShieldMax(f),
        ids::INSP_VIDA_SHIELD_DURATION => E::ShieldDurationS(f),
        ids::INSP_VIDA_SHIELD_REGEN => E::ShieldRegen(f),
        ids::INSP_VIDA_SHIELD_REGEN_DELAY => E::ShieldRegenDelayS(f),
        ids::INSP_VIDA_ARMOR => E::ArmorFlat(f),
        ids::INSP_VIDA_ARMOR_PCT => E::ArmorPercent(f),
        ids::INSP_VIDA_DODGE => E::Dodge(f),
        // ⚠️ A semente é INTEIRA: arredondada, nunca truncada por um `as` que a faria andar
        // para baixo a cada gesto.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        ids::INSP_VIDA_SEED => E::Seed(v.max(0.0).round() as u64),
        ids::INSP_DANO_AMOUNT => E::DamageAmount(f),
        ids::INSP_VIDA_DEATH_HITSTOP => E::DeathHitstopS(f),
        ids::INSP_VIDA_BLINK => E::BlinkS(f),
        ids::INSP_VIDA_KNOCKBACK_TAKEN => E::KnockbackTaken(f),
        ids::INSP_VIDA_NUMBERS_SIZE => E::NumbersSize(f),
        ids::INSP_DANO_HITSTOP => E::HitstopS(f),
        ids::INSP_DANO_KNOCKBACK => E::Knockback(f),
        ids::INSP_DANO_KNOCKBACK_LIFT => E::KnockbackLift(f),
        ids::INSP_BARRA_WIDTH => E::BarWidth(f),
        ids::INSP_BARRA_HEIGHT => E::BarHeight(f),
        ids::INSP_BARRA_OFFSET_X => E::BarOffsetX(f),
        ids::INSP_BARRA_OFFSET_Y => E::BarOffsetY(f),
        ids::INSP_BARRA_TRAIL_DELAY => E::BarTrailDelayS(f),
        ids::INSP_BARRA_TRAIL_SPEED => E::BarTrailSpeed(f),
        _ => return None,
    })
}

/// Um texto editado.
fn texto(id: ph2d_a11y::NodeId, t: String) -> Option<E> {
    Some(match id {
        ids::INSP_VIDA_TEAM => E::Team(t),
        ids::INSP_VIDA_ON_DAMAGE => E::OnDamage(t),
        ids::INSP_VIDA_ON_HEAL => E::OnHeal(t),
        ids::INSP_VIDA_ON_DEATH => E::OnDeath(t),
        ids::INSP_DANO_TEAM => E::DamageTeam(t),
        ids::INSP_BARRA_TARGET => E::BarTarget(t),
        _ => return None,
    })
}

/// Despacha um evento das secções HEALTH e DAMAGE. `true` = consumido.
pub(crate) fn apply_vida_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_vida() else {
        return false;
    };
    let edit = match ev {
        WidgetEvent::Toggled(id) => caixa(id, &info),
        WidgetEvent::ValueChanged(id) => {
            let v = host.store().number_value(id).unwrap_or(0.0);
            numero(id, v)
        }
        WidgetEvent::TextChanged(id) => {
            let t = host.store().text(id).unwrap_or("").to_string();
            texto(id, t)
        }
        _ => None,
    };
    let Some(edit) = edit else {
        return false;
    };
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits: info.entity_bits,
        edit: ComponentEdit::Vida(edit),
    });
    true
}

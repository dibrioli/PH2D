//! **O despacho das secções HEALTH e DAMAGE** (plano 28, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão das irmãs.
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual — a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::vida_edits::{InspectorVidaInfo, VidaFieldEdit as E};

use crate::ids;
use crate::state::InspectorState;

/// A caixa clicada, e a edição que ela pede — o INVERTIDO do que o snapshot diz. `aberta` é a
/// resistência cujo editor está pintado.
fn caixa(id: ph2d_a11y::NodeId, i: &InspectorVidaInfo, aberta: Option<usize>) -> Option<E> {
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
        ids::INSP_VIDA_RESIST_ABSORBS => {
            let k = aberta?;
            E::ResistanceAbsorbs(u8::try_from(k).ok()?, !h?.resistances.get(k)?.absorbs)
        }
        _ => return None,
    })
}

/// Um número editado. ⚠️ **Um `match` e não um `if` por campo** — com `if`s, o terceiro acaba a
/// escrever no primeiro (a lição dos três campos de texto da tabela de acções).
fn numero(id: ph2d_a11y::NodeId, v: f64, aberta: Option<usize>) -> Option<E> {
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
        ids::INSP_DANO_OT_PER_S => E::OverTimePerS(f),
        ids::INSP_DANO_OT_S => E::OverTimeS(f),
        ids::INSP_DANO_OT_EVERY => E::OverTimeEveryS(f),
        ids::INSP_VIDA_RESIST_RATE => E::ResistanceRate(u8::try_from(aberta?).ok()?, f),
        _ => return None,
    })
}

/// Um texto editado.
fn texto(id: ph2d_a11y::NodeId, t: String, aberta: Option<usize>) -> Option<E> {
    Some(match id {
        ids::INSP_VIDA_TEAM => E::Team(t),
        ids::INSP_VIDA_ON_DAMAGE => E::OnDamage(t),
        ids::INSP_VIDA_ON_HEAL => E::OnHeal(t),
        ids::INSP_VIDA_ON_DEATH => E::OnDeath(t),
        ids::INSP_DANO_TEAM => E::DamageTeam(t),
        ids::INSP_BARRA_TARGET => E::BarTarget(t),
        ids::INSP_DANO_KIND => E::DamageKind(t),
        ids::INSP_VIDA_RESIST_KIND => E::ResistanceKind(u8::try_from(aberta?).ok()?, t),
        _ => return None,
    })
}

/// ⭐ **Os cliques da lista de RESISTÊNCIAS** (W6) — abrir uma linha, juntar, tirar. `Some(edição)`
/// quando o clique vai ao barramento, `Some(None)` quando ele só mexe na selecção (⚠️ abrir uma
/// linha NÃO vai ao barramento: um passo de undo por clique sobre um facto que a cena não tem), e
/// `None` quando o clique não é desta lista.
fn clique_da_lista(
    panel: &mut InspectorState,
    id: ph2d_a11y::NodeId,
    info: &InspectorVidaInfo,
    aberta: Option<usize>,
) -> Option<Option<E>> {
    let n = info.health.as_ref()?.resistances.len();
    if let Some(i) = ids::INSP_VIDA_RESIST_ROW.iter().position(|&o| o == id)
        && i < n
    {
        panel.resist_selected = i;
        return Some(None);
    }
    if id == ids::INSP_VIDA_RESIST_ADD {
        // ⚠️ **Abre a que acabou de nascer** — senão o `+` lê-se como se não fizesse nada.
        panel.resist_selected = n;
        return Some(Some(E::AddResistance));
    }
    if id == ids::INSP_VIDA_RESIST_REMOVE {
        let k = aberta?;
        panel.resist_selected = k.saturating_sub(1);
        return Some(Some(E::RemoveResistance(u8::try_from(k).ok()?)));
    }
    None
}

/// Despacha um evento das secções HEALTH e DAMAGE. `true` = consumido.
pub(crate) fn apply_vida_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = crate::state_components::current_inspector_vida() else {
        return false;
    };
    let aberta = crate::sync_vida::resistencia_aberta(&info, panel.resist_selected);
    let edit = match ev {
        WidgetEvent::Click(id) => {
            let Some(pedido) = clique_da_lista(panel, id, &info, aberta) else {
                return false;
            };
            // Repõe o visual do botão momentâneo — senão ele fica `Pressed` depois do clique.
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ph2d_editor_core::widget::ButtonState::Normal;
            }
            match pedido {
                Some(e) => Some(e),
                None => return true,
            }
        }
        WidgetEvent::Toggled(id) => caixa(id, &info, aberta),
        WidgetEvent::ValueChanged(id) => {
            let v = host.store().number_value(id).unwrap_or(0.0);
            numero(id, v, aberta)
        }
        WidgetEvent::TextChanged(id) => {
            let t = host.store().text(id).unwrap_or("").to_string();
            texto(id, t, aberta)
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

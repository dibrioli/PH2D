//! **O despacho da secção WEAPON** — a arma do jogador.
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
use ph2d_editor_core::weapon_edits::WeaponFieldEdit;

/// Despacha um evento da secção WEAPON. `true` = consumido.
pub(crate) fn apply_weapon_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_weapon() else {
        return false;
    };
    let bits = info.entity_bits;

    if let WidgetEvent::TextChanged(id) = ev {
        // ⚠️ **O NOME cru** — quem o apara é quem o lê (a regra do `SignalOnHit`).
        let nome = host.store().text(id).unwrap_or("").to_string();
        // ⚠️ **Um `match` e não um `if` por campo** — com `if`s, o terceiro acaba a escrever no
        // primeiro (a lição dos três campos de texto da tabela de acções).
        let edit = match id {
            crate::ids::INSP_WEAPON_ON_SIGNAL => WeaponFieldEdit::OnSignal(nome),
            crate::ids::INSP_WEAPON_AMMO => WeaponFieldEdit::AmmoCounter(nome),
            crate::ids::INSP_WEAPON_RELOAD_ON => WeaponFieldEdit::ReloadOn(nome),
            crate::ids::INSP_WEAPON_ON_FIRE => WeaponFieldEdit::OnFire(nome),
            crate::ids::INSP_WEAPON_ON_EMPTY => WeaponFieldEdit::OnEmpty(nome),
            crate::ids::INSP_WEAPON_ON_RELOADED => WeaponFieldEdit::OnReloaded(nome),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let ms = v.max(0.0) as u64;
        let edit = match id {
            crate::ids::INSP_WEAPON_COOLDOWN => WeaponFieldEdit::CooldownMs(ms),
            crate::ids::INSP_WEAPON_RELOAD_MS => WeaponFieldEdit::ReloadMs(ms),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: WeaponFieldEdit) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::Weapon(edit),
    });
}

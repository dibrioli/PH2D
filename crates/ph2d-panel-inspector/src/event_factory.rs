//! **O despacho das secções FACTORY e LIFECYCLE** (TOP-20 #11 e #12, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão do [`crate::event_camera`].
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.
//!
//! # ⚠️ Os dois eixos da área viajam JUNTOS
//!
//! A caixa é um `[f32; 2]` **no componente**, e o descritor espelha a ESTRUTURA. ⇒ mexer na largura
//! manda o par inteiro, com a altura lida do snapshot. ⛔ Mandar meio par obrigaria a shell a ler o
//! outro eixo do mundo, e é assim que dois escritores do mesmo campo nascem.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{FactoryFieldEdit, InspectorSpawnWhere};

/// Despacha um evento das secções FACTORY / LIFECYCLE. `true` = consumido.
pub(crate) fn apply_factory_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_factory() else {
        return false;
    };
    let bits = info.entity_bits;
    let fab = info.factory.clone().unwrap_or_default();
    let vida = info.lifecycle.clone().unwrap_or_default();

    if let WidgetEvent::Click(id) = ev
        && let Some(i) = crate::ids::INSP_FACTORY_WHERE.iter().position(|&b| b == id)
    {
        let modo = InspectorSpawnWhere::ALL[i];
        push(host, bits, FactoryFieldEdit::Where(modo));
        return true;
    }

    if let WidgetEvent::Toggled(id) = ev {
        // ⚠️ **O estado vem do SNAPSHOT**, e a edição é o INVERTIDO dele.
        if id == crate::ids::INSP_FACTORY_PICK_RANDOM {
            push(host, bits, FactoryFieldEdit::PickRandom(!fab.pick_random));
            return true;
        }
        if id == crate::ids::INSP_FACTORY_AIM {
            push(
                host,
                bits,
                FactoryFieldEdit::AimFromSpawner(!fab.aim_from_spawner),
            );
            return true;
        }
    }

    if let WidgetEvent::TextChanged(id) = ev {
        let text = host.store().text(id).unwrap_or("").to_string();
        let edit = match id {
            crate::ids::INSP_FACTORY_RECIPE => FactoryFieldEdit::Recipe(text),
            crate::ids::INSP_FACTORY_ON_SIGNAL => FactoryFieldEdit::OnSignal(text),
            crate::ids::INSP_FACTORY_TAG => FactoryFieldEdit::Tag(text),
            crate::ids::INSP_FACTORY_ON_SPAWNED => FactoryFieldEdit::OnSpawned(text),
            crate::ids::INSP_FACTORY_ON_EXHAUSTED => FactoryFieldEdit::OnExhausted(text),
            crate::ids::INSP_LIFE_ON_DEATH => FactoryFieldEdit::OnDeath(text),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation)]
        let f = v as f32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = v.max(0.0) as u32;
        // ⚠️ **Um `match` e não um `if` por campo** — a mesma razão que pôs os três campos de texto
        // da tabela de acções numa porta só: com `if`s, o terceiro acaba a escrever no primeiro.
        let edit = match id {
            crate::ids::INSP_FACTORY_AREA_W => FactoryFieldEdit::Area([f, fab.area[1]]),
            crate::ids::INSP_FACTORY_AREA_H => FactoryFieldEdit::Area([fab.area[0], f]),
            crate::ids::INSP_FACTORY_BURST => FactoryFieldEdit::Burst(n),
            crate::ids::INSP_FACTORY_ALIVE_MAX => FactoryFieldEdit::AliveMax(n),
            crate::ids::INSP_FACTORY_TOTAL_MAX => FactoryFieldEdit::TotalMax(n),
            crate::ids::INSP_FACTORY_SEED => FactoryFieldEdit::Seed(u64::from(n)),
            crate::ids::INSP_LIFE_SECONDS => FactoryFieldEdit::LifetimeSeconds(f),
            crate::ids::INSP_LIFE_OUTSIDE_MARGIN => FactoryFieldEdit::OutsideMargin(f),
            _ => return false,
        };
        let _ = &vida;
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: FactoryFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorFactoryEdit { entity_bits, edit });
}

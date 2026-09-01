//! ⭐⭐ **RENOMEAR uma linha da Hierarquia** — irmão por RESPONSABILIDADE do
//! [`super::hierarchy`], de onde saiu quando aquele bateu no teto de 600 LOC do shell (HR-18).
//!
//! Semear o campo com o nome actual, gravar o que o `Enter` deixou, limpar o buffer e **honrar as
//! chaves que o nome declara** são uma coisa só; os verbos de instância que o irmão dreno são
//! outra.
//!
//! ⚠️ **A ordem é load-bearing:** o `Name` é escrito ANTES de as chaves valerem, porque a lei
//! ([`crate::instance_declared_value`]) lê o **mundo** e não o texto do campo.

use crate::HeroLive;
use ph2d_ecs::{Name, SimWorld};
use ph2d_editor::{HeroScreen, NodeId, Toast, ToastQueue};

/// Devolve `true` quando o título da janela ficou por refazer.
pub(super) fn drain(
    rename_seed_row: Option<NodeId>,
    rename_commit: Option<(NodeId, String)>,
    hero: &mut HeroScreen,
    hero_live: &Option<HeroLive>,
    sim: &mut SimWorld,
    echo: &mut crate::instance_sync::MasterEcho,
    toasts: &mut ToastQueue,
) -> bool {
    let mut title_dirty = false;
    if let Some(row) = rename_seed_row
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row)
    {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        let value = sim
            .world()
            .get::<Name>(entity)
            .map(|n| n.as_str().to_owned())
            .unwrap_or_default();
        if let Some(ph2d_editor::interaction::InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) = hero
            .store
            .get_mut(ph2d_editor::screens::hero::ids::HIER_RENAME_INPUT)
        {
            let len = value.len();
            *text = value;
            *caret = len;
            *selection_anchor = Some(0); // select all
        }
    }
    // Drain a finalized rename commit (Enter pressed in rename
    // input). Write the new Name component on the entity; toast
    // confirms.
    if let Some((row, new_name)) = rename_commit
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row)
    {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        // Reject user-typed collisions with other entities. `_excluding`
        // ignores `entity`'s own current name so committing the same
        // name (no-op rename) doesn't auto-suffix into "(1)".
        let final_name = crate::name_unique::unique_name_excluding(sim, &new_name, entity);
        let was_adjusted = final_name != new_name;
        let sim_w = sim.world_mut();
        if let Ok(mut entry) = sim_w.get_entity_mut(entity) {
            entry.insert(Name::new(final_name.clone()));
            if was_adjusted {
                toasts.push(Toast::warning(format!(
                    "Name in use — renamed to {final_name}"
                )));
            } else {
                toasts.push(Toast::success(format!("Renamed to {final_name}")));
            }
            title_dirty = true;
        }
        // ⭐ **E as CHAVES do nome valem** — lei em [`crate::instance_declared_value`]; aqui
        // DEPOIS de o `Name` estar escrito, porque ela lê o mundo e não o campo.
        crate::instance_declared_value::speak(
            crate::instance_declared_value::apply(sim, echo, entity),
            toasts,
        );
        // Clear the rename TextInput buffer for next session.
        if let Some(ph2d_editor::interaction::InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) = hero
            .store
            .get_mut(ph2d_editor::screens::hero::ids::HIER_RENAME_INPUT)
        {
            text.clear();
            *caret = 0;
            *selection_anchor = None;
        }
    }

    title_dirty
}

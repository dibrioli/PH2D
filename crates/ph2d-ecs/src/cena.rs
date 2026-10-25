//! ⭐ **A CENA como alvo da timeline** (F30 §B, [plano](../../../docs/UI_New_and_Simple/F30_plano.md)) —
//! uma entidade com [`Cena`] e [`StableId::CENA`], sem `Transform`: o que é do mundo e não de um
//! objecto (a gravidade, `canal_mundo`) tem um binding como qualquer outro, e a resolução, a cura
//! e a persistência da timeline são as que existem (`wire_id = StableId::CENA`).
//!
//! Sem `Transform` ela fica fora da hierarquia, do save (a DFS parte das raízes com `Transform`) e
//! do restauro do undo (que só limpa quem tem `Transform`); a captura incremental salta-a pelo
//! marcador. Um mundo reconstruído (o load) perde-a: [`garante_cena`] repõe-na, a cada quadro.

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Entity, With, World};

use crate::StableId;

/// O marcador da entidade da Cena.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Cena;

/// A entidade da Cena, se existir.
#[must_use]
pub fn entidade_da_cena(world: &World) -> Option<Entity> {
    world
        .try_query_filtered::<Entity, With<Cena>>()?
        .iter(world)
        .next()
}

/// Os bits da entidade da Cena, se existir — o endereço dos canais dela.
#[must_use]
pub fn bits_da_cena(world: &World) -> Option<u64> {
    entidade_da_cena(world).map(Entity::to_bits)
}

/// ⭐ **Garante a entidade da Cena** — idempotente: devolve a que existe ou cria-a.
pub fn garante_cena(world: &mut World) -> Entity {
    if let Some(e) = entidade_da_cena(world) {
        return e;
    }
    world.spawn((Cena, StableId::CENA)).id()
}

#[cfg(test)]
#[path = "cena_tests.rs"]
mod tests;

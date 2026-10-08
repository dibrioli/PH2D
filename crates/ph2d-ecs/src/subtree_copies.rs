//! ⭐ **As cópias que o EDITOR fez neste quadro** (doc ObjectsAndComponents 01 §6).
//!
//! *Duplicate*, *Make*, *Instantiate* e a colocação de um prefab anunciam aqui o par `(original,
//! cópia)` de cada entidade copiada; quem guarda documentos POR OBJECTO fora do mundo — a
//! Animação, na Timeline — drena-o e dá à cópia o seu (escolha do dono, 08/10: clips próprios).
//! Um recurso e não uma chamada: as portas de cópia não conhecem a Timeline (ADR-0075).
//!
//! ⛔ **A Fábrica não anuncia** (`instantiate_master_many`): as cópias de JOGO nascem e morrem às
//! dezenas, e uma bala que ganhasse clips encheria o documento a cada disparo.

use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;

use crate::DeepCopy;

/// Os pares `(bits do original, bits da cópia)` anunciados e ainda por drenar.
#[derive(Resource, Debug, Default)]
pub struct SubtreeCopies(Vec<(u64, u64)>);

/// Anuncia uma cópia do editor.
pub fn announce_copy(world: &mut World, copy: &DeepCopy) {
    world
        .get_resource_or_insert_with(SubtreeCopies::default)
        .0
        .extend(
            copy.entities
                .iter()
                .map(|(s, d)| (s.to_bits(), d.to_bits())),
        );
}

/// Drena o que foi anunciado (vazio sem alocar quando não houve cópia).
pub fn take_copies(world: &mut World) -> Vec<(u64, u64)> {
    world
        .get_resource_mut::<SubtreeCopies>()
        .map(|mut c| std::mem::take(&mut c.0))
        .unwrap_or_default()
}

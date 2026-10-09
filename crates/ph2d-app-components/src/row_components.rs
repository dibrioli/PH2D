//! ⭐⭐ **As famílias de componentes de cada objecto, para a linha da Hierarquia** (doc 01 §7.1).
//!
//! ⚠️ **Pela tabela de tipos do mundo, nunca serializando:** a paleta pergunta *«este objecto já tem
//! X?»* serializando cada componente registado (`component_attach::present_on`), o que é certo para
//! um clique e caro demais para cada linha em cada quadro. Aqui a pergunta é a do archetype
//! (`EntityRef::contains_id`), com os ids resolvidos UMA vez por quadro.
//!
//! ⭐ **E o conjunto dos AUTORADOS** ([`ComponentSet`], o filtro ▾ da Hierarquia, F6) responde-se UMA
//! vez por ARCHETYPE: todas as entidades de um archetype têm os mesmos componentes, e uma cena de
//! milhares de objectos tem dezenas de archetypes — a linha paga uma consulta, não uma sonda por tipo.

use std::cell::RefCell;

use bevy_ecs::archetype::ArchetypeId;
use bevy_ecs::component::ComponentId;
use ph2d_component_desc::{ComponentCategory, ComponentSet};
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_editor_core::hier_rows::{component_icon, row_categories};

/// As sondas do quadro: um id do bevy por componente registado cuja família ganha ícone, e um por
/// componente autorado com o bit dele no [`ComponentSet`].
pub struct RowComponents {
    probes: Vec<(ComponentId, ComponentCategory)>,
    authored: Vec<(ComponentId, u8)>,
    by_archetype: RefCell<Vec<(ArchetypeId, ComponentSet)>>,
}

impl RowComponents {
    /// Resolve as sondas contra ESTE mundo (o `ComponentId` é do mundo, não do tipo).
    #[must_use]
    pub fn of(world: &ph2d_ecs::World, registry: &ComponentRegistry) -> Self {
        let probes = registry
            .iter()
            .filter_map(|e| {
                let category = e.desc?.category;
                component_icon(category)?;
                Some(((e.bevy_component_id)(world)?, category))
            })
            .collect();
        let authored = ComponentSet::authored()
            .filter_map(|(bit, d)| {
                let entry = registry
                    .iter()
                    .find(|e| e.desc.is_some_and(|x| x.canonical_name == d.canonical_name))?;
                Some(((entry.bevy_component_id)(world)?, bit))
            })
            .collect();
        Self {
            probes,
            authored,
            by_archetype: RefCell::new(Vec::new()),
        }
    }

    /// As famílias com ícone que `entity` tem, na ordem da paleta.
    #[must_use]
    pub fn of_entity(&self, world: &ph2d_ecs::World, entity: u64) -> Vec<ComponentCategory> {
        let Some(e) =
            ph2d_ecs::Entity::try_from_bits(entity).and_then(|e| world.get_entity(e).ok())
        else {
            return Vec::new();
        };
        row_categories(
            self.probes
                .iter()
                .filter(|(id, _)| e.contains_id(*id))
                .map(|(_, c)| *c),
        )
    }

    /// ⭐ Os componentes AUTORADOS que `entity` tem — sondados uma vez por archetype.
    #[must_use]
    pub fn set_of(&self, world: &ph2d_ecs::World, entity: u64) -> ComponentSet {
        let Some(e) =
            ph2d_ecs::Entity::try_from_bits(entity).and_then(|e| world.get_entity(e).ok())
        else {
            return ComponentSet::default();
        };
        let arch = e.archetype().id();
        if let Some((_, set)) = self.by_archetype.borrow().iter().find(|(a, _)| *a == arch) {
            return *set;
        }
        let mut set = ComponentSet::default();
        for (id, bit) in &self.authored {
            if e.contains_id(*id) {
                set.insert(*bit);
            }
        }
        self.by_archetype.borrow_mut().push((arch, set));
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ O conjunto de um objecto diz os autorados que ele TEM — e dois objectos com os mesmos
    /// componentes respondem pela mesma sonda de archetype. (Mutação: `set_of` devolver o conjunto
    /// do 1.º archetype para todos ⇒ o objecto sem Health passa a tê-lo — RED.)
    #[test]
    fn the_set_of_an_object_is_what_it_has() {
        let registry = crate::component_registry_for_tests::registo();
        let mut world = ph2d_ecs::World::new();
        let heroi = world.spawn(ph2d_physics_ecs::Health::default()).id();
        let outro = world.spawn(ph2d_physics_ecs::Health::default()).id();
        let moeda = world.spawn(ph2d_ecs::Transform::default()).id();
        let rows = RowComponents::of(&world, &registry);
        let vida = ComponentSet::authored()
            .find(|(_, d)| d.canonical_name == "ph2d::physics::Health")
            .map(|(b, _)| b)
            .expect("Health é autorado");
        assert!(rows.set_of(&world, heroi.to_bits()).contains(vida));
        assert!(rows.set_of(&world, outro.to_bits()).contains(vida));
        assert!(!rows.set_of(&world, moeda.to_bits()).contains(vida));
    }
}

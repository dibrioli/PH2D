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
use ph2d_editor_core::hier_types::ObjectType;

/// As sondas do quadro: um id do bevy por componente registado cuja família ganha ícone, e um por
/// componente autorado com o bit dele no [`ComponentSet`].
pub struct RowComponents {
    probes: Vec<(ComponentId, ComponentCategory)>,
    authored: Vec<(ComponentId, u8)>,
    /// Um id por componente que DEFINE um tipo de objecto ([`ObjectType::marker`], F42).
    markers: Vec<(ComponentId, &'static str)>,
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
                // `desc_for` resolve pelo próprio nome: o nome do descritor É o da entrada (N10 F6c).
                let entry = registry
                    .get_by_name(d.canonical_name)
                    .filter(|e| e.desc.is_some())?;
                Some(((entry.bevy_component_id)(world)?, bit))
            })
            .collect();
        let markers = ObjectType::ALL
            .into_iter()
            .filter_map(|t| {
                let name = t.marker()?;
                let entry = registry.get_by_name(name)?;
                Some(((entry.bevy_component_id)(world)?, name))
            })
            .collect();
        Self {
            probes,
            authored,
            markers,
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

    /// ⭐ **O TIPO do objecto** — o ícone e a cor da linha (F42).
    #[must_use]
    pub fn object_type(&self, world: &ph2d_ecs::World, entity: u64) -> ObjectType {
        let Some(e) = ph2d_ecs::Entity::try_from_bits(entity) else {
            return ObjectType::Empty;
        };
        let Ok(r) = world.get_entity(e) else {
            return ObjectType::Empty;
        };
        let kind = crate::component_attach::kind_of(world, e);
        ObjectType::of(kind, |name| {
            self.markers
                .iter()
                .any(|(id, n)| *n == name && r.contains_id(*id))
        })
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

    /// ⭐ F42 — o tipo da linha sai dos marcadores do REGISTO: o componente definidor faz o tipo num
    /// vazio, e o jogador ganha à imagem. (Mutação: a sonda dos marcadores responder `false` ⇒ tudo
    /// volta a Empty/Image — RED.)
    #[test]
    fn the_row_type_comes_from_the_registered_markers() {
        use ph2d_editor_core::hier_types::ObjectType;
        let registry = crate::component_registry_for_tests::registo();
        let mut world = ph2d_ecs::World::new();
        let caixa = world
            .spawn(ph2d_physics_ecs::Collider::default())
            .id()
            .to_bits();
        let camera = world.spawn(ph2d_ecs::GameCamera::default()).id().to_bits();
        let vazio = world.spawn(ph2d_ecs::Transform::default()).id().to_bits();
        let rows = RowComponents::of(&world, &registry);
        assert_eq!(rows.object_type(&world, caixa), ObjectType::Collider);
        assert_eq!(rows.object_type(&world, camera), ObjectType::Camera);
        assert_eq!(rows.object_type(&world, vazio), ObjectType::Empty);
    }

    /// N10 F6c: a busca por nome dá as MESMAS sondas que a varredura linear de antes (o perfil
    /// achou-a por quadro). Controlo: o oráculo é a varredura, sobre o registo inteiro.
    #[test]
    fn a_busca_por_nome_da_as_mesmas_sondas_que_a_varredura() {
        let registry = crate::component_registry_for_tests::registo();
        let mut world = ph2d_ecs::World::new();
        for e in registry.iter() {
            if let Some(f) = e.insert_default {
                let id = world.spawn_empty().id();
                f(&mut world, id).expect("default");
            }
        }
        let rows = RowComponents::of(&world, &registry);
        let authored: Vec<_> = ComponentSet::authored()
            .filter_map(|(bit, d)| {
                let entry = registry
                    .iter()
                    .find(|e| e.desc.is_some_and(|x| x.canonical_name == d.canonical_name))?;
                Some(((entry.bevy_component_id)(&world)?, bit))
            })
            .collect();
        let markers: Vec<_> = ObjectType::ALL
            .into_iter()
            .filter_map(|t| {
                let name = t.marker()?;
                let entry = registry.iter().find(|e| e.canonical_name == name)?;
                Some(((entry.bevy_component_id)(&world)?, name))
            })
            .collect();
        assert!(
            !authored.is_empty() && !markers.is_empty(),
            "o controlo tem população"
        );
        assert_eq!(rows.authored, authored);
        assert_eq!(rows.markers, markers);
    }

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

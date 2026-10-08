//! ⭐⭐ **As famílias de componentes de cada objecto, para a linha da Hierarquia** (doc 01 §7.1).
//!
//! ⚠️ **Pela tabela de tipos do mundo, nunca serializando:** a paleta pergunta *«este objecto já tem
//! X?»* serializando cada componente registado (`component_attach::present_on`), o que é certo para
//! um clique e caro demais para cada linha em cada quadro. Aqui a pergunta é a do archetype
//! (`EntityRef::contains_id`), com os ids resolvidos UMA vez por quadro.

use bevy_ecs::component::ComponentId;
use ph2d_component_desc::ComponentCategory;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_editor_core::hier_rows::{component_icon, row_categories};

/// As sondas do quadro: um id do bevy por componente registado cuja família ganha ícone.
pub struct RowComponents {
    probes: Vec<(ComponentId, ComponentCategory)>,
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
        Self { probes }
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
}

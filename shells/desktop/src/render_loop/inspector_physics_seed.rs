//! ⭐ **O que ANEXAR um componente de física faz além de inserir o ponto neutro** (ADR-0166 / F3).
//!
//! ⚠️ Irmão de [`super::inspector_physics_apply`] pelo teto de 600 LOC da shell, e o corte é por
//! ASSUNTO: lá fica o que uma EDIÇÃO do Inspector faz; aqui, o que a CRIAÇÃO semeia. ⛔ Não devolva
//! uma destas funções ao irmão — o teto volta a estourar no gate seguinte.

use ph2d_ecs::{Entity, SimWorld};

/// ⭐ **A caixa que casa com o DESENHO** — as meias-extensões do `Sprite` desta entidade, ou o
/// meio-metro de reserva quando não há sprite nenhuma.
///
/// ⚠️ **Existe porque a lei estava escrita TRÊS vezes** (o `Add`, o `AddShape` e — desde a F3 — o
/// seed da paleta), e uma lei em três sítios diverge no dia em que um deles for corrigido. *Uma lei
/// escrita em dois sítios ainda não é uma lei; só uma PORTA é.*
pub(crate) fn sprite_half_extents(world: &ph2d_ecs::World, entity: Entity) -> [f32; 2] {
    world
        .get::<ph2d_render::Sprite>(entity)
        .map_or([0.5, 0.5], |s| {
            [(s.size[0] * 0.5).max(1e-3), (s.size[1] * 0.5).max(1e-3)]
        })
}

/// ⭐ **O que anexar um `Collider` faz além de inserir o ponto neutro** (ADR-0166 / F3 · a emenda
/// medida na F0 — ver [`crate::component_seed`]).
///
/// O `Collider::default()` é uma **bola de meio metro**; debaixo de um sprite quadrado ela é
/// exatamente o desencontro que o Enio apanhou em 2026-07-18. O `insert_default` do registo é
/// type-erased e não pode saber o tamanho do desenho.
///
/// ⚠️ **Só morde na forma AINDA NEUTRA**, e é o que o torna idempotente e seguro: um collider que o
/// artista já autorou nunca é reescrito — a mesma lei que o `AddShape` honra («a porta que CRIA a
/// peça recusa reescrever uma que existe»), medida numa peça `0,17 × 0,91` que voltava `0,10 × 0,50`
/// com tudo zerado.
pub(crate) fn seed_attached_collider(sim: &mut SimWorld, entity_bits: u64) {
    use ph2d_physics_ecs::{Collider, ColliderShape};
    let entity = Entity::from_bits(entity_bits);
    let Some(mut col) = sim.world().get::<Collider>(entity).copied() else {
        return;
    };
    if col.shape != ColliderShape::default() {
        return;
    }
    let half = sprite_half_extents(sim.world(), entity);
    col.shape = ColliderShape::Cuboid {
        half_x: half[0],
        half_y: half[1],
    };
    sim.world_mut().entity_mut(entity).insert(col);
}

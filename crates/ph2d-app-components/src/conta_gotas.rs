//! **Os conta-gotas do Inspector que escolhem um objecto no palco** — o Target da bala (GEF 06) e o
//! «Ignore up to» do Ray Sensor (GEF 07 C8). A shell compõe UMA porta: um clique, um `Esc`, um cursor.

use ph2d_ecs::World;
use ph2d_editor_core::action_bus::ActionBus;

/// Algum está armado (o cursor de mira).
#[must_use]
pub fn armado(world: &World) -> bool {
    crate::projectile_inspector::armado(world) || crate::ray_topo::armado(world, None)
}

/// Desarma todos (o `Esc`); `true` = algum estava armado.
pub fn desarma(world: &mut World) -> bool {
    crate::projectile_inspector::desarma(world).is_some()
        | crate::ray_topo::desarma(world).is_some()
}

/// ⭐ **O clique na Hierarchy com um conta-gotas armado** (dono 15/11): enquanto `clique` corre, a linha clicada vai
/// para o conta-gotas e NÃO selecciona — o dono dele fica no Inspector. Devolve o objecto da linha (`Entity::to_bits`).
/// Arma só durante o clique: a aba do som (`ph2d-app-sfxlab`) re-arma a dela a cada quadro.
pub fn pela_hierarquia(armado: bool, clique: impl FnOnce()) -> Option<u64> {
    if !armado {
        clique();
        return None;
    }
    ph2d_panel_hierarchy::arma_escolha(true);
    clique();
    let linha = ph2d_panel_hierarchy::toma_escolha();
    ph2d_panel_hierarchy::arma_escolha(false);
    linha
}

/// O ponto de mundo do objecto `bits` — o «onde» de um clique que veio da Hierarchy (a cola prende ali).
#[must_use]
pub fn ponto_de(world: &World, bits: u64) -> [f32; 2] {
    ph2d_ecs::Entity::try_from_bits(bits)
        .and_then(|e| ph2d_ecs::world_transform(world, e))
        .map_or([0.0; 2], |t| [t.translation.x, t.translation.y])
}

/// O clique no palco com `debaixo` (de cima para baixo); `true` = um conta-gotas consumiu-o.
pub fn escolher(world: &mut World, bus: &mut ActionBus, debaixo: &[u64]) -> bool {
    crate::projectile_inspector::escolher_alvo(world, bus, debaixo)
        || crate::ray_topo::escolher_topo(world, bus, debaixo)
}

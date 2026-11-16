//! ⭐ **O `Motion Input` mora num objeto filho** (GEF 08 MI-10, dono 16/11, ADR-0199 por analogia): o pick
//! da paleta e o «Add …» por nome criam o filho «Motion Input» sob o objeto e anexam lá — nunca no objeto.

use ph2d_ecs::{Entity, MotionInput, SimWorld};

const CANON: &str = "ph2d::ecs::MotionInput";

/// Para onde vai o pick `name` pedido para `bits`: o `MotionInput` ganha um filho novo de `bits` (salvo se
/// `bits` já é um objeto «Motion Input»); todo o resto fica em `bits`.
pub fn alvo_do_pick(sim: &mut SimWorld, bits: u64, name: &str) -> u64 {
    let world = sim.world_mut();
    let Some(pai) = Entity::try_from_bits(bits).filter(|&e| world.get_entity(e).is_ok()) else {
        return bits;
    };
    if name != CANON || world.get::<MotionInput>(pai).is_some() {
        return bits;
    }
    let base = ph2d_game_dicas::add::MOTION_INPUT.key.tr();
    ph2d_physics_ecs::pendura_entrada(world, None, Some(pai), &base).to_bits()
}

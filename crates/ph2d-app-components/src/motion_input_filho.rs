//! ⭐ **O `Motion Input` mora num objeto filho** (GEF 08 MI-10): o pick cria o filho e anexa lá.

use ph2d_ecs::{Entity, MotionInput, SimWorld};

const CANON: &str = "ph2d::ecs::MotionInput";

/// Para onde vai o pick `name` de `bits`: o `MotionInput` num filho novo (salvo se `bits` já o tem).
pub fn alvo_do_pick(sim: &mut SimWorld, bits: u64, name: &str) -> u64 {
    let world = sim.world_mut();
    let Some(pai) = Entity::try_from_bits(bits).filter(|&e| world.get_entity(e).is_ok()) else {
        return bits;
    };
    if name != CANON || world.get::<MotionInput>(pai).is_some() {
        return bits;
    }
    let base = ph2d_game_dicas::add::MOTION_INPUT.key.tr();
    let filho = ph2d_physics_ecs::pendura_entrada(world, None, Some(pai), &base).to_bits();
    NOVO.with(|c| c.set(Some(filho)));
    filho
}

thread_local! {
    static NOVO: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

/// O filho que o último pick criou (lê e limpa): a seleção vai para ele, como no Add ▸ Motion Input.
pub fn toma_o_novo() -> Option<u64> {
    NOVO.with(std::cell::Cell::take)
}

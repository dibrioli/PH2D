//! **A metade da fronteira**: os gates da cena `physics_smoke_part` que exercitam um
//! GESTO DA SHELL (`crate::physics::…`) sobre uma cena que vive na crate da família.
//!
//! ⚠️ **Eles estão aqui porque um `#[cfg(test)]` é INVISÍVEL do outro lado da
//! fronteira de crate** (HOWTO §2): o sujeito é da crate, o gesto é da shell, e um
//! teste não pode ter um pé em cada. Os outros gates da mesma cena ficaram **com o
//! sujeito**, na crate — *o corte é por quem o teste EXERCITA, não por quem ele nomeia.*

use crate::physics_smoke_part::*;
use ph2d_ecs::{Entity, Name, SimWorld};
use ph2d_physics_ecs::PhysicsBridge;

fn entity(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn world_y(sim: &mut SimWorld, name: &str) -> f32 {
    let e = entity(sim, name);
    ph2d_ecs::world_transform(sim.world(), e)
        .expect("transform")
        .translation
        .y
}

/// **Afinar a PEÇA é o que liberta a chave** — o gesto inteiro da wave, medido
/// pelo caminho do produto (o edit do §11 sobre a entidade da peça).
///
/// Sem este gate a cena afirmaria uma promessa que ninguém verificou: que o
/// número digitado no campo *Half Width* de uma peça de fato muda a simulação.
#[test]
fn narrowing_the_part_lets_the_wide_key_through() {
    let mut sim = SimWorld::new();
    crate::common::spawn_floor(sim.world_mut());
    build_keys(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let bit = entity(&mut sim, "Wide Bit");
    // O MESMO edit que o campo do painel emite.
    crate::physics_tests::apply(
        &mut sim,
        bit,
        ph2d_editor_core::PhysicsFieldEdit::HalfX(BIT_HALF_X[1]),
    );
    let mut bridge = PhysicsBridge::new();
    for t in 0..=300u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = world_y(&mut sim, "Wide Handle");
    assert!(
        y < 1.2,
        "a chave larga continuou entalada em {y:.3} depois de afinar o palhetão — \
         o valor digitado não chegou ao solver"
    );
}

/// **Remover a PEÇA é o segundo caminho**, e ele tem de dar o mesmo resultado.
#[test]
fn removing_the_part_also_lets_the_wide_key_through() {
    let mut sim = SimWorld::new();
    crate::common::spawn_floor(sim.world_mut());
    build_keys(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let bit = entity(&mut sim, "Wide Bit");
    crate::physics_tests::apply(&mut sim, bit, ph2d_editor_core::PhysicsFieldEdit::Remove);
    let mut bridge = PhysicsBridge::new();
    for t in 0..=300u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = world_y(&mut sim, "Wide Handle");
    assert!(
        y < 1.2,
        "a chave continuou entalada em {y:.3} depois de *Remove Shape* — a forma \
         removida ainda alcança o solver"
    );
}

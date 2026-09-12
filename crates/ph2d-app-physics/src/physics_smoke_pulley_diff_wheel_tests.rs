//! **A metade da fronteira**: os gates da cena `physics_smoke_pulley_diff` que exercitam um
//! GESTO DA SHELL (`crate::physics::…`) sobre uma cena que vive na crate da família.
//!
//! ⚠️ **Eles estão aqui porque um `#[cfg(test)]` é INVISÍVEL do outro lado da
//! fronteira de crate** (HOWTO §2): o sujeito é da crate, o gesto é da shell, e um
//! teste não pode ter um pé em cada. Os outros gates da mesma cena ficaram **com o
//! sujeito**, na crate — *o corte é por quem o teste EXERCITA, não por quem ele nomeia.*

use crate::physics_smoke_pulley_diff::*;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("corpo vivo")
}

fn entity_of(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

/// **O segundo diâmetro é autorável, e digitar nele MUDA a cena** — a quarta
/// condição da política de UI do plano: as outras três podem estar verdes com a
/// sequência não levando a lugar nenhum.
#[test]
fn typing_an_out_radius_turns_a_plain_wheel_into_a_drum() {
    let travel = |out: f32| {
        let mut sim = SimWorld::new();
        build_differential(sim.world_mut());
        ph2d_physics_ecs::resolve_body_names(sim.world_mut());
        let drum = entity_of(&mut sim, "Plain Rope Drum");
        // A porta pura que o painel alcança — a mesma que a row do Inspector usa.
        let current = *sim
            .world()
            .get::<ph2d_physics_ecs::PulleyWheel>(drum)
            .expect("o tambor existe");
        let next = crate::joint_wheel::wheel_with_edit(
            current,
            ph2d_editor_core::WheelFieldEdit::RadiusOut(out),
        )
        .expect("editar o raio de saída é uma escrita de componente");
        *sim.world_mut()
            .get_mut::<ph2d_physics_ecs::PulleyWheel>(drum)
            .expect("o tambor existe") = next;
        let mut bridge = PhysicsBridge::new();
        let y0 = y_of(&mut sim, "Plain Load");
        for t in 1..=120 {
            bridge.dispatch(&mut sim, false, t);
        }
        y_of(&mut sim, "Plain Load") - y0
    };
    let plain = travel(0.0);
    assert!(
        plain < -1.0,
        "com 0 no raio de saída a roldana é COMUM e a carga tinha de cair; ela \
         andou {plain:.3} m — a fixture não contém o fenômeno"
    );
    let geared = travel(R_OUT);
    assert!(
        geared > plain + 1.0,
        "digitar {R_OUT} no raio de saída faz do MESMO tambor um diferencial de \
         {:.0}x, e a mesma carga tinha de parar de cair; ela andou {geared:.3} m \
         contra {plain:.3}",
        R_IN / R_OUT
    );
}

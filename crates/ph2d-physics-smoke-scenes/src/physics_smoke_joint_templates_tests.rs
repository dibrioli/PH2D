//! A cena 123 ensina o que acontece: quatro montagens nomeadas, e cada máquina faz a sua coisa.

use super as scene;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{Collider, JointAssembly, PhysicsBridge, PhysicsJoint, RigidBody};

fn y_of(sim: &mut SimWorld, name: &str) -> Transform {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| *t)
        .unwrap_or_else(|| panic!("{name} is not in the scene"))
}

#[test]
fn the_template_scene_shows_four_named_machines_that_work() {
    let mut sim = SimWorld::new();
    scene::build(sim.world_mut());
    let mut names: Vec<String> = sim
        .world_mut()
        .query::<(&Name, &JointAssembly)>()
        .iter(sim.world())
        .map(|(n, _)| n.as_str().to_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["Bridge 1", "Car 1", "Door 1", "Elevator 1"]);
    let joints = sim
        .world_mut()
        .query::<&PhysicsJoint>()
        .iter(sim.world())
        .count();
    assert_eq!(joints, 1 + 9 + 1 + 2, "door + bridge + elevator + car");
    // ADR-0199: a forma de cada corpo é a filha «Collision Shape» na Hierarchy, nunca o corpo.
    let bodies: Vec<_> = sim
        .world_mut()
        .query_filtered::<ph2d_ecs::Entity, ph2d_ecs::With<RigidBody>>()
        .iter(sim.world())
        .collect();
    for b in bodies {
        let w = sim.world();
        assert!(w.get::<Collider>(b).is_none(), "a body carries its shape");
        let f = ph2d_physics_ecs::forma_para_editar(w, b).expect("every body has a shape");
        let name = w.get::<Name>(f).map_or("", Name::as_str);
        assert!(name.starts_with("Collision Shape"), "the shape is «{name}»");
    }

    let (car0, lift0) = (
        y_of(&mut sim, "Car Body"),
        y_of(&mut sim, "Elevator Platform"),
    );
    let mut bridge = PhysicsBridge::new();
    for t in 1..=300 {
        bridge.dispatch(&mut sim, false, t);
    }
    let car = y_of(&mut sim, "Car Body");
    assert!(
        car.translation.x - car0.translation.x > 3.0,
        "the car did not drive"
    );
    assert!(
        car.translation.x < scene::END_WALL_X,
        "the car went through the end wall"
    );
    let lift = y_of(&mut sim, "Elevator Platform");
    assert!(
        (lift.translation.y - lift0.translation.y - 3.0).abs() < 0.1,
        "the lift"
    );
    let crate_y = y_of(&mut sim, "Crate").translation.y;
    assert!(
        // Pousada a 1,30 m (tábua 1,05 + meia caixa); medido 1,288 com a ponte como uma corrente (W2).
        crate_y > 1.2,
        "the crate sank into the bridge (y = {crate_y:.2})"
    );
    let door = y_of(&mut sim, "Door").rotation.abs();
    assert!(
        door < std::f32::consts::FRAC_PI_2 + 0.1,
        "the door swung past its stop"
    );
}

/// **A ponte APANHA a caixa, não a atira** — e isso não depende de 1 mm de onde ela cai.
///
/// A caixa de 10 kg cai de 3 m sobre a emenda das duas árvores (`ph2d_physics::world::chains`). Com a emenda
/// rígida ela subia 0,7–2 m e caía onde calhasse (às vezes no chão): o «segura» da cena era sorte do fim.
/// Nove quedas a ±4 mm: depois de tocar, a caixa nunca sobe mais de 10 cm acima do repouso (1,30), e acaba
/// sobre a ponte, perto de onde caiu.
#[test]
fn the_bridge_catches_a_dropped_crate_without_throwing_it() {
    const REST: f32 = 1.30;
    for k in -4i32..=4 {
        let dx = k as f32 * 1e-3;
        let mut sim = SimWorld::new();
        scene::build(sim.world_mut());
        {
            let mut q = sim.world_mut().query::<(&Name, &mut Transform)>();
            for (n, mut t) in q.iter_mut(sim.world_mut()) {
                if n.as_str() == "Crate" {
                    t.translation.x += dx;
                }
            }
        }
        let mut bridge = PhysicsBridge::new();
        let (mut touched, mut peak) = (false, f32::MIN);
        for t in 1..=300 {
            bridge.dispatch(&mut sim, false, t);
            let y = y_of(&mut sim, "Crate").translation.y;
            touched |= y < REST + 0.1;
            if touched {
                peak = peak.max(y);
            }
        }
        let end = y_of(&mut sim, "Crate").translation;
        assert!(
            peak < REST + 0.1,
            "dx {dx:+.3}: the bridge threw the crate {:.2} m up",
            peak - REST
        );
        assert!(
            end.y > 1.2 && (end.x - 7.0 - dx).abs() < 0.3,
            "dx {dx:+.3}: the crate ended at ({:.2}, {:.2})",
            end.x,
            end.y
        );
    }
}

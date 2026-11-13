//! A cena 123 ensina o que acontece: quatro montagens nomeadas, e cada máquina faz a sua coisa.

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
    super::build(sim.world_mut());
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
        car.translation.x < super::END_WALL_X,
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

/// SONDA (dono 12/11): a densidade das TÁBUAS (caixa fixa, 10 kg) contra o quanto a ponte afunda.
#[test]
#[ignore = "sonda"]
fn probe_plank_density_against_bridge_sag() {
    for (d, drop) in [0.1_f32, 1.0, 10.0, 40.0]
        .into_iter()
        .flat_map(|d| [(d, true), (d, false)])
    {
        let mut sim = SimWorld::new();
        super::build(sim.world_mut());
        if !drop {
            // A caixa POUSADA na tábua do meio (topo 1,05 + meia caixa 0,25), sem queda.
            let mut q = sim.world_mut().query::<(&Name, &mut Transform)>();
            for (n, mut t) in q.iter_mut(sim.world_mut()) {
                if n.as_str() == "Crate" {
                    t.translation.y = 1.31;
                }
            }
        }
        print!("drop {drop:>5} ");
        let planks: Vec<ph2d_ecs::Entity> = {
            let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
            q.iter(sim.world())
                .filter(|(_, n)| n.as_str().starts_with("Plank "))
                .map(|(e, _)| e)
                .collect()
        };
        for p in planks {
            crate::physics_tests::apply(
                &mut sim,
                p,
                ph2d_editor_core::PhysicsFieldEdit::Density(d),
            );
        }
        let mut bridge = PhysicsBridge::new();
        let mut low = f32::MAX;
        for t in 1..=300 {
            bridge.dispatch(&mut sim, false, t);
            low = low.min(y_of(&mut sim, "Plank 5").translation.y);
        }
        let p5 = y_of(&mut sim, "Plank 5").translation.y;
        let cy = y_of(&mut sim, "Crate").translation.y;
        let gap = y_of(&mut sim, "Plank 5").translation.x - y_of(&mut sim, "Plank 4").translation.x;
        println!(
            "plank density {d:>5} plank5 end {p5:+.3} lowest {low:+.3} crate y {cy:+.3} p4-p5 dx {gap:.3}"
        );
    }
}

/// SONDA (dono 12/11): Rope × Pin entre as tábuas, tábuas de densidade `d`, caixa de 10 kg.
#[test]
#[ignore = "sonda"]
fn probe_rope_against_pin_bridge() {
    use ph2d_physics_ecs::JointKind;
    for d in [0.1_f32, 1.0, 10.0] {
        for pin in [false, true] {
            let mut sim = SimWorld::new();
            super::build(sim.world_mut());
            let planks: Vec<ph2d_ecs::Entity> = {
                let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
                q.iter(sim.world())
                    .filter(|(_, n)| n.as_str().starts_with("Plank "))
                    .map(|(e, _)| e)
                    .collect()
            };
            for p in planks {
                crate::physics_tests::apply(
                    &mut sim,
                    p,
                    ph2d_editor_core::PhysicsFieldEdit::Density(d),
                );
            }
            if pin {
                let w = sim.world_mut();
                let named: Vec<(u64, [f32; 2])> = w
                    .query::<(&ph2d_ecs::StableId, &Transform)>()
                    .iter(w)
                    .map(|(n, t)| (n.0, [t.translation.x, t.translation.y]))
                    .collect();
                let pos = |_: &ph2d_ecs::World, id: u64| {
                    named.iter().find(|(k, _)| *k == id).map(|(_, p)| *p)
                };
                let mut q = w.query::<&mut PhysicsJoint>();
                let ropes: Vec<(u64, u64, [f32; 2], [f32; 2])> = q
                    .iter(w)
                    .filter(|j| j.kind == JointKind::Rope)
                    .map(|j| (j.body_a, j.body_b, j.local_a, j.local_b))
                    .collect();
                let mut fixed = Vec::new();
                for (a, b, la, lb) in ropes {
                    let (Some(pa), Some(pb)) = (pos(w, a), pos(w, b)) else {
                        continue;
                    };
                    let wa = [pa[0] + la[0], pa[1] + la[1]];
                    let wb = [pb[0] + lb[0], pb[1] + lb[1]];
                    let m = [(wa[0] + wb[0]) / 2.0, (wa[1] + wb[1]) / 2.0];
                    fixed.push((
                        a,
                        b,
                        [m[0] - pa[0], m[1] - pa[1]],
                        [m[0] - pb[0], m[1] - pb[1]],
                    ));
                }
                for mut j in q.iter_mut(w) {
                    if let Some(f) = fixed.iter().find(|f| f.0 == j.body_a && f.1 == j.body_b) {
                        let keep = (j.body_a, j.body_b, j.anchored, j.assembly, j.role);
                        *j = PhysicsJoint::of_kind(JointKind::Pin);
                        (j.body_a, j.body_b, j.anchored, j.assembly, j.role) = keep;
                        j.local_a = f.2;
                        j.local_b = f.3;
                    }
                }
            }
            let mut bridge = PhysicsBridge::new();
            let mut low = f32::MAX;
            for t in 1..=300 {
                bridge.dispatch(&mut sim, false, t);
                low = low.min(y_of(&mut sim, "Plank 5").translation.y);
            }
            let p5 = y_of(&mut sim, "Plank 5").translation.y;
            let kind = if pin { "Pin " } else { "Rope" };
            println!("{kind} plank {d:>4}  end {p5:+.3} lowest {low:+.3}");
        }
    }
}

/// SONDA (dono 12/11): o solver (sub-passos × iterações) contra o esmagamento da ponte leve.
#[test]
#[ignore = "sonda"]
fn probe_solver_against_light_bridge() {
    for d in [0.1_f32, 1.0] {
        for (sub, it) in [(4, 4), (8, 4), (12, 4), (4, 16), (12, 16)] {
            let mut sim = SimWorld::new();
            super::build(sim.world_mut());
            let planks: Vec<ph2d_ecs::Entity> = {
                let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
                q.iter(sim.world())
                    .filter(|(_, n)| n.as_str().starts_with("Plank "))
                    .map(|(e, _)| e)
                    .collect()
            };
            for p in planks {
                crate::physics_tests::apply(
                    &mut sim,
                    p,
                    ph2d_editor_core::PhysicsFieldEdit::Density(d),
                );
            }
            let mut bridge = PhysicsBridge::new();
            let mut s = bridge.settings();
            s.substeps = sub;
            s.solver_iterations = it;
            bridge.set_settings(s);
            let mut low = f32::MAX;
            for t in 1..=300 {
                bridge.dispatch(&mut sim, false, t);
                low = low.min(y_of(&mut sim, "Plank 5").translation.y);
            }
            let p5 = y_of(&mut sim, "Plank 5").translation.y;
            println!("plank {d:>4} sub {sub:>2} it {it:>2}  end {p5:+.3} lowest {low:+.3}");
        }
    }
}

/// SONDA (dono 12/11): a densidade da caixa contra o quanto a ponte afunda.
#[test]
#[ignore = "sonda"]
fn probe_crate_density_against_bridge_sag() {
    for d in [0.1_f32, 1.0, 10.0, 40.0, 400.0] {
        let mut sim = SimWorld::new();
        super::build(sim.world_mut());
        let c = {
            let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
            q.iter(sim.world())
                .find(|(_, n)| n.as_str() == "Crate")
                .map(|(e, _)| e)
                .expect("crate")
        };
        // O gesto do Inspector: a caixa (o CORPO) selecionada, «Density» editada.
        crate::physics_tests::apply(&mut sim, c, ph2d_editor_core::PhysicsFieldEdit::Density(d));
        let f = ph2d_physics_ecs::forma_para_editar(sim.world(), c).expect("shape");
        let got = sim.world().get::<Collider>(f).map(|c| c.density);
        let on_body = sim.world().get::<Collider>(c).map(|c| c.density);
        let ov = sim
            .world()
            .get::<ph2d_physics_ecs::MassOverride>(c)
            .map(|m| m.0);
        println!("  shape density {got:?} body collider {on_body:?} mass override {ov:?}");
        let mut bridge = PhysicsBridge::new();
        let mut low = f32::MAX;
        for t in 1..=300 {
            bridge.dispatch(&mut sim, false, t);
            low = low.min(y_of(&mut sim, "Plank 5").translation.y);
        }
        let p5 = y_of(&mut sim, "Plank 5").translation.y;
        let cy = y_of(&mut sim, "Crate").translation.y;
        println!("density {d:>6} plank5 end {p5:+.3} lowest {low:+.3} crate y {cy:+.3}");
    }
}

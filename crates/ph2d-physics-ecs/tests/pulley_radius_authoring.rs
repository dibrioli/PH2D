//! **Dimensionar uma roldana NÃO explode o rig** (W-Pulley, 2026-07-29).
//!
//! Report do Enio: *"aumentar o diâmetro da polia … na simulação ocorre aqueles
//! saltos explosivos"*.
//!
//! O `L0` da corda (`PhysicsJoint::max_length`) é semeado UMA vez da rota que a
//! montagem tem em repouso e depois congelado por `anchored = true`. Crescer o
//! raio **cresce a rota** — o abraço é maior —, então a restrição `L(rota) ≤ L0`
//! nascia **violada** e o solver comia a diferença num tick.
//!
//! A cura é a lei que o W-AnchorFollow já escreveu para a âncora — *autorar
//! re-deriva, o runtime congela* — pela porta `reseat_wheel_geometry`.
//!
//! ⚠️ **Dois gates e nenhum é redundante:** o primeiro afirma a PROPRIEDADE (a
//! restrição não nasce violada) e não olha o relógio; o segundo afirma a
//! CONSEQUÊNCIA (a sim não salta), que é o que o artista vê. Um passe futuro que
//! violasse por outra via passaria pelo primeiro e cairia no segundo.
//!
//! Os números vêm de `tests/measure_pulley_radius.rs`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics::world::rope_route;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody,
};

/// Um elevador: carga de 3 kg e contrapeso de 1 kg por duas roldanas no alto.
fn rig() -> SimWorld {
    let mut sim = SimWorld::new();
    let mut body = |name: &str, x: f32, y: f32, kind: BodyKind, mass: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: mass / (std::f32::consts::PI * 0.04),
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    };
    body("Floor", 0.0, -4.0, BodyKind::Static, 1.0);
    body("Load", -1.5, 2.0, BodyKind::Dynamic, 3.0);
    body("Counter", 1.5, 2.0, BodyKind::Dynamic, 1.0);
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Load"),
            body_b: stable_name_id("Counter"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(-1.5, 2.0)),
    ));
    for (i, x) in [-1.5f32, 1.5].into_iter().enumerate() {
        sim.world_mut().spawn((
            Name::new(format!("Rope Wheel {}", i + 1)),
            PulleyWheel {
                rope: stable_name_id("Rope"),
                order: u16::try_from(i).expect("duas roldanas"),
                radius: 0.3,
                ..Default::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ));
    }
    sim
}

fn entity_of(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let e = entity_of(sim, name);
    sim.world().get::<Transform>(e).expect("t").translation.y
}

/// A violação com que a restrição NASCE, em metros: `L(rota) − L0`.
fn violation(sim: &mut SimWorld, bridge: &PhysicsBridge) -> f32 {
    let e = entity_of(sim, "Rope");
    let l0 = sim.world().get::<PhysicsJoint>(e).expect("j").max_length;
    let v = bridge.joint_views().next().expect("uma joint");
    let arena = bridge.pulley_wheel_arena();
    let start = v.wheel_start as usize;
    let wheels = &arena[start..start + v.wheel_count as usize];
    let mut segs = Vec::new();
    rope_route::route(v.anchor_a, v.anchor_b, wheels, &mut segs).map_or(f32::NAN, |r| r.length) - l0
}

/// **O artista dimensiona a roldana** — a MESMA sequência do produto: escrever o
/// raio e re-abrir o que depende da geometria dela.
fn author_radius(sim: &mut SimWorld, r: f32) {
    let w = entity_of(sim, "Rope Wheel 1");
    if let Some(mut c) = sim.world_mut().get_mut::<PulleyWheel>(w) {
        c.radius = r;
    }
    ph2d_physics_ecs::reseat_wheel_geometry(sim.world_mut(), w);
}

/// **Crescer o raio em repouso não deixa a corda violada.**
///
/// Mutação: tirar a metade do `L0` de `reseat_wheel_geometry` ⇒ **+1,0931 m** a
/// raio 0,90 (medido), e este gate nomeia o número.
#[test]
fn authoring_a_radius_at_rest_leaves_the_rope_unviolated() {
    for r in [0.45f32, 0.60, 0.90, 1.50] {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        author_radius(&mut sim, r);
        bridge.dispatch(&mut sim, false, 0);
        let v = violation(&mut sim, &bridge);
        assert!(
            v.abs() < 1.0e-3,
            "raio {r:.2}: a corda nasce violada em {v:+.4} m — o L0 ficou parado \
             enquanto a rota crescia, e o solver come isso num tick"
        );
    }
}

/// **E a sim não salta.**
///
/// ⚠️ **O oráculo é o CONTROLE, não um literal**: o maior passo num tick com o
/// raio de fábrica é o que esta cena faz de qualquer jeito (medido 0,0817 m), e
/// o que se afirma é que dimensionar a roldana **não muda a ordem de grandeza**.
/// Um limite absoluto mediria o rig, não o defeito.
///
/// Mutação: sem a porta ⇒ **14,1247 m** a raio 0,90 e **50,4327 m** a 1,50, com a
/// carga arremessada de +2 m para **+53 m**.
#[test]
fn and_the_sim_does_not_jump_when_the_radius_grows() {
    let worst_of = |r: f32, door: bool| -> (f32, f32) {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        if door {
            author_radius(&mut sim, r);
        } else if let Some(mut c) = {
            let w = entity_of(&mut sim, "Rope Wheel 1");
            sim.world_mut().get_mut::<PulleyWheel>(w)
        } {
            c.radius = r;
        }
        bridge.dispatch(&mut sim, false, 0);
        let mut prev = (y_of(&mut sim, "Load"), y_of(&mut sim, "Counter"));
        let mut worst = 0.0f32;
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
            let now = (y_of(&mut sim, "Load"), y_of(&mut sim, "Counter"));
            worst = worst.max((now.0 - prev.0).abs().max((now.1 - prev.1).abs()));
            prev = now;
        }
        (worst, prev.0)
    };

    let (control, control_y) = worst_of(0.30, false);
    assert!(
        control < 0.2,
        "o CONTROLE já salta ({control:.4} m) — a fixture não mede o defeito"
    );
    assert!(
        control_y < 1.0,
        "no controle a carga de 3 kg tem de DESCER (ficou em {control_y:+.3})"
    );
    for r in [0.60f32, 0.90, 1.50] {
        let (worst, end_y) = worst_of(r, true);
        assert!(
            worst < control * 1.5,
            "raio {r:.2}: maior salto {worst:.4} m contra {control:.4} do controle — \
             dimensionar a roldana passou a arremessar o rig"
        );
        assert!(
            end_y < 1.0,
            "raio {r:.2}: a carga acabou em {end_y:+.3} — ela devia DESCER como no \
             controle, não ser lançada para cima"
        );
    }
}

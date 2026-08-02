//! **A sonda:** o que acontece HOJE ao arrastar o corpo que carrega a **âncora A**?
//!
//! O pedido do Enio (2026-08-02) é sobre esse gesto: *"se arrasto o objeto onde está
//! a âncora A, todo o sistema vai junto — FK de forma rígida, IK com o último objeto
//! da cadeia se movendo por último, como arrastando uma corda pela ponta"*.
//!
//! Antes de construir, medir: **quanto cada corpo anda** quando o gesto é aberto sobre
//! o corpo-líder, nos dois modos, com e sem âncora estática.
//!
//! `cargo test -p ph2d-physics-ecs --release measure_lead_drag -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, IkOptions, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

fn body(sim: &mut SimWorld, name: &str, x: f32, kind: BodyKind) -> Entity {
    let _ = sim.world_mut().spawn((
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(x, 0.0)),
    ));
    named(sim, name)
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

fn joint(sim: &mut SimWorld, a: &str, b: &str, kind: JointKind, at: f32) -> Entity {
    let n = format!("J-{a}-{b}");
    let _ = sim.world_mut().spawn((
        Name::new(&n),
        PhysicsJoint {
            body_a: stable_name_id(a),
            body_b: stable_name_id(b),
            kind,
            ..PhysicsJoint::of_kind(kind)
        },
        Transform::from_translation(Vec2::new(at, 0.0)),
    ));
    named(sim, &n)
}

/// Quatro elos de 1 m pinados ponta a ponta, **na ordem autorada L1→L2→L3→L4**:
/// cada joint tem `body_a` no elo de trás, então **L1 carrega a âncora A** da
/// primeira junta e é o "líder" no modelo do artista.
///
/// `anchored` prende L4 numa parede estática, que é o rig realista; sem ela a
/// cadeia flutua.
fn chain(anchored: bool, kind: JointKind) -> (SimWorld, PhysicsBridge, Vec<Entity>) {
    let mut sim = SimWorld::new();
    let l1 = body(&mut sim, "L1", 0.0, BodyKind::Dynamic);
    let l2 = body(&mut sim, "L2", 1.0, BodyKind::Dynamic);
    let l3 = body(&mut sim, "L3", 2.0, BodyKind::Dynamic);
    let l4 = body(&mut sim, "L4", 3.0, BodyKind::Dynamic);
    joint(&mut sim, "L1", "L2", kind, 0.5);
    joint(&mut sim, "L2", "L3", kind, 1.5);
    joint(&mut sim, "L3", "L4", kind, 2.5);
    let mut out = vec![l1, l2, l3, l4];
    if anchored {
        let w = body(&mut sim, "Wall", 4.0, BodyKind::Static);
        joint(&mut sim, "L4", "Wall", kind, 3.5);
        out.push(w);
    }
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge, out)
}

fn pose(sim: &SimWorld, e: Entity) -> [f32; 2] {
    sim.world()
        .get::<Transform>(e)
        .map(|t| [t.translation.x, t.translation.y])
        .unwrap_or([f32::NAN; 2])
}

fn write(sim: &mut SimWorld, poses: &[(Entity, [f32; 2], f32)]) {
    for &(e, t, r) in poses {
        if let Some(mut tr) = sim.world_mut().get_mut::<Transform>(e) {
            tr.translation = Vec2::new(t[0], t[1]);
            tr.rotation = r;
        }
    }
}

const NAMES: [&str; 4] = ["L1", "L2", "L3", "L4"];

/// Distância percorrida por cada elo, da pose inicial à final.
fn moved(before: &[[f32; 2]], sim: &SimWorld, e: &[Entity]) -> Vec<f32> {
    (0..4)
        .map(|i| {
            let a = before[i];
            let b = pose(sim, e[i]);
            (b[0] - a[0]).hypot(b[1] - a[1])
        })
        .collect()
}

fn row(label: &str, d: &[f32]) {
    print!("  {label:<26}");
    for (n, v) in NAMES.iter().zip(d) {
        print!(" {n} {v:>7.3}");
    }
    println!();
}

#[test]
#[ignore = "sonda"]
fn measure_lead_drag() {
    for kind in [JointKind::Pin, JointKind::Weld] {
        for anchored in [false, true] {
            println!(
                "\n=== {kind:?} · cadeia {} · ARRASTO EM L1 (o corpo da âncora A) ===",
                if anchored { "ANCORADA em L4" } else { "SOLTA" }
            );

            // ---- o plano: quem a IK/FK considera a RAIZ ao pegar L1?
            let (_s, b, e) = chain(anchored, kind);
            match b.ik_plan(e[0]) {
                Some(p) => {
                    let name = NAMES
                        .iter()
                        .zip(&e)
                        .find(|&(_, &x)| x == p.root)
                        .map_or("Wall", |(n, _)| n);
                    println!("  plano: raiz = {name}, {} arestas", p.edges.len());
                }
                None => println!("  plano: NENHUM (ik_plan devolveu None)"),
            }

            // ---- FK: o gesto abre? o que ele move?
            let (mut sim, mut b, e) = chain(anchored, kind);
            let before: Vec<_> = (0..4).map(|i| pose(&sim, e[i])).collect();
            let opened = b.fk_begin(&sim, e[0], [0.0, 0.0]);
            if opened {
                let n = b.fk_bodies().len();
                let poses = b.fk_move([0.0, 2.0]);
                write(&mut sim, &poses);
                println!("  FK: abriu, move {n} corpo(s)");
                row("FK deslocamento:", &moved(&before, &sim, &e));
            } else {
                println!("  FK: **NAO ABRIU** (fk_begin = false) -- nada se move");
            }

            // ---- IK: o gesto abre? o que ele move, e quem chega por último?
            let (mut sim, mut b, e) = chain(anchored, kind);
            let before: Vec<_> = (0..4).map(|i| pose(&sim, e[i])).collect();
            if b.ik_begin(e[0]) {
                let n = b.posing_bodies().len();
                // Arrasta L1 dois metros para CIMA, em dez passos (como a mão faz).
                for k in 1..=10i16 {
                    let t = [0.0, 2.0 * f32::from(k) / 10.0];
                    let poses = b.ik_move(t, 0.0, IkOptions::default());
                    write(&mut sim, &poses);
                    b.dispatch(&mut sim, false, 0);
                }
                println!("  IK: abriu, arvore de {n} corpo(s)");
                row("IK deslocamento:", &moved(&before, &sim, &e));
                println!("  IK: L1 pedido em (0.000, 2.000), chegou em {:?}", pose(&sim, e[0]));
            } else {
                println!("  IK: **NAO ABRIU** (ik_begin = false) -- nada se move");
            }
        }
    }
    println!(
        "\nLeitura: 'todo o sistema vai junto' pede deslocamento NAO-ZERO nos quatro; \
         'a corda pela ponta' pede que ele CAIA de L1 para L4."
    );
}

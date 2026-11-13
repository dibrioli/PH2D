//! **SONDA: a ponte de tábuas sob carga** — impulso (Rope, Pin) × multibody (coordenadas reduzidas).
//!
//! Correr: `cargo test -p ph2d-physics --profile smoke sonda_da_ponte -- --nocapture --ignored`
//!
//! A geometria é a do gabarito Bridge (`ph2d-joint-templates`, catálogo `lines.rs`): 8 tábuas
//! 0,5 × 0,1 m com 2 cm de folga, postes fixos, caixa 0,5 m de 10 kg caindo de 2 m acima.
//! Tabela e leitura: `docs/JointEditor/02_gabaritos_medidos.md` §«A ponte sob carga».

use super::*;
use super::joint_desc::{JointDesc, JointKind};
use rapier2d::dynamics::{RevoluteJointBuilder, RigidBodyBuilder, RopeJointBuilder};
use rapier2d::geometry::ColliderBuilder;

const N: usize = 8;
const HALF: [f32; 2] = [0.25, 0.05];
const PITCH: f32 = 0.52;
const X0: f32 = -1.82;
const Y: f32 = 1.0;
const POST_X: f32 = 2.24;

#[derive(Clone, Copy, Debug)]
enum Way {
    /// Hoje: `Rope` de impulso com o comprimento da folga.
    Rope,
    /// `Pin` de impulso no meio da folga.
    Pin,
    /// Uma árvore multibody do poste esquerdo à última tábua; o poste direito fecha por um Pin de impulso.
    TreePin,
    /// A mesma árvore; o poste direito fecha por um Rope de impulso.
    TreeRope,
    /// Duas árvores, uma de cada poste, que se encontram no meio por um Pin de impulso.
    TwoTrees,
    /// O caminho do produto: `spawn_joint` com Pins `one_chain` ([`super::chains`]).
    Product,
}

struct Out {
    end: f32,
    low: f32,
    crate_y: f32,
    crate_x: f32,
    crate_v: f32,
    worst_gap: f32,
}

fn run(way: Way, plank_density: f32, substeps: u32) -> Out {
    let mut w = PhysicsWorld::new();
    w.substeps = substeps;
    w.set_dt(PhysicsWorld::DEFAULT_DT);
    let fixed = |w: &mut PhysicsWorld, x: f32, y: f32, hx: f32, hy: f32| {
        let h = w.insert_body(RigidBodyBuilder::fixed().translation(Vector::new(x, y)).build());
        w.colliders
            .insert_with_parent(ColliderBuilder::cuboid(hx, hy).build(), h, &mut w.bodies);
        h
    };
    fixed(&mut w, 0.0, -0.5, 6.0, 0.5);
    let left = fixed(&mut w, -POST_X, 0.6, 0.15, 0.5);
    let right = fixed(&mut w, POST_X, 0.6, 0.15, 0.5);
    let planks: Vec<RigidBodyHandle> = (0..N)
        .map(|i| {
            let x = X0 + PITCH * i as f32;
            let h = w.insert_body(RigidBodyBuilder::dynamic().translation(Vector::new(x, Y)).build());
            w.colliders.insert_with_parent(
                ColliderBuilder::cuboid(HALF[0], HALF[1])
                    .density(plank_density)
                    .build(),
                h,
                &mut w.bodies,
            );
            h
        })
        .collect();
    let load = w.insert_body(RigidBodyBuilder::dynamic().translation(Vector::new(0.0, 3.0)).build());
    w.colliders.insert_with_parent(
        ColliderBuilder::cuboid(0.25, 0.25).density(40.0).build(),
        load,
        &mut w.bodies,
    );

    // As ligações em ordem: (a, b, ponta em a, ponta em b) — pontas nas bordas, 2 cm de folga.
    let post_tip = [0.15, 0.4];
    let mut links: Vec<(RigidBodyHandle, RigidBodyHandle, [f32; 2], [f32; 2])> = Vec::new();
    links.push((left, planks[0], post_tip, [-HALF[0], 0.0]));
    for i in 0..N - 1 {
        links.push((planks[i], planks[i + 1], [HALF[0], 0.0], [-HALF[0], 0.0]));
    }
    links.push((planks[N - 1], right, [HALF[0], 0.0], [-post_tip[0], post_tip[1]]));
    let gap = 0.02_f32;
    // O pino no meio da folga: cada ponta desloca 1 cm para fora, no eixo x.
    let mid = |p: [f32; 2], s: f32| [p[0] + s * gap / 2.0, p[1]];
    let pin = |la: [f32; 2], lb: [f32; 2]| {
        RevoluteJointBuilder::new()
            .local_anchor1(Vector::new(mid(la, 1.0)[0], la[1]))
            .local_anchor2(Vector::new(mid(lb, -1.0)[0], lb[1]))
            .build()
    };
    let rope = |la: [f32; 2], lb: [f32; 2]| {
        RopeJointBuilder::new(gap)
            .local_anchor1(Vector::new(la[0], la[1]))
            .local_anchor2(Vector::new(lb[0], lb[1]))
            .build()
    };
    let last = links.len() - 1;
    for (k, &(a, b, la, lb)) in links.iter().enumerate() {
        match way {
            Way::Rope => {
                w.impulse_joints.insert(a, b, rope(la, lb), true);
            }
            Way::Pin => {
                w.impulse_joints.insert(a, b, pin(la, lb), true);
            }
            Way::TreePin | Way::TreeRope if k < last => {
                w.multibody_joints.insert(a, b, pin(la, lb), true);
            }
            Way::TreePin => {
                w.impulse_joints.insert(a, b, pin(la, lb), true);
            }
            Way::TreeRope => {
                w.impulse_joints.insert(a, b, rope(la, lb), true);
            }
            Way::Product => {
                let d = JointDesc {
                    kind: JointKind::Pin,
                    anchor_a: mid(la, 1.0),
                    anchor_b: mid(lb, -1.0),
                    one_chain: true,
                    ..JointDesc::default()
                };
                assert!(w.spawn_joint(a, b, d).is_some());
            }
            Way::TwoTrees => {
                let meet = links.len() / 2;
                if k < meet {
                    w.multibody_joints.insert(a, b, pin(la, lb), true);
                } else if k == meet {
                    w.impulse_joints.insert(a, b, pin(la, lb), true);
                } else {
                    // A árvore da direita cresce do poste para dentro: o pai é `b`.
                    let j = RevoluteJointBuilder::new()
                        .local_anchor1(Vector::new(mid(lb, -1.0)[0], lb[1]))
                        .local_anchor2(Vector::new(mid(la, 1.0)[0], la[1]))
                        .build();
                    w.multibody_joints.insert(b, a, j, true);
                }
            }
        }
    }

    let mut low = f32::MAX;
    let mut worst_gap = 0.0_f32;
    for _ in 0..600 {
        w.step();
        let y = w.bodies[planks[N / 2]].translation().y;
        low = low.min(y);
        for &(a, b, la, lb) in &links {
            let pa = w.bodies[a].position().transform_point(Vector::new(la[0], la[1]));
            let pb = w.bodies[b].position().transform_point(Vector::new(lb[0], lb[1]));
            worst_gap = worst_gap.max((pa - pb).length());
        }
    }
    Out {
        end: w.bodies[planks[N / 2]].translation().y,
        low,
        crate_y: w.bodies[load].translation().y,
        crate_x: w.bodies[load].translation().x,
        crate_v: w.bodies[load].linvel().length(),
        worst_gap,
    }
}

#[test]
#[ignore = "sonda"]
fn sonda_da_ponte() {
    println!("way        planks  sub | plank5 end  lowest  crate y  crate x  crate v  worst gap (m)");
    for sub in [4_u32, 8] {
        for d in [0.1_f32, 1.0, 10.0] {
            for way in [Way::Rope, Way::Pin, Way::TwoTrees, Way::Product] {
                let o = run(way, d, sub);
                println!(
                    "{:<10} {d:>6} {sub:>4} | {:+.3}     {:+.3}  {:+.3}  {:+.3}  {:.3}   {:.3}",
                    format!("{way:?}"),
                    o.end,
                    o.low,
                    o.crate_y,
                    o.crate_x,
                    o.crate_v,
                    o.worst_gap
                );
            }
        }
    }
}

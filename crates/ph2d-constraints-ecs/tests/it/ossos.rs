//! A IK como item da pilha, nos ossos (W2): corre na posição da lista, escreve a corrente, parte do
//! autorado e respeita o espelho. Fixture: o braço dos casos `ik_*` do oráculo.

use bevy_ecs::entity::Entity;
use ph2d_constraints::{BendSide, Canais, ChildOf as Filho, CopyTransform, Ik, Kind, Pose2};
use ph2d_constraints_ecs::{Constraints, Estado, Estados, evaluate};
use ph2d_ecs::{ChildOf, SimWorld, StableId, Transform, world_transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_skeleton_ecs::Bone;

const FIXTURE: &str =
    include_str!("../../../../docs/BlenderConstraints/fixtures/blender_5_2_3_constraints_2d.json");
/// O resíduo do solver iterativo do Blender (`ph2d-constraints/tests/it/oraculo_ossos.rs`).
const TOL_IK: f64 = 3e-3;

fn em(x: f32, y: f32, rot_deg: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t.rotation = rot_deg.to_radians();
    t
}

fn osso(sim: &mut SimWorld, id: u64, t: Transform, pai: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            StableId(id),
            t,
            Bone {
                length: 1.0,
                strength: 1.0,
                ..Default::default()
            },
        ))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

fn mundo(sim: &SimWorld, e: Entity) -> Transform {
    world_transform(sim.world(), e).unwrap()
}

fn cabeca(sim: &SimWorld, e: Entity) -> [f64; 2] {
    let t = mundo(sim, e);
    [f64::from(t.translation.x), f64::from(t.translation.y)]
}

fn cauda(sim: &SimWorld, e: Entity) -> [f64; 2] {
    let t = mundo(sim, e);
    let (s, c) = f64::from(t.rotation).sin_cos();
    let l = f64::from(t.scale.x);
    let [x, y] = cabeca(sim, e);
    [x + c * l, y + s * l]
}

#[track_caller]
fn perto(a: [f64; 2], b: [f64; 2], tol: f64) {
    assert!(
        (a[0] - b[0]).abs() < tol && (a[1] - b[1]).abs() < tol,
        "{a:?} ≠ {b:?}"
    );
}

fn oraculo() -> serde_json::Value {
    serde_json::from_str(FIXTURE).unwrap()
}

fn ponto(v: &serde_json::Value) -> [f64; 2] {
    [
        v["cauda_x"].as_f64().unwrap(),
        v["cauda_y"].as_f64().unwrap(),
    ]
}

fn ik(chain: u32) -> Kind {
    Kind::Ik(Ik {
        chain,
        ..Ik::default()
    })
}

fn quadro(sim: &mut SimWorld, drive: &mut PreviewDrive) {
    evaluate(sim, drive);
    drive.settle();
}

/// O braço do caso `ik_com_copy_rotation_abaixo`: dois ossos de 1 deitados no +X.
fn braco_recto(sim: &mut SimWorld) -> (Entity, Entity) {
    let b1 = osso(sim, 1, em(0.0, 0.0, 0.0), None);
    let b2 = osso(sim, 2, em(1.0, 0.0, 0.0), Some(b1));
    (b1, b2)
}

#[test]
fn ik_is_an_ordinary_item_in_the_stack() {
    let o = oraculo();
    let blender = &o["ik_com_copy_rotation_abaixo"];
    for ik_primeiro in [true, false] {
        let mut sim = SimWorld::new();
        let mut drive = PreviewDrive::default();
        let (b1, b2) = braco_recto(&mut sim);
        sim.world_mut().spawn((StableId(10), em(1.2, 0.9, 0.0)));
        sim.world_mut().spawn((StableId(11), em(0.0, 0.0, 90.0)));
        let mut c = Constraints::default();
        let copia = Kind::CopyTransform(CopyTransform {
            canais: Canais::ROTATION,
            ..Default::default()
        });
        if ik_primeiro {
            c.push(ik(2), Some(StableId(10)));
            c.push(copia, Some(StableId(11)));
        } else {
            c.push(copia, Some(StableId(11)));
            c.push(ik(2), Some(StableId(10)));
        }
        sim.world_mut().entity_mut(b2).insert(c);
        quadro(&mut sim, &mut drive);
        // O cotovelo é o da IK nas duas ordens (o Copy Rotation só mexe na mão).
        perto(cauda(&sim, b1), ponto(&blender["b1"]), TOL_IK);
        perto(cabeca(&sim, b2), cauda(&sim, b1), 1e-5);
        if ik_primeiro {
            // ⛔ Ao contrário do Blender (onde o Copy Rotation some): a mão RODA para 90°.
            let rot = f64::from(mundo(&sim, b2).rotation);
            assert!(
                ph2d_constraints::wrap_pi(rot - 90f64.to_radians()).abs() < 1e-5,
                "{rot}"
            );
            let longe = cauda(&sim, b2);
            assert!(
                (longe[0] - 1.2).hypot(longe[1] - 0.9) > 0.1,
                "a cauda não fica no alvo"
            );
        } else {
            // A IK em baixo vence: a mão chega ao alvo, como no Blender.
            perto(cauda(&sim, b2), ponto(&blender["b2"]), TOL_IK);
        }
        assert_eq!(
            sim.world().resource::<Estados>().of(b2, 0),
            Some(Estado::Activa)
        );
    }
}

/// O braço dobrado dos casos `ik_dois_ossos`, sob uma raiz que se pode espelhar.
fn braco_dobrado(sim: &mut SimWorld, sx: f32) -> (Entity, Entity, Entity) {
    let mut t = Transform::IDENTITY;
    t.scale.x = sx;
    let raiz = sim.world_mut().spawn((StableId(20), t)).id();
    let b1 = osso(sim, 1, em(0.0, 0.0, 0.0), Some(raiz));
    let b2 = osso(sim, 2, em(1.0, 0.0, 30.0), Some(b1));
    (raiz, b1, b2)
}

#[test]
fn bend_side_flips_with_the_mirror() {
    let o = oraculo();
    let c30 = 30f64.to_radians();
    let lado = ph2d_skeleton::bend_side_of(
        &[[0.0, 0.0], [1.0, 0.0], [1.0 + c30.cos(), c30.sin()]],
        [1.0 + c30.cos(), c30.sin()],
        2.0,
    );
    assert_ne!(lado, BendSide::Keep);
    for (sx, caso) in [
        (1.0f32, "perto"),
        (-1.0, "espelho.perto"),
        (1.0, "baixo"),
        (-1.0, "espelho.baixo"),
    ] {
        let v = &o["ik_dois_ossos"][caso];
        let mut sim = SimWorld::new();
        let mut drive = PreviewDrive::default();
        let (_, b1, b2) = braco_dobrado(&mut sim, sx);
        let alvo = [
            v["alvo"][0].as_f64().unwrap(),
            v["alvo"][1].as_f64().unwrap(),
        ];
        #[expect(clippy::cast_possible_truncation, reason = "o Transform é f32")]
        sim.world_mut()
            .spawn((StableId(10), em(alvo[0] as f32, alvo[1] as f32, 0.0)));
        let mut c = Constraints::default();
        c.push(
            Kind::Ik(Ik {
                bend: lado,
                ..Ik::default()
            }),
            Some(StableId(10)),
        );
        sim.world_mut().entity_mut(b2).insert(c);
        for _ in 0..3 {
            quadro(&mut sim, &mut drive);
            // O joelho do espelho é o espelho do joelho — e não salta de quadro para quadro.
            perto(cauda(&sim, b1), ponto(&v["b1"]), TOL_IK);
            perto(cauda(&sim, b2), ponto(&v["b2"]), TOL_IK);
        }
    }
}

#[test]
fn a_reader_of_the_chain_runs_after_the_ik() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let (b1, b2) = braco_recto(&mut sim);
    sim.world_mut().spawn((StableId(10), em(1.2, 0.9, 0.0)));
    // O leitor nasce DEPOIS (bits menores — o desempate da ordem corre-o primeiro): sem a aresta da
    // corrente leria o ombro de antes da IK (medido pela mutação M2).
    let leitor = sim
        .world_mut()
        .spawn((StableId(30), Transform::IDENTITY))
        .id();
    assert!(leitor.to_bits() < b2.to_bits());
    let mut c = Constraints::default();
    c.push(ik(2), Some(StableId(10)));
    sim.world_mut().entity_mut(b2).insert(c);
    let mut segue = Constraints::default();
    segue.push(
        Kind::ChildOf(Filho {
            canais: Canais::ALL,
            offset: Pose2::IDENTITY,
        }),
        Some(StableId(1)),
    );
    sim.world_mut().entity_mut(leitor).insert(segue);
    quadro(&mut sim, &mut drive);
    assert_eq!(mundo(&sim, leitor).rotation, mundo(&sim, b1).rotation);
}

#[test]
fn a_target_inside_the_chain_is_a_cycle() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let (b1, b2) = braco_recto(&mut sim);
    let dentro = sim
        .world_mut()
        .spawn((StableId(10), em(0.5, 0.5, 0.0), ChildOf(b1)))
        .id();
    let mut c = Constraints::default();
    c.push(ik(2), Some(StableId(10)));
    sim.world_mut().entity_mut(b2).insert(c);
    let antes = (mundo(&sim, b1), mundo(&sim, b2), mundo(&sim, dentro));
    quadro(&mut sim, &mut drive);
    assert_eq!(
        sim.world().resource::<Estados>().of(b2, 0),
        Some(Estado::Ciclo)
    );
    assert_eq!(
        (mundo(&sim, b1), mundo(&sim, b2), mundo(&sim, dentro)),
        antes
    );
}

#[test]
fn a_half_ik_is_a_function_of_the_authored_pose() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let (b1, b2) = braco_recto(&mut sim);
    sim.world_mut().spawn((StableId(10), em(1.2, 0.9, 0.0)));
    let mut c = Constraints::default();
    let id = c.push(ik(2), Some(StableId(10)));
    c.item_mut(id).unwrap().influence = 0.5;
    sim.world_mut().entity_mut(b2).insert(c);
    quadro(&mut sim, &mut drive);
    let primeiro = (mundo(&sim, b1), mundo(&sim, b2));
    for _ in 0..10 {
        quadro(&mut sim, &mut drive);
        assert_eq!((mundo(&sim, b1), mundo(&sim, b2)), primeiro, "não rasteja");
    }
    // Gravar a meio devolve o autorado da corrente inteira.
    let vivo = drive.substitute(&mut sim);
    assert_eq!(
        *sim.world().get::<Transform>(b1).unwrap(),
        em(0.0, 0.0, 0.0)
    );
    assert_eq!(
        *sim.world().get::<Transform>(b2).unwrap(),
        em(1.0, 0.0, 0.0)
    );
    PreviewDrive::restore(&mut sim, &vivo);
    // Apagar o item devolve o autorado.
    sim.world_mut().entity_mut(b2).remove::<Constraints>();
    quadro(&mut sim, &mut drive);
    assert_eq!(
        *sim.world().get::<Transform>(b1).unwrap(),
        em(0.0, 0.0, 0.0)
    );
    assert_eq!(
        *sim.world().get::<Transform>(b2).unwrap(),
        em(1.0, 0.0, 0.0)
    );
}

#[test]
fn ik_on_an_object_that_is_not_a_bone_says_so() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    sim.world_mut().spawn((StableId(10), em(1.2, 0.9, 0.0)));
    let mut c = Constraints::default();
    c.push(ik(2), Some(StableId(10)));
    let e = sim
        .world_mut()
        .spawn((StableId(1), Transform::IDENTITY, c))
        .id();
    quadro(&mut sim, &mut drive);
    assert_eq!(
        sim.world().resource::<Estados>().of(e, 0),
        Some(Estado::NaoEOsso)
    );
    assert_eq!(
        *sim.world().get::<Transform>(e).unwrap(),
        Transform::IDENTITY
    );
}

#[test]
fn the_stretch_reaches_a_far_target_by_scaling_the_chain_root() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let (b1, b2) = braco_recto(&mut sim);
    sim.world_mut().spawn((StableId(10), em(3.0, 1.0, 0.0)));
    let mut c = Constraints::default();
    c.push(
        Kind::Ik(Ik {
            stretch: true,
            ..Ik::default()
        }),
        Some(StableId(10)),
    );
    sim.world_mut().entity_mut(b2).insert(c);
    quadro(&mut sim, &mut drive);
    perto(cauda(&sim, b2), [3.0, 1.0], 1e-5);
    let k = 10f64.sqrt() / 2.0;
    assert!((f64::from(sim.world().get::<Transform>(b1).unwrap().scale.x) - k).abs() < 1e-6);
    assert_eq!(sim.world().get::<Transform>(b2).unwrap().scale.x, 1.0);
}

/// Um `IkGoal` de um ficheiro anterior à W2 vira o item IK com os mesmos números — e o braço obedece.
#[test]
fn an_old_ik_goal_becomes_the_ik_item() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let (b1, b2) = braco_recto(&mut sim);
    sim.world_mut().spawn((StableId(10), em(1.2, 0.9, 0.0)));
    sim.world_mut().entity_mut(b2).insert(ph2d_skeleton_ecs::IkGoal {
        target: StableId(10),
        chain: 2,
        mix: 0.75,
        softness: 0.1,
        bend: BendSide::Cw,
        offset: 0.0,
    });
    assert_eq!(ph2d_constraints_ecs::converter_ik_goals(sim.world_mut()), 1);
    assert!(sim.world().get::<ph2d_skeleton_ecs::IkGoal>(b2).is_none());
    let (_, it, ik) = ph2d_constraints_ecs::ik_de(sim.world(), b2).expect("o item IK");
    assert_eq!((it.target, it.influence), (Some(StableId(10)), 0.75));
    assert_eq!((ik.chain, ik.softness, ik.bend, ik.stretch), (2, 0.1, BendSide::Cw, false));
    assert_eq!(ph2d_constraints_ecs::converter_ik_goals(sim.world_mut()), 0, "idempotente");
    let antes = mundo(&sim, b1);
    quadro(&mut sim, &mut drive);
    assert_ne!(mundo(&sim, b1), antes, "a corrente obedece ao item convertido");
}

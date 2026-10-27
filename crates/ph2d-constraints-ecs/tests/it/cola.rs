//! A cola no mundo (05_W5 §4): o gesto escreve o DOCUMENTO dos parceiros no mesmo quadro, a curva conduz pelo
//! `PreviewDrive`, tocar × scrub dá o mesmo bit e todo parceiro entra na população do Auto Key.

use bevy_ecs::entity::Entity;
use ph2d_constraints::Pose2;
use ph2d_constraints_cola::Cola;
use ph2d_constraints_ecs::{Colas, colar};
use ph2d_ecs::{AlvosDasFamilias, Locked, SimWorld, StableId, Transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

#[expect(
    clippy::cast_possible_truncation,
    reason = "o `Transform` da casa é f32"
)]
fn tf(x: f64, y: f64, rot: f64) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x as f32;
    t.translation.y = y as f32;
    t.rotation = rot as f32;
    t
}

fn pose(t: Transform) -> Pose2 {
    Pose2 {
        x: f64::from(t.translation.x),
        y: f64::from(t.translation.y),
        rot: f64::from(t.rotation),
        sx: f64::from(t.scale.x),
        sy: f64::from(t.scale.y),
    }
}

fn objeto(sim: &mut SimWorld, id: u64, t: Transform) -> Entity {
    sim.world_mut().spawn((StableId(id), t)).id()
}

fn tf_de(sim: &SimWorld, e: Entity) -> Transform {
    *sim.world().get::<Transform>(e).unwrap()
}

fn mexe(sim: &mut SimWorld, e: Entity, t: Transform) {
    *sim.world_mut().get_mut::<Transform>(e).unwrap() = t;
}

#[test]
fn dragging_either_glued_object_moves_the_other() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let (ta, tb) = (tf(0.0, 0.0, 0.0), tf(3.0, 1.0, 0.4));
    let a = objeto(&mut sim, 1, ta);
    let b = objeto(&mut sim, 2, tb);
    let cola = Cola::rigida(1, 1, 2, pose(ta), pose(tb)).unwrap();
    sim.world_mut().entity_mut(a).insert(Colas {
        items: vec![cola],
        next_id: 2,
    });
    colar(&mut sim, &mut drive);
    assert_eq!(
        (tf_de(&sim, a), tf_de(&sim, b)),
        (ta, tb),
        "colar sem gesto não mexe ninguém"
    );

    let ph2d_constraints_cola::Tipo::Rigida { rel } = cola.tipo else {
        unreachable!()
    };
    for (mao, outro, alvo) in [(a, b, tf(2.0, -1.0, 0.9)), (b, a, tf(-4.0, 2.5, -1.2))] {
        mexe(&mut sim, mao, alvo);
        colar(&mut sim, &mut drive);
        assert_eq!(tf_de(&sim, mao), alvo, "a mão fica onde o gesto a pôs");
        let (pa, pb) = (pose(tf_de(&sim, a)), pose(tf_de(&sim, b)));
        let esperado = rel.under(pa);
        let erro = (esperado.x - pb.x)
            .abs()
            .max((esperado.y - pb.y).abs())
            .max((esperado.rot - pb.rot).abs());
        assert!(
            erro < 1e-5,
            "o parceiro {outro:?} seguiu a mão {mao:?} (erro {erro:e})"
        );
        assert!(
            drive.driven_by(Driver::ColaPose).is_empty(),
            "o gesto escreve o DOCUMENTO (um passo de undo)"
        );
    }
}

/// O quadrilátero articulado do oráculo: chão (cadeado), manivela 1, biela 3,5, balancim 3.
fn quadrilatero(sim: &mut SimWorld) -> [Entity; 4] {
    let (l_chao, l_bal) = (4.0, 3.0);
    let a = (3.5f64.powi(2) - l_bal * l_bal + 9.0) / 6.0;
    let h = (3.5f64.powi(2) - a * a).sqrt();
    let p = [
        tf(0.0, 0.0, 0.0),
        tf(0.0, 0.0, 0.0),
        tf(1.0, 0.0, h.atan2(a)),
        tf(l_chao, 0.0, h.atan2(1.0 + a - l_chao)),
    ];
    let es = [1, 2, 3, 4].map(|i| objeto(sim, i, p[i as usize - 1]));
    sim.world_mut().entity_mut(es[0]).insert(Locked);
    let q = p.map(pose);
    let colas = vec![
        Cola::pino(1, 1, 2, q[0], q[1], [0.0, 0.0]).unwrap(),
        Cola::pino(2, 2, 3, q[1], q[2], [1.0, 0.0]).unwrap(),
        Cola::pino(3, 3, 4, q[2], q[3], [1.0 + a, h]).unwrap(),
        Cola::pino(4, 1, 4, q[0], q[3], [l_chao, 0.0]).unwrap(),
    ];
    sim.world_mut().entity_mut(es[0]).insert(Colas {
        items: colas,
        next_id: 5,
    });
    es
}

/// Um quadro: a curva conduz a manivela (como a timeline, pelo `PreviewDrive`), a cola, e o fim do quadro.
fn quadro(
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
    manivela: Entity,
    f: u32,
) -> [Transform; 2] {
    let agora = tf_de(sim, manivela);
    let novo = tf(0.0, 0.0, f64::from(f) * std::f64::consts::TAU / 48.0);
    drive.driven(
        manivela,
        Driven::SolverPose(agora),
        Driven::SolverPose(novo),
    );
    mexe(sim, manivela, novo);
    colar(sim, drive);
    drive.settle();
    [3, 4].map(|i| {
        let e = ph2d_ecs::entity_with_stable_id(sim.world(), StableId(i)).unwrap();
        tf_de(sim, e)
    })
}

#[test]
fn glued_play_and_scrub_are_the_same_bit() {
    let tocar: Vec<[Transform; 2]> = {
        let mut sim = SimWorld::new();
        let mut drive = PreviewDrive::default();
        let [chao, manivela, ..] = quadrilatero(&mut sim);
        colar(&mut sim, &mut drive);
        let v = (0..49)
            .map(|f| quadro(&mut sim, &mut drive, manivela, f))
            .collect();
        assert_eq!(tf_de(&sim, chao), tf(0.0, 0.0, 0.0), "o cadeado é o chão");
        v
    };
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let [_, manivela, ..] = quadrilatero(&mut sim);
    colar(&mut sim, &mut drive);
    for f in (0..49).map(|k| (k * 17 + 5) % 49) {
        let lido = quadro(&mut sim, &mut drive, manivela, f);
        assert_eq!(lido, tocar[f as usize], "quadro {f}: scrub ≠ tocar");
    }
    assert!(tocar[12] != tocar[0], "a manivela a 90° move o balancim");
}

#[test]
fn autokey_records_every_glued_partner() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let es = quadrilatero(&mut sim);
    colar(&mut sim, &mut drive);
    let alvos: Vec<u64> = sim.world().resource::<AlvosDasFamilias>().todas().collect();
    for e in es {
        assert!(
            alvos.contains(&e.to_bits()),
            "{e:?} fora da população do Auto Key ANTES do gesto"
        );
    }
}

#[test]
fn a_glue_survives_save_and_open() {
    use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot, snapshot_to_world, world_to_snapshot};
    use ph2d_ecs::{TransformPropagationState, WorklistBuf};
    let mut reg = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_constraints_ecs::register_constraint_components(&mut reg);
    let mut sim = SimWorld::new();
    quadrilatero(&mut sim);
    let gravadas: Vec<Colas> = sim
        .world_mut()
        .query::<&Colas>()
        .iter(sim.world())
        .cloned()
        .collect();
    let mut snap = WorldSnapshot::new();
    let (mut st, mut wl) = (
        TransformPropagationState::new(sim.world_mut()),
        WorklistBuf::default(),
    );
    world_to_snapshot(sim.world_mut(), &mut st, &mut wl, &reg, &mut snap).unwrap();
    let bytes = postcard::to_allocvec(&snap).unwrap();
    let lido: WorldSnapshot = postcard::from_bytes(&bytes).unwrap();
    let mut outro = SimWorld::new();
    snapshot_to_world(outro.world_mut(), &lido, &reg).unwrap();
    let abertas: Vec<Colas> = outro
        .world_mut()
        .query::<&Colas>()
        .iter(outro.world())
        .cloned()
        .collect();
    assert_eq!(abertas, gravadas);
    assert_eq!(abertas[0].items.len(), 4);
}

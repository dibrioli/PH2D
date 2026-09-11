//! **A FK sem junta acima: a peça ANDA INTEIRA** (W-LeadDrag, etapa A).
//!
//! A subida do `build_fk_session` procura a primeira junta com grau de liberdade
//! acima do corpo pego. Quando não há nenhuma — uma cadeia toda soldada, ou o
//! corpo-raiz da árvore — ela chegava ao topo e terminava num `?`, então o gesto
//! **nascia morto**: `fk_begin` devolvia `false` e nada se movia.
//!
//! ⚠️ **E o cabeçalho do módulo já prometia o contrário** (*"a peça soldada
//! viaja junta, que é o que 'soldado' quer dizer"*), o que torna isto uma
//! promessa não cumprida e não uma capacidade nova.
//!
//! O oráculo é o mesmo do irmão `fk_gesture`: um movimento rígido preserva TODA
//! distância — e, aqui, também **todo ângulo**, que é o que separa *a peça
//! andou* de *a peça girou em torno de alguma coisa*.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
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

fn joint(sim: &mut SimWorld, a: &str, b: &str, kind: JointKind, at: f32) {
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
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
}

/// Três elos de 1 m **SOLDADOS** ponta a ponta, sem parede — a peça rígida vai
/// até a raiz da árvore em qualquer direção que a subida tome.
fn welded() -> (SimWorld, PhysicsBridge, Vec<Entity>) {
    let mut sim = SimWorld::new();
    let l1 = body(&mut sim, "L1", 0.5, BodyKind::Dynamic);
    let l2 = body(&mut sim, "L2", 1.5, BodyKind::Dynamic);
    let l3 = body(&mut sim, "L3", 2.5, BodyKind::Dynamic);
    joint(&mut sim, "L1", "L2", JointKind::Weld, 1.0);
    joint(&mut sim, "L2", "L3", JointKind::Weld, 2.0);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge, vec![l1, l2, l3])
}

fn pose_of(poses: &[(Entity, [f32; 2], f32)], e: Entity) -> ([f32; 2], f32) {
    poses
        .iter()
        .find(|(x, _, _)| *x == e)
        .map(|&(_, t, r)| (t, r))
        .expect("body is in the moved set")
}

/// **O gesto EXISTE.** Este é o repro: antes da correção `fk_begin` devolvia
/// `false` numa cadeia toda soldada e o artista arrastava sem nada acontecer.
///
/// Mutação (devolver a subida ao `?` que fechava no topo) ⇒ `fk_begin` false,
/// RED aqui e nos três abaixo.
#[test]
fn a_fully_welded_chain_opens_a_gesture_instead_of_refusing() {
    let (sim, mut bridge, e) = welded();
    assert!(
        bridge.fk_begin(&sim, e[1], [1.5, 0.0]),
        "pegar um elo de uma peça soldada nao abriu gesto nenhum"
    );
    assert_eq!(
        bridge.fk_bodies().len(),
        3,
        "a peca soldada tem de viajar INTEIRA"
    );
    assert!(
        bridge.fk_session().expect("sessao viva").is_rigid(),
        "sem junta com grau de liberdade acima, o gesto e' uma TRANSLACAO"
    );
}

/// **A peça anda e não vira.** É a diferença entre este ramo e o da dobradiça,
/// que move o mesmo conjunto quando a peça é a subárvore toda.
///
/// Mutação (rodar os corpos junto com a translação) ⇒ RED na rotação.
#[test]
fn the_piece_travels_without_turning_and_keeps_every_distance() {
    let (sim, mut bridge, e) = welded();
    assert!(bridge.fk_begin(&sim, e[1], [1.5, 0.0]));
    let poses = bridge.fk_move([1.5 + 2.0, 0.0 + 3.0]);
    assert_eq!(poses.len(), 3);

    for (i, (&x0, &ent)) in [0.5f32, 1.5, 2.5].iter().zip(&e).enumerate() {
        let (t, r) = pose_of(&poses, ent);
        assert!(
            (t[0] - (x0 + 2.0)).abs() < 1e-4 && (t[1] - 3.0).abs() < 1e-4,
            "elo {i} devia seguir o cursor: {t:?}"
        );
        assert!(
            r.abs() < 1e-6,
            "uma translacao nao gira nada, e o elo {i} girou {r}"
        );
    }
}

/// **CONTROLE: o ramo novo não roubou o caso da dobradiça.** Uma solda ABAIXO
/// de uma dobradiça continua subindo até ela e a peça gira, como sempre.
///
/// Sem este gate a correção poderia ter feito TODO gesto de FK virar translação
/// e os quatro gates acima ficariam mais verdes ainda.
#[test]
fn a_weld_under_a_hinge_still_bends_at_the_hinge() {
    let mut sim = SimWorld::new();
    let _wall = body(&mut sim, "Wall", -0.5, BodyKind::Static);
    let l1 = body(&mut sim, "L1", 0.5, BodyKind::Dynamic);
    let l2 = body(&mut sim, "L2", 1.5, BodyKind::Dynamic);
    joint(&mut sim, "Wall", "L1", JointKind::Pin, 0.0);
    joint(&mut sim, "L1", "L2", JointKind::Weld, 1.0);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);

    assert!(bridge.fk_begin(&sim, l2, [1.5, 0.0]));
    assert!(
        !bridge.fk_session().expect("sessao viva").is_rigid(),
        "ha' uma dobradica acima: o gesto tem de DOBRAR, nao transladar"
    );
    // Um quarto de volta em torno da âncora em x = 0.
    let poses = bridge.fk_move([0.0, 1.5]);
    let (t1, r1) = pose_of(&poses, l1);
    let (t2, _) = pose_of(&poses, l2);
    assert!(
        r1 > 1.0,
        "o elo preso a' dobradica tem de GIRAR, e girou {r1} rad"
    );
    assert!(
        t1[1] > 0.3 && t2[1] > 1.0,
        "a peca soldada acompanha o giro: {t1:?} {t2:?}"
    );
}

/// **A peça só viaja se o TOPO dela puder se mover — a PAREDE não anda.**
///
/// ⚠️ Este gate existe porque a sonda `measure_lead_drag` achou o defeito que
/// eu tinha acabado de escrever: uma corrente soldada a um corpo `Static` movia
/// **cinco** corpos, o estático incluso, e a cena inteira saía do lugar por um
/// arrasto que devia não fazer nada. A lei já estava escrita no módulo irmão —
/// *a raiz é o que **não se move** ao posar* — e o ramo novo a atropelava.
///
/// Mutação (tirar a condição de `Dynamic`) ⇒ `fk_begin` abre, move 3 corpos e
/// a parede viaja, RED.
#[test]
fn a_piece_welded_to_a_wall_has_no_gesture() {
    let mut sim = SimWorld::new();
    let _wall = body(&mut sim, "Wall", -0.5, BodyKind::Static);
    let _l1 = body(&mut sim, "L1", 0.5, BodyKind::Dynamic);
    let l2 = body(&mut sim, "L2", 1.5, BodyKind::Dynamic);
    joint(&mut sim, "Wall", "L1", JointKind::Weld, 0.0);
    joint(&mut sim, "L1", "L2", JointKind::Weld, 1.0);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);

    assert!(
        !bridge.fk_begin(&sim, l2, [1.5, 0.0]),
        "uma peca soldada a uma parede nao tem grau de liberdade nenhum"
    );
    assert!(
        bridge.fk_bodies().is_empty(),
        "e nada foi posto no conjunto que o gesto moveria"
    );
}

/// **O gesto é função do CURSOR, não da lista de Moves.** A lei que esta linha
/// e a do Painter pagaram várias vezes: um produto sobre os eventos faz o
/// resultado depender da taxa de polling do mouse.
///
/// Mutação (acumular `raw` por incrementos em vez de medir contra o press) ⇒ o
/// caminho de dez passos diverge do de um.
#[test]
fn the_same_cursor_gives_the_same_pose_however_many_moves_it_took() {
    let target = [1.5 + 1.7, -2.3];

    let (sim, mut bridge, e) = welded();
    assert!(bridge.fk_begin(&sim, e[1], [1.5, 0.0]));
    let one = bridge.fk_move(target);

    let (sim, mut bridge, e) = welded();
    assert!(bridge.fk_begin(&sim, e[1], [1.5, 0.0]));
    let mut many = Vec::new();
    for k in 1..=10i16 {
        let f = f32::from(k) / 10.0;
        many = bridge.fk_move([1.5 + 1.7 * f, -2.3 * f]);
    }

    for (i, &ent) in e.iter().enumerate().take(3) {
        let (a, _) = pose_of(&one, ent);
        let (b, _) = pose_of(&many, ent);
        assert!(
            (a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5,
            "elo {i}: um passo {a:?} contra dez {b:?}"
        );
    }
}

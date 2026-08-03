//! **A CORDA PELA PONTA** (W-LeadDrag, etapa B) — arrastar a RAIZ da cadeia.
//!
//! A IK responde *"ponha a ponta ali"* com a raiz parada. Pegando a própria raiz
//! não há nada atrás para resolver, e a versão anterior **recusava** o gesto
//! (`root == tip` devolvia `None`) — sem resposta justamente para o gesto de
//! levar o rig de lugar.
//!
//! ⚠️ **O oráculo é o PERFIL no caminho, nunca o deslocamento final.** *"O
//! último se move por último"* é afirmação sobre o INSTANTE: puxada longa o
//! bastante, tudo acaba andando, e o número final não distingue uma corda de um
//! bloco. Foi essa distinção que expôs a primeira lei desta wave como errada.

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
}

/// Quatro elos de 1 m em +X, **autorados L1→L2→L3→L4**: L1 é a cabeça, e é ela
/// que a mão pega. Âncoras nas fronteiras (0,5 · 1,5 · 2,5).
fn chain(kind: JointKind) -> (SimWorld, PhysicsBridge, Vec<Entity>) {
    let mut sim = SimWorld::new();
    let l1 = body(&mut sim, "L1", 0.0, BodyKind::Dynamic);
    let l2 = body(&mut sim, "L2", 1.0, BodyKind::Dynamic);
    let l3 = body(&mut sim, "L3", 2.0, BodyKind::Dynamic);
    let l4 = body(&mut sim, "L4", 3.0, BodyKind::Dynamic);
    joint(&mut sim, "L1", "L2", kind, 0.5);
    joint(&mut sim, "L2", "L3", kind, 1.5);
    joint(&mut sim, "L3", "L4", kind, 2.5);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge, vec![l1, l2, l3, l4])
}

fn pose(sim: &SimWorld, e: Entity) -> [f32; 2] {
    sim.world()
        .get::<Transform>(e)
        .map(|t| [t.translation.x, t.translation.y])
        .expect("body has a transform")
}

fn write(sim: &mut SimWorld, poses: &[(Entity, [f32; 2], f32)]) {
    for &(e, t, r) in poses {
        if let Some(mut tr) = sim.world_mut().get_mut::<Transform>(e) {
            tr.translation = Vec2::new(t[0], t[1]);
            tr.rotation = r;
        }
    }
}

/// Arrasta a cabeça `dist` metros para cima em `steps` passos e devolve o
/// deslocamento de cada elo.
fn drag(kind: JointKind, dist: f32, steps: i16) -> Vec<f32> {
    let (mut sim, mut b, e) = chain(kind);
    let before: Vec<_> = (0..4).map(|i| pose(&sim, e[i])).collect();
    assert!(b.ik_begin(e[0]), "pegar a cabeca tem de abrir gesto");
    for k in 1..=steps {
        let t = [0.0, dist * f32::from(k) / f32::from(steps)];
        let poses = b.ik_move(t, 0.0, IkOptions::default());
        write(&mut sim, &poses);
        b.dispatch(&mut sim, false, 0);
    }
    (0..4)
        .map(|i| {
            let (a, c) = (before[i], pose(&sim, e[i]));
            (c[0] - a[0]).hypot(c[1] - a[1])
        })
        .collect()
}

/// **O gesto EXISTE.** Repro: antes da correção `ik_begin` devolvia `false` ao
/// pegar a cabeça de uma cadeia livre — o plano recusava com `root == tip`.
///
/// Mutação (restaurar a recusa) ⇒ RED aqui e nos quatro abaixo.
#[test]
fn grabbing_the_head_of_a_free_chain_opens_a_gesture() {
    let (_sim, mut bridge, e) = chain(JointKind::Pin);
    assert!(bridge.ik_begin(e[0]));
    assert_eq!(
        bridge.posing_bodies().len(),
        4,
        "a corda inteira e' o que este gesto move"
    );
}

/// **Todo o sistema vai junto.** O pedido do Enio, na sua forma mais direta.
#[test]
fn the_whole_chain_comes_along() {
    let d = drag(JointKind::Pin, 2.0, 20);
    for (i, v) in d.iter().enumerate() {
        assert!(*v > 0.05, "o elo {i} ficou parado ({v:.3} m)");
    }
    assert!(
        (d[0] - 2.0).abs() < 1e-3,
        "a cabeca segue o cursor exatamente: {:.3}",
        d[0]
    );
}

/// **O último se move por ÚLTIMO** — o perfil decai da mão para a cauda no
/// começo da puxada, que é o que arrastar uma corda pela ponta parece.
///
/// ⚠️ Medido no INSTANTE (10 cm de puxada), não no fim: com 2 m tudo já andou.
/// Números do produto: **0,100 · 0,050 · 0,005 · 0,005**.
///
/// Mutação (apontar o CENTRO do corpo em vez da junta seguinte) ⇒ a alavanca de
/// gangorra inverte o perfil, `0,100 · 0,010 · 0,028 · 0,043`, e o gate sangra
/// no primeiro par.
#[test]
fn the_far_end_moves_last() {
    let d = drag(JointKind::Pin, 0.1, 4);
    for i in 0..3 {
        assert!(
            d[i] >= d[i + 1] - 1e-4,
            "o elo {} andou {:.4} e o {} andou {:.4} -- a corda inverteu",
            i,
            d[i],
            i + 1,
            d[i + 1]
        );
    }
    assert!(
        d[3] < d[0] * 0.2,
        "a cauda mal devia se mexer: {:.4} contra {:.4} da cabeca",
        d[3],
        d[0]
    );
}

/// **Uma cadeia SOLDADA não é corda: ela vai inteira.** O mesmo gesto, e a lei
/// angular nem chega a ser consultada porque nenhum elo dobra.
#[test]
fn a_welded_chain_travels_rigidly_instead_of_trailing() {
    let d = drag(JointKind::Weld, 2.0, 20);
    for (i, v) in d.iter().enumerate() {
        assert!(
            (v - 2.0).abs() < 1e-3,
            "o elo {i} de uma peca soldada tem de seguir a mao: {v:.3}"
        );
    }
}

/// **A forma CONVERGE com a amostragem em vez de divergir.**
///
/// Esta é a única sessão desta linha com MEMÓRIA — a corda depende do CAMINHO,
/// não da posição em que a mão parou —, e o que justifica a exceção é que
/// refinar os passos aproxima um limite (a tractriz) em vez de mudar a
/// resposta sem parar. O gate mede exatamente essa frase: dobrar a amostragem
/// tem de mexer MENOS que a metade anterior.
#[test]
fn a_finer_drag_of_the_same_path_converges_instead_of_diverging() {
    let a = drag(JointKind::Pin, 1.0, 10);
    let b = drag(JointKind::Pin, 1.0, 20);
    let c = drag(JointKind::Pin, 1.0, 40);
    let gap =
        |x: &[f32], y: &[f32]| -> f32 { (0..4).map(|i| (x[i] - y[i]).abs()).fold(0.0, f32::max) };
    let (g1, g2) = (gap(&a, &b), gap(&b, &c));
    assert!(
        g2 < g1,
        "dobrar a amostragem tem de mexer MENOS: 10->20 mexeu {g1:.4}, 20->40 mexeu {g2:.4}"
    );
    assert!(g2 < 0.05, "e o resto tem de ser pequeno: {g2:.4}");
}

/// **CONTROLE: pegar a CAUDA continua sendo a IK de sempre.** O modo é
/// escolhido por *quem foi pego*, e roubá-lo do caso da ponta apagaria o gesto
/// que a wave anterior construiu.
///
/// ⚠️ **O oráculo é a LEI, não um limite que eu escolhi.** A primeira versão
/// deste gate afirmava *"a cabeça anda menos de 1 m"* e falhou sobre produto
/// correto (ela andou **1,135** — uma raiz livre tem três graus de liberdade e
/// o solver pode transladar o conjunto). O que de fato separa os dois modos é
/// muito mais simples: **quem a mão pegou é quem mais se move**, e nos dois ela
/// vale — o que muda é de que ponta a cadeia decai.
#[test]
fn whoever_the_hand_grabbed_is_the_one_that_moves_most() {
    // A CAUDA pega: a IK clássica dobra a cadeia atrás dela.
    let (mut sim, mut b, e) = chain(JointKind::Pin);
    let before: Vec<_> = (0..4).map(|i| pose(&sim, e[i])).collect();
    assert!(b.ik_begin(e[3]), "pegar a cauda abre a IK classica");
    for k in 1..=20i16 {
        let t = [3.0, 1.5 * f32::from(k) / 20.0];
        let poses = b.ik_move(t, 0.0, IkOptions::default());
        write(&mut sim, &poses);
        b.dispatch(&mut sim, false, 0);
    }
    let moved: Vec<f32> = (0..4)
        .map(|i| {
            let (a, c) = (before[i], pose(&sim, e[i]));
            (c[0] - a[0]).hypot(c[1] - a[1])
        })
        .collect();
    assert!(
        moved[3] > moved[0],
        "com a CAUDA na mao, ela e' que tem de andar mais: cabeca {:.3} contra cauda {:.3}",
        moved[0],
        moved[3]
    );

    // E a metade oposta, com a mesma régua: a CABEÇA pega inverte a ordem.
    let d = drag(JointKind::Pin, 1.5, 20);
    assert!(
        d[0] > d[3],
        "com a CABECA na mao, e' ela: cabeca {:.3} contra cauda {:.3}",
        d[0],
        d[3]
    );
}

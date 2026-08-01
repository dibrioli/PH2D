//! **A solda que CEDE, atravessando a PONTE** (W-SoftWeld).
//!
//! O wrapper já prova a física (`ph2d-physics/tests/joint_soft_weld.rs`: verga,
//! não se abre, assenta). O que estes gates provam é o que só existe deste lado:
//! que o flag AUTORADO chega ao solver, que ele é **gateado pelo TIPO**, e que um
//! rewind o RE-ARMA — o `rebuild_from_rest` reconstrói o mundo do descriptor, e
//! um param que não viaja nele desaparece no primeiro scrub.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

/// Uma parede estática e um braço de 1 × 0,2 m soldado à ponta ESQUERDA dele —
/// a viga em balanço do wrapper, montada em componentes.
fn cantilever(kind: JointKind, soft: bool) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Wall"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.1,
                half_y: 0.3,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Arm"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.5, 5.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Joint"),
        PhysicsJoint {
            body_a: stable_name_id("Wall"),
            body_b: stable_name_id("Arm"),
            kind,
            soft,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    sim
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

/// Quanto o braço pendeu, em graus, depois de `ticks` ticks de simulação.
fn droop_after(sim: &mut SimWorld, bridge: &mut PhysicsBridge, ticks: u64) -> f32 {
    for t in 0..=ticks {
        bridge.dispatch(sim, true, t);
    }
    let e = named(sim, "Arm");
    -sim.world()
        .get::<Transform>(e)
        .expect("transform")
        .rotation
        .to_degrees()
}

/// **O flag autorado chega ao solver.** Mutação: fazer o `joint_desc` sempre
/// mandar `soft: false` deixa o braço mole em 0,00° e este gate vermelho.
#[test]
fn the_bridge_hands_the_soft_flag_to_the_solver() {
    let mut rigid_sim = cantilever(JointKind::Weld, false);
    let mut soft_sim = cantilever(JointKind::Weld, true);
    let rigid = droop_after(&mut rigid_sim, &mut PhysicsBridge::new(), 360);
    let soft = droop_after(&mut soft_sim, &mut PhysicsBridge::new(), 360);

    assert!(
        rigid.abs() < 0.05,
        "o controle se moveu: a solda rígida pendeu {rigid:.3}°"
    );
    assert!(
        soft > 2.0,
        "o flag não chegou ao solver: a solda mole pendeu {soft:.3}°"
    );
}

/// **Um rewind RE-ARMA a solda mole.** O `rebuild_from_rest` reconstrói o mundo a
/// partir do `BodyDesc`/`JointDesc` guardados; um param que não viaja neles
/// existe no play e some no primeiro scrub para trás — o modo de falha que o W8,
/// o W9 e o W-CCD cada um gateou por conta própria.
#[test]
fn a_rewind_re_arms_the_soft_weld() {
    let mut sim = cantilever(JointKind::Weld, true);
    let mut bridge = PhysicsBridge::new();

    let live = droop_after(&mut sim, &mut bridge, 360);
    assert!(live > 2.0, "a cena não cedeu nem ao vivo: {live:.3}°");

    // Scrub para trás até o zero e re-simula pelo MESMO caminho.
    bridge.dispatch(&mut sim, true, 0);
    let replayed = droop_after(&mut sim, &mut bridge, 360);

    assert!(
        (replayed - live).abs() < 0.5,
        "o replay divergiu: {replayed:.3}° contra {live:.3}° ao vivo — o `soft` \
         não sobreviveu ao rebuild"
    );
}

/// **O `soft` de um tipo que não pode ser mole é INERTE**, e é o `can_be_soft` da
/// ponte que o torna inerte. Sem essa pergunta um `soft` deixado para trás por
/// uma troca de tipo seguiria em vigor em silêncio — a mesma razão pela qual o
/// `limits` pergunta `has_limits` na linha de cima.
#[test]
fn a_soft_flag_on_a_kind_that_cannot_be_soft_changes_nothing() {
    let mut plain = cantilever(JointKind::Pin, false);
    let mut flagged = cantilever(JointKind::Pin, true);
    let a = droop_after(&mut plain, &mut PhysicsBridge::new(), 120);
    let b = droop_after(&mut flagged, &mut PhysicsBridge::new(), 120);

    assert!(
        (a - b).abs() < 1e-4,
        "o `soft` mudou um Pin: {a:.6}° contra {b:.6}°"
    );
}

/// **A resposta é do TIPO, exaustiva.** Um oitavo tipo não compila até dizer a
/// sua; o que este gate acrescenta é que a resposta de hoje é *só o Weld* —
/// mutá-la para `true` num vizinho deixa este vermelho.
#[test]
fn only_a_weld_can_be_soft() {
    for kind in JointKind::ALL {
        assert_eq!(
            kind.can_be_soft(),
            kind == JointKind::Weld,
            "{kind:?}: só uma solda segura a ORIENTAÇÃO dos dois corpos, logo só \
             ela tem uma orientação a amolecer"
        );
    }
}

/// **Uma solda MOLE pode partir sob torção, e uma RÍGIDA não** — rapier publica a
/// reação de um eixo motorizado e nada de um travado, e o `soft` é o que troca um
/// pelo outro (medido: 0,9619 N·m contra 0,0000).
///
/// ⚠️ **A metade RÍGIDA é o controle e ela não é decorativa:** sem ela o gate
/// ficaria verde sobre um `breaks_on_torque` cravado em `true`, que pintaria na
/// caixa de Break de toda solda um limiar que nunca dispara — o *controle em nome
/// apenas* que a porta existe para impedir.
#[test]
fn only_a_soft_weld_can_break_on_torque() {
    let weld = |soft: bool| PhysicsJoint {
        kind: JointKind::Weld,
        soft,
        ..PhysicsJoint::default()
    };
    assert!(
        !weld(false).breaks_on_torque(),
        "uma solda RÍGIDA travou o eixo angular: rapier não publica reação dele"
    );
    assert!(
        weld(true).breaks_on_torque(),
        "uma solda MOLE motoriza o eixo angular — negar a row deixaria a torção \
         arrancá-la sem que exista o número que a segura"
    );
    // E nada mais mudou de resposta.
    for kind in JointKind::ALL {
        let j = PhysicsJoint {
            kind,
            soft: true,
            ..PhysicsJoint::default()
        };
        assert_eq!(
            j.breaks_on_torque(),
            kind.breaks_on_torque() || kind == JointKind::Weld,
            "{kind:?}: o `soft` mudou a resposta de um tipo que não pode ser mole"
        );
    }
}

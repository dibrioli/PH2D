//! **A metade do PAR do seam da §12** (W-J8) — o Active, o Collide, o Swap e o
//! nome com que um joint nasce.
//!
//! Irmão de `inspector_joint_tests`, separado dele pelo cap de 600 LOC da shell e
//! cortado pela MESMA linha que a §12 desenha na tela: aqui *entre quais dois
//! isto está, e como eles se tratam*; lá *o que a restrição faz*.

use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::JointFieldEdit;
use ph2d_physics_ecs::{JointKind, PhysicsJoint};

use super::inspector_joint::{build_joint_info, create_joint, joint_with_edit};

/// **O Swap chega ao componente pelo FUNIL que todo edit usa.**
///
/// A aritmética inteira mora em `PhysicsJoint::swapped` e tem gates próprios no
/// crate; o que este afirma é o outro lado — que o botão da §12 leva até lá, e
/// que a rota é a mesma que uma row numérica toma (`joint_with_edit` →
/// `clamped()` → fila do editor).
///
/// Mutação: o braço `Swap` virando um no-op → as duas pontas não se movem,
/// vermelho.
#[test]
fn the_swap_edit_exchanges_the_pair_through_the_same_funnel() {
    let before = PhysicsJoint {
        body_a: 101,
        body_b: 202,
        local_a: [0.5, 0.0],
        local_b: [-0.5, 0.25],
        anchored: true,
        motor_speed: 2.0,
        ..PhysicsJoint::default()
    };
    let after = joint_with_edit(before, JointFieldEdit::Swap).expect("swap is a component write");
    assert_eq!((after.body_a, after.body_b), (202, 101));
    assert_eq!(
        after.local_a,
        [-0.5, 0.25],
        "a ancora viaja com o corpo dela"
    );
    assert_eq!(after.motor_speed, -2.0, "e o motor reverte para compensar");
    assert!(
        after.anchored,
        "e NAO re-semeia: as locais seguem certas, so trocaram de rotulo"
    );
}

/// **Os dois interruptores da §12 vão e voltam sem conversão.**
///
/// Escritos pelo funil e lidos de volta pelo snapshot — o par que impede o
/// controle write-only que a família W-Area inteira teve até `9ec4b43b` (autorar
/// funcionava, re-selecionar lia zero).
#[test]
fn the_pair_switches_round_trip_through_the_snapshot() {
    for (active, collide) in [(false, true), (true, false)] {
        let mut j = PhysicsJoint::default();
        j = joint_with_edit(j, JointFieldEdit::Active(active)).expect("active edit");
        j = joint_with_edit(j, JointFieldEdit::CollideConnected(collide)).expect("collide edit");
        assert_eq!((j.active, j.collide_connected), (active, collide));
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((Name::new("J"), j, Transform::default()))
            .id();
        let info = build_joint_info(&mut sim, e.to_bits(), 0).expect("info");
        assert_eq!(
            (info.active, info.collide_connected),
            (active, collide),
            "o snapshot tem de espelhar o que o componente guarda"
        );
    }
}

/// **Um joint nasce com o nome do que ele une** (W-J8) — "Post : Plank" e não
/// "Joint (3)".
///
/// Numa cena com uma dúzia deles, uma Hierarquia de "Joint", "Joint (2)",
/// "Joint (3)" obriga a clicar cada linha para descobrir qual é qual; o par de
/// nomes responde de onde ela está. É o idioma do Constraints Graph do Unreal.
///
/// A segunda metade é o que mantém o nome utilizável: dois joints entre o MESMO
/// par ainda recebem nomes distintos, porque a unicidade continua sendo imposta
/// (é ela que a Hierarquia e o log de auditoria dependem).
///
/// Mutação: voltar para `unique_name(sim, "Joint")` → o primeiro assert cai.
#[test]
fn a_new_joint_is_named_after_what_it_joins() {
    let mut sim = SimWorld::new();
    let post = sim
        .world_mut()
        .spawn((Name::new("Post"), Transform::default()))
        .id();
    let plank = sim
        .world_mut()
        .spawn((Name::new("Plank"), Transform::default()))
        .id();
    let first =
        create_joint(&mut sim, post.to_bits(), plank.to_bits(), JointKind::Pin).expect("joint");
    assert_eq!(
        sim.world()
            .get::<Name>(first)
            .map(|n| n.as_str().to_string()),
        Some("Post : Plank".to_string())
    );
    let second =
        create_joint(&mut sim, post.to_bits(), plank.to_bits(), JointKind::Pin).expect("joint");
    let second_name = sim
        .world()
        .get::<Name>(second)
        .map(|n| n.as_str().to_string())
        .unwrap();
    assert_ne!(
        second_name, "Post : Plank",
        "o segundo joint entre o mesmo par ainda precisa de nome proprio"
    );
    assert!(
        second_name.starts_with("Post : Plank"),
        "e continua dizendo o que ele une: {second_name}"
    );
}

//! A OUTRA MÃO num facto de vários campos (a junta): só o que ela mudou passa a documento.

use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsJoint;

use crate::{Driven, Driver, PreviewDrive};

fn rig(stiffness: f32, anchored: bool) -> PhysicsJoint {
    PhysicsJoint {
        stiffness,
        anchored,
        ..PhysicsJoint::default()
    }
}

fn autorado(pd: &PreviewDrive, e: ph2d_ecs::Entity) -> PhysicsJoint {
    match pd.authored(e.to_bits(), Driver::JointParams) {
        Some(Driven::JointParams(j)) => j,
        o => panic!("sem junta autorada: {o:?}"),
    }
}

/// A curva conduz a rigidez; a ponte semeia as âncoras ⇒ o documento fica 30 e ancorado.
/// Se a mão mexe na MESMA rigidez, o documento passa a ser a dela.
#[test]
fn a_outra_mao_so_leva_os_campos_que_mudou() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn_empty().id();
    let mut pd = PreviewDrive::default();
    let j = |s, a| Driven::JointParams(rig(s, a));
    pd.driven(e, j(30.0, false), j(5.0, false));
    pd.driven(e, j(5.0, true), j(300.0, true));
    assert_eq!(
        autorado(&pd, e),
        rig(30.0, true),
        "a semente entra, a curva não"
    );
    pd.driven(e, j(42.0, true), j(5.0, true));
    assert_eq!(
        autorado(&pd, e),
        rig(42.0, true),
        "a rigidez da mão é documento"
    );
}

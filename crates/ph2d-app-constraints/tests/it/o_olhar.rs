//! A IK de UM osso que chega de fora (ficheiro, colagem, o `Chain = 1` do painel) é um Look At no
//! quadro seguinte — pela porta do quadro, não por uma chamada à mão (decisão do dono 25/10).

use ph2d_constraints::{Ik, Kind, LookAt};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_skeleton_ecs::Bone;
use ph2d_vec_scene::VecScene;

#[test]
fn the_frame_turns_a_one_bone_ik_into_a_look_at() {
    let mut sim = SimWorld::new();
    let osso = sim
        .world_mut()
        .spawn((StableId(1), Transform::IDENTITY, Bone::default()))
        .id();
    let objecto = sim
        .world_mut()
        .spawn((StableId(2), Transform::IDENTITY))
        .id();
    let mut alvo = Transform::IDENTITY;
    alvo.translation.y = 3.0;
    sim.world_mut().spawn((StableId(10), alvo));
    let um = Kind::Ik(Ik {
        chain: 1,
        offset: 0.25,
        ..Ik::default()
    });
    for e in [osso, objecto] {
        let mut c = Constraints::default();
        c.push(um, Some(StableId(10)));
        sim.world_mut().entity_mut(e).insert(c);
    }
    let mut preview = PreviewDrive::default();
    ph2d_app_constraints::quadro(&mut sim, &mut VecScene::new(), &mut preview, None);
    let kind = |e| sim.world().get::<Constraints>(e).unwrap().items[0].kind;
    assert_eq!(kind(osso), Kind::LookAt(LookAt { offset: 0.25 }));
    assert_eq!(
        kind(objecto),
        um,
        "num objecto a IK é «não é osso» e fica como está"
    );
    let r = sim.world().get::<Transform>(osso).unwrap().rotation;
    assert!(
        (f64::from(r) - (std::f64::consts::FRAC_PI_2 + 0.25)).abs() < 1e-6,
        "{r}"
    );
}

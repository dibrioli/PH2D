//! Follow Path sobre uma forma DESENHADA (W2): a família lê a poligonal da cena vectorial pela porta do
//! `motion.path` e a fase leva-a ao mundo com o afim da forma de agora — mover a forma move o dono.

use ph2d_constraints::{FollowPath, Kind, PathBy};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::{SimWorld, StableId, Transform, VecPathRef, world_transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_vec_scene::{VecPath, VecScene, VecVertex, VertexKind};

fn quina(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
        id: ph2d_vec_scene::VertId::NOVO,
    }
}

#[track_caller]
fn perto(t: Transform, x: f32, y: f32, rot_deg: f32) {
    assert!(
        (t.translation.x - x).abs() < 1e-4
            && (t.translation.y - y).abs() < 1e-4
            && ph2d_constraints::wrap_pi(f64::from(t.rotation - rot_deg.to_radians())).abs() < 1e-5,
        "{t:?} ≠ ({x}, {y}, {rot_deg}°)"
    );
}

#[test]
fn follow_path_walks_the_drawn_shape_where_it_is() {
    let mut sim = SimWorld::new();
    let mut cena = VecScene::new();
    // O «L» do oráculo (`follow_path.L.*`): (0,0) → (4,0) → (4,3), comprimento 7.
    let id = cena.push_path(VecPath {
        verts: vec![quina(0.0, 0.0), quina(4.0, 0.0), quina(4.0, 3.0)],
        closed: false,
        ..VecPath::default()
    });
    let mut pose = Transform::IDENTITY;
    pose.translation.x = 10.0;
    pose.translation.y = 5.0;
    pose.rotation = 90f32.to_radians();
    let forma = sim
        .world_mut()
        .spawn((StableId(1), pose, VecPathRef(id)))
        .id();
    let mut c = Constraints::default();
    let item = c.push(
        Kind::FollowPath(FollowPath {
            by: PathBy::Fraction,
            value: 0.25,
            ..FollowPath::default()
        }),
        Some(StableId(1)),
    );
    let dono = sim
        .world_mut()
        .spawn((StableId(2), Transform::IDENTITY, c))
        .id();
    let mut drive = PreviewDrive::default();
    ph2d_app_constraints::quadro(&mut sim, &cena, &mut drive, None);
    drive.settle();
    // Local (1,75; 0) a 0° ⇒ rodado 90° e levado a (10, 5).
    perto(
        world_transform(sim.world(), dono).unwrap(),
        10.0,
        6.75,
        90.0,
    );
    // Por distância, no segundo trecho: local (4; 1,25) a 90°.
    if let Some(mut c) = sim.world_mut().get_mut::<Constraints>(dono)
        && let Some(it) = c.item_mut(item)
    {
        it.kind = Kind::FollowPath(FollowPath {
            by: PathBy::Distance,
            value: 5.25,
            ..FollowPath::default()
        });
    }
    ph2d_app_constraints::quadro(&mut sim, &cena, &mut drive, None);
    drive.settle();
    perto(
        world_transform(sim.world(), dono).unwrap(),
        8.75,
        9.0,
        180.0,
    );
    // Mover a forma move o dono no mesmo quadro.
    sim.world_mut()
        .get_mut::<Transform>(forma)
        .unwrap()
        .translation
        .x = 0.0;
    ph2d_app_constraints::quadro(&mut sim, &cena, &mut drive, None);
    perto(
        world_transform(sim.world(), dono).unwrap(),
        -1.25,
        9.0,
        180.0,
    );
}

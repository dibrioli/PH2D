//! **O gate das ALÇAS de um pino de mundo** — o desenho é da crate da família e as
//! alças são do gizmo DESTA shell (`render_loop::point_gizmo`).
//!
//! ⚠️ Os outros oito gates do mesmo assunto ficaram **com o sujeito**, na
//! [`ph2d_app_physics::overlay::joints`]: *o corte é por quem o teste EXERCITA.*

use ph2d_app_physics::overlay::joints::*;
use ph2d_physics_ecs::{JointKind, JointView};
use ph2d_vector::{BezPath, PathEl};

/// **O pino de mundo é OFERECIDO com DUAS alças** (W-WorldPinLocal).
///
/// A metade de shell da wave: `joint_anchor_handles` pula um lado cujo
/// `joint_anchor_world` diz `None`, então antes desta wave um pino de mundo saía
/// da porta com **uma** alça — e o número que a outra autoraria (`local_a`, *onde
/// no corpo isto prende*) não tinha porta nenhuma, nem alça nem row numérica.
///
/// ⚠️ **As duas nascem no MESMO ponto, e isso é correto:** um pino satisfeito tem
/// prego e ponto-de-corpo no mesmo lugar. O que as mantém sendo duas é a infra de
/// par coincidente — *A pega o quadrado interno e B a banda de fora*.
#[test]
fn a_world_pin_is_offered_two_handles_that_start_on_the_same_point() {
    use ph2d_core::Vec2;
    use ph2d_ecs::{Name, SimWorld, Transform};
    use ph2d_physics_ecs::{
        BodyKind, Collider, ColliderShape, JointWorldAnchor, PhysicsBridge, PhysicsJoint, RigidBody,
    };
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Plank"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.0,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(1.0, 2.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Pin"),
        PhysicsJoint {
            body_a: ph2d_ecs::stable_name_id("Plank"),
            ..PhysicsJoint::default()
        },
        JointWorldAnchor,
        Transform::from_translation(Vec2::new(0.0, 2.0)),
    ));
    let mut bridge = PhysicsBridge::new();
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    bridge.dispatch(&mut sim, false, 0);

    let handles = crate::render_loop::point_gizmo::joint_anchor_handles(&sim, &bridge, true);
    assert_eq!(
        handles.len(),
        2,
        "o pino de mundo saiu da porta com {} alça(s)",
        handles.len()
    );
    let kinds: Vec<_> = handles.iter().map(|h| h.kind).collect();
    assert!(
        kinds.contains(&ph2d_editor::gizmo::PointHandleKind::AnchorA)
            && kinds.contains(&ph2d_editor::gizmo::PointHandleKind::AnchorB),
        "faltou um dos dois lados: {kinds:?}"
    );
    let (p, q) = (handles[0].world, handles[1].world);
    assert!(
        (p[0] - q[0]).abs() < 1e-4 && (p[1] - q[1]).abs() < 1e-4,
        "um pino satisfeito tem as duas no mesmo ponto: {p:?} contra {q:?}"
    );
}

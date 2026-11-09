//! The smoke/photo of «Convert to Collision Outline» (`PH2D_PHYSICS_SMOKE=120 PH2D_PHYSICS_SMOKE_VETOR=1`):
//! a VECTOR «Frame» (a filled square with a square hole) under the static body «Frame Body», right of
//! the scene, with a «Drop Ball» above it. Right-click the «Frame» row → the item; after it, Space: the
//! ball lands on the top bar. `=2` starts already converted (the photo of the result).

use ph2d_ecs::{ChildOf, Entity, Name, Transform, VecPathRef, World};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_vec_scene::{Contour, FillRule, Paint, Rgba8, VecPathId, VecScene, rectangle};

/// Where the frame stands and the view that shows it.
pub const FRAME_AT: [f32; 2] = [11.0, 2.0];
pub const VIEW_HEIGHT: f32 = 10.0;

/// What [`seed`] made: the path and its entity (the shell registers the pair), the view centre.
pub struct Seeded {
    /// `None` once converted (`=2`: the photo of the result).
    pub path: Option<VecPathId>,
    pub drawing: Entity,
    pub centre: [f32; 2],
}

/// `Some` when `PH2D_PHYSICS_SMOKE_VETOR` is set: the frame drawing, its body and the ball.
pub fn seed(world: &mut World, doc: &mut VecScene) -> Option<Seeded> {
    let mode = std::env::var("PH2D_PHYSICS_SMOKE_VETOR").ok()?;
    let ink = Rgba8 {
        r: 70,
        g: 120,
        b: 200,
        a: 255,
    };
    let path = doc.push_path(ph2d_vec_scene::VecPath {
        fill: Some(Paint::solid(ink)),
        subpaths: vec![Contour {
            verts: rectangle([-0.8, -0.8], [0.8, 0.8]).verts,
            closed: true,
        }],
        fill_rule: FillRule::EvenOdd,
        ..rectangle([-1.5, -1.5], [1.5, 1.5])
    });
    let at = |p: [f32; 2]| Transform::from_translation(ph2d_core::Vec2::new(p[0], p[1]));
    let body = world
        .spawn((
            Name::new("Frame Body"),
            at(FRAME_AT),
            RigidBody {
                kind: BodyKind::Static,
            },
        ))
        .id();
    let drawing = world
        .spawn((
            Name::new("Frame"),
            Transform::default(),
            VecPathRef(path),
            ChildOf(body),
        ))
        .id();
    world.spawn((
        Name::new("Drop Ball"),
        at([FRAME_AT[0] - 0.9, FRAME_AT[1] + 3.0]),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.4 },
            density: 1.0,
            ..Collider::default()
        },
    ));
    ph2d_ecs::assign_missing_root_order(world);
    ph2d_ecs::assign_missing_stable_ids(world);
    // `=2`: already converted, by the same door as the menu (the photo of the result).
    if mode.trim() == "2"
        && let Ok(shape) = crate::convert(world, doc, drawing)
    {
        return Some(Seeded {
            path: None,
            drawing: shape,
            centre: [FRAME_AT[0], FRAME_AT[1]],
        });
    }
    Some(Seeded {
        path: Some(path),
        drawing,
        centre: [FRAME_AT[0], FRAME_AT[1]],
    })
}

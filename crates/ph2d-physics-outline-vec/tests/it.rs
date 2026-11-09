//! «Convert to Collision Outline»: a real vector drawing (a frame with a hole) under a body becomes
//! the body's «Collision Shape» in the drawing's place; a ball rests on the frame and one falls through
//! the hole. An open stroke is a platform. Refusals change nothing.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, VecPathRef, ordered_children};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderOutline, ColliderShape, PhysicsBridge, RigidBody,
};
use ph2d_physics_outline_vec::{Refusal, convert};
use ph2d_vec_scene::{Contour, FillRule, Paint, Rgba8, StrokeSpec, VecPath, VecScene, rectangle};

const INK: Rgba8 = Rgba8 {
    r: 20,
    g: 20,
    b: 20,
    a: 255,
};

/// A 4×4 square with a 2×2 hole (even-odd), filled.
fn frame() -> VecPath {
    let hole = rectangle([-1.0, -1.0], [1.0, 1.0]);
    VecPath {
        fill: Some(Paint::solid(INK)),
        subpaths: vec![Contour {
            verts: hole.verts,
            closed: true,
        }],
        fill_rule: FillRule::EvenOdd,
        ..rectangle([-2.0, -2.0], [2.0, 2.0])
    }
}

/// Static body at the origin with three children: `a`, the drawing (moved +0,5 m in x), `b`.
fn scene(path: VecPath) -> (SimWorld, VecScene, Entity, [Entity; 3]) {
    let mut sim = SimWorld::new();
    let mut doc = VecScene::new();
    let id = doc.push_path(path);
    let w = sim.world_mut();
    let body = w
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Transform::default(),
        ))
        .id();
    let a = w.spawn((Name::new("a"), Transform::default(), ChildOf(body))).id();
    let v = w
        .spawn((
            Name::new("Drawing"),
            Transform::from_translation(Vec2::new(0.5, 0.0)),
            VecPathRef(id),
            ChildOf(body),
        ))
        .id();
    let b = w.spawn((Name::new("b"), Transform::default(), ChildOf(body))).id();
    ph2d_ecs::set_sibling_order(w, body, &[a, v, b]);
    (sim, doc, body, [a, v, b])
}

fn ball(sim: &mut SimWorld, x: f32, y: f32) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ))
        .id()
}

fn settle(sim: &mut SimWorld, balls: &[Entity]) -> Vec<[f32; 2]> {
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=240 {
        bridge.dispatch(sim, true, tick);
    }
    balls
        .iter()
        .map(|&e| {
            let p = bridge.body_pose(e).expect("body");
            [p.0, p.1]
        })
        .collect()
}

/// *Mutations: the hole dropped (fill rule ignored) ⇒ the inner ball is pushed out; the offset not set ⇒ the
/// frame sits at the origin and the side ball misses it; the slot not kept ⇒ order `[a, b, shape]`.*
#[test]
fn o_desenho_com_buraco_vira_a_forma_do_corpo_no_lugar_dele() {
    let (mut sim, doc, body, [a, v, b]) = scene(frame());
    let shape = convert(sim.world_mut(), &doc, v).expect("converts");
    let w = sim.world();
    assert!(w.get_entity(v).is_err(), "the drawing is replaced");
    assert_eq!(w.get::<Name>(shape).map(Name::as_str), Some("Collision Shape"));
    assert_eq!(w.get::<ChildOf>(shape).map(|c| c.0), Some(body));
    assert_eq!(ordered_children(w, body), vec![a, shape, b], "same slot");
    assert_eq!(
        w.get::<Transform>(shape).map(|t| t.translation),
        Some(Vec2::new(0.5, 0.0))
    );
    let o = w.get::<ColliderOutline>(shape).expect("outline");
    assert_eq!(o.rings.len(), 2, "outer ring + hole");
    assert!(o.custom, "no picture to follow");
    // Frame in world: x ∈ [−1,5; 2,5], top y = 2; the hole x ∈ [−0,5; 1,5], y ∈ [−1; 1] (a pocket in 2D).
    let balls = [ball(&mut sim, -1.0, 4.0), ball(&mut sim, 0.5, 0.5)];
    let at = settle(&mut sim, &balls);
    assert!(
        (at[0][1] - 2.3).abs() < 0.08,
        "the side ball rests on the frame at y ≈ 2,3, got {:?}",
        at[0]
    );
    assert!(
        (at[1][1] + 0.7).abs() < 0.08,
        "the ball dropped INSIDE the hole falls to its floor at y ≈ −0,7, got {:?}",
        at[1]
    );
}

/// An open line with a stroke is a platform (its stroke band); without a stroke it paints nothing and
/// is refused, as is a non-vector object — and a refusal changes nothing.
#[test]
fn o_traco_aberto_e_plataforma_e_as_recusas_nao_mexem() {
    let line = VecPath {
        verts: rectangle([-2.0, 0.0], [2.0, 0.0]).verts[..2].to_vec(),
        closed: false,
        stroke: Some(StrokeSpec::new(INK, 0.4)),
        ..VecPath::default()
    };
    let (mut sim, doc, _, [_, v, _]) = scene(line.clone());
    let shape = convert(sim.world_mut(), &doc, v).expect("a stroked line converts");
    let one = ball(&mut sim, 0.5, 3.0);
    let at = settle(&mut sim, &[one]);
    assert!(
        (at[0][1] - 0.5).abs() < 0.08,
        "rests on the 0,4-wide line at y ≈ 0,5, got {:?}",
        at[0]
    );
    assert!(sim.world().get::<ColliderOutline>(shape).is_some());

    let bare = VecPath {
        stroke: None,
        ..line
    };
    let (mut sim, doc, body, kids) = scene(bare);
    assert_eq!(convert(sim.world_mut(), &doc, kids[1]), Err(Refusal::NoArea));
    assert_eq!(convert(sim.world_mut(), &doc, kids[0]), Err(Refusal::NotAVector));
    assert_eq!(ordered_children(sim.world(), body), kids.to_vec(), "nothing moved");
}


//! «Convert to Collision Outline»: a real vector drawing (a frame with a hole) under a body becomes
//! the body's «Collision Shape» in the drawing's place; a ball rests on the frame and one falls through
//! the hole. An open stroke is a platform. Refusals change nothing.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, VecPathRef, ordered_children};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderOutline, ColliderShape, PhysicsBridge, RigidBody,
};
use ph2d_physics_outline_vec::{Refusal, convert};
use ph2d_vec_entities::entities::VecEntityMap;
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
    let a = w
        .spawn((Name::new("a"), Transform::default(), ChildOf(body)))
        .id();
    let v = w
        .spawn((
            Name::new("Drawing"),
            Transform::from_translation(Vec2::new(0.5, 0.0)),
            VecPathRef(id),
            ChildOf(body),
        ))
        .id();
    let b = w
        .spawn((Name::new("b"), Transform::default(), ChildOf(body)))
        .id();
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
    let (mut sim, mut doc, body, [a, v, b]) = scene(frame());
    let shape = convert(sim.world_mut(), &mut doc, v).expect("converts");
    assert!(
        doc.is_empty(),
        "the path leaves the document with the drawing"
    );
    let w = sim.world();
    assert!(w.get_entity(v).is_err(), "the drawing is replaced");
    assert_eq!(
        w.get::<Name>(shape).map(Name::as_str),
        Some("Collision Shape")
    );
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
    let (mut sim, mut doc, _, [_, v, _]) = scene(line.clone());
    let shape = convert(sim.world_mut(), &mut doc, v).expect("a stroked line converts");
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
    let (mut sim, mut doc, body, kids) = scene(bare);
    assert_eq!(
        convert(sim.world_mut(), &mut doc, kids[1]),
        Err(Refusal::NoArea)
    );
    assert_eq!(
        convert(sim.world_mut(), &mut doc, kids[0]),
        Err(Refusal::NotAVector)
    );
    assert_eq!(
        ordered_children(sim.world(), body),
        kids.to_vec(),
        "nothing moved"
    );
}

/// The project state the editor's undo stores (the shell's `ProjectState::capture`/`restore`, minus the
/// parts this law never touches): the incremental world capture plus the vector document.
type State = (ph2d_ecs::scene::WorldSnapshot, VecScene);
fn capture(sim: &mut SimWorld, doc: &VecScene, reg: &ph2d_ecs::scene::ComponentRegistry) -> State {
    let mut w = ph2d_ecs::scene::WorldSnapshot::new();
    let mut cache = ph2d_ecs::scene::incremental::CaptureCache::new();
    ph2d_ecs::scene::incremental::capture_incremental(sim.world_mut(), &mut cache, reg, &mut w)
        .expect("capture");
    (w, doc.clone())
}
fn restore(
    s: &State,
    sim: &mut SimWorld,
    reg: &ph2d_ecs::scene::ComponentRegistry,
) -> VecEntityMap {
    let mut q = sim
        .world_mut()
        .query_filtered::<Entity, ph2d_ecs::With<Transform>>();
    for e in q.iter(sim.world()).collect::<Vec<_>>() {
        let _ = sim.world_mut().despawn(e);
    }
    ph2d_ecs::scene::snapshot_to_world(sim.world_mut(), &s.0, reg).expect("restore");
    ph2d_vec_entities::entities::rebuild_map(sim)
}
fn by_name(sim: &mut SimWorld, n: &str) -> Option<Entity> {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, x)| x.as_str() == n)
        .map(|(e, _)| e)
}

/// Undo through the editor's per-frame diff (moved from the shell, `undo_convert_outline_tests.rs`): the
/// click frame (`convert` + the frame-end vector `sync`) is ONE step; undo gives back the drawing — name,
/// parent, slot, `Transform` and the SAME vector document — and drops the shape; redo converts again.
/// *Mutation: no map rebuild (`rebuild_map` empty) ⇒ the `sync` spawns a twin (red, 09/11). Dropping
/// `convert`'s own `remove_path` stays green: the same frame's `sync` removes the orphan path.*
#[test]
fn converter_o_desenho_e_um_passo_e_o_ctrl_z_devolve_o_vetor() {
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let (mut sim, mut doc, body, [_, v, _]) = scene(frame());
    sim.world_mut().entity_mut(body).insert(Name::new("Body"));
    let (id, pose) = (
        sim.world().get::<VecPathRef>(v).expect("ref").0,
        Transform::from_translation(Vec2::new(0.5, 0.0)),
    );
    let mut map = VecEntityMap::default();
    map.insert(id, v.to_bits());
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_vec_entities::entities::sync(&mut sim, &mut doc, &mut map);
    let (doc_before, before) = (doc.clone(), capture(&mut sim, &doc, &reg));
    let (mut base, mut steps) = (before.clone(), Vec::new());
    let shape = convert(sim.world_mut(), &mut doc, v).expect("converts");
    for _ in 0..2 {
        ph2d_vec_entities::entities::sync(&mut sim, &mut doc, &mut map);
        let now = capture(&mut sim, &doc, &reg);
        if now != base {
            steps.push(std::mem::replace(&mut base, now));
        }
    }
    assert_eq!(steps.len(), 1, "converting must be ONE step");
    assert!(sim.world().get::<ColliderOutline>(shape).is_some());
    let names = |sim: &SimWorld, p| -> Vec<String> {
        let kids = ordered_children(sim.world(), p);
        kids.iter()
            .map(|&c| {
                sim.world()
                    .get::<Name>(c)
                    .map_or_else(String::new, |n| n.as_str().to_owned())
            })
            .collect()
    };
    assert_eq!(names(&sim, body), ["a", "Collision Shape", "b"]);
    let after = base.clone();

    let back = steps.pop().expect("a step");
    assert_eq!(
        back, before,
        "undo gives back the state before, vector document included"
    );
    let mut map = restore(&back, &mut sim, &reg);
    let mut doc = back.1.clone();
    assert_eq!(doc, doc_before, "the SAME vector document");
    let body = by_name(&mut sim, "Body").expect("body");
    let f = by_name(&mut sim, "Drawing").expect("the drawing is back");
    assert!(
        by_name(&mut sim, "Collision Shape").is_none(),
        "the shape is gone"
    );
    assert_eq!(names(&sim, body), ["a", "Drawing", "b"], "the same slot");
    assert_eq!(sim.world().get::<ChildOf>(f).map(|c| c.0), Some(body));
    assert_eq!(sim.world().get::<Transform>(f), Some(&pose));
    let r = sim
        .world()
        .get::<VecPathRef>(f)
        .copied()
        .expect("VecPathRef");
    assert_eq!(doc.path(r.0), doc_before.path(id), "the same path");
    ph2d_vec_entities::entities::sync(&mut sim, &mut doc, &mut map);
    assert_eq!(doc, doc_before, "the sync does not undo the restore");
    let refs = sim
        .world_mut()
        .query::<&VecPathRef>()
        .iter(sim.world())
        .count();
    assert_eq!(
        refs, 1,
        "ONE object for the path (map rebuilt; no twin from the sync)"
    );

    let _ = restore(&after, &mut sim, &reg);
    assert!(after.1.is_empty(), "redo converts again");
    assert!(by_name(&mut sim, "Collision Shape").is_some());
    assert!(by_name(&mut sim, "Drawing").is_none());
}

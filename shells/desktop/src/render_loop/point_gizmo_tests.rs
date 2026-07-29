//! **Which anchors get a handle** — the publish rule, headless.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Locked, Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::Sprite;

fn camera() -> Camera2d {
    Camera2d {
        center: [0.0, 0.0],
        height_world: 10.0,
        ..Camera2d::default()
    }
}

fn window() -> WindowSize {
    WindowSize {
        width: 1000,
        height: 1000,
    }
}

fn body(sim: &mut SimWorld, name: &str, kind: BodyKind, at: [f32; 2]) {
    sim.world_mut().spawn((
        Name::new(name.to_string()),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.3,
                half_y: 0.3,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(at[0], at[1])),
    ));
}

fn joint(sim: &mut SimWorld, name: &str, a: &str, b: &str, at: [f32; 2]) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(name.to_string()),
            PhysicsJoint {
                body_a: stable_name_id(a),
                body_b: stable_name_id(b),
                kind: JointKind::Rope,
                max_length: 2.0,
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(at[0], at[1])),
        ))
        .id()
}

/// One post, one arm, one rope between them — plus a plain sprite that is not a
/// joint and must never grow a handle.
fn rig() -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    body(&mut sim, "Post", BodyKind::Static, [0.0, 6.0]);
    body(&mut sim, "Arm", BodyKind::Dynamic, [1.0, 5.0]);
    sim.world_mut().spawn((
        Name::new("JustASprite".to_string()),
        Transform::from_translation(Vec2::new(-3.0, 0.0)),
        Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
    ));
    let j = joint(&mut sim, "Link", "Post", "Arm", [0.0, 6.0]);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge, j)
}

/// **A joint nobody selected still gets its handles** — the whole of W-J2b.
///
/// A joint has no sprite, so a canvas click cannot reach it; when the handles
/// were selection-gated the only way to see them was to find the joint in the
/// Hierarchy first. Nothing here is selected, and both ends are offered.
///
/// Mutation-tested: re-adding a `selection` guard makes this return an empty
/// list and the gate goes red.
#[test]
fn an_unselected_joint_still_publishes_both_handles() {
    let (sim, bridge, j) = rig();
    let handles = joint_anchor_handles(&sim, &bridge, true);
    assert_eq!(
        handles.len(),
        2,
        "the rope's two ends must both be grabbable with nothing selected, got {handles:?}"
    );
    assert!(handles.iter().all(|h| h.key == j.to_bits()));
    assert!(handles.iter().any(|h| h.kind == PointHandleKind::AnchorA));
    assert!(handles.iter().any(|h| h.kind == PointHandleKind::AnchorB));
}

/// **Two joints are four handles, and a plain sprite is none of them.** The
/// list is per-joint, so the count is the fact that says the rule reads the
/// whole scene rather than one entity.
#[test]
fn every_joint_in_the_scene_is_offered_and_a_sprite_is_not() {
    let (mut sim, mut bridge, _) = rig();
    body(&mut sim, "Post2", BodyKind::Static, [4.0, 6.0]);
    body(&mut sim, "Arm2", BodyKind::Dynamic, [5.0, 5.0]);
    let j2 = joint(&mut sim, "Link2", "Post2", "Arm2", [4.0, 6.0]);
    bridge.dispatch(&mut sim, false, 0);

    let handles = joint_anchor_handles(&sim, &bridge, true);
    assert_eq!(handles.len(), 4, "two joints, two ends each: {handles:?}");
    assert_eq!(
        handles.iter().filter(|h| h.key == j2.to_bits()).count(),
        2,
        "the second joint must be offered too — it is the one that proves the rule is not \
         'the selected joint' wearing another name"
    );
}

/// **The order is deterministic.** It decides which of two overlapping joints
/// paints on top, and that must not depend on archetype layout.
#[test]
fn the_handle_order_is_stable() {
    let (sim, bridge, _) = rig();
    let a = joint_anchor_handles(&sim, &bridge, true);
    let b = joint_anchor_handles(&sim, &bridge, true);
    assert_eq!(a, b);
    let mut sorted = a.clone();
    sorted.sort_by_key(|h| (h.key, h.kind));
    assert_eq!(a, sorted, "handles must come out sorted by (entity, side)");
}

/// **No handles while the clock runs.** During play the overlay draws the
/// SOLVER's anchors — the live ones — and a grabbable dot authoring against a
/// swinging pose would write an anchor nobody chose. `sync_joint_pivots` has
/// claimed this in its own doc since W-AnchorFollow; nothing enforced it until
/// W-J2.
///
/// Mutation-tested: dropping the `at_rest` guard returns the two handles.
#[test]
fn the_handles_are_not_offered_while_the_clock_runs() {
    let (sim, bridge, _) = rig();
    assert!(joint_anchor_handles(&sim, &bridge, false).is_empty());
}

/// **A locked joint is not offered a handle.** `open_drag` refuses a locked
/// entity, and a dot that paints, registers a hit and then declines the gesture
/// is worse than one that is not there.
#[test]
fn a_locked_joint_offers_nothing_to_grab() {
    let (mut sim, bridge, j) = rig();
    sim.world_mut().entity_mut(j).insert(Locked);
    assert!(joint_anchor_handles(&sim, &bridge, true).is_empty());
}

/// **A dormant joint keeps its A handle and is refused a B one.** Half-authored
/// (a body renamed, deleted, or never picked) is precisely the joint the artist
/// is in the middle of fixing, and its authored pivot is the `Transform` the
/// seed will convert. B has no body, so no anchor, so no handle.
#[test]
fn a_dormant_joints_a_end_is_offered_and_its_b_end_is_not() {
    let mut sim = SimWorld::new();
    body(&mut sim, "Post", BodyKind::Static, [0.0, 6.0]);
    let j = joint(&mut sim, "Link", "Post", "NoSuchBody", [0.0, 6.0]);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);

    let handles = joint_anchor_handles(&sim, &bridge, true);
    assert_eq!(handles.len(), 1, "only the A end, got {handles:?}");
    assert_eq!(handles[0].kind, PointHandleKind::AnchorA);
    assert_eq!(handles[0].key, j.to_bits());
    assert_eq!(handles[0].world, [0.0, 6.0]);
}

/// **Nothing to grab publishes no view.** An empty list would paint nothing and
/// register nothing anyway; `None` says so at the boundary instead of shipping
/// an empty struct every frame of every scene without joints.
#[test]
fn an_empty_scene_publishes_no_view() {
    assert!(build_point_view(Vec::new(), &camera(), window(), None, false).is_none());
    let v = build_point_view(
        vec![PointHandle {
            key: 1,
            kind: PointHandleKind::AnchorA,
            world: [1.0, 2.0],
        }],
        &camera(),
        window(),
        Some([3.0, 3.0]),
        false,
    )
    .expect("a handle publishes a view");
    assert_eq!(v.handles.len(), 1);
    assert_eq!(v.snap_world, Some([3.0, 3.0]));
    assert!(!v.inert, "nothing armed => the handles are grabbable");
}

/// **A hit id resolves back to the joint and the end it was painted for.** This
/// is the half that makes several joints safe: the ids are keyed, so the map is
/// the only thing that knows them, and it is filled by the same pass that
/// registers them.
#[test]
fn a_hit_id_resolves_to_its_joint_and_side() {
    let (sim, bridge, j) = rig();
    let handles = joint_anchor_handles(&sim, &bridge, true);
    let mut map = std::collections::BTreeMap::new();
    for h in &handles {
        map.insert(ph2d_editor::gizmo::point_handle_id(h.key, h.kind), *h);
    }
    for h in &handles {
        let id = ph2d_editor::gizmo::point_handle_id(h.key, h.kind);
        let (e, kind) = resolve_anchor_hit(&map, id).expect("a painted handle resolves");
        assert_eq!(e, j);
        assert_eq!(kind, h.kind, "the resolved kind must be the one painted");
        assert_eq!(
            anchor_side(kind) == Some(JointSide::A),
            h.kind == PointHandleKind::AnchorA,
            "an anchor kind must map back to its side"
        );
    }
    assert!(resolve_anchor_hit(&map, ph2d_editor::NodeId(7)).is_none());
}

// ── W-J3: the parameter grips ────────────────────────────────────────────────

/// A hinge with limits + a spring, so both grip families are on screen at once.
fn param_rig() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    body(&mut sim, "HingePost", BodyKind::Static, [0.0, 0.0]);
    body(&mut sim, "HingeArm", BodyKind::Dynamic, [1.0, 0.0]);
    sim.world_mut().spawn((
        Name::new("Hinge".to_string()),
        PhysicsJoint {
            body_a: stable_name_id("HingePost"),
            body_b: stable_name_id("HingeArm"),
            kind: JointKind::Pin,
            limits_enabled: true,
            limit_min: -0.5,
            limit_max: 0.9,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    body(&mut sim, "SpringPost", BodyKind::Static, [5.0, 0.0]);
    body(&mut sim, "SpringBob", BodyKind::Dynamic, [6.0, 0.0]);
    sim.world_mut().spawn((
        Name::new("Spring".to_string()),
        PhysicsJoint {
            body_a: stable_name_id("SpringPost"),
            body_b: stable_name_id("SpringBob"),
            kind: JointKind::Spring,
            rest_length: 1.0,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(5.0, 0.0)),
    ));
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge)
}

fn kinds(hs: &[PointHandle]) -> Vec<PointHandleKind> {
    let mut k: Vec<_> = hs.iter().map(|h| h.kind).collect();
    k.sort_unstable();
    k
}

/// **Each joint gets the grips its own kind has**, and no others: a hinge with
/// limits gets two walls, a spring gets its ring. A parameter that a joint does
/// not use is not a grip that quietly authors an unused field.
#[test]
fn a_hinge_gets_two_walls_and_a_spring_gets_its_ring() {
    let (_sim, bridge) = param_rig();
    let hs = joint_param_handles(&bridge, &camera(), window(), true, true);
    assert_eq!(
        kinds(&hs),
        vec![
            PointHandleKind::LimitMin,
            PointHandleKind::LimitMax,
            PointHandleKind::Length
        ],
        "got {hs:?}"
    );
}

/// **A free hinge has no walls to grip.** `limits_enabled` off means the arc is
/// not drawn, so there is nothing there to grab.
#[test]
fn a_hinge_without_limits_has_no_walls() {
    let (mut sim, mut bridge) = param_rig();
    let hinge = sim
        .world_mut()
        .query::<(Entity, &Name)>()
        .iter(sim.world())
        .find(|(_, n)| n.as_str() == "Hinge")
        .map(|(e, _)| e)
        .expect("the hinge");
    if let Some(mut c) = sim.world_mut().get_mut::<PhysicsJoint>(hinge) {
        c.limits_enabled = false;
    }
    bridge.dispatch(&mut sim, false, 0);
    let hs = joint_param_handles(&bridge, &camera(), window(), true, true);
    assert_eq!(kinds(&hs), vec![PointHandleKind::Length], "got {hs:?}");
}

/// **The grips follow the joint overlay's visibility.**
///
/// They are grips on ITS geometry — the arc, the ring. With `B` off there is no
/// arc on screen, and a dot that moves an invisible line is a control the artist
/// cannot reason about. (The ANCHOR dots are drawn by the gizmo itself, so they
/// are not gated: they are always visible.)
///
/// Mutation-tested: dropping the `show_overlay` guard leaves the grips live with
/// nothing drawn under them.
#[test]
fn the_parameter_grips_are_not_offered_with_the_overlay_hidden() {
    let (_sim, bridge) = param_rig();
    assert!(joint_param_handles(&bridge, &camera(), window(), false, true).is_empty());
    // …nor while the clock runs, for the reason the anchors are not.
    assert!(joint_param_handles(&bridge, &camera(), window(), true, false).is_empty());
}

/// **A wall's grip re-projects to the pixel the arc drew it at.** The handle
/// carries WORLD (one projection serves every kind), and the arc lives at a
/// fixed SCREEN radius, so the publish unprojects — a round trip that must land
/// back where it started or the hit rect drifts off the tick as you zoom.
#[test]
fn a_walls_grip_round_trips_through_world_back_to_its_pixel() {
    let (_sim, bridge) = param_rig();
    let (cam, win) = (camera(), window());
    let hs = joint_param_handles(&bridge, &cam, win, true, true);
    for h in hs.iter().filter(|h| h.kind != PointHandleKind::Length) {
        let v = bridge
            .joint_views()
            .find(|v| v.entity.to_bits() == h.key)
            .expect("the joint");
        let l = v.limits.expect("limits");
        let i = usize::from(h.kind == PointHandleKind::LimitMax);
        let drawn = crate::render_loop::physics_overlay_joint_glyphs::limit_end_screen(
            &cam, win, v.anchor_a, v.angle_a, l[i],
        );
        let (sx, sy) = cam.world_to_screen(h.world, win);
        let d = (f64::from(sx) - drawn.x).hypot(f64::from(sy) - drawn.y);
        assert!(
            d < 0.05,
            "the {:?} grip re-projected {d:.4} px away from the wall it grips",
            h.kind
        );
    }
}

/// **A rail's stroke grips sit ON THE RAIL, in metres — not on an arc, in
/// radians.**
///
/// A Slider is limited too (W-J5), and this publisher asked only *"does it have
/// limits?"*: it put a 0.5 **metre** stroke on the hinge arc at 0.5 **radians**,
/// 21 screen px from the anchor and pointing nowhere the rail goes. The drag that
/// opened there then authored a bearing into a field read as metres — Enio's
/// runaway (2026-07-26).
///
/// The oracle is the position, because that is the whole defect: the grip has to
/// be at `anchor + axis · limit`, which is exactly where `slider_rail` already
/// draws its end-of-travel tick. Grip and tick are one place, which is the law
/// `limit_end_screen` states for the hinge.
///
/// Mutation: the publisher dropping the `limits_in_metres` branch — RED, the max
/// grip lands 0.37 m from where the rail's tick is.
#[test]
fn a_rails_stroke_grips_sit_on_the_rail_not_on_an_arc() {
    let mut sim = SimWorld::new();
    body(&mut sim, "Post", BodyKind::Static, [2.0, 3.0]);
    body(&mut sim, "Car", BodyKind::Dynamic, [2.0, 3.0]);
    // A rail at 90°: the axis is +Y, so the two ends are directly above and
    // below the anchor and a grip that stayed on the arc cannot coincide by luck.
    let mut t = Transform::from_translation(Vec2::new(2.0, 3.0));
    t.rotation = std::f32::consts::FRAC_PI_2;
    sim.world_mut().spawn((
        Name::new("Rail".to_string()),
        PhysicsJoint {
            body_a: stable_name_id("Post"),
            body_b: stable_name_id("Car"),
            kind: JointKind::Slider,
            limits_enabled: true,
            limit_min: -0.4,
            limit_max: 0.9,
            ..PhysicsJoint::default()
        },
        t,
    ));
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let hs = joint_param_handles(&bridge, &camera(), window(), true, true);
    assert_eq!(
        kinds(&hs),
        vec![PointHandleKind::LimitMin, PointHandleKind::LimitMax],
        "a rail has a stroke and no ring: got {hs:?}"
    );
    let at = |k: PointHandleKind| hs.iter().find(|h| h.kind == k).expect("grip").world;
    for (k, want) in [
        (PointHandleKind::LimitMin, [2.0f32, 3.0 - 0.4]),
        (PointHandleKind::LimitMax, [2.0, 3.0 + 0.9]),
    ] {
        let p = at(k);
        assert!(
            (p[0] - want[0]).abs() < 1e-3 && (p[1] - want[1]).abs() < 1e-3,
            "{k:?} must sit at anchor + axis*limit = {want:?}, got {p:?}"
        );
    }
}

/// **TODO tipo com comprimento OFERECE o anel** — a metade complementar do gate
/// de escrita em `joint_anchor_drag_tests`.
///
/// As duas metades existem porque cada uma passa sozinha sobre o defeito da
/// outra: um anel publicado com a escrita morta é o que o **Rod** shipou (o
/// grip pegava e o arrasto não fazia nada), e uma escrita viva sem anel é uma
/// capacidade que ninguém alcança. A lista vem dos CHIPS que o painel pinta,
/// então o sétimo tipo nasce coberto.
#[test]
fn every_kind_with_a_length_offers_the_ring_to_grab() {
    for tag in 0..u8::try_from(ph2d_editor::ids::INSP_JOINT_KIND.len()).expect("cabe") {
        let kind = crate::render_loop::inspector_joint::kind_of(tag);
        if kind.length_field().is_none() {
            continue;
        }
        let (mut sim, mut bridge) = param_rig();
        let joint = sim
            .world_mut()
            .query::<(Entity, &Name)>()
            .iter(sim.world())
            .find(|(_, n)| n.as_str() == "Hinge")
            .map(|(e, _)| e)
            .expect("the hinge");
        if let Some(mut c) = sim.world_mut().get_mut::<PhysicsJoint>(joint) {
            c.kind = kind;
            c.limits_enabled = false;
        }
        bridge.dispatch(&mut sim, false, 0);
        let hs = joint_param_handles(&bridge, &camera(), window(), true, true);
        assert!(
            hs.iter().any(|h| h.kind == PointHandleKind::Length),
            "{kind:?} tem comprimento e nao oferece o anel: {hs:?}"
        );
    }
}

/// **Uma polia NÃO oferece anel de comprimento, e as alças dela são da RODA.**
///
/// Duas decisões vistas de um lugar só. O anel é um raio em volta da âncora A, e
/// o comprimento de uma polia é a CORDA — que não é a distância entre as âncoras
/// —, então ele descreveria uma medida que não existe na cena. E as roldanas
/// deixaram de ser alças do joint quando viraram entidades (W-Pulley W1): quem as
/// publica é `wheel_handles`, para a roda SELECIONADA, e é ele que impede uma
/// corda de seis rodas de publicar doze alças sobrepostas.
#[test]
fn a_pulley_offers_no_length_ring_and_its_wheels_are_the_wheels_own_handles() {
    let mut sim = ph2d_ecs::SimWorld::new();
    for (name, x) in [("Load", -2.0_f32), ("Counterweight", 2.0)] {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 2.0)),
        ));
    }
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Load"),
            body_b: stable_name_id("Counterweight"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(-2.0, 2.0)),
    ));
    let wheel = sim
        .world_mut()
        .spawn((
            Name::new("Rope Wheel 1"),
            ph2d_physics_ecs::PulleyWheel {
                rope: stable_name_id("Rope"),
                order: 0,
                radius: 0.4,
                wrap: ph2d_physics_ecs::WrapSide::Auto,
                motor_speed: 0.0,
                break_enabled: false,
                break_force: ph2d_physics_ecs::PulleyWheel::DEFAULT_BREAK_FORCE,
                ..Default::default()
            },
            Transform::from_translation(Vec2::new(-2.0, 4.0)),
        ))
        .id();
    let mut bridge = ph2d_physics_ecs::PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    assert!(
        joint_param_handles(&bridge, &camera(), window(), true, true).is_empty(),
        "uma polia não tem parâmetro agarrável na corda"
    );

    // Sem seleção, nenhuma alça de roda — senão uma corda de seis publicaria
    // doze, e o aro de uma cairia sobre o centro da vizinha.
    assert!(wheel_handles(&sim, None, true, true).is_empty());
    let hs = wheel_handles(&sim, Some(wheel.to_bits()), true, true);
    assert_eq!(
        kinds(&hs),
        vec![PointHandleKind::WheelCentre, PointHandleKind::WheelRim],
        "got {hs:?}"
    );
    // O centro está ONDE a roda está; o aro, a um raio dele.
    assert!((hs[0].world[0] - (-2.0)).abs() < 1e-4 && (hs[0].world[1] - 4.0).abs() < 1e-4);
    let d = (hs[1].world[0] - hs[0].world[0]).hypot(hs[1].world[1] - hs[0].world[1]);
    assert!(
        (d - 0.4).abs() < 1e-4,
        "o aro tem de ficar a um RAIO do centro: {d:.4}"
    );

    // **O SEGUNDO diâmetro** (W6): a roldana comum acima NÃO o oferece — é a
    // metade de AUSÊNCIA, e ela é o gate, não decoração. Duas alças no mesmo
    // pixel seriam uma alça que às vezes faz outra coisa.
    if let Some(mut w) = sim
        .world_mut()
        .get_mut::<ph2d_physics_ecs::PulleyWheel>(wheel)
    {
        w.radius_out = 0.1;
    }
    let hs = wheel_handles(&sim, Some(wheel.to_bits()), true, true);
    assert_eq!(
        kinds(&hs),
        vec![
            PointHandleKind::WheelCentre,
            PointHandleKind::WheelRim,
            PointHandleKind::WheelRimOut
        ],
        "got {hs:?}"
    );
    // ⚠️ Do lado OPOSTO ao aro de entrada: com os dois raios próximos, sair do
    // mesmo lado poria as duas alças a poucos pixels uma da outra.
    assert!(
        hs[2].world[0] < hs[0].world[0] && hs[1].world[0] > hs[0].world[0],
        "os dois aros saem de lados opostos do centro: {hs:?}"
    );
    let d_out = (hs[2].world[0] - hs[0].world[0]).hypot(hs[2].world[1] - hs[0].world[1]);
    assert!(
        (d_out - 0.1).abs() < 1e-4,
        "o aro de saída tem de ficar ao raio de SAÍDA do centro: {d_out:.4}"
    );
}

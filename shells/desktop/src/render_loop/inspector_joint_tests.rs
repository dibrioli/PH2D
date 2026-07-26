//! **The other half of the §12 seam: does the click produce a joint that
//! HOLDS?**
//!
//! `ph2d-panel-inspector/tests/seam_joint.rs` proves panel → bus. These prove
//! bus → ECS → simulation — the half a tool can fail while every gate in its
//! own crate stays green ([[feedback_tool_unit_green_integration_dead]]).

use ph2d_core::Vec2;
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, apply_editor_commands, register_ecs_components,
};
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_editor::JointFieldEdit;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

use super::inspector_joint::{
    apply_joint_edit, build_joint_info, create_joint, joint_with_edit, kind_of, set_joint_body,
};

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    reg
}

/// A hook and a plank, both physical, neither jointed. `named` decides whether
/// they arrive with names — the case that decides whether a joint can refer to
/// them at all.
fn two_bodies(named: bool) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let mut spawn = |x: f32, y: f32, kind: BodyKind, name: &str| {
        let e = sim
            .world_mut()
            .spawn((
                RigidBody { kind },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.25 },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(x, y)),
            ))
            .id();
        if named {
            sim.world_mut()
                .get_entity_mut(e)
                .expect("just spawned")
                .insert(Name::new(name));
        }
        e
    };
    let hook = spawn(0.0, 6.0, BodyKind::Static, "Hook");
    let plank = spawn(0.0, 5.0, BodyKind::Dynamic, "Plank");
    (sim, hook, plank)
}

/// **The gesture produces a joint that actually holds the plank up.**
///
/// The oracle is the simulation, not the components: after Join, playing for
/// two seconds must leave the plank hanging near where it started instead of
/// falling away.
#[test]
fn joining_two_bodies_makes_a_joint_that_holds() {
    let (mut sim, hook, plank) = two_bodies(true);
    create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Pin).expect("join");

    let mut bridge = PhysicsBridge::new();
    for tick in 1..=120 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(
        bridge.joint_count(),
        1,
        "the joint never reached the solver"
    );
    let y = sim
        .world()
        .get::<Transform>(plank)
        .expect("plank")
        .translation
        .y;
    assert!(
        y > 4.0,
        "the plank fell to y={y} — it was joined to a static hook and should \
         still be hanging near y=5"
    );
}

/// **Bodies with no name are named, because a joint refers to them by name.**
///
/// Not a side effect to apologise for: an unnamed body is one a joint cannot
/// point at, and the timeline's bindings have the same requirement.
#[test]
fn joining_unnamed_bodies_names_them_first() {
    let (mut sim, hook, plank) = two_bodies(false);
    assert!(
        sim.world().get::<Name>(hook).is_none(),
        "fixture precondition"
    );

    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Pin).expect("join");

    let a = sim.world().get::<Name>(hook).expect("hook was named");
    let b = sim.world().get::<Name>(plank).expect("plank was named");
    assert_ne!(a.as_str(), b.as_str(), "both bodies got the SAME name");
    let j = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    assert_eq!(j.body_a, stable_name_id(a.as_str()));
    assert_eq!(j.body_b, stable_name_id(b.as_str()));
    assert!(j.names_two_bodies());
}

/// **A body cannot be joined to itself.**
#[test]
fn joining_a_body_to_itself_creates_nothing() {
    let (mut sim, hook, _) = two_bodies(true);
    assert!(create_joint(&mut sim, hook.to_bits(), hook.to_bits(), JointKind::Pin).is_none());
    let mut q = sim.world_mut().query::<&PhysicsJoint>();
    assert_eq!(q.iter(sim.world()).count(), 0);
}

/// **The new joint lands at the midpoint of the two bodies.**
///
/// One rule for every kind — and for a Pin between two touching bodies, which
/// is the chain-link case, the midpoint IS the correct pivot.
#[test]
fn the_new_joint_lands_between_the_two_bodies() {
    let (mut sim, hook, plank) = two_bodies(true);
    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Pin).expect("join");
    let t = sim.world().get::<Transform>(joint).expect("transform");
    assert_eq!(
        t.translation.y, 5.5,
        "hook at 6, plank at 5 -> pivot at 5.5"
    );
}

/// **Degrees in the Inspector, radians in the component.**
///
/// The boundary `Transform::rotation_rad` already keeps. A value that crossed
/// it unconverted would be off by a factor of 57 and still look like a number.
#[test]
fn the_angle_fields_convert_at_the_boundary() {
    let (mut sim, hook, plank) = two_bodies(true);
    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Pin).expect("join");
    let reg = registry();
    let queue = EditorCommandQueue::default();

    apply_joint_edit(
        &sim,
        joint.to_bits(),
        JointFieldEdit::LimitMax(90.0),
        &queue,
        &reg,
    );
    apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commands apply");

    let j = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    assert!(
        (j.limit_max - std::f32::consts::FRAC_PI_2).abs() < 1e-6,
        "90° should be stored as π/2 radians; it is {}",
        j.limit_max
    );
    // And it comes back out in degrees, so the round trip is closed.
    let info = build_joint_info(&mut sim, joint.to_bits(), 0).expect("info");
    assert!((info.limit_max_ui - 90.0).abs() < 1e-3);
}

/// **The snapshot shows the two bodies by NAME, and says when one is gone.**
///
/// The joint stores hashes; a hash is not something to show a person, and an
/// empty gap where a name should be is not either.
#[test]
fn the_snapshot_resolves_the_body_names_and_reports_a_broken_link() {
    let (mut sim, hook, plank) = two_bodies(true);
    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Pin).expect("join");

    let info = build_joint_info(&mut sim, joint.to_bits(), 0).expect("info");
    assert_eq!(info.body_a_name, "Hook");
    assert_eq!(info.body_b_name, "Plank");
    assert!(info.bound);

    *sim.world_mut().get_mut::<Name>(plank).expect("name") = Name::new("Renamed");
    let info = build_joint_info(&mut sim, joint.to_bits(), 0).expect("info");
    assert!(
        !info.bound && info.body_b_name.is_empty(),
        "after the rename the section must report the link as broken, not \
         keep showing the old name as if nothing happened"
    );
}

/// **§12 is offered only for a joint.**
#[test]
fn a_plain_body_has_no_joint_section() {
    let (mut sim, hook, _) = two_bodies(true);
    assert!(build_joint_info(&mut sim, hook.to_bits(), 0).is_none());
}

/// **Two bodies that share a name cannot be joined, and the gesture says so.**
///
/// The `a == b` guard compares ENTITIES; a joint stores name HASHES. Two
/// distinct bodies with the same name resolve to one id, so the joint could
/// never bind — and before this it was still created, handing the artist an
/// object that does nothing and (until the ring fix) cleared the scrub cache
/// every frame for as long as it existed.
#[test]
fn two_bodies_sharing_a_name_cannot_be_joined() {
    let (mut sim, hook, plank) = two_bodies(true);
    *sim.world_mut().get_mut::<Name>(plank).expect("name") = Name::new("Hook");
    assert!(
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Pin).is_none(),
        "a joint was created between two bodies that share a name — it can \
         never bind, because the two ids it stores are the same number"
    );
    let mut q = sim.world_mut().query::<&PhysicsJoint>();
    assert_eq!(q.iter(sim.world()).count(), 0);
}

/// Spawn a named physical body — a third body for the re-pick tests.
fn body(sim: &mut SimWorld, x: f32, y: f32, kind: BodyKind, name: &str) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
            Name::new(name),
        ))
        .id()
}

/// **Setting a body re-binds that slot IN PLACE, and the joint still binds.**
///
/// Re-pick slot A from Hook to a new anchor Post: the component names Post, slot
/// B is untouched, `set_joint_body` returns `true`, and the joint still reaches
/// the solver. It writes in place (no editor queue) because the pick resolves
/// mid-frame in the pointer handler, and the global diff-based undo captures a
/// direct write. Mutation-tested: writing the WRONG slot leaves `body_a` on
/// Hook, and not writing does the same — both go red on the `body_a == Post`
/// assertion.
#[test]
fn set_joint_body_rebinds_slot_a_and_the_joint_still_binds() {
    let (mut sim, hook, plank) = two_bodies(true);
    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Pin).expect("join");
    let post = body(&mut sim, 3.0, 6.0, BodyKind::Static, "Post");

    assert!(
        set_joint_body(&mut sim, joint.to_bits(), false, post),
        "the re-bind was refused"
    );

    let j = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    assert_eq!(
        j.body_a,
        stable_name_id("Post"),
        "slot A did not re-bind to Post"
    );
    assert_eq!(
        j.body_b,
        stable_name_id("Plank"),
        "slot B was touched — the wrong slot moved"
    );

    // Not dormant: the re-bound joint still reaches the solver.
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=60 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(
        bridge.joint_count(),
        1,
        "the re-bound joint never reached the solver"
    );
    let _ = plank;
}

/// **A self-joint is refused, and the joint is left untouched.**
///
/// Re-picking slot A to the body already in slot B would name both ends the same
/// body — a joint that can never bind. `set_joint_body` returns `false` and
/// writes nothing (so the shell keeps the pick armed for another click), rather
/// than leaving a silently-dormant joint. Mutation-tested: dropping the guard
/// writes the self-joint and this goes red on the "untouched" assertion.
#[test]
fn set_joint_body_refuses_a_self_joint() {
    let (mut sim, hook, plank) = two_bodies(true);
    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Pin).expect("join");
    let before = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    // Slot A is currently Hook; re-pick it to Plank, which is already slot B.
    assert!(
        !set_joint_body(&mut sim, joint.to_bits(), false, plank),
        "picking the body already in the other slot must be refused"
    );
    let after = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    assert_eq!(before, after, "a refused re-pick must not touch the joint");
}

/// **Creating a joint makes the KIND the artist chose** (gold standard: create
/// the type you want, not a Pin you convert in §12). Mutation-tested:
/// `create_joint` ignoring its `kind` and spawning the default Pin makes the
/// Spring/Rope/Weld iterations go red.
#[test]
fn create_joint_makes_the_requested_kind() {
    for kind in [
        JointKind::Pin,
        JointKind::Spring,
        JointKind::Rope,
        JointKind::Weld,
    ] {
        let (mut sim, hook, plank) = two_bodies(true);
        let joint = create_joint(&mut sim, hook.to_bits(), plank.to_bits(), kind).expect("join");
        let j = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
        assert_eq!(
            j.kind, kind,
            "create_joint ignored the chosen kind {kind:?}"
        );
    }
}

/// **Changing the kind re-seeds the anchor.** The anchor POLICY depends on the
/// kind — a Pin/Weld shares a point, a Spring/Rope anchors body B at its centre
/// — so a kind change marks the joint un-anchored, and the next reconcile
/// re-derives the body-local anchors under the new policy. Without it a Pin
/// turned into a Rope keeps the shared-point anchor and the rope hangs from the
/// wrong spot on body B. Mutation-tested: dropping `next.anchored = false` in
/// `apply_joint_edit`'s Kind arm leaves it anchored and this goes red.
#[test]
fn changing_the_kind_re_seeds_the_anchor() {
    let (mut sim, hook, plank) = two_bodies(true);
    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Pin).expect("join");

    // Seed the anchors: the first reconcile flips `anchored` to true.
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    assert!(
        sim.world().get::<PhysicsJoint>(joint).unwrap().anchored,
        "the pin should be seeded after a dispatch"
    );

    let reg = registry();
    let queue = EditorCommandQueue::default();
    apply_joint_edit(
        &sim,
        joint.to_bits(),
        JointFieldEdit::Kind(2), // Rope
        &queue,
        &reg,
    );
    apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commands apply");

    let j = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    assert_eq!(j.kind, JointKind::Rope, "the kind did not change");
    assert!(
        !j.anchored,
        "a kind change must mark the joint for re-seed, so the two-ended Rope \
         anchors body B at its centre instead of keeping the Pin's shared point"
    );
}

/// **A parameter edit only LANDS after the queue is flushed — the render loop's
/// job, and the one it forgot (W-JointParams, 2026-07-25).**
///
/// The user's report was in TWO parts: the bridge gated the re-describe on
/// `at_rest` (fixed in `bridge/joints.rs`), AND the shell's §12 edit block
/// pushed a `SetComponent` without flushing it — so a joint slider did nothing
/// until some OTHER Inspector edit happened to drain the queue ("às vezes
/// funciona"). This pins the fact that block must honour: `apply_joint_edit`
/// only QUEUES; the component changes on `apply_editor_commands`. The arch-gate
/// `the_joint_edit_loop_flushes_the_command_queue` proves the render loop calls
/// it; this proves that call is load-bearing.
#[test]
fn a_joint_param_edit_lands_only_when_the_queue_is_flushed() {
    let (mut sim, hook, plank) = two_bodies(true);
    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Spring).expect("join");
    {
        let mut j = sim
            .world_mut()
            .get_mut::<PhysicsJoint>(joint)
            .expect("joint");
        j.stiffness = 30.0;
    }
    let reg = registry();
    let queue = EditorCommandQueue::default();

    // The edit the panel emits: stiffen the spring.
    apply_joint_edit(
        &sim,
        joint.to_bits(),
        JointFieldEdit::Stiffness(300.0),
        &queue,
        &reg,
    );
    // ⚠️ Before the flush the component is UNCHANGED — the edit is only queued.
    // This is the whole reason the render loop must flush; skipping it is what
    // made the slider inert.
    assert_eq!(
        sim.world().get::<PhysicsJoint>(joint).unwrap().stiffness,
        30.0,
        "apply_joint_edit must only QUEUE — if it wrote the component directly \
         the flush would not be load-bearing and the render-loop bug would be \
         invisible here"
    );

    apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commands apply");
    assert_eq!(
        sim.world().get::<PhysicsJoint>(joint).unwrap().stiffness,
        300.0,
        "after the flush the component must carry the new stiffness — this is the \
         edit the render loop failed to flush, so the joint sat at k=30. The \
         bridge picking a flushed component change up and tightening the spring \
         is proven end to end in ph2d-physics-ecs/tests/joint_live_edit.rs"
    );
}

/// **O curso vai e volta na unidade do TIPO** (W-J5).
///
/// `limit_min/max` carregam radianos num Pin e metros num Slider — o modelo do
/// próprio rapier (um campo `limits`, pertencente ao grau de liberdade que o
/// joint deixou livre). Este gate pina o par de portas: o que o artista digita na
/// row Min é o que a row Min mostra no frame seguinte, nos DOIS tipos.
///
/// Mutação: `limit_in`/`limit_out` ignorarem o tipo (converter sempre) ⇒ o
/// Slider volta 0,5 m como 28,6 e isto fica vermelho.
#[test]
fn a_limit_round_trips_in_its_kinds_own_unit() {
    for (kind_tag, typed) in [(0u8, 45.0_f32), (4, 0.5)] {
        let base = PhysicsJoint {
            kind: kind_of(kind_tag),
            limits_enabled: true,
            ..PhysicsJoint::default()
        };
        let after =
            joint_with_edit(base, JointFieldEdit::LimitMax(typed)).expect("a limit edit lands");
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((Name::new("J"), after, Transform::default()))
            .id();
        let info = build_joint_info(&mut sim, e.to_bits(), 0).expect("info");
        assert!(
            (info.limit_max_ui - typed).abs() < 1e-3,
            "kind {kind_tag}: digitou {typed}, a row mostra {}",
            info.limit_max_ui
        );
        // ⚠️ **E o número GUARDADO, que é o que chega ao solver.** O round-trip
        // sozinho é oráculo fraco: um par de conversões consistentemente ERRADO
        // (converter sempre, ignorando o tipo) vai e volta perfeitamente enquanto
        // o trilho fica 57x curto. A mutação que apaga o `if limits_in_metres`
        // sobreviveu à asserção acima e é esta que a mata.
        let stored = if kind_tag == 4 {
            typed // metros, verbatim
        } else {
            typed.to_radians()
        };
        assert!(
            (after.limit_max - stored).abs() < 1e-4,
            "kind {kind_tag}: {typed} tem de VIRAR {stored} no componente, got {}",
            after.limit_max
        );
    }
}

/// **Trocar entre dobradiça e trilho RE-SEMEIA o alcance** — e trocar entre dois
/// tipos da mesma unidade NÃO.
///
/// Sem a re-semeadura os ±45° de um Pin (±0,785 rad) viram ±0,785 **metros** de
/// curso, um número que ninguém digitou. Com ela sempre, Pin→Weld→Pin jogaria
/// fora os ângulos do artista — que é a promessa que o componente faz sobre
/// trocar de tipo.
///
/// Mutação: re-semear em toda troca ⇒ a 2ª metade fica vermelha; nunca re-semear
/// ⇒ a 1ª.
#[test]
fn changing_the_limit_unit_re_seeds_the_range_and_nothing_else_does() {
    let pin = PhysicsJoint {
        kind: JointKind::Pin,
        limit_min: -0.4,
        limit_max: 0.4,
        ..PhysicsJoint::default()
    };
    // Pin -> Slider: a unidade muda, o alcance é re-semeado em METROS.
    let slider = joint_with_edit(pin, JointFieldEdit::Kind(4)).expect("kind edit");
    let want = PhysicsJoint::default_limits(JointKind::Slider);
    assert!(
        (slider.limit_max - want[1]).abs() < 1e-6,
        "Pin->Slider tem de re-semear o curso, got {}",
        slider.limit_max
    );
    // Pin -> Weld: MESMA unidade, o alcance do artista sobrevive.
    let weld = joint_with_edit(pin, JointFieldEdit::Kind(3)).expect("kind edit");
    assert!(
        (weld.limit_max - 0.4).abs() < 1e-6,
        "Pin->Weld tem de PRESERVAR os angulos, got {}",
        weld.limit_max
    );
}

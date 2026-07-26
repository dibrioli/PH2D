//! **`jointed_group` — a bake of one link is a bake of the whole rig.**
//!
//! Baking flips a body to `Kinematic`; with the transport's Physics toggle off,
//! any DYNAMIC body left un-baked freezes (nothing steps the solver) while the
//! baked links play, and the joint stretches between a moving anchor and a
//! still one. So a bake of any body must pull in every Dynamic body it is
//! coupled to. These gates pin the graph rule the shell's bake relies on —
//! pure, headless, no dispatch (the function reads the authored ECS).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody, jointed_group,
};
use std::collections::BTreeSet;

fn body(sim: &mut SimWorld, name: &str, kind: BodyKind, y: f32) {
    sim.world_mut().spawn((
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Ball { radius: 0.1 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, y)),
    ));
}

fn joint(sim: &mut SimWorld, name: &str, a: &str, b: &str) {
    sim.world_mut().spawn((
        Name::new(name),
        PhysicsJoint {
            body_a: stable_name_id(a),
            body_b: stable_name_id(b),
            kind: JointKind::Pin,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

/// The set of NAMES in a group, so assertions read in the fixture's vocabulary
/// rather than in entity bits.
fn group_names(sim: &mut SimWorld, seed: &[Entity]) -> BTreeSet<String> {
    let group = jointed_group(sim.world_mut(), seed);
    let bits: BTreeSet<u64> = group.iter().map(|e| e.to_bits()).collect();
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .filter(|(e, _)| bits.contains(&e.to_bits()))
        .map(|(_, n)| n.as_str().to_string())
        .collect()
}

fn names(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

/// **A bake of one link pulls in the whole dynamic chain.**
///
/// Hook(static) – L1 – L2 – L3 (dynamic), pinned in a line. Selecting ANY one
/// link must return all three dynamic links — the static Hook is a boundary, so
/// it is not in the group.
///
/// Mutation-tested: dropping the BFS (returning only the seed) leaves L2's group
/// as `{L2}`, and this goes red — the shell would bake one link and freeze the
/// rest when Physics is off.
#[test]
fn baking_one_link_pulls_in_the_whole_dynamic_chain() {
    let mut sim = SimWorld::new();
    body(&mut sim, "Hook", BodyKind::Static, 6.0);
    body(&mut sim, "L1", BodyKind::Dynamic, 5.0);
    body(&mut sim, "L2", BodyKind::Dynamic, 4.0);
    body(&mut sim, "L3", BodyKind::Dynamic, 3.0);
    joint(&mut sim, "J0", "Hook", "L1");
    joint(&mut sim, "J1", "L1", "L2");
    joint(&mut sim, "J2", "L2", "L3");

    // From the middle, and from an end — the answer is the whole chain either way.
    for seed_name in ["L1", "L2", "L3"] {
        let seed = named(&mut sim, seed_name);
        assert_eq!(
            group_names(&mut sim, &[seed]),
            names(&["L1", "L2", "L3"]),
            "baking {seed_name} did not pull in the whole dynamic chain"
        );
    }
}

/// **A body with no joints is its own group.** Nothing to pull in, and nothing
/// pulled in by mistake.
#[test]
fn a_lone_body_is_its_own_group() {
    let mut sim = SimWorld::new();
    body(&mut sim, "Free", BodyKind::Dynamic, 5.0);
    // A second, jointed rig that must NOT be dragged in.
    body(&mut sim, "Hook", BodyKind::Static, 6.0);
    body(&mut sim, "Swing", BodyKind::Dynamic, 5.0);
    joint(&mut sim, "Pin", "Hook", "Swing");

    let free = named(&mut sim, "Free");
    assert_eq!(group_names(&mut sim, &[free]), names(&["Free"]));
}

/// **Two pendulums on the SAME static hook stay independent.**
///
/// The static hook is a wall, not a wire: P1 and P2 are each pinned to Hook but
/// are not coupled to each other, so baking P1 must leave P2 alone.
///
/// Mutation-tested: this is the gate that earns "conduct through DYNAMIC bodies
/// only". Let the walk cross a non-dynamic neighbour (seed the frontier with, or
/// step through, the static Hook) and the two pendulums merge into one group —
/// this goes red where the chain gate stays green.
#[test]
fn two_pendulums_on_one_static_hook_stay_independent() {
    let mut sim = SimWorld::new();
    body(&mut sim, "Hook", BodyKind::Static, 6.0);
    body(&mut sim, "P1", BodyKind::Dynamic, 5.0);
    body(&mut sim, "P2", BodyKind::Dynamic, 5.0);
    joint(&mut sim, "JA", "Hook", "P1");
    joint(&mut sim, "JB", "Hook", "P2");

    let p1 = named(&mut sim, "P1");
    assert_eq!(
        group_names(&mut sim, &[p1]),
        names(&["P1"]),
        "baking one pendulum dragged in the other through the shared static hook"
    );
}

/// **A kinematic link is a boundary too.** A Dynamic body jointed to a Kinematic
/// one (already curve-driven, never moved by a joint) is not coupled through it:
/// baking the Dynamic side does not pull the Kinematic one in, and does not
/// cross it to a body on its far side.
#[test]
fn a_kinematic_neighbour_is_a_boundary() {
    let mut sim = SimWorld::new();
    body(&mut sim, "Dyn", BodyKind::Dynamic, 5.0);
    body(&mut sim, "Kine", BodyKind::Kinematic, 4.0);
    body(&mut sim, "Far", BodyKind::Dynamic, 3.0);
    joint(&mut sim, "J0", "Dyn", "Kine");
    joint(&mut sim, "J1", "Kine", "Far");

    let d = named(&mut sim, "Dyn");
    assert_eq!(
        group_names(&mut sim, &[d]),
        names(&["Dyn"]),
        "a kinematic body conducted the coupling it should have blocked"
    );
}

/// **The seed passes through verbatim.** A Static floor caught in a marquee
/// selection stays in the returned set (the shell filters it downstream — a
/// static body's trajectory is constant, so bake writes no track and never
/// flips its kind), and it does not conduct.
#[test]
fn the_seed_is_kept_even_when_it_does_not_conduct() {
    let mut sim = SimWorld::new();
    body(&mut sim, "Floor", BodyKind::Static, 0.0);
    body(&mut sim, "Hook", BodyKind::Static, 6.0);
    body(&mut sim, "Swing", BodyKind::Dynamic, 5.0);
    joint(&mut sim, "Pin", "Hook", "Swing");

    let floor = named(&mut sim, "Floor");
    // The static floor stays; it pulls in nothing (it has no joints and would
    // not conduct anyway).
    assert_eq!(group_names(&mut sim, &[floor]), names(&["Floor"]));
}

// ─────────────────────────────────────────────────────────────────────────────
// `jointed_rig` — a MESMA travessia, a OUTRA política (W-JG)
// ─────────────────────────────────────────────────────────────────────────────

/// O mesmo helper, pela porta do ARRASTO.
fn rig_names(sim: &mut SimWorld, seed: &[Entity]) -> BTreeSet<String> {
    let group = ph2d_physics_ecs::jointed_rig(sim.world_mut(), seed);
    let bits: BTreeSet<u64> = group.iter().map(|e| e.to_bits()).collect();
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .filter(|(e, _)| bits.contains(&e.to_bits()))
        .map(|(_, n)| n.as_str().to_string())
        .collect()
}

/// **AS DUAS PORTAS DISCORDAM, e a divergência é o contrato.**
///
/// O gate mais importante deste par: sobre a MESMA cena, `jointed_group` (bake)
/// para no gancho estático e `jointed_rig` (arrasto) o atravessa. Elas respondem
/// perguntas diferentes — *quem CONGELA quando a física é desligada?* contra
/// *quem tem de andar junto para a pose de repouso ficar coerente?* — e uma
/// futura "simplificação" que as unificasse quebraria exatamente uma das duas,
/// em silêncio, porque as assinaturas são idênticas.
///
/// Mutação-testada nas duas direções: `jointed_rig` com a política do Dynamic
/// perde o gancho; `jointed_group` com a política de todo-tipo o ganha.
#[test]
fn the_bake_door_and_the_drag_door_disagree_about_a_static_neighbour() {
    let mut sim = SimWorld::new();
    body(&mut sim, "Hook", BodyKind::Static, 6.0);
    body(&mut sim, "L1", BodyKind::Dynamic, 5.0);
    body(&mut sim, "L2", BodyKind::Dynamic, 4.0);
    joint(&mut sim, "J0", "Hook", "L1");
    joint(&mut sim, "J1", "L1", "L2");

    let l2 = named(&mut sim, "L2");
    assert_eq!(
        group_names(&mut sim, &[l2]),
        names(&["L1", "L2"]),
        "o BAKE para na fronteira estatica: um Static nao congela e nao se assa"
    );
    assert_eq!(
        rig_names(&mut sim, &[l2]),
        names(&["Hook", "L1", "L2"]),
        "o ARRASTO atravessa: a ancora do joint no gancho tambem viaja com ele"
    );
}

/// **Um Kinematic também é carregado pelo arrasto**, e pelo bake também não.
///
/// O irmão do gate acima para o terceiro tipo — o `jointed_group` o recusa por
/// um motivo DIFERENTE do Static (ele já segue curva, então não congela), e é
/// por isso que os dois tipos precisam de gate cada.
#[test]
fn the_drag_door_crosses_a_kinematic_the_bake_door_stops_at() {
    let mut sim = SimWorld::new();
    body(&mut sim, "Load", BodyKind::Dynamic, 5.0);
    body(&mut sim, "Lift", BodyKind::Kinematic, 4.0);
    body(&mut sim, "Far", BodyKind::Dynamic, 3.0);
    joint(&mut sim, "J0", "Load", "Lift");
    joint(&mut sim, "J1", "Lift", "Far");

    let load = named(&mut sim, "Load");
    assert_eq!(
        group_names(&mut sim, &[load]),
        names(&["Load"]),
        "o BAKE nao conduz por um Kinematic"
    );
    assert_eq!(
        rig_names(&mut sim, &[load]),
        names(&["Far", "Lift", "Load"]),
        "o ARRASTO conduz por ele e o leva junto"
    );
}

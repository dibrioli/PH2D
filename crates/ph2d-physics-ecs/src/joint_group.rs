//! [`jointed_group`] — the whole articulated rig, from any of its bodies.
//!
//! A bake flips a body to `Kinematic` and drives its pose from a curve. That is
//! coherent for a WHOLE jointed group: the sim runs once, every link's
//! trajectory already reflects the coupling, they all become curves, the joints
//! go inert (a kinematic body is not moved by a joint — infinite mass), and
//! playback reproduces the swing. It is INCOHERENT for a PART of one. Bake only
//! the links the artist selected and the un-baked DYNAMIC neighbours stay
//! dynamic; once the transport's Physics toggle is off nothing steps the solver,
//! so those neighbours FREEZE at their authored pose while the baked links play,
//! and the joint stretches between a moving anchor and a still one. There is no
//! partial bake of a coupled rig that looks right.
//!
//! So a bake of any body pulls in every body it is dynamically coupled to. This
//! computes that set.
//!
//! # Only DYNAMIC bodies conduct, and that is physics, not tidiness
//!
//! Three body kinds behave differently under "Physics off after a bake":
//!
//! - **Dynamic** freezes (nothing integrates it), and its simulated motion
//!   depends on its Dynamic neighbours (they push and pull each other through
//!   the joint). So it must be baked AND it transmits the coupling.
//! - **Kinematic** already follows a curve (`settle` tracks it every frame), so
//!   it does not freeze; and a joint does not move it, so it is a scripted
//!   anchor the Dynamic side is pulled toward, not a coupler.
//! - **Static** is fixed: it does not freeze and does not move.
//!
//! So the group is the connected component of **Dynamic** bodies reachable
//! through joints. Static and Kinematic bodies are BOUNDARIES — a joint edge
//! reaches them but the walk does not continue through them, and they are not
//! added to the group (baking a Static writes nothing — its trajectory is
//! constant — and a Kinematic is already curve-driven). This is what makes two
//! independent pendulums hung from the *same* static hook stay independent: the
//! hook is a wall between them, not a wire.
//!
//! # Read from the authored ECS, not the live graph
//!
//! The edges come straight from the [`PhysicsJoint`] components, resolved
//! through [`Name`] exactly as [`crate::PhysicsBridge`] resolves them (same key,
//! [`stable_name_id`]). It reads the ECS rather than the bridge's reconciled
//! `self.joints` because the authored graph is always current — it includes a
//! joint the artist made this very frame — and needs no dispatch to have run,
//! which is what lets a headless gate build a rig and ask for its group without
//! stepping the world. For a bake, the authored scene is the truth.
//!
//! # Two policies over ONE walk (W-JG)
//!
//! The **drag** (arrastar um rig no canvas em repouso) asks a different question
//! from the bake, and the difference is exactly the kind filter:
//!
//! | | pergunta | quem conduz |
//! |---|---|---|
//! | [`jointed_group`] (bake) | *quem CONGELA quando a física é desligada?* | só Dynamic |
//! | [`jointed_rig`] (arrasto) | *quem tem de andar junto para a pose ficar coerente?* | **todo corpo** |
//!
//! Um joint tem **duas** âncoras e as duas viajam com os corpos delas — então,
//! para um arrasto, um gancho Static ou uma plataforma Kinematic que ficam para
//! trás deixam o joint esticado exatamente como um elo Dynamic deixaria.
//! Congelar, não: um Static não congela e um Kinematic segue curva, e é por isso
//! que o bake não os assa.
//!
//! ⚠️ **A travessia é UMA** ([`walk`]) e as políticas são dois nomes sobre ela.
//! *Que aresta existe* é a pergunta que não pode ter duas respostas — duas
//! cópias do grafo divergiriam no dia em que um tipo novo de joint chegasse.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::{Entity, Name, World, stable_name_id};

use crate::components::{BodyKind, RigidBody};
use crate::joint::PhysicsJoint;
use crate::joint_tool::DragReach;

/// `seed` plus every Dynamic body it is coupled to through a chain of joints —
/// the articulated rig **as a BAKE sees it**. Returned sorted by entity bits
/// (HR-5), so a bake of the same rig produces the same target order whichever
/// link the artist clicked.
///
/// The `seed` is kept verbatim (a body with no joints returns just itself; a
/// Static floor caught in a marquee passes through and bakes to nothing). What
/// is ADDED is only the Dynamic bodies reachable through Dynamic bodies — see
/// the module docs for why Static and Kinematic are boundaries here and NOT in
/// [`jointed_rig`].
#[must_use]
pub fn jointed_group(world: &mut World, seed: &[Entity]) -> Vec<Entity> {
    walk(world, seed, |k| k == BodyKind::Dynamic)
}

/// `seed` plus every body it is jointed to, **whatever kind they are** — the rig
/// as a CANVAS DRAG sees it (W-JG). Same order guarantee as [`jointed_group`].
///
/// A hinge's two anchors are body-local, so any member left behind leaves the
/// joint stretched — a Static hook and a Kinematic platform included. Moving a
/// static anchor at rest is legitimate authoring (you are relocating the wall),
/// which is why nothing is a boundary here.
///
/// ⚠️ **Consequência:** dois pêndulos no MESMO gancho são **um** rig por esta
/// porta (o gancho conduz), enquanto pelo [`jointed_group`] eles são dois. As
/// duas respostas estão certas para as suas perguntas, e há gate pinando que
/// elas DIVERGEM — para ninguém as unificar de volta.
#[must_use]
pub fn jointed_rig(world: &mut World, seed: &[Entity]) -> Vec<Entity> {
    walk(world, seed, |_| true)
}

/// **A porta única do arrasto** (W-JointTools): o conjunto que um gesto carrega,
/// escolhido pela política que o artista tem em mãos.
///
/// As duas respostas já existiam ([`jointed_rig`] e [`jointed_group`]); o que
/// esta função acrescenta é que a ESCOLHA entre elas é feita **uma vez**, aqui,
/// e não num `match` por chamador. O rádio da seção Joints é o que a alimenta —
/// e o dia em que uma terceira política chegar, ela nasce nesta função em vez de
/// nascer em cada sítio de press que alguém lembrou de atualizar.
///
/// ⚠️ [`DragReach::Dynamic`] delega ao `jointed_group`, que o BAKE também usa —
/// e é a MESMA pergunta feita por dois motivos (*"quem tem de vir junto para a
/// cena ficar coerente sem mexer nas âncoras fixas?"*), não uma coincidência de
/// implementação.
#[must_use]
pub fn jointed_by(world: &mut World, seed: &[Entity], reach: DragReach) -> Vec<Entity> {
    match reach {
        DragReach::Whole => jointed_rig(world, seed),
        DragReach::Dynamic => jointed_group(world, seed),
    }
}

/// A travessia: BFS pelo grafo de joints AUTORADO, conduzindo (e acrescentando)
/// só pelos corpos que `carries` aceita.
fn walk(world: &mut World, seed: &[Entity], carries: impl Fn(BodyKind) -> bool) -> Vec<Entity> {
    // name hash -> body entity, and the set of CONDUCTING body bits. Same key the
    // reconcile uses; the source is the ECS because a bake is about the authored
    // scene. Only NAMED bodies can be part of the joint graph (a joint names its
    // bodies), so requiring `Name` here is not a filter, it is the domain.
    let mut name_to_body: BTreeMap<u64, Entity> = BTreeMap::new();
    let mut conducting: BTreeSet<u64> = BTreeSet::new();
    let mut bodies = world.query::<(Entity, &Name, &RigidBody)>();
    for (e, name, rb) in bodies.iter(world) {
        name_to_body.insert(stable_name_id(name.as_str()), e);
        if carries(rb.kind) {
            conducting.insert(e.to_bits());
        }
    }

    // Undirected adjacency over body bits. A joint that does not name two
    // distinct resolvable bodies contributes no edge — half-authored, or a body
    // deleted / renamed / absent — which is the reconcile's `names_two_bodies` +
    // resolve guard, made here for the authored graph.
    let mut adj: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let mut joints = world.query::<&PhysicsJoint>();
    for j in joints.iter(world) {
        if !j.names_two_bodies() {
            continue;
        }
        let (Some(&ea), Some(&eb)) = (name_to_body.get(&j.body_a), name_to_body.get(&j.body_b))
        else {
            continue;
        };
        adj.entry(ea.to_bits()).or_default().push(eb.to_bits());
        adj.entry(eb.to_bits()).or_default().push(ea.to_bits());
    }

    // BFS from the seed's CONDUCTING bodies. A reached body is ADDED only if it
    // conducts too — so under the bake's policy a Static / Kinematic neighbour is
    // a boundary (touched by the edge, never crossed), and under the drag's
    // policy there is no boundary at all.
    let mut group: BTreeSet<u64> = seed.iter().map(|e| e.to_bits()).collect();
    let mut frontier: Vec<u64> = seed
        .iter()
        .map(|e| e.to_bits())
        .filter(|b| conducting.contains(b))
        .collect();
    while let Some(b) = frontier.pop() {
        let Some(neighbours) = adj.get(&b) else {
            continue;
        };
        for &n in neighbours {
            if conducting.contains(&n) && group.insert(n) {
                frontier.push(n);
            }
        }
    }

    group.into_iter().map(Entity::from_bits).collect()
}

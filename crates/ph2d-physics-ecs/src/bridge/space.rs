//! **The solver speaks WORLD, the scene stores LOCAL** — the one place the
//! bridge converts between them (ADR-0131 W5).
//!
//! For a root entity the two coincide, which is why this cost nothing to get
//! wrong for four waves: every gate, every smoke scene and every demo used
//! root-level bodies. Parent a physics object to anything in the Hierarchy and
//! both directions break at once — measured before the fix, a body authored at
//! local `(0, 4)` under a parent at `(5, 0)` **simulated at x = 0 while it drew
//! at x = 5**. The collider was not where the sprite was: it collided with
//! things the artist could not see it touching, and nothing errored.
//!
//! Two functions, because the bug is symmetric and half a fix is worse than
//! none — a scene where entry composes and readback does not would be *stable*
//! and wrong, drifting one parent-offset per frame.
//!
//! ⚠️ **Neither invents any algebra.** `ph2d-ecs` owns composition
//! (`parent_world_transform_into`) and its exact inverse
//! (`Transform::inverse_compose`), next to the `compose` they mirror. A second
//! copy here would be a second answer to *"where is this entity?"*, and this
//! module exists precisely because there was already one too many.

use bevy_ecs::world::World;
use ph2d_ecs::{Entity, Transform, parent_world_transform_into};

/// The entity's pose in WORLD space — what the solver must be told.
///
/// `None` when the entity has no `Transform` (it is not placeable, so it is
/// not simulatable either).
pub(super) fn world_transform(
    world: &World,
    e: Entity,
    scratch: &mut Vec<Transform>,
) -> Option<Transform> {
    ph2d_ecs::world_transform_into(world, e, scratch)
}

/// **Onde `e` está DENTRO de `ancestor`** — a pose local de uma PEÇA no corpo
/// dela (W-Compound / W-PartFace).
///
/// ⚠️ **Compõe a cadeia AUTORADA e nunca pergunta ao solver**, e essa é a
/// diferença inteira. A versão anterior derivava por
/// `inverse_compose(mundo_do_dono, mundo_da_peça)` — um ROUND-TRIP pela pose que
/// o solver acabou de escrever —, e por isso o `reconcile_parts` só podia
/// re-descrever uma peça **em repouso**: durante o play o resultado é estável mas
/// não é EXATO (medido, 300 ticks sobre uma rampa: 1,2e-7 em translação e 3,0e-8
/// em rotação), então a comparação `local == local` falharia quase todo frame e
/// a peça seria despendurada e re-pendurada sem parar.
///
/// O preço daquele gate era o defeito que o Enio reportou: mover uma peça com o
/// relógio ANDANDO movia o desenho e **deixava o collider onde estava** — a forma
/// atravessava o estático, sem erro e sem warning. Composta pela cadeia autorada,
/// a resposta não depende do solver, é **bit-estável** enquanto ninguém edita, e
/// o gate de repouso deixa de ter razão de existir.
///
/// ⚠️ **Um GRUPO no meio é atravessado**, pela mesma regra do `owner_body`: a
/// cadeia é composta do ancestral para baixo, então pôr as formas de uma peça
/// dentro de uma pasta continua sendo transparente.
///
/// `None` se `ancestor` não está na linhagem de `e`, ou se falta `Transform` a
/// alguém no caminho.
pub(super) fn local_in_ancestor(
    world: &World,
    e: Entity,
    ancestor: Entity,
    scratch: &mut Vec<Transform>,
) -> Option<Transform> {
    scratch.clear();
    let mut cur = e;
    loop {
        scratch.push(*world.get::<Transform>(cur)?);
        let parent = world
            .get::<ph2d_ecs::ChildOf>(cur)
            .map(ph2d_ecs::ChildOf::parent)?;
        if parent == ancestor {
            break;
        }
        cur = parent;
    }
    // `scratch` está da PEÇA para cima; compor de cima para baixo é
    // `compose(pai, filho)`, então a dobra vai do fim para o começo.
    let mut acc = *scratch.last()?;
    for t in scratch.iter().rev().skip(1) {
        acc = Transform::compose(acc, *t);
    }
    Some(acc)
}

/// Store a WORLD pose the solver produced onto the entity's LOCAL `Transform`.
///
/// Writes translation and rotation only: a rigid body has a pose, not a scale,
/// so the artist's scale and skew are left exactly as authored. (The collider
/// shape does not follow scale either — that is true of root bodies too, is
/// orthogonal to this wave, and is named in the tracker rather than smuggled
/// in here.)
///
/// Returns `false` and **writes nothing** when the parent chain cannot be
/// inverted. That refusal is not a formality: the alternative is `±inf`/`NaN`
/// in a `Transform`, and one non-finite field poisons the entire subtree's
/// `GlobalTransform` through propagation — a scene-wide corruption produced by
/// a single body under a parent someone scaled to zero.
pub(super) fn write_world_pose(
    world: &mut World,
    e: Entity,
    x: f32,
    y: f32,
    rotation: f32,
    scratch: &mut Vec<Transform>,
) -> bool {
    // The parent chain is read while `world` is only borrowed shared; the
    // mutable borrow below cannot overlap it, so the lookup finishes first.
    let parent = parent_world_transform_into(world, e, scratch);
    let Some(local) = world.get::<Transform>(e).copied() else {
        return false;
    };
    // Feed the inverse a world transform that differs from the current one
    // ONLY in the pose, so scale/skew round-trip to exactly what is stored
    // rather than through a division and back.
    let target = Transform {
        translation: ph2d_core::Vec2::new(x, y),
        rotation,
        ..Transform::compose(parent, local)
    };
    let Some(next) = Transform::inverse_compose(parent, target) else {
        return false;
    };
    let Some(mut t) = world.get_mut::<Transform>(e) else {
        return false;
    };
    t.translation = next.translation;
    t.rotation = next.rotation;
    true
}

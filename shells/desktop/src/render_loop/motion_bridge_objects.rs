//! **Publishing engine OBJECTS into the cook** (doc 86 §2) — the shell half of
//! `source.object`, the sibling of `motion_bridge_shapes` (drawn curves).
//!
//! Every entity that has a **name** and a `Sprite` goes into the `Cook`'s
//! external table under that name, as **one instance carrying its appearance**:
//! `(P, size, tint, uv_rect, texture_id)`. The node reads it by name
//! (`ctx.external("Ball")`), the memo sees the tile's content-revision, and the
//! `motion.duplicator` downstream stamps it at every point.
//!
//! ## Three decisions, all about a boundary
//!
//! **1. The instance is at the ORIGIN, and it carries appearance, not pose.** A
//! `source.object` names WHAT to draw; WHERE is the point-set's job (the
//! duplicator adds the point's `P`). So the sprite's own canvas position is
//! irrelevant — the tile is a *template*. This is also why the membrane needs
//! only `Sprite` + `Name` (both live in `SimWorld`) and never `GlobalTransform`
//! (a `PresentComponent`, absent here): the appearance is `size`, `tint`, and
//! which atlas cell — none of them the world transform.
//!
//! **2. The name is the artist's, and it is the WHOLE reference** — the same
//! decision `motion_bridge_shapes` makes. An unnamed sprite is not published;
//! rename it and the node's `Object` field stops matching, which is exactly what
//! renaming a thing you refer to by name means. Two objects (or an object and a
//! curve) with the same name is the artist's business — the last publish wins,
//! and objects publish *after* curves, deterministically.
//!
//! **3. The tile is resolved the SAME way the sprite renderer resolves it.** An
//! atlas sprite's `uv_rect` is the packed cell (`TextureAtlas::region_uv`), an
//! individual's is the unit rect with its store handle — the exact branch
//! `sim_extract` runs. Reading the atlas here (and not one frame late through a
//! stashed copy) is possible because `renderer.atlas()` is in hand at the motion
//! phase; publishing an atlas UV the extract would derive differently is the
//! two-doors bug this repo hunts, so it is the *same* `region_uv`.

use ph2d_ecs::{
    Children, Entity, FlipObjectRef, GroupedChildren, Name, SimWorld, Transform, VecPathRef, With,
};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_render::{RenderInstance, Sprite, SpriteSource, TextureAtlas};

use crate::motion_object_bake::BakedTile;
use crate::motion_state::MotionState;

/// The appearance tile for one sprite: one instance at the origin carrying
/// `(P, size, tint, uv_rect, texture_id)`. `None` for a source the atlas cannot
/// resolve here (a cooked KTX2 sprite needs the renderer's cooked-texture store,
/// not in hand — deferred to a later wave; it is skipped, not guessed).
fn sprite_tile(spr: &Sprite, atlas: &TextureAtlas) -> Option<Stream> {
    let (uv_rect, texture_id) = sprite_appearance(spr, atlas)?;
    // `collapsed_tint` = self_tint × tint (the per-sprite modulate). The
    // inherited ancestor cascade the extract folds in is a refinement of a
    // template, deferred: a source is *this object's* appearance.
    Some(appearance_tile(
        spr.size,
        spr.collapsed_tint(),
        uv_rect,
        texture_id,
    ))
}

/// The `(uv_rect, texture_id)` a sprite resolves to — the branch `sim_extract`
/// runs. `None` for a cooked KTX2 (its store isn't in hand here). Shared by the
/// single-sprite path and the group-child path (doc 86 §2 A4).
fn sprite_appearance(spr: &Sprite, atlas: &TextureAtlas) -> Option<([f32; 4], u32)> {
    match spr.source {
        // Atlas → the packed cell's UV, sampling the shared atlas (`0`); the
        // cheap direct path (no bake) the sprite renderer already uses.
        SpriteSource::Atlas { key } => {
            Some((atlas.region_uv(key), RenderInstance::ATLAS_TEXTURE_ID))
        }
        // Individual → the full unit rect + the store handle it already carries.
        SpriteSource::Individual { texture_id } => Some(([0.0, 0.0, 1.0, 1.0], texture_id)),
        // Cooked KTX2 resolves through `renderer.cooked_texture_id`, not in hand.
        SpriteSource::CookedTexture { .. } => None,
    }
}

/// The one-instance appearance stream, at the origin. ⚠️ **The columns here are
/// exactly the ones `lower_to_instances` reads** — `P` (world_pos), `size`,
/// `tint`, `uv_rect` (atlas_uv), `texture_id` — so what the membrane publishes
/// and what the sink lowers cannot diverge (the two-doors bug). Pure, so that
/// column contract is unit-tested without a GPU atlas.
fn appearance_tile(size: [f32; 2], tint: [f32; 4], uv_rect: [f32; 4], texture_id: u32) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![size]))
        .with("tint", Column::Vec4(vec![tint]))
        .with("uv_rect", Column::Vec4(vec![uv_rect]))
        // A small integer id, exact in f32; the lowering reads it back.
        .with("texture_id", Column::Scalar(vec![texture_id as f32]))
}

/// Publish every **named sprite** into the cook (doc 86 §2).
///
/// ⚠️ **Appends — does NOT clear.** `motion_bridge_shapes::publish` clears the
/// external table first (its decision), and this pass adds objects into it, so
/// the cook sees both the curves (`motion.path`) and the objects
/// (`source.object`). It MUST run *after* `shapes::publish`; the caller
/// (`motion_bridge::publish_objects`) guarantees the order.
///
/// Called once a frame, before the pump. Republishing is free: the external's
/// revision is a hash of its content, so a sprite nobody touched invalidates
/// nothing (`ph2d_nodegraph::external`).
pub(super) fn publish(
    cook: &mut Cook,
    sim: &mut SimWorld,
    atlas: &TextureAtlas,
    bakes: &crate::motion_object_bake::ObjectBake,
    flip_bakes: &crate::motion_flip_bake::FlipObjectBake,
) {
    // Sprites resolve directly (a sprite already IS a tile). A `(&Sprite, &Name)`
    // query walks exactly the entities that can be a source; `world_mut()` builds
    // the `QueryState` (it caches on the world), the iteration is read-only.
    let mut q = sim.world_mut().query::<(&Sprite, &Name)>();
    let world = sim.world();
    for (spr, name) in q.iter(world) {
        if name.0.trim().is_empty() {
            continue; // unnamed: nothing for the artist to type into the node
        }
        if let Some(tile) = sprite_tile(spr, atlas) {
            cook.set_external(name.0.clone(), tile);
        }
    }

    // Vectors come as BAKED tiles (doc 86 §2 A2): the fx phase rasterized each
    // named shape once into an individual texture; here the tile's `texture_id`
    // rides the same appearance stream a sprite's does. The shape's colours are
    // baked in, so the stream tint is WHITE (the tile is not re-tinted).
    for (name, tile) in bakes.tiles() {
        cook.set_external(
            name.to_string(),
            appearance_tile(
                tile.size,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
                tile.texture_id,
            ),
        );
    }

    // Flip objects come as BAKED tiles too (doc 86 §2 A3): the fx phase composed each
    // named object's layers at the current frame into an individual texture. Same
    // appearance stream, same WHITE tint (the colours are baked in), so the sink can't
    // tell a baked Flip from a baked vector — the `source.object` node stays media-
    // agnostic. Objects publish after curves; the last write on a name clash wins.
    for (name, tile) in flip_bakes.tiles() {
        cook.set_external(
            name.to_string(),
            appearance_tile(
                tile.size,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
                tile.texture_id,
            ),
        );
    }

    // Groups (doc 86 §2 A4) publish LAST: a named GROUP resolves to N live
    // instances (its subtree's leaves), and its children may also be published
    // individually above — the two coexist (different names).
    group_externals(cook, sim, atlas, bakes, flip_bakes);
}

/// **Publish every named GROUP as N live instances** (doc 86 §2 A4). A group is an
/// entity with `GroupedChildren`; its subtree's leaves (sprite / vector / flip)
/// become one instance each, at their transform **relative to the group** (the
/// group's own world pose is excluded — the tile is a template stamped at each
/// point's `P`). Each child keeps its own live tile, so a group of mixed, animated,
/// simulated objects is stamped with its whole live layout in lockstep.
///
/// ⚠️ **VIVO, not a frozen composite:** the group is NOT baked into one tile (which
/// would freeze it); it emits its children as N quads, each resolving its own tile
/// every frame. This is the liveness the tier (§2) prioritises.
///
/// ⚠️ **v1 limits, named not silent:** a vector/flip child is resolved by its NAME
/// (its A2/A3 tile) — an UNNAMED vector/flip child is skipped (a sprite child needs
/// no name). And a child tile carries the orientation it was baked at (its world
/// pose), so the layout is exact for an axis-aligned group; a rotated/scaled GROUP
/// re-orienting its vector/flip children is a follow-up.
fn group_externals(
    cook: &mut Cook,
    sim: &mut SimWorld,
    atlas: &TextureAtlas,
    bakes: &crate::motion_object_bake::ObjectBake,
    flip_bakes: &crate::motion_flip_bake::FlipObjectBake,
) {
    // The named groups present this frame (collected first so the walk below can
    // borrow the world immutably without the query iterator alive).
    let mut named_groups: Vec<(String, Entity)> = Vec::new();
    {
        let mut q = sim
            .world_mut()
            .query_filtered::<(Entity, &Name), With<GroupedChildren>>();
        let world = sim.world();
        for (e, name) in q.iter(world) {
            if !name.0.trim().is_empty() {
                named_groups.push((name.0.clone(), e));
            }
        }
    }
    let world = sim.world();
    for (name, group) in named_groups {
        // The group's OWN transform is excluded: `IDENTITY` at the group ⇒ the
        // children are laid out in the group's local frame (position-independent).
        let mut subtree: Vec<(Entity, Transform)> = Vec::new();
        walk_group_transforms(world, group, Transform::IDENTITY, &mut subtree);
        let leaves: Vec<LeafInstance> = subtree
            .iter()
            .filter_map(|(e, acc)| resolve_leaf(world, *e, acc, atlas, bakes, flip_bakes))
            .collect();
        if leaves.is_empty() {
            continue;
        }
        cook.set_external(name, group_stream(&leaves));
    }
}

/// One resolved leaf of a group: an instance carrying its appearance + its pose
/// relative to the group. The columns are exactly the ones `lower_to_instances`
/// reads (`P`, `size`, `rot` in DEGREES, `tint`, `uv_rect`, `texture_id`).
struct LeafInstance {
    p: [f32; 2],
    rot_deg: f32,
    size: [f32; 2],
    tint: [f32; 4],
    uv: [f32; 4],
    tid: u32,
}

/// Walk the group subtree depth-first, accumulating each entity's transform
/// relative to the group (`compose` down the chain). PURE ECS (no resolution/atlas)
/// so the layout — the load-bearing part — is unit-testable headless. Recurses into
/// nested groups (a group of groups lays out correctly).
fn walk_group_transforms(
    world: &ph2d_ecs::World,
    entity: Entity,
    acc: Transform,
    out: &mut Vec<(Entity, Transform)>,
) {
    out.push((entity, acc));
    if let Some(children) = world.get::<Children>(entity) {
        for &child in children.iter() {
            let child_t = world
                .get::<Transform>(child)
                .copied()
                .unwrap_or(Transform::IDENTITY);
            walk_group_transforms(world, child, Transform::compose(acc, child_t), out);
        }
    }
}

/// Resolve ONE entity's appearance as a group leaf, at `acc` (its pose relative to
/// the group). Sprite → direct (atlas/individual); vector/flip → its A2/A3 tile by
/// name. `None` for a pure container (a group node) or an unresolvable leaf.
fn resolve_leaf(
    world: &ph2d_ecs::World,
    entity: Entity,
    acc: &Transform,
    atlas: &TextureAtlas,
    bakes: &crate::motion_object_bake::ObjectBake,
    flip_bakes: &crate::motion_flip_bake::FlipObjectBake,
) -> Option<LeafInstance> {
    let p = [acc.translation.x, acc.translation.y];
    if let Some(spr) = world.get::<Sprite>(entity) {
        let (uv, tid) = sprite_appearance(spr, atlas)?;
        // A sprite's atlas cell is orientation-free ⇒ the child's rotation/scale
        // relative to the group is applied here (rot in DEGREES, size scaled).
        return Some(LeafInstance {
            p,
            rot_deg: acc.rotation.to_degrees(),
            size: [
                spr.size[0] * acc.scale.x.abs(),
                spr.size[1] * acc.scale.y.abs(),
            ],
            tint: spr.collapsed_tint(),
            uv,
            tid,
        });
    }
    // A vector/flip child is resolved by its NAME (its baked tile).
    let name = world.get::<Name>(entity).map(|n| n.0.clone());
    let tile = if world.get::<VecPathRef>(entity).is_some() {
        name.as_deref().and_then(|n| bakes.tile_for(n))?
    } else if world.get::<FlipObjectRef>(entity).is_some() {
        name.as_deref().and_then(|n| flip_bakes.tile_for(n))?
    } else {
        return None; // a group container or an entity with no drawable appearance
    };
    Some(leaf_from_tile(acc, tile))
}

/// A baked-tile leaf (vector/flip): the tile carries the child's own orientation
/// (⚠️ its world bake — v1 limit), so `rot` is 0 and only position + scale apply.
fn leaf_from_tile(acc: &Transform, tile: BakedTile) -> LeafInstance {
    LeafInstance {
        p: [acc.translation.x, acc.translation.y],
        rot_deg: 0.0,
        size: [
            tile.size[0] * acc.scale.x.abs(),
            tile.size[1] * acc.scale.y.abs(),
        ],
        tint: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 0.0, 1.0, 1.0],
        tid: tile.texture_id,
    }
}

/// Build the N-instance appearance stream from the group's leaves — the same
/// columns the single-object tile publishes, one row per leaf.
fn group_stream(leaves: &[LeafInstance]) -> Stream {
    Stream::new(leaves.len())
        .with("P", Column::Vec2(leaves.iter().map(|l| l.p).collect()))
        .with(
            "size",
            Column::Vec2(leaves.iter().map(|l| l.size).collect()),
        )
        .with(
            "rot",
            Column::Scalar(leaves.iter().map(|l| l.rot_deg).collect()),
        )
        .with(
            "tint",
            Column::Vec4(leaves.iter().map(|l| l.tint).collect()),
        )
        .with(
            "uv_rect",
            Column::Vec4(leaves.iter().map(|l| l.uv).collect()),
        )
        .with(
            "texture_id",
            Column::Scalar(leaves.iter().map(|l| l.tid as f32).collect()),
        )
}

/// **Publish the engine objects into the cook** (doc 86 §2) — the sibling of
/// `publish_shapes` for `source.object`. Every named sprite (live from the atlas),
/// baked vector tile and baked Flip tile becomes an external the graph can bring in.
///
/// ⚠️ **Call AFTER `publish_shapes`:** it clears the external table first and this
/// APPENDS objects into it, so the cook sees both the curves and the objects.
pub(crate) fn publish_objects(motion: &mut MotionState, sim: &mut SimWorld, atlas: &TextureAtlas) {
    // `&mut pump.cook` and the two `_bake` reads are disjoint fields — sprites resolve
    // live from the atlas; the fx phase filled `object_bake`/`flip_object_bake` last
    // frame (doc 86 §2 A2/A3).
    publish(
        &mut motion.pump.cook,
        sim,
        atlas,
        &motion.object_bake,
        &motion.flip_object_bake,
    );
}

/// **Bake the named vector shapes to tiles** (doc 86 §2 A2) — at the fx phase, where
/// `renderer` + `gpu` + the vector scene/transforms are in hand. Cached by content, so
/// a static scene bakes once; `publish_objects` publishes the results next frame.
#[allow(clippy::too_many_arguments)]
pub(crate) fn bake_objects(
    motion: &mut MotionState,
    scene: &ph2d_vec_scene::VecScene,
    map: &crate::vec_entities::VecEntityMap,
    xforms: &ph2d_vec_scene::VecXforms,
    live: &ph2d_vec_render::LiveGeometry,
    gpu: &ph2d_gpu::GpuContext,
    surface_format: wgpu::TextureFormat,
    renderer: &mut ph2d_render::SpriteRenderer,
    sim: &SimWorld,
) {
    motion
        .object_bake
        .bake(scene, map, xforms, live, gpu, surface_format, renderer, sim);
}

/// **Bake the named Flip objects to tiles** (doc 86 §2 A3) — sibling of `bake_objects`,
/// at the fx phase. Composes each object's layers at the current frame through a
/// scratch Flip raster + compositor into an individual texture; cached by resolved-frame
/// content, so a static hold bakes once.
pub(crate) fn bake_flip_objects(
    motion: &mut MotionState,
    flip: &ph2d_flip::FlipDoc,
    map: &crate::flip_entities::FlipEntityMap,
    playhead: &ph2d_core::Playhead,
    gpu: &ph2d_gpu::GpuContext,
    renderer: &mut ph2d_render::SpriteRenderer,
    sim: &SimWorld,
) {
    motion
        .flip_object_bake
        .bake(flip, map, playhead, gpu, renderer, sim);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_membrane_publishes_exactly_the_columns_the_sink_reads() {
        // doc 86 §6 gate 3 (`bake_source_is_the_same_columns_the_sink_reads`):
        // the tile the membrane emits, lowered by the render sink, carries the
        // appearance back to the `RenderInstance` — the publish side and the
        // read side cannot diverge (the two-doors bug this repo hunts). A DUMMY
        // texture_id 7 that is neither the atlas sentinel (0) nor a size/tint
        // value, so a lowering that ignored the column could not pass.
        let tile = appearance_tile([2.0, 3.0], [1.0, 0.0, 0.0, 1.0], [0.1, 0.2, 0.3, 0.4], 7);
        let inst = ph2d_eval_motion::lower_to_instances(&tile);
        assert_eq!(inst.len(), 1);
        assert_eq!(inst[0].texture_id, 7, "which texture");
        assert_eq!(inst[0].size, [2.0, 3.0]);
        assert_eq!(inst[0].tint, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(inst[0].atlas_uv, [0.1, 0.2, 0.3, 0.4], "the atlas cell");
        assert_eq!(inst[0].world_pos, [0.0, 0.0], "at the origin — a template");
    }

    #[test]
    fn a_group_lays_its_children_out_relative_to_the_group() {
        // doc 86 §2 A4: a named group emits its subtree's leaves at their transform
        // RELATIVE to the group — the group's OWN pose is EXCLUDED (the tile is a
        // template stamped at each point's `P`). The walk composes down the chain,
        // so a nested subgroup lays out correctly. This is the load-bearing layout;
        // the per-medium appearance is the column gate above + the smoke.
        use ph2d_core::Vec2;
        use ph2d_ecs::ChildOf;
        let at = |x: f32, y: f32| Transform {
            translation: Vec2::new(x, y),
            ..Transform::IDENTITY
        };
        let mut sim = SimWorld::new();
        // The group carries its OWN translation (100,50) — it must NOT leak into
        // the children's layout.
        let group = sim
            .world_mut()
            .spawn((Name::new("Group"), GroupedChildren, at(100.0, 50.0)))
            .id();
        let c1 = sim.world_mut().spawn((at(2.0, 0.0), ChildOf(group))).id();
        let c2 = sim.world_mut().spawn((at(-2.0, 1.0), ChildOf(group))).id();
        // A nested subgroup at (5,0) with a grandchild at (1,0) ⇒ the grandchild
        // relative to the TOP group is (6,0).
        let sub = sim
            .world_mut()
            .spawn((GroupedChildren, at(5.0, 0.0), ChildOf(group)))
            .id();
        let gc = sim.world_mut().spawn((at(1.0, 0.0), ChildOf(sub))).id();

        let mut out: Vec<(Entity, Transform)> = Vec::new();
        walk_group_transforms(sim.world(), group, Transform::IDENTITY, &mut out);

        let pos = |e: Entity| {
            out.iter()
                .find(|(x, _)| *x == e)
                .map(|(_, t)| [t.translation.x, t.translation.y])
        };
        // ⚠️ If the walk folded the group's own transform in, the group would be at
        // (100,50) and c1 at (102,50) — these assertions are the mutation guard.
        assert_eq!(
            pos(group),
            Some([0.0, 0.0]),
            "the group's OWN pose is excluded"
        );
        assert_eq!(
            pos(c1),
            Some([2.0, 0.0]),
            "a direct child at its local position"
        );
        assert_eq!(pos(c2), Some([-2.0, 1.0]), "the second direct child");
        assert_eq!(
            pos(gc),
            Some([6.0, 0.0]),
            "a grandchild composed down the chain (5+1)"
        );
    }

    #[test]
    fn the_group_stream_lowers_to_one_instance_per_child() {
        // doc 86 §2 A4: the N-instance stream a group publishes, lowered by the
        // sink, yields ONE `RenderInstance` per child at its position + texture —
        // the two-doors coverage for the group path (the single-object version is
        // the column gate above). DISTINCT texture_ids (3, 9) that are neither the
        // atlas sentinel nor a coordinate, so a lowering that dropped the column
        // could not pass.
        let leaf = |p: [f32; 2], size: [f32; 2], tid: u32| LeafInstance {
            p,
            rot_deg: 0.0,
            size,
            tint: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0, 1.0, 1.0],
            tid,
        };
        let leaves = vec![
            leaf([-1.0, 0.0], [0.5, 0.5], 3),
            leaf([1.0, 0.5], [0.7, 0.7], 9),
        ];
        let inst = ph2d_eval_motion::lower_to_instances(&group_stream(&leaves));
        assert_eq!(inst.len(), 2, "one instance per child");
        assert_eq!(inst[0].world_pos, [-1.0, 0.0]);
        assert_eq!(inst[0].texture_id, 3);
        assert_eq!(inst[1].world_pos, [1.0, 0.5]);
        assert_eq!(inst[1].texture_id, 9);
        assert_eq!(inst[1].size, [0.7, 0.7], "each child keeps its own size");
    }
}

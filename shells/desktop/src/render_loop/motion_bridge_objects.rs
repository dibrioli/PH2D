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
    ChildOf, Children, Entity, FlipObjectRef, GroupedChildren, Name, SimWorld, Transform,
    VecPathRef, With,
};
use ph2d_eval_motion::VectorInstance;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_render::{RenderInstance, Sprite, SpriteSource, TextureAtlas};

// The LOD partition (the 160k freeze fix) lives in a sibling FILHO via `#[path]`
// so this parent stays under the shell LOC cap; the re-export keeps
// `objects::apply_object_lod` / `objects::LOD_COUNT` resolving unchanged.
#[path = "motion_bridge_objects_lod.rs"]
mod lod;
// O canal da aparência DESLOCADA (doc 89, folha 14) mora num irmão: ele é um
// ASSUNTO — *com que cara este objeto está em OUTRO tempo* — e não uma variação do
// publicador de agora. O pai fica com "o que a cena tem", o filho com "quando".
#[path = "motion_bridge_objects_shift.rs"]
mod shift;
pub(crate) use lod::{LOD_COUNT, apply_object_lod};
pub(crate) use shift::publish_shifted;
pub(crate) use shift::wanted_shifts;
// The faithful VectorInstance→tile conversion. ⚠️ **Era `cfg(test)`** — a partição
// usava-a internamente e só o gate a nomeava. Ganhou um consumidor de PRODUÇÃO em
// 2026-08-20: o [`super::super::motion_glow_layer`] deriva com ela a camada que o
// bright-pass do glow desenha, e as duas vistas TÊM de ser a mesma conversão (uma
// segunda cópia poria o halo noutro sítio que o tile do LOD).
pub(crate) use lod::vector_instance_as_tile;

use crate::motion_flip_bake::FlipTile;
use crate::motion_object_bake::ObjectVector;
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

/// The one-instance appearance stream for a LIVE VECTOR (a `source.object` that
/// names a vector, ADR-0154 reused for objects): `(P, size, tint, geometry_id)` —
/// no `uv_rect`/`texture_id`, because a live vector is drawn crisp by the vector
/// pass, not sampled as an atlas quad. The lowering routes a `geometry_id > 0` row
/// there automatically (its `> 0.5` split), and the sprite lowering SKIPS it, so a
/// mixed group stream draws each part once. The tint is WHITE — the drawing's own
/// fill/stroke carry its colours ([`ph2d_vec_render::draw_shape_instance`]).
fn appearance_vector(size: [f32; 2], tint: [f32; 4], geometry_id: u32) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![size]))
        .with("tint", Column::Vec4(vec![tint]))
        .with("geometry_id", Column::Scalar(vec![geometry_id as f32]))
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
/// **A metade VETORIAL do publicador de objetos** — um passo nomeado porque é ela
/// que disputa o nome com o publicador de curvas, e um gate tem de poder dirigi-la
/// sem um atlas (que é da metade dos SPRITES, e precisa de GPU).
pub(super) fn publish_vector_bakes(cook: &mut Cook, bakes: &crate::motion_object_bake::ObjectBake) {
    for (name, obj) in bakes.objects() {
        if super::shapes::is_reserved(name) {
            continue; // the editor's namespace (`motion_bridge_shapes::is_reserved`)
        }
        cook.set_external(
            name.to_string(),
            appearance_vector(obj.size, [1.0, 1.0, 1.0, 1.0], obj.geometry_id),
        );
    }
}

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
    let mut q = sim
        .world_mut()
        .query::<(&Sprite, &Name, Option<&ph2d_ecs::Transform>)>();
    let world = sim.world();
    for (spr, name, xf) in q.iter(world) {
        if name.0.trim().is_empty() {
            continue; // unnamed: nothing for the artist to type into the node
        }
        if super::shapes::is_reserved(&name.0) {
            continue; // the editor's namespace (`motion_bridge_shapes::is_reserved`)
        }
        if let Some(tile) = sprite_tile(spr, atlas) {
            cook.set_external(name.0.clone(), tile);
        }
        // ⚠️ **And WHERE it is, on its own channel.** The appearance stream above puts
        // `P` at the origin on purpose — it says what the object looks like, and the
        // graph decides where copies of it go. A node asking "where is the thing called
        // X" that read THAT `P` would aim at the origin for every object in the scene,
        // which is how `motion.look_at`'s Object mode shipped broken: a column with the
        // right name holding another question's answer.
        if let Some(t) = xf {
            cook.set_external(
                ph2d_nodegraph::external::position_of(&name.0),
                Stream::new(1).with(
                    "P",
                    Column::Vec2(vec![[t.translation[0], t.translation[1]]]),
                ),
            );
        }
    }

    // Vectors come as LIVE geometry (ADR-0154, reused for objects): the fx phase
    // parked each named shape in the vector store; here its `geometry_id` rides the
    // appearance stream and the vector pass draws it CRISP at any zoom (no more
    // fixed-DPI raster tile). The drawing carries its own colours, so the tint is
    // WHITE. ⚠️ A graph with a live vector recuses to the CPU cook — the GPU has no
    // `geometry_id` route (`motion_bridge_gpu::cook_publishes_live_geometry`).
    publish_vector_bakes(cook, bakes);

    // Flip objects come as BAKED tiles too (doc 86 §2 A3): the fx phase composed each
    // named object's layers at the current frame into an individual texture. Same
    // appearance stream, same WHITE tint (the colours are baked in), so the sink can't
    // tell a baked Flip from a baked vector — the `source.object` node stays media-
    // agnostic. Objects publish after curves; the last write on a name clash wins.
    for (name, tile) in flip_bakes.tiles() {
        if super::shapes::is_reserved(name) {
            continue;
        }
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
/// ⚠️ **An UNNAMED vector/flip child stamps too** (doc 86 §9.6 follow-up): a child is
/// resolved by its DRAWING id (`resolve_leaf`), and the bake tiles exactly the drawings
/// a named group references ([`entity_is_in_a_named_group`]), so a child needs no name.
/// A **v1 limit remains:** a child tile carries the orientation it was baked at (its world
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
/// relative to the group. The columns are exactly the ones the lowering reads
/// (`P`, `size`, `rot` in DEGREES, `tint`, `uv_rect`, `texture_id`, `geometry_id`).
/// ⚠️ A leaf is EITHER a raster quad (`tid > 0`, sprite/Flip) OR a live vector
/// (`gid > 0`, `source.object` of a vector) — never both; the lowering's
/// `geometry_id > 0.5` split routes each row to exactly one pass, so a group of
/// mixed media draws each child once.
struct LeafInstance {
    p: [f32; 2],
    rot_deg: f32,
    size: [f32; 2],
    tint: [f32; 4],
    uv: [f32; 4],
    tid: u32,
    gid: u32,
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

/// **Is `entity` inside a named GROUP's subtree?** (doc 86 §9.6 follow-up) — the bake
/// tiles a vector/flip drawing iff it is named OR this is true, so an unnamed group child
/// gets a tile without wasting one on unnamed canvas art. Walks UP the `ChildOf` chain
/// (bounded, the `container_of` precedent), which is the SAME tree relation
/// `group_externals` descends DOWN from every named group — a gate pins that the two
/// agree, so the set the bake tiles and the set the membrane stamps cannot diverge.
///
/// A "named group" is a `GroupedChildren` entity with a non-empty `Name` — exactly what
/// `group_externals` starts from. INCLUSIVE of `entity` itself, because a named group is
/// in its own subtree (`walk_group_transforms` pushes the group), so a group that also
/// carries geometry resolves as a leaf.
pub(crate) fn entity_is_in_a_named_group(world: &ph2d_ecs::World, entity: Entity) -> bool {
    const MAX_DEPTH: usize = 64;
    let mut cur = entity;
    for _ in 0..MAX_DEPTH {
        let named = world
            .get::<Name>(cur)
            .is_some_and(|n| !n.0.trim().is_empty());
        if named && world.get::<GroupedChildren>(cur).is_some() {
            return true;
        }
        match world.get::<ChildOf>(cur) {
            Some(c) => cur = c.parent(),
            None => break,
        }
    }
    false
}

/// Resolve ONE entity's appearance as a group leaf, at `acc` (its pose relative to
/// the group). Sprite → direct (atlas/individual); vector/flip → its A2/A3 tile by
/// its DRAWING id. `None` for a pure container (a group node) or an unresolvable leaf.
///
/// ⚠️ **The tile is keyed by the drawing's own id** (`VecPathRef`/`FlipObjectRef`,
/// the `VecPathId`/`FlipObjectId`), NOT by the entity's `Name` — so a group child
/// with **no name** still resolves (its drawing was baked under its id). The id is
/// also undo/rename-stable (unlike `Entity::to_bits`, which the bake never uses), so
/// the lookup survives a respawn. The bake's [`crate::motion_object_bake::select_present`]
/// bakes exactly the drawings a named group references, via
/// [`entity_is_in_a_named_group`] — the same tree relation this walk descends.
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
            gid: 0, // a sprite is a raster quad, not a live vector
        });
    }
    resolve_drawing_leaf(world, entity, acc, bakes, flip_bakes)
}

/// Resolve a vector/flip group child by its **drawing id** (its baked tile) — name-free,
/// so an unnamed child still stamps. Split from [`resolve_leaf`] so it is headless-testable:
/// the sprite branch needs the atlas, this one needs only the bakes (and the drawing id
/// off the `VecPathRef`/`FlipObjectRef`). `None` for a group container or a non-drawable.
fn resolve_drawing_leaf(
    world: &ph2d_ecs::World,
    entity: Entity,
    acc: &Transform,
    bakes: &crate::motion_object_bake::ObjectBake,
    flip_bakes: &crate::motion_flip_bake::FlipObjectBake,
) -> Option<LeafInstance> {
    if let Some(r) = world.get::<VecPathRef>(entity) {
        // A vector child stamps LIVE (`geometry_id`) — crisp at any zoom.
        Some(leaf_from_object(acc, bakes.vector_for_id(r.0)?))
    } else if let Some(r) = world.get::<FlipObjectRef>(entity) {
        // A Flip child stamps as its baked tile (`texture_id`) — no live path.
        Some(leaf_from_flip(acc, flip_bakes.tile_for_id(r.0)?))
    } else {
        None // a group container or an entity with no drawable appearance
    }
}

/// A live-vector leaf (`source.object` of a vector): the drawing rides `geometry_id`
/// (⚠️ baked at its world orientation — v1 limit), so `rot` is 0 and only position +
/// scale apply; `tid` is 0 so the sprite lowering skips it.
fn leaf_from_object(acc: &Transform, obj: ObjectVector) -> LeafInstance {
    LeafInstance {
        p: [acc.translation.x, acc.translation.y],
        rot_deg: 0.0,
        size: [
            obj.size[0] * acc.scale.x.abs(),
            obj.size[1] * acc.scale.y.abs(),
        ],
        tint: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 0.0, 1.0, 1.0], // unused for a geometry_id row
        tid: 0,
        gid: obj.geometry_id,
    }
}

/// A baked-tile Flip leaf: the tile carries the child's own orientation (⚠️ its
/// world bake — v1 limit), so `rot` is 0 and only position + scale apply; `gid` is
/// 0 so the vector lowering skips it.
fn leaf_from_flip(acc: &Transform, tile: FlipTile) -> LeafInstance {
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
        gid: 0,
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
        // The live-vector column: `> 0` for a vector child, `0` for a sprite/Flip.
        // The lowering's `geometry_id > 0.5` split routes each row to exactly one
        // pass, so a mixed group draws each child once (never a blank atlas quad).
        .with(
            "geometry_id",
            Column::Scalar(leaves.iter().map(|l| l.gid as f32).collect()),
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
    // ⚠️ **DEPOIS de tudo, e a ordem é load-bearing:** o canal deslocado começa por
    // COPIAR o que o nome cru acabou de publicar, então ele tem de correr quando esse
    // canal já existe. Rodá-lo antes daria um external vazio, e um `time_offset` num
    // sprite faria o objeto SUMIR — que é a forma mais cara de um param novo estar
    // errado, porque ela parece a feature funcionando em outro objeto.
    publish_shifted(
        &mut motion.pump.cook,
        &motion.doc.graph,
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
    renderer: &mut ph2d_render::SpriteRenderer,
    surface_format: wgpu::TextureFormat,
    sim: &SimWorld,
) {
    // `object_bake` and `shape_store` are disjoint fields of `MotionState`: the store
    // the vector `encode` reads is where the live geometry is parked (→ `geometry_id`),
    // so there is ONE store for `source.shape` AND `source.object` vectors. `renderer`
    // is where the LOD tile is uploaded (→ `texture_id`), refcounted.
    let MotionState {
        object_bake,
        shape_store,
        ..
    } = motion;
    object_bake.bake(
        shape_store,
        scene,
        map,
        xforms,
        live,
        gpu,
        renderer,
        surface_format,
        sim,
    );
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
    let shifts = wanted_shifts(&motion.doc.graph);
    motion
        .flip_object_bake
        .bake(flip, map, playhead, &shifts, gpu, renderer, sim);
}

#[cfg(test)]
#[path = "motion_bridge_objects_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_bridge_objects_shift_tests.rs"]
mod shift_tests;

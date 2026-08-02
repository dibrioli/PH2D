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

use ph2d_ecs::{Name, SimWorld};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_render::{RenderInstance, Sprite, SpriteSource, TextureAtlas};

/// The appearance tile for one sprite: one instance at the origin carrying
/// `(P, size, tint, uv_rect, texture_id)`. `None` for a source the atlas cannot
/// resolve here (a cooked KTX2 sprite needs the renderer's cooked-texture store,
/// not in hand — deferred to a later wave; it is skipped, not guessed).
fn sprite_tile(spr: &Sprite, atlas: &TextureAtlas) -> Option<Stream> {
    let (uv_rect, texture_id) = match spr.source {
        // Atlas → the packed cell's UV, sampling the shared atlas (`0`); the
        // cheap direct path (no bake) the sprite renderer already uses.
        SpriteSource::Atlas { key } => (atlas.region_uv(key), RenderInstance::ATLAS_TEXTURE_ID),
        // Individual → the full unit rect + the store handle it already carries.
        SpriteSource::Individual { texture_id } => ([0.0, 0.0, 1.0, 1.0], texture_id),
        // Cooked KTX2 resolves through `renderer.cooked_texture_id`, which this
        // pass does not hold — skip (invisible), the same shape a not-yet-
        // uploaded cooked sprite takes in the extract.
        SpriteSource::CookedTexture { .. } => return None,
    };
    // `collapsed_tint` = self_tint × tint (the per-sprite modulate). The
    // inherited ancestor cascade the extract folds in is a refinement of a
    // template, deferred: a source is *this object's* appearance.
    Some(appearance_tile(spr.size, spr.collapsed_tint(), uv_rect, texture_id))
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
            appearance_tile(tile.size, [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], tile.texture_id),
        );
    }
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
}

//! Sim tick + extract phase.
//!
//! Wave 3.2 stage A — extracted from `render_loop::mod.rs` as a free
//! function. Runs the M5 demo's bouncing-motion sim tick, then the
//! ADR-0021 / ADR-0025 extract pass that propagates Transforms +
//! emits `RenderInstance`s into PresentWorld. Behavior-preserving
//! lift.
//!
//! HR-3: `worklist`'s capacity is reused across frames so this hot
//! path stays zero-alloc after warm-up (`tests/propagate_no_alloc.rs`).

use crate::{Velocity, WORLD_HALF};
use ph2d_ecs::{
    PresentWorld, SimRef, SimWorld, Transform, TransformPropagationState, WorklistBuf,
    propagate_transforms,
};
use ph2d_render::{RenderInstance, Sprite, SpriteRenderer};

/// Per-frame override that swaps a sprite entity's texture binding
/// for a transient one — used by the BG-Removal live preview (Lens F,
/// 2026-05-26) so the preview pixels render through the SAME sprite
/// pipeline (`Rgba8UnormSrgb` + `sprite.wgsl` + premul blend) as the
/// Apply bake. Replaces the previous Vello-overlay path, which
/// diverged from Apply in gamma + blend space and produced a visible
/// halo at edge pixels.
///
/// When the extract pass emits a `RenderInstance` for `entity_bits`,
/// it substitutes `texture_id` + `premultiplied` while leaving every
/// other instance field (world position, size, rotation, anchor,
/// tint, z_order) intact — so the override paints the SAME quad the
/// source sprite would have, but sampling from the preview texture.
#[derive(Copy, Clone, Debug)]
pub(crate) struct PreviewOverride {
    pub entity_bits: u64,
    pub texture_id: u32,
    pub premultiplied: bool,
}

/// Sim tick → extract pass. Caller provides the destructured
/// `AppGfx` refs.
pub(super) fn run(
    dt: f32,
    sim: &mut SimWorld,
    present: &mut PresentWorld,
    renderer: &SpriteRenderer,
    prop_state: &mut TransformPropagationState,
    worklist: &mut WorklistBuf,
    // Tool live-preview override: when `Some`, the entity's
    // `RenderInstance` is emitted with `texture_id` + `premultiplied`
    // replaced. `None` = emit every sprite from its own `Sprite`
    // source.
    preview_override: Option<PreviewOverride>,
) {
    // Sim tick: bouncing motion. Single substep per frame for the
    // M5 demo (we don't yet honor the FixedStep substep count for
    // gameplay — that lands in M10 with the physics integrator).
    {
        let mut q = sim.world_mut().query::<(&mut Transform, &mut Velocity)>();
        for (mut t, mut vel) in q.iter_mut(sim.world_mut()) {
            let mut p = t.translation;
            let mut v = vel.0;
            p += v * dt;
            if p.x.abs() > WORLD_HALF {
                v.x = -v.x;
                p.x = p.x.clamp(-WORLD_HALF, WORLD_HALF);
            }
            if p.y.abs() > WORLD_HALF {
                v.y = -v.y;
                p.y = p.y.clamp(-WORLD_HALF, WORLD_HALF);
            }
            t.translation = p;
            vel.0 = v;
        }
    }

    // Extract (ADR-0021 + ADR-0025): hierarchical Transform →
    // GlobalTransform propagation plus per-entity sprite emit.
    // `propagate_transforms` walks the `ChildOf` tree once, and the
    // closure spawns one mirror entity per sim entity in PresentWorld
    // carrying `(SimRef, GlobalTransform)` plus an optional
    // `RenderInstance` for sprite-bearing entities.
    let atlas = renderer.atlas();
    present.world_mut().clear_entities();
    ph2d_ecs::extract!(*sim => *present, |sim_w, present_w| {
        // Sequential `z_order` assigned in `propagate_transforms`
        // traversal order (DFS of the ChildOf tree, with roots
        // collated by `RootOrder`). Stamping the counter onto every
        // `RenderInstance` lets the renderer sort by `(z_order,
        // texture_id)` so the visual order mirrors the Hierarchy
        // panel — without it, a Color-EQ/BgRemoval/Padding bake that
        // promotes an Atlas sprite (texture_id=0) to Individual
        // (>0) would silently float to the front of the scene.
        let mut z_counter: u32 = 0;
        propagate_transforms(
            sim_w,
            prop_state,
            present_w,
            worklist,
            |sim, present, sim_entity, gt| {
                let mut builder = present.spawn((SimRef(sim_entity), gt));
                // M14.6A: respect the Visibility component (eye
                // toggle in the Hierarchy panel). Absence of the
                // component = visible by default.
                let hidden = sim
                    .get::<ph2d_ecs::Visibility>(sim_entity)
                    .is_some_and(|v| v.hidden);
                let override_for_entity =
                    preview_override.filter(|o| o.entity_bits == sim_entity.to_bits());
                if !hidden
                    && let Some(spr) = sim.get::<Sprite>(sim_entity)
                {
                    let p = gt.translation();
                    // M14.7 polish: extract scale + rotation from
                    // the entity's `GlobalTransform` matrix so the
                    // gizmo's scale handles AND rotation reach the
                    // shader. Column-major affine:
                    //   col0 = (cos*sx, sin*sx)
                    //   col1 = (-sin*sy, cos*sy)
                    //   col2 = (tx, ty)
                    // Scale magnitudes come from column lengths;
                    // rotation comes from atan2(col0.y, col0.x).
                    // The Sprite's raw `size` is the import-time
                    // world rect; multiplying here keeps the gizmo
                    // pipeline orthogonal to the import pipeline
                    // (no double-scaling).
                    let affine = gt.affine();
                    let col0_x = affine[0];
                    let col0_y = affine[1];
                    let col1_x = affine[2];
                    let col1_y = affine[3];
                    let scale_x = (col0_x * col0_x + col0_y * col0_y).sqrt();
                    let scale_y = (col1_x * col1_x + col1_y * col1_y).sqrt();
                    let rotation = col0_y.atan2(col0_x);
                    // M14.5 C: branch on the sprite source. Atlas
                    // sprites resolve UV via `region_uv`; individual
                    // sprites use the full (0..1) UV rect and carry
                    // the renderer-side texture_id so the batcher
                    // can pick the right bind group at draw time.
                    let (atlas_uv, texture_id) = match spr.source {
                        ph2d_render::SpriteSource::Atlas { key } => (
                            atlas.region_uv(key),
                            ph2d_render::RenderInstance::ATLAS_TEXTURE_ID,
                        ),
                        ph2d_render::SpriteSource::Individual { texture_id } => {
                            ([0.0, 0.0, 1.0, 1.0], texture_id)
                        }
                    };
                    // Lens F (2026-05-26): if a tool's live preview
                    // claims this entity, substitute the texture binding
                    // for the preview's transient Individual texture +
                    // its premultiplied flag. UV stays the full unit
                    // rect (preview textures are atlas-free); every
                    // other instance field (transform, size, anchor,
                    // tint, z) is identical so the override paints the
                    // SAME quad the source sprite would have.
                    let (atlas_uv, texture_id, premultiplied_flag) =
                        if let Some(ov) = override_for_entity {
                            ([0.0, 0.0, 1.0, 1.0], ov.texture_id, ov.premultiplied)
                        } else {
                            (atlas_uv, texture_id, spr.premultiplied)
                        };
                    let z_order = z_counter;
                    z_counter += 1;
                    // Sprite-Inspector-v2 v4 channel collapse (W1.T1.8/T1.10,
                    // anatomia §4.2/§4.3). `collapsed_tint` = self_tint × tint
                    // (per-component); the per-sprite collapse logic + its
                    // unit test live in ph2d-render. The ancestor modulate
                    // product Π(ancestors.tint) is NOT folded — that needs a
                    // GlobalTint propagation pass (does not exist yet) and is
                    // W2 work (the 3-level smoke_w2_color_tint.scene validates
                    // it). For W1, self_tint defaults WHITE → identity → render
                    // unchanged. per_corner_tint + opacity passthrough.
                    let cascade_tint = spr.collapsed_tint();
                    // Packed flip/fill flags (ADR-0070-amendment-3): bit0=flip_x,
                    // bit1=flip_y, bit2=tint_fill. Encoded via the canonical
                    // helper so the bit layout stays single-sourced with the
                    // WGSL decode. All default false → 0 → no-op.
                    let flip_uv = ph2d_render::RenderInstance::pack_flip_flags(
                        spr.flip_x,
                        spr.flip_y,
                        spr.tint_fill,
                    );
                    builder.insert(RenderInstance {
                        world_pos: [p.x, p.y],
                        size: [spr.size[0] * scale_x, spr.size[1] * scale_y],
                        atlas_uv,
                        tint: cascade_tint,
                        rotation,
                        texture_id,
                        // Flag the BG-Removal-baked premultiplied texture
                        // so the fragment skips its post-sample premultiply
                        // (fringe fix). Straight for every other sprite.
                        premultiplied: if premultiplied_flag { 1.0 } else { 0.0 },
                        // Pivot offset: scale the intrinsic-local anchor
                        // by the same `GlobalTransform` scale as `size`,
                        // so the shader's `anchor + quad*size` stays in
                        // one consistent (world-scaled) local frame.
                        anchor: [spr.anchor[0] * scale_x, spr.anchor[1] * scale_y],
                        per_corner_tint: spr.per_corner_tint,
                        opacity: spr.opacity,
                        flip_uv,
                        z_order,
                    });
                }
            },
        );
    });
}

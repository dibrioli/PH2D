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

/// Sim tick → extract pass. Caller provides the destructured
/// `AppGfx` refs.
pub(super) fn run(
    dt: f32,
    sim: &mut SimWorld,
    present: &mut PresentWorld,
    renderer: &SpriteRenderer,
    prop_state: &mut TransformPropagationState,
    worklist: &mut WorklistBuf,
    // Entity (`to_bits`) to suppress from the sprite emit this frame.
    // Used by the Background-Removal live preview: while the tool shows
    // its on-canvas preview overlay, the *original* sprite must not
    // render underneath, otherwise the preview's transparent (removed)
    // regions would reveal the untouched original instead of the canvas
    // backdrop. `None` = render everything.
    exclude_entity: Option<u64>,
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
                let suppressed = exclude_entity == Some(sim_entity.to_bits());
                if !hidden
                    && !suppressed
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
                    builder.insert(RenderInstance {
                        world_pos: [p.x, p.y],
                        size: [spr.size[0] * scale_x, spr.size[1] * scale_y],
                        atlas_uv,
                        tint: spr.tint,
                        rotation,
                        texture_id,
                        _pad: [0; 2],
                    });
                }
            },
        );
    });
}

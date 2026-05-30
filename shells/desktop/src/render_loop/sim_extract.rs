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

/// Select the sprite-sheet cell `frame` from a base UV rect
/// `[u_min, v_min, u_max, v_max]`, dividing it into an `hframes × vframes`
/// grid (anatomia §03 §3.4). Frame 0 = top-left, `col = frame % hframes`,
/// `row = frame / hframes` (row increases downward, matching V=0 = top).
/// `hframes`/`vframes` floor at 1 and `frame` is clamped into the grid,
/// so the default 1×1 sheet returns the input rect unchanged (no-op for
/// every legacy sprite). Render-only (PresentWorld), HR-5 exempt.
fn sprite_sheet_subrect(uv: [f32; 4], hframes: u32, vframes: u32, frame: u32) -> [f32; 4] {
    let hf = hframes.max(1);
    let vf = vframes.max(1);
    if hf == 1 && vf == 1 {
        return uv;
    }
    let cells = hf.saturating_mul(vf).max(1);
    let frame = frame.min(cells - 1);
    let col = frame % hf;
    let row = frame / hf;
    let [u0, v0, u1, v1] = uv;
    let cw = (u1 - u0) / hf as f32;
    let ch = (v1 - v0) / vf as f32;
    let nu0 = u0 + col as f32 * cw;
    let nv0 = v0 + row as f32 * ch;
    [nu0, nv0, nu0 + cw, nv0 + ch]
}

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
                    // ADR-0070-amendment-4: pass the FULL 2x2 world basis
                    // (col0, col1) to the shader instead of decomposing it
                    // to atan2(col0) + per-column scale. The old
                    // decomposition collapsed any skew (a non-orthogonal
                    // basis) into a rotated rectangle — skew read as
                    // rotation + stretched scale. The basis carries
                    // rotation + scale + skew EXACTLY, and the shader maps
                    // the local quad through it (sheared parallelogram).
                    // `size`/`anchor` stay LOCAL (the basis applies scale),
                    // so no double-scaling. Column-major affine:
                    //   col0 = basis.xy (x axis), col1 = basis.zw (y axis).
                    let affine = gt.affine();
                    let basis = [affine[0], affine[1], affine[2], affine[3]];
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
                    // Sprite-sheet sub-UV (anatomia §03 §3.4): divide the
                    // base atlas_uv rect into an hframes×vframes grid and
                    // select `frame`'s cell. The default 1×1 grid is a
                    // no-op, so legacy sprites render unchanged. Skipped
                    // under a tool preview override (audit E-3): the
                    // transient preview texture is a full-frame bake, not
                    // a sheet, so slicing it would show only one cell.
                    let atlas_uv = if override_for_entity.is_some() {
                        atlas_uv
                    } else {
                        sprite_sheet_subrect(atlas_uv, spr.hframes, spr.vframes, spr.frame)
                    };
                    builder.insert(RenderInstance {
                        world_pos: [p.x, p.y],
                        // LOCAL size — the basis applies world scale.
                        size: spr.size,
                        atlas_uv,
                        tint: cascade_tint,
                        basis,
                        texture_id,
                        // Flag the BG-Removal-baked premultiplied texture
                        // so the fragment skips its post-sample premultiply
                        // (fringe fix). Straight for every other sprite.
                        premultiplied: if premultiplied_flag { 1.0 } else { 0.0 },
                        // Pivot offset in LOCAL meters — the basis maps it
                        // to world along with the quad corners, so the quad
                        // orbits `world_pos` (the pivot) under skew too.
                        anchor: spr.anchor,
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

#[cfg(test)]
mod sprite_sheet_tests {
    use super::sprite_sheet_subrect;

    const FULL: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    #[test]
    fn default_grid_is_identity() {
        assert_eq!(sprite_sheet_subrect(FULL, 1, 1, 0), FULL);
        // Zero counts floor to 1 → still identity.
        assert_eq!(sprite_sheet_subrect(FULL, 0, 0, 5), FULL);
    }

    #[test]
    fn two_by_two_selects_cells() {
        // frame 0 = top-left, 1 = top-right, 2 = bottom-left, 3 = bottom-right.
        assert_eq!(sprite_sheet_subrect(FULL, 2, 2, 0), [0.0, 0.0, 0.5, 0.5]);
        assert_eq!(sprite_sheet_subrect(FULL, 2, 2, 1), [0.5, 0.0, 1.0, 0.5]);
        assert_eq!(sprite_sheet_subrect(FULL, 2, 2, 2), [0.0, 0.5, 0.5, 1.0]);
        assert_eq!(sprite_sheet_subrect(FULL, 2, 2, 3), [0.5, 0.5, 1.0, 1.0]);
    }

    #[test]
    fn frame_past_grid_clamps_to_last_cell() {
        assert_eq!(sprite_sheet_subrect(FULL, 2, 2, 99), [0.5, 0.5, 1.0, 1.0]);
    }

    #[test]
    fn subrect_respects_a_non_unit_base_rect() {
        // An atlas region [0.2, 0.4, 0.6, 0.8] split 2×1 → left half.
        let base = [0.2, 0.4, 0.6, 0.8];
        let left = sprite_sheet_subrect(base, 2, 1, 0);
        assert!((left[0] - 0.2).abs() < 1e-6 && (left[2] - 0.4).abs() < 1e-6);
        assert!((left[1] - 0.4).abs() < 1e-6 && (left[3] - 0.8).abs() < 1e-6);
    }
}

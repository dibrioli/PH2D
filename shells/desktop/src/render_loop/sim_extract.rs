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
    ChildOf, Entity, PresentWorld, SimRef, SimWorld, Transform, TransformPropagationState,
    WorklistBuf, World, propagate_transforms,
};
use ph2d_render::{RenderInstance, Sprite, SpriteRenderer};

/// Render tint folding the ancestor modulate chain (anatomia §4.3):
/// `self_tint × tint × Π(ancestor.tint)`. Each ancestor contributes its
/// `tint` — the inheriting "modulate" (Godot) — NOT its `self_tint`,
/// which is local-only. Non-sprite ancestors (empty parents, pure
/// transforms) contribute nothing. Walks `ChildOf` bottom-up (O(depth));
/// the common no-parent case is one component miss + the per-sprite
/// `collapsed_tint`. No allocation (HR-3). Alpha cascades too (spec §4.4
/// `Π(modulate_ancestors.a)`); `opacity` stays per-sprite and is NOT
/// folded here.
fn cascade_tint_with_ancestors(world: &World, entity: Entity, sprite: &Sprite) -> [f32; 4] {
    let mut tint = sprite.collapsed_tint(); // self_tint × own tint
    let mut cur = world.get::<ChildOf>(entity).map(|c| c.parent());
    while let Some(parent) = cur {
        if let Some(ps) = world.get::<Sprite>(parent) {
            tint[0] *= ps.tint[0];
            tint[1] *= ps.tint[1];
            tint[2] *= ps.tint[2];
            tint[3] *= ps.tint[3];
        }
        cur = world.get::<ChildOf>(parent).map(|c| c.parent());
    }
    tint
}

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

/// Narrow a UV rect to a sprite's pixel-space `region_rect` (anatomia
/// §03 §3.5). `region` is `[x, y, w, h]` in SOURCE pixels; `(src_w,
/// src_h)` are the source image's pixel dimensions, so the rect maps to
/// the fraction `region / src` of the base `uv`. A zero/negative region
/// or unknown source dims leaves `uv` untouched (region = no-op). When
/// `filter_clip_half_texel` is `Some((htu, htv))`, the result is inset
/// by half a texel per side (Godot `region_filter_clip`) so bilinear
/// sampling can't bleed past the region edge into neighbouring atlas
/// content. The `htu`/`htv` are in the SAMPLED texture's UV space.
fn region_subrect(
    uv: [f32; 4],
    region: [f32; 4],
    src_w: f32,
    src_h: f32,
    filter_clip_half_texel: Option<(f32, f32)>,
) -> [f32; 4] {
    let [u0, v0, u1, v1] = uv;
    let [rx, ry, rw, rh] = region;
    if rw <= 0.0 || rh <= 0.0 || src_w <= 0.0 || src_h <= 0.0 {
        return uv;
    }
    let du = u1 - u0;
    let dv = v1 - v0;
    // Region beyond the source edges is clamped to the base rect.
    let mut nu0 = (u0 + du * (rx / src_w)).clamp(u0, u1);
    let mut nv0 = (v0 + dv * (ry / src_h)).clamp(v0, v1);
    let mut nu1 = (u0 + du * ((rx + rw) / src_w)).clamp(u0, u1);
    let mut nv1 = (v0 + dv * ((ry + rh) / src_h)).clamp(v0, v1);
    if let Some((htu, htv)) = filter_clip_half_texel {
        nu0 += htu;
        nv0 += htv;
        nu1 -= htu;
        nv1 -= htv;
        // A region thinner than one texel would invert under the inset;
        // collapse it to its center so the sample stays inside.
        if nu1 < nu0 {
            let m = 0.5 * (nu0 + nu1);
            nu0 = m;
            nu1 = m;
        }
        if nv1 < nv0 {
            let m = 0.5 * (nv0 + nv1);
            nv0 = m;
            nv1 = m;
        }
    }
    [nu0, nv0, nu1, nv1]
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
#[allow(clippy::too_many_arguments)]
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
    // Project pixels-per-meter — converts a sprite's intrinsic-px
    // `offset` into the LOCAL meters `resolve_anchor` works in.
    pixels_per_meter: f32,
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
                    // anatomia §4.2/§4.3): `self_tint × tint × Π(ancestor.tint)`.
                    // The per-sprite `collapsed_tint` (self_tint × tint) lives +
                    // is unit-tested in ph2d-render; `cascade_tint_with_ancestors`
                    // folds in the inherited modulate chain (W2 — so a parent's
                    // `tint` actually tints its children, while `self_tint` does
                    // not). Skipped under a preview override (the override emits
                    // the SAME tint as the source). per_corner_tint + opacity
                    // pass through unchanged.
                    let cascade_tint = cascade_tint_with_ancestors(sim, sim_entity, spr);
                    // Packed flip/fill flags (ADR-0070-amendment-3): bit0=flip_x,
                    // bit1=flip_y, bit2=tint_fill. Encoded via the canonical
                    // helper so the bit layout stays single-sourced with the
                    // WGSL decode. All default false → 0 → no-op.
                    let flip_uv = ph2d_render::RenderInstance::pack_flip_flags(
                        spr.flip_x,
                        spr.flip_y,
                        spr.tint_fill,
                    );
                    // Region sub-UV (anatomia §03 §3.5): when enabled,
                    // narrow the base rect to `region_rect` (source pixels
                    // → UV). Applied BEFORE the sheet grid so the grid
                    // divides the region, not the whole source. Skipped
                    // under a tool preview override (the preview texture is
                    // a full-frame bake whose pixel space doesn't match the
                    // authored source's `region_rect`). Default
                    // `region_enabled = false` → no-op for legacy sprites.
                    let atlas_uv = if spr.region_enabled && override_for_entity.is_none() {
                        let src_dims = match spr.source {
                            ph2d_render::SpriteSource::Atlas { key } => atlas.region_px(key),
                            ph2d_render::SpriteSource::Individual { texture_id } => {
                                renderer.individual().dims(texture_id)
                            }
                        };
                        match src_dims {
                            Some((sw, sh)) => {
                                // Half-texel size in the SAMPLED texture's UV
                                // space: atlas sprites sample the shared atlas
                                // (1 / size_px); individual sprites sample
                                // their own texture (1 / w, 1 / h).
                                let half_texel = if spr.region_filter_clip {
                                    let (tw, th) = match spr.source {
                                        ph2d_render::SpriteSource::Atlas { .. } => {
                                            let s = atlas.size_px.max(1) as f32;
                                            (s, s)
                                        }
                                        ph2d_render::SpriteSource::Individual { .. } => {
                                            (sw.max(1) as f32, sh.max(1) as f32)
                                        }
                                    };
                                    Some((0.5 / tw, 0.5 / th))
                                } else {
                                    None
                                };
                                region_subrect(
                                    atlas_uv,
                                    spr.region_rect,
                                    sw as f32,
                                    sh as f32,
                                    half_texel,
                                )
                            }
                            None => atlas_uv,
                        }
                    } else {
                        atlas_uv
                    };
                    // Sprite-sheet sub-UV (anatomia §03 §3.4): divide the
                    // (possibly region-narrowed) atlas_uv rect into an
                    // hframes×vframes grid and select `frame`'s cell. The
                    // default 1×1 grid is a no-op, so legacy sprites render
                    // unchanged. Skipped under a tool preview override
                    // (audit E-3): the transient preview texture is a
                    // full-frame bake, not a sheet, so slicing it would
                    // show only one cell.
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
                        // Effective pivot offset in LOCAL meters — folds
                        // the Godot `centered`/`offset` authoring onto the
                        // tool `anchor` (resolve_anchor). The basis maps it
                        // to world with the quad corners, so the quad orbits
                        // `world_pos` (the pivot) under skew too. Default
                        // (centered, offset 0, anchor 0) = [0,0] (legacy).
                        anchor: spr.resolve_anchor(pixels_per_meter),
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

#[cfg(test)]
mod region_subrect_tests {
    use super::region_subrect;

    const FULL: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    fn close(a: [f32; 4], b: [f32; 4]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6)
    }

    #[test]
    fn zero_or_negative_region_is_identity() {
        // A degenerate (zero-area) region is treated as "no region".
        assert_eq!(
            region_subrect(FULL, [0.0, 0.0, 0.0, 0.0], 64.0, 64.0, None),
            FULL
        );
        assert_eq!(
            region_subrect(FULL, [10.0, 10.0, -5.0, 8.0], 64.0, 64.0, None),
            FULL
        );
        // Unknown source dims (0) is also identity.
        assert_eq!(
            region_subrect(FULL, [0.0, 0.0, 32.0, 32.0], 0.0, 0.0, None),
            FULL
        );
    }

    #[test]
    fn full_source_region_is_identity() {
        // region == whole source → the base rect, unchanged.
        assert!(close(
            region_subrect(FULL, [0.0, 0.0, 100.0, 100.0], 100.0, 100.0, None),
            FULL
        ));
    }

    #[test]
    fn right_half_region_maps_to_right_half_uv() {
        // 100×100 source, region = right half → U in [0.5, 1.0].
        assert!(close(
            region_subrect(FULL, [50.0, 0.0, 50.0, 100.0], 100.0, 100.0, None),
            [0.5, 0.0, 1.0, 1.0]
        ));
    }

    #[test]
    fn region_maps_into_a_non_unit_atlas_rect() {
        // The source occupies atlas-UV [0.2, 0.2, 0.6, 0.6] (a 0.4×0.4
        // patch); a centered quarter region [25,25,50,50] of the 100px
        // source → the middle 0.2×0.2 of that patch = [0.3, 0.3, 0.5, 0.5].
        let base = [0.2, 0.2, 0.6, 0.6];
        assert!(close(
            region_subrect(base, [25.0, 25.0, 50.0, 50.0], 100.0, 100.0, None),
            [0.3, 0.3, 0.5, 0.5]
        ));
    }

    #[test]
    fn region_beyond_source_edges_is_clamped() {
        // A region wider than the source can't sample past the base rect.
        let r = region_subrect(FULL, [0.0, 0.0, 200.0, 200.0], 100.0, 100.0, None);
        assert!(close(r, FULL));
    }

    #[test]
    fn filter_clip_insets_by_half_a_texel() {
        // 64px atlas texel = 1/64; half-texel = 1/128. Right-half region
        // [0.5,0,1,1] inset by 1/128 on every side.
        let ht = 0.5 / 64.0;
        let r = region_subrect(FULL, [32.0, 0.0, 32.0, 64.0], 64.0, 64.0, Some((ht, ht)));
        assert!((r[0] - (0.5 + ht)).abs() < 1e-6, "u0 {r:?}");
        assert!((r[2] - (1.0 - ht)).abs() < 1e-6, "u1 {r:?}");
        assert!((r[1] - ht).abs() < 1e-6, "v0 {r:?}");
        assert!((r[3] - (1.0 - ht)).abs() < 1e-6, "v1 {r:?}");
    }

    #[test]
    fn sub_texel_region_collapses_to_center_under_filter_clip() {
        // A 1px-wide region on a 64px source spans 1/64 in U; insetting by
        // a half-texel (1/128) each side would invert → collapse to center.
        let ht = 0.5 / 64.0;
        let r = region_subrect(FULL, [10.0, 0.0, 1.0, 64.0], 64.0, 64.0, Some((ht, ht)));
        assert!((r[0] - r[2]).abs() < 1e-6, "U collapsed to a point: {r:?}");
        // Center sits at 10.5/64 in U.
        assert!((r[0] - 10.5 / 64.0).abs() < 1e-6, "{r:?}");
    }
}

#[cfg(test)]
mod cascade_tint_tests {
    use super::cascade_tint_with_ancestors;
    use ph2d_ecs::{ChildOf, SimWorld};
    use ph2d_render::Sprite;

    /// Root(tint A, self_tint B) → child(white) → grandchild(tint C).
    /// Render tint = self_tint × tint × Π(ancestor.tint); an ancestor
    /// contributes its `tint` (modulate, cascades) but NOT its `self_tint`
    /// (local) — the exact distinction Enio's smoke flagged as missing.
    #[test]
    fn cascade_folds_ancestor_modulate_not_self_modulate() {
        let mut sim = SimWorld::new();
        // Root: tint = half-red, self_tint = half-green (local only).
        let mut root_s = Sprite::atlas(0, [1.0, 1.0], [0.5, 1.0, 1.0, 1.0]);
        root_s.self_tint = [1.0, 0.5, 1.0, 1.0];
        let root = sim.world_mut().spawn(root_s).id();
        let child = sim
            .world_mut()
            .spawn((
                Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
                ChildOf(root),
            ))
            .id();
        let grandchild = sim
            .world_mut()
            .spawn((
                Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 0.5, 1.0]),
                ChildOf(child),
            ))
            .id();

        let w = sim.world();
        let cascade = |e| {
            let s = w.get::<Sprite>(e).unwrap();
            cascade_tint_with_ancestors(w, e, s)
        };
        // Root: self_tint × tint, no ancestors.
        assert_eq!(cascade(root), [0.5, 0.5, 1.0, 1.0]);
        // Child: root's TINT (half-red) cascades; root's SELF_TINT
        // (half-green) does NOT — so green stays 1.0. THE key assertion.
        assert_eq!(cascade(child), [0.5, 1.0, 1.0, 1.0]);
        // Grandchild: own tint (half-blue) × child (white) × root tint.
        assert_eq!(cascade(grandchild), [0.5, 1.0, 0.5, 1.0]);
    }

    #[test]
    fn root_sprite_cascade_equals_per_sprite_collapse() {
        // No parent → identical to the old `collapsed_tint` (zero
        // regression for every root sprite).
        let mut sim = SimWorld::new();
        let mut s = Sprite::atlas(0, [1.0, 1.0], [0.4, 0.5, 0.6, 0.7]);
        s.self_tint = [0.5, 0.5, 1.0, 0.8];
        let e = sim.world_mut().spawn(s).id();
        let w = sim.world();
        assert_eq!(
            cascade_tint_with_ancestors(w, e, w.get::<Sprite>(e).unwrap()),
            w.get::<Sprite>(e).unwrap().collapsed_tint()
        );
    }
}

//! Snapshot publication phase — once per frame, before paint.
//!
// ph2d-loc-cap: accreted one producer per W3 Inspector section
// (sprite/transform/visibility/ordering/sampling/name + §8 visibility-
// section). Was already AT the 600-LOC ceiling before §8; +7 LOC for the
// §8 producer tips it. Follow-up: lift the per-section producers into
// their sibling `inspector_*` modules (build_* already live there) and
// leave this file as the thin publish orchestrator.
//!
//! Wave 3.2 stage A — extracted from `render_loop::mod.rs` as a free
//! function taking explicit refs to the destructured `AppGfx` fields
//! it needs. Behavior-preserving lift.
//!
//! Publishes the live hierarchy snapshot, grid view, telemetry stats,
//! gizmo projection, and 4 inspector snapshots
//! (sprite/transform/visibility/name) onto the `HeroScreen` so the
//! subsequent paint pass reads them via the HR-8 / ADR-0021 boundary
//! (Inspector never reads SimWorld directly).

use crate::HeroLive;
use ph2d_asset::AssetDb;
use ph2d_asset::AssetId;
use ph2d_ecs::{Name, PresentWorld, SimRef, SimWorld, Transform, Visibility};
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite};
use std::collections::BTreeMap;

/// BulkSelect (T2.0): compute which editable `Sprite` fields diverge
/// across the `selected` entities, relative to `primary`. Exact equality
/// is intentional — "Mixed" means the stored values literally differ, so
/// editing the field would stomp the divergence. `selected` includes the
/// primary (a no-op self-compare); unknown / non-sprite entities are
/// skipped. Returns all-`false` for a single selection.
#[allow(clippy::float_cmp)] // exact compare: same stored value = not mixed
fn compute_sprite_mixed(
    world: &ph2d_ecs::World,
    selected: &[u64],
    primary: &Sprite,
) -> ph2d_editor::InspectorSpriteMixed {
    let mut m = ph2d_editor::InspectorSpriteMixed::default();
    for &bits in selected {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let Some(s) = world.get::<Sprite>(entity) else {
            continue;
        };
        m.flip_x |= s.flip_x != primary.flip_x;
        m.flip_y |= s.flip_y != primary.flip_y;
        m.tint_fill |= s.tint_fill != primary.tint_fill;
        m.centered |= s.centered != primary.centered;
        m.region_enabled |= s.region_enabled != primary.region_enabled;
        m.region_filter_clip |= s.region_filter_clip != primary.region_filter_clip;
        m.opacity |= s.opacity != primary.opacity;
        m.hframes |= s.hframes != primary.hframes;
        m.vframes |= s.vframes != primary.vframes;
        m.frame |= s.frame != primary.frame;
        m.offset_x |= s.offset[0] != primary.offset[0];
        m.offset_y |= s.offset[1] != primary.offset[1];
        m.region_x |= s.region_rect[0] != primary.region_rect[0];
        m.region_y |= s.region_rect[1] != primary.region_rect[1];
        m.region_w |= s.region_rect[2] != primary.region_rect[2];
        m.region_h |= s.region_rect[3] != primary.region_rect[3];
        m.tint |= s.tint != primary.tint;
        m.self_tint |= s.self_tint != primary.self_tint;
        m.per_corner |= s.per_corner_tint != primary.per_corner_tint;
    }
    m
}

/// Walks PresentWorld + SimWorld to build the per-frame snapshots
/// and writes them onto the `HeroScreen`. Caller (orchestrator)
/// already holds the destructured `AppGfx` refs and the per-frame
/// EWMA stats; this is purely the publication logic.
#[allow(clippy::too_many_arguments)]
pub(super) fn publish(
    hero: &mut HeroScreen,
    hero_live: &mut Option<HeroLive>,
    sim: &mut SimWorld,
    present: &mut PresentWorld,
    camera: &Camera2d,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    renderer: &ph2d_render::SpriteRenderer,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    frame_ms_ewma: f32,
    frame_cpu_ms_ewma: f32,
) {
    // M14.4a: if live-bridge enabled, rebuild HierarchySnapshot
    // from SimWorld + push into HeroScreen BEFORE paint. The
    // snapshot's DFS visit order = hierarchy panel display
    // order. ADR-0029 Phase C.2: the typed Hierarchy panel owns the
    // live-entries thread-local; we call into the panel crate
    // directly here (the shell already gates `panel-hierarchy` via
    // feature).
    #[cfg(feature = "panel-hierarchy")]
    if let Some(live) = hero_live.as_mut() {
        crate::build_hierarchy_snapshot(
            sim.world(),
            &mut live.walk_state,
            &mut live.walk_scratch,
            &mut live.snapshot,
        );
        let (ordered, mut entries) = live.bridge.sync_from_snapshot(&live.snapshot);
        // Fase 0 hotfix: mark every multi-selection row's
        // `HierarchyEntity.selected` BEFORE the panel paints, so
        // the row painter highlights N rows instead of just the
        // primary (paint.rs falls back to label match only when
        // `selected` is still false — fixture/demo path).
        for bits in hero.gizmo.iter_selected() {
            if let Some(node_id) = live.bridge.node_for(bits)
                && let Some(entry) = entries.get_mut(&node_id)
            {
                entry.selected = true;
            }
        }
        // Onda 1 hotfix: centralise the header label sync to the
        // multi-selection primary. Input handlers (canvas pick,
        // Hierarchy panel click, modifier override) used to stamp
        // hero.selection themselves and could race — e.g. Hierarchy
        // Cmd+click on row A stamped label="A" BEFORE the bus drain
        // toggled A out of the selection, leaving paint's label-match
        // fallback to re-highlight A. Snapshotting it once here
        // post-drain, against the post-toggle primary, removes the
        // race entirely.
        let primary_label = hero
            .gizmo
            .selection
            .and_then(|bits| live.bridge.node_for(bits))
            .and_then(|node| {
                entries
                    .get(&node)
                    .map(|e| (e.name.clone(), e.badge.clone()))
            });
        ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &ordered, entries);
        if let Some((label, badge)) = primary_label {
            hero.selection = Some(ph2d_editor::HeroSelection {
                label,
                kind: badge.unwrap_or_else(|| "ENT".to_string()),
                world_pos: (0.0, 0.0),
            });
        } else if hero.gizmo.selection.is_none() {
            hero.selection = None;
        }
    }
    // M14.4b: publish the demo camera + window dims so the
    // hero paints its world grid overlay. `canvas` is a
    // placeholder — `paint_hero_screen` overrides it with
    // the layout-computed canvas rect.
    hero.set_grid_view(Some(ph2d_editor::GridView {
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: window_size.width as f32,
        window_h: window_size.height as f32,
        canvas: ph2d_editor::zones::Rect::new(0.0, 0.0, 0.0, 0.0),
    }));
    // M14.4g Telemetry Phase A: publish real stats. Sprite
    // and entity counts come from PresentWorld (the source of
    // truth for "what we shipped to the GPU this frame"); fps
    // is derived from the EWMA frame_ms.
    let sprite_count = present
        .world_mut()
        .query::<&ph2d_render::RenderInstance>()
        .iter(present.world_mut())
        .count() as u32;
    let entity_count = present
        .world_mut()
        .query::<&SimRef>()
        .iter(present.world_mut())
        .count() as u32;
    let fps = if frame_ms_ewma > 0.001 {
        1000.0 / frame_ms_ewma
    } else {
        0.0
    };
    // M14.7 polish (10.1): raw fps = inverse of pure
    // CPU/command-encode time. Floored at 1 ms (1000 fps) so
    // a startup-edge measurement of 0 doesn't blow up to
    // `inf`; real workloads stabilize within a few frames.
    let raw_fps = 1000.0 / frame_cpu_ms_ewma.max(0.001);
    hero.stats = ph2d_editor::BottomHudStats {
        fps,
        frame_ms: frame_ms_ewma,
        draws: 1,
        sprite_count,
        entity_count,
        raw_fps,
    };
    // Hierarchy counts use PresentWorld's archetype components
    // (Transform + Sprite + Visibility + ChildOf + Children).
    // It's a proxy — exactly the components the editor's
    // snapshot pipeline observes per entity. Multiplying by
    // entity count is a rough estimate; counting via archetype
    // walk is cheap enough at editor scales.
    let component_count = {
        let world = sim.world();
        let mut total = 0u32;
        for archetype in world.archetypes().iter() {
            let len = archetype.len();
            let comps = archetype.components().len() as u32;
            total = total.saturating_add(len.saturating_mul(comps));
        }
        total
    };
    #[cfg(feature = "panel-hierarchy")]
    ph2d_panel_hierarchy::set_live_component_count(component_count);
    // M14.7 B: publish the gizmo's per-frame projection. When
    // the selection still resolves to a present entity (it can
    // vanish if the user deleted it between frames) we build a
    // `GizmoView` from the world-space bbox + camera. Empty
    // selection → clear the view so the painter skips.
    //
    // M14.7 polish (parent-fix): the gizmo MUST read
    // `GlobalTransform` from PresentWorld — not the entity's
    // local `Transform` in SimWorld. After a hierarchy reparent
    // the child's local Transform stays the same but its world
    // position is now parent.world ∘ local; the sprite renders
    // at the new world position via the extract path (which
    // reads GlobalTransform), so the gizmo has to do the same
    // or it drifts away from the sprite by exactly the parent's
    // world offset. The Sprite's local `size` is still pulled
    // from SimWorld — it's the import-time author rect,
    // multiplied here by the world scale extracted from the
    // matrix to match the renderer's RenderInstance build.
    // Whether the Pivot transform tool is the active radio selection —
    // captured as a Copy bool so the gizmo-view closure (which can't
    // re-borrow `hero`) can emphasize the pivot dot.
    let pivot_tool_active = hero.store.button_state(ph2d_editor::ids::TOOL_PIVOT)
        == Some(ph2d_editor::widget::ButtonState::Pressed);
    // Captured Copy so the closure (which can't re-borrow `hero`) can
    // resolve the same effective anchor the extract stamps — keeping the
    // selection box aligned with the rendered quad under centered/offset.
    let gizmo_ppm = hero.project.pixels_per_meter;
    // Onda 2: factor the per-sprite GizmoView build into a closure so
    // the primary, each extra, and the global union all share the
    // exact same world→view math. Single source of truth for the
    // affine decomposition + anchor compensation; any future render-
    // path tweak only touches this closure.
    let build_view =
        |bits: u64, sim: &SimWorld, present: &mut PresentWorld| -> Option<ph2d_editor::GizmoView> {
            let sim_entity = ph2d_ecs::Entity::from_bits(bits);
            let sprite = sim.world().get::<Sprite>(sim_entity)?;
            let mut q = present
                .world_mut()
                .query::<(&SimRef, &ph2d_ecs::GlobalTransform)>();
            let gt = q.iter(present.world()).find_map(|(sref, gt)| {
                if sref.0 == sim_entity {
                    Some(*gt)
                } else {
                    None
                }
            })?;
            let affine = gt.affine();
            let col0_x = affine[0];
            let col0_y = affine[1];
            let col1_x = affine[2];
            let col1_y = affine[3];
            let scale_x = (col0_x * col0_x + col0_y * col0_y).sqrt();
            let scale_y = (col1_x * col1_x + col1_y * col1_y).sqrt();
            let rotation = col0_y.atan2(col0_x);
            let p = gt.translation();
            let half_w = sprite.size[0] * scale_x * 0.5;
            let half_h = sprite.size[1] * scale_y * 0.5;
            // Effective anchor (folds centered/offset) so the box tracks
            // the rendered quad, not just the raw tool pivot.
            let eff_anchor = sprite.resolve_anchor(gizmo_ppm);
            let ax = eff_anchor[0] * scale_x;
            let ay = eff_anchor[1] * scale_y;
            // T1.3.5 cross-OS bit-identical.
            let (sin_r, cos_r) = libm::sincosf(rotation);
            let cx = p.x + ax * cos_r - ay * sin_r;
            let cy = p.y + ax * sin_r + ay * cos_r;
            Some(ph2d_editor::GizmoView {
                bbox_min_world: [cx - half_w, cy - half_h],
                bbox_max_world: [cx + half_w, cy + half_h],
                pivot_world: [p.x, p.y],
                pivot_tool_active,
                rotation,
                camera_center: camera.center,
                camera_height_world: camera.height_world,
                window_w: window_size.width as f32,
                window_h: window_size.height as f32,
                canvas: ph2d_editor::zones::Rect::new(
                    0.0,
                    0.0,
                    window_size.width as f32,
                    window_size.height as f32,
                ),
                cursor_screen: Some(last_pointer),
            })
        };
    hero.gizmo.view = hero
        .gizmo
        .selection
        .and_then(|bits| build_view(bits, sim, present));
    // Onda 2: rebuild the extras' views every frame. Cleared first so
    // a sprite that left the selection between frames stops painting.
    hero.gizmo.extra_views.clear();
    let mut alive_extras: Vec<u64> = Vec::with_capacity(hero.gizmo.extra_selection.len());
    for bits in hero.gizmo.extra_selection.clone() {
        if let Some(v) = build_view(bits, sim, present) {
            hero.gizmo.extra_views.push(v);
            alive_extras.push(bits);
        }
    }
    // Onda 2 hotfix: prune the selection set when sprites disappear
    // (cascade despawn, parent delete, etc.) — otherwise selected_len
    // stays >1 and the global gizmo keeps painting over the surviving
    // single sprite (user-reported: "se deletar algumas e sobrar 1,
    // o gizmo global fica aparecendo mesmo com uma sprite"). The
    // bridge / world don't notify; we detect by absence of a view.
    hero.gizmo.extra_selection = alive_extras;
    if hero.gizmo.view.is_none() && hero.gizmo.selection.is_some() {
        // Primary disappeared. Promote oldest extra if any; else clear.
        hero.gizmo.selection = if !hero.gizmo.extra_selection.is_empty() {
            let promoted = hero.gizmo.extra_selection.remove(0);
            // The promoted entity already had a view in extra_views;
            // re-point hero.gizmo.view to it.
            hero.gizmo.view = build_view(promoted, sim, present);
            Some(promoted)
        } else {
            None
        };
    }
    // Onda 2 polish: while a Global gizmo drag is alive, derive the
    // global view from the cached `global_view_start` snapshot +
    // primary's transform deltas. This is what makes the global gizmo
    // **rotate visually** during a Global Rotate (and scale rigidly
    // during a Global Scale) instead of being the axis-aligned union
    // of rotated sprites — that union grows under rotation, which
    // would make the gizmo "balloon" rather than rotate.
    let global_from_drag = if let (Some(start), Some(drag)) = (
        hero.gizmo.global_view_start.as_ref().copied(),
        hero.gizmo.drag.as_ref().copied(),
    ) && matches!(drag.target, ph2d_editor::GizmoTarget::Global)
    {
        let primary_entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
        let world = sim.world();
        let (delta_rot, factor_x, factor_y) =
            if let Some(t) = world.get::<Transform>(primary_entity) {
                let dr = t.rotation - drag.start_transform.rotation;
                let fx = if drag.start_transform.scale[0].abs() > f32::EPSILON {
                    t.scale.x / drag.start_transform.scale[0]
                } else {
                    1.0
                };
                let fy = if drag.start_transform.scale[1].abs() > f32::EPSILON {
                    t.scale.y / drag.start_transform.scale[1]
                } else {
                    1.0
                };
                (dr, fx, fy)
            } else {
                (0.0, 1.0, 1.0)
            };
        let cx_s = (start.bbox_min_world[0] + start.bbox_max_world[0]) * 0.5;
        let cy_s = (start.bbox_min_world[1] + start.bbox_max_world[1]) * 0.5;
        let hw_s = (start.bbox_max_world[0] - start.bbox_min_world[0]) * 0.5;
        let hh_s = (start.bbox_max_world[1] - start.bbox_min_world[1]) * 0.5;
        // Onda 2 hotfix: global drags (Scale + Rotate) PIVOT around the
        // start centre. The primary's translation shifts as a side
        // effect of the rotation/scale, but the gizmo's centre stays
        // at the original pivot — using the primary's delta_translation
        // here was making the gizmo drift away from the sprites it
        // covers (smoke: "o desenho do gizmo não rotaciona corretamente
        // em seu centro causando um drift entre as sprites e o
        // desenho do gizmo"). Global has no Translate handle (we
        // dropped BBOX_INTERIOR for keyed gizmos), so this branch only
        // sees Scale + Rotate.
        let new_cx = cx_s;
        let new_cy = cy_s;
        let new_hw = hw_s * factor_x.abs();
        let new_hh = hh_s * factor_y.abs();
        Some(ph2d_editor::GizmoView {
            bbox_min_world: [new_cx - new_hw, new_cy - new_hh],
            bbox_max_world: [new_cx + new_hw, new_cy + new_hh],
            pivot_world: [new_cx, new_cy],
            pivot_tool_active: false,
            rotation: delta_rot,
            camera_center: start.camera_center,
            camera_height_world: start.camera_height_world,
            window_w: start.window_w,
            window_h: start.window_h,
            canvas: start.canvas,
            cursor_screen: Some(last_pointer),
        })
    } else {
        None
    };
    // Onda 2: global view = union of every selected sprite's bbox,
    // EXPANDED by a fixed screen offset so the global gizmo's handles
    // sit clear of the individual gizmos' handles (Enio: "o gizmo da
    // multiseleção com offset em relação aos gizmos individuais para
    // não conflitar as alças de manipulação"). 32 px in screen space,
    // converted to world units at the current zoom so the offset
    // tracks the zoom level — handles stay one handle-size + a gap
    // outside the individuals at any scale.
    hero.gizmo.global_view = if let Some(v) = global_from_drag {
        Some(v)
    } else if hero.gizmo.selected_len() > 1 {
        let primary = hero.gizmo.view.as_ref();
        let mut iter = primary.into_iter().chain(hero.gizmo.extra_views.iter());
        iter.next().map(|first| {
            let mut min_x = first.bbox_min_world[0];
            let mut min_y = first.bbox_min_world[1];
            let mut max_x = first.bbox_max_world[0];
            let mut max_y = first.bbox_max_world[1];
            for v in iter {
                min_x = min_x.min(v.bbox_min_world[0]);
                min_y = min_y.min(v.bbox_min_world[1]);
                max_x = max_x.max(v.bbox_max_world[0]);
                max_y = max_y.max(v.bbox_max_world[1]);
            }
            let pixel_to_world = first.camera_height_world / first.window_h.max(1.0);
            let offset_world = 32.0 * pixel_to_world;
            ph2d_editor::GizmoView {
                bbox_min_world: [min_x - offset_world, min_y - offset_world],
                bbox_max_world: [max_x + offset_world, max_y + offset_world],
                pivot_world: [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5],
                pivot_tool_active: false,
                rotation: 0.0,
                camera_center: first.camera_center,
                camera_height_world: first.camera_height_world,
                window_w: first.window_w,
                window_h: first.window_h,
                canvas: first.canvas,
                cursor_screen: first.cursor_screen,
            }
        })
    } else {
        None
    };
    // M14.5 inspector phase (6.4/§9): publish a per-frame
    // snapshot of the selected sprite so `paint_inspector` can
    // surface the Render Source section + Reimport button
    // without crossing the ADR-0021 boundary into SimWorld.
    // BulkSelect (T2.0): the full selection (primary + extras). Only
    // collected (one alloc) for a MULTI-selection — single-select (the
    // common case) takes the empty path and skips the Mixed compare.
    let selected_count = hero.gizmo.selected_len();
    let inspector_selection: Vec<u64> = if selected_count > 1 {
        hero.gizmo.iter_selected().collect()
    } else {
        Vec::new()
    };
    let inspector_sprite = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let world = sim.world();
        let sprite = world.get::<Sprite>(entity)?;
        let transform = world.get::<Transform>(entity)?;
        let mixed = if inspector_selection.len() > 1 {
            compute_sprite_mixed(world, &inspector_selection, sprite)
        } else {
            ph2d_editor::InspectorSpriteMixed::default()
        };
        let name = world
            .get::<Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity_{bits:x}"));
        let (source_kind, source_pixels, can_reimport) = match sprite.source {
            ph2d_render::SpriteSource::Atlas { key } => {
                let dims = atlas_asset_map.get(&key).and_then(|aid| {
                    asset_db.get(aid).and_then(|asset| match &*asset {
                        ph2d_asset::Asset::ImageRgba8 { width, height, .. } => {
                            Some((*width, *height))
                        }
                        _ => None,
                    })
                });
                (
                    ph2d_editor::InspectorSpriteSource::Atlas { key },
                    dims,
                    dims.is_some(),
                )
            }
            ph2d_render::SpriteSource::Individual { texture_id } => {
                // Source dims come from the renderer's individual-texture
                // store (the bake's own size) so the Region UI can show
                // "Source W×H" and seed `region_rect` to the full source —
                // the extract already supports Individual region sampling.
                let dims = renderer.individual().dims(texture_id);
                (
                    ph2d_editor::InspectorSpriteSource::Individual { texture_id },
                    dims,
                    // Reimport recomputes world size from an Atlas asset's
                    // px/m; Individual bakes have no atlas asset to re-decode.
                    false,
                )
            }
        };
        let world_size = [
            sprite.size[0] * transform.scale.x,
            sprite.size[1] * transform.scale.y,
        ];
        Some(ph2d_editor::InspectorSpriteInfo {
            entity_bits: bits,
            name,
            world_size,
            source_kind,
            source_pixels,
            can_reimport,
            flip_x: sprite.flip_x,
            flip_y: sprite.flip_y,
            opacity: sprite.opacity,
            tint_fill: sprite.tint_fill,
            hframes: sprite.hframes,
            vframes: sprite.vframes,
            frame: sprite.frame,
            tint: sprite.tint,
            self_tint: sprite.self_tint,
            per_corner_tint: sprite.per_corner_tint,
            region_enabled: sprite.region_enabled,
            region_rect: sprite.region_rect,
            region_filter_clip: sprite.region_filter_clip,
            centered: sprite.centered,
            offset: sprite.offset,
            selected_count,
            mixed,
        })
    });
    // M14.A: live Transform snapshot for the inspector. Same
    // ADR-0021 / HR-8 boundary as sprite snapshot — Inspector
    // never reads SimWorld; the host bridges. Lands on every
    // entity that has a `Transform` component, not just sprites
    // (so non-renderable entities still show their pose).
    let inspector_transform = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let t = sim.world().get::<Transform>(entity)?;
        Some(ph2d_editor::InspectorTransformInfo {
            entity_bits: bits,
            translation: [t.translation.x, t.translation.y],
            rotation_rad: t.rotation,
            scale: [t.scale.x, t.scale.y],
            skew_rad: [t.skew_x, t.skew_y],
        })
    });
    // M14.D: live Visibility snapshot. Absence-equals-visible
    // is the canonical invariant — entities without a
    // `Visibility` component render normally, so `None` from
    // `world.get::<Visibility>` maps to `visible = true`.
    // Only published when the selection has a `Transform`
    // (i.e. it's an Inspector-worthy entity); without a
    // Transform the Inspector hides the whole panel content.
    let inspector_visibility = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        sim.world().get::<Transform>(entity)?;
        let visible = sim
            .world()
            .get::<Visibility>(entity)
            .map(|v| !v.hidden)
            .unwrap_or(true);
        Some(ph2d_editor::InspectorVisibilityInfo {
            entity_bits: bits,
            visible,
        })
    });
    // M14.E: live `Name` snapshot. Falls back to
    // `Entity_{hex}` when the entity has no Name component
    // yet — matches the existing `InspectorSpriteInfo::name`
    // shape. Same Transform-presence gate.
    let inspector_name = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        sim.world().get::<Transform>(entity)?;
        let name = sim
            .world()
            .get::<Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity_{bits:x}"));
        Some(ph2d_editor::InspectorNameInfo {
            entity_bits: bits,
            name,
        })
    });
    let sel = &inspector_selection; // W3 §7/§9 snapshots (§7 sibling module)
    let inspector_ordering = hero.gizmo.selection.and_then(|b| {
        super::inspector_ordering::build_ordering_info(sim.world(), b, sel, selected_count)
    });
    let inspector_sampling = hero.gizmo.selection.and_then(|b| {
        super::inspector_ordering::build_sampling_info(sim.world(), b, sel, selected_count)
    });
    let inspector_blend = hero.gizmo.selection.and_then(|b| {
        super::inspector_ordering::build_blend_info(sim.world(), b, sel, selected_count)
    });
    let inspector_visibility_section = hero.gizmo.selection.and_then(|b| {
        super::inspector_visibility::build_visibility_section_info(
            sim.world(),
            b,
            sel,
            selected_count,
        )
    });
    // ADR-0029 Phase C.1: publish snapshots to the panel crate's
    // thread-locals (replaces the pre-C.1 `hero.inspector.<field>`
    // writes — the field no longer exists; the panel-owned state +
    // its thread-local snapshot setters do).
    #[cfg(feature = "panel-inspector")]
    {
        ph2d_panel_inspector::set_current_inspector_sprite(inspector_sprite);
        ph2d_panel_inspector::set_current_inspector_ordering(inspector_ordering);
        ph2d_panel_inspector::set_current_inspector_sampling(inspector_sampling);
        ph2d_panel_inspector::set_current_inspector_blend(inspector_blend);
        ph2d_panel_inspector::set_current_inspector_visibility_section(
            inspector_visibility_section,
        );
        ph2d_panel_inspector::set_current_inspector_transform(inspector_transform);
        ph2d_panel_inspector::set_current_inspector_visibility(inspector_visibility);
        ph2d_panel_inspector::set_current_inspector_name(inspector_name);
        ph2d_panel_inspector::set_current_display_unit(
            hero.project.display_unit,
            hero.project.pixels_per_meter,
        );
    }
    #[cfg(not(feature = "panel-inspector"))]
    {
        let _ = (
            inspector_sprite,
            inspector_transform,
            inspector_visibility,
            inspector_visibility_section,
            inspector_name,
        );
    }
}

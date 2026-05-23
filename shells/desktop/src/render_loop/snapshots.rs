//! Snapshot publication phase — once per frame, before paint.
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
            .and_then(|node| entries.get(&node).map(|e| (e.name.clone(), e.badge.clone())));
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
    // Onda 2: factor the per-sprite GizmoView build into a closure so
    // the primary, each extra, and the global union all share the
    // exact same world→view math. Single source of truth for the
    // affine decomposition + anchor compensation; any future render-
    // path tweak only touches this closure.
    let build_view = |bits: u64,
                      sim: &SimWorld,
                      present: &mut PresentWorld|
     -> Option<ph2d_editor::GizmoView> {
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
        let ax = sprite.anchor[0] * scale_x;
        let ay = sprite.anchor[1] * scale_y;
        let (sin_r, cos_r) = rotation.sin_cos();
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
    hero.gizmo.view = hero.gizmo.selection.and_then(|bits| build_view(bits, sim, present));
    // Onda 2: rebuild the extras' views every frame. Cleared first so
    // a sprite that left the selection between frames stops painting.
    hero.gizmo.extra_views.clear();
    for bits in hero.gizmo.extra_selection.clone() {
        if let Some(v) = build_view(bits, sim, present) {
            hero.gizmo.extra_views.push(v);
        }
    }
    // Onda 2: global view = union of every selected sprite's bbox.
    // Only painted when multi-select is alive (selected_len > 1). The
    // global gizmo is axis-aligned in world space (no rotation) since
    // each individual bbox may rotate independently; the union of
    // their rotated AABBs is itself axis-aligned by construction.
    hero.gizmo.global_view = if hero.gizmo.selected_len() > 1 {
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
            ph2d_editor::GizmoView {
                bbox_min_world: [min_x, min_y],
                bbox_max_world: [max_x, max_y],
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
    let inspector_sprite = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let world = sim.world();
        let sprite = world.get::<Sprite>(entity)?;
        let transform = world.get::<Transform>(entity)?;
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
            ph2d_render::SpriteSource::Individual { texture_id } => (
                ph2d_editor::InspectorSpriteSource::Individual { texture_id },
                None,
                false,
            ),
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
    // ADR-0029 Phase C.1: publish snapshots to the panel crate's
    // thread-locals (replaces the pre-C.1 `hero.inspector.<field>`
    // writes — the field no longer exists; the panel-owned state +
    // its thread-local snapshot setters do).
    #[cfg(feature = "panel-inspector")]
    {
        ph2d_panel_inspector::set_current_inspector_sprite(inspector_sprite);
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
            inspector_name,
        );
    }
}

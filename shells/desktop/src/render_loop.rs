// ph2d-loc-cap: 1603 LOC, Wave 3.1 stage C lifted `App::render_frame`
// here verbatim (was 1582 LOC inside main.rs::impl App). Splitting
// the body into per-phase siblings (snapshots / dispatch / paint /
// present) is deferred to Wave 3.2 — each phase needs careful
// borrow-restructure on the AppGfx destructure that the body relies
// on heavily.

//! Per-frame render orchestration.
//!
//! Wave 3.1 stage C — `App::render_frame`'s body lifted verbatim from
//! `main.rs` into this sibling. The orchestrator function stays a
//! method on `App` (via a split `impl` block) so all the `self.X`
//! field accesses and `Self::method()` calls continue to work
//! unchanged. main.rs's `render_frame` is now a 1-line delegate.
//!
//! Behavior-preserving lift — no logic changes. Splitting this file
//! further (into per-phase siblings) is deferred to Wave 3.2 if the
//! current file size still violates HR-18.

use crate::*;

use ph2d_ecs::{Name, SimRef, Transform, Visibility, propagate_transforms};
use ph2d_editor::interaction::WidgetEvent;
use ph2d_editor::paint::PaintCtx;
use ph2d_editor::zones::Rect as EditorRect;
use ph2d_editor::{Layout as EditorLayout, RequestedSpriteStrategy, Toast, paint_hero_screen};
use ph2d_render::{Camera2d, RenderInstance, Sprite};
use std::time::Instant;

impl crate::App {
    pub(super) fn run_render_frame(&mut self) {
        // M14.7 polish (10.1): tag the start of CPU work for the
        // raw-fps measurement. Stopped after `queue.submit` (before
        // the present blocks on vsync) so the EWMA tracks pure
        // hardware capacity, independent of refresh rate.
        let cpu_start = Instant::now();
        // Pump gamepad events first so InputState reflects the latest
        // state by the time sim/extract run. Order: input → script
        // input snapshot → sim → extract → render.
        self.pump_gamepad();
        self.push_input_to_script();

        // M14.7 polish (7.3 fix): drain `pending_drops` atomically
        // BEFORE the render walks PresentWorld. Each path imports
        // exactly once, so a batch drop of N files always produces
        // exactly N sprites (winit's per-event timing no longer
        // matters). Clear the hover overlay here too — the gesture
        // is over the moment the first DroppedFile arrived.
        if !self.pending_drops.is_empty() {
            let paths = std::mem::take(&mut self.pending_drops);
            self.hovered_files.clear();
            if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                hero.dragging_files = None;
            }
            self.handler.on_file_drop(&paths);
            self.handle_dropped_files(&paths);
        }

        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let AppGfx {
            surface,
            renderer,
            sim,
            present,
            camera,
            asset_db,
            atlas_is_real,
            script,
            theme,
            zen,
            toasts,
            tools,
            layout,
            game_rt,
            tonemap,
            compositor,
            vello_pass,
            vector_scene,
            text_system,
            hero_screen,
            hero_arena,
            clipboard: _,
            prop_state,
            worklist,
            hero_live,
            next_import_cell,
            atlas_asset_map,
            component_registry,
            editor_queue,
            transform_type_id,
            visibility_type_id,
            name_type_id,
            sprite_type_id,
            image_edit_undo,
        } = gfx;
        let Some(host) = self.host.as_ref() else {
            return;
        };

        // M12 per-frame ticks: ZenMode debounce cooldown + ToastQueue
        // TTL decay. Both are pure data-layer (no Vello paint here).
        zen.tick();
        let prev_toasts = toasts.len();
        toasts.tick();
        if toasts.len() != prev_toasts {
            self.title_dirty = true;
        }

        // M14.A: drive the NumberInput stepper continuous-hold. Each
        // frame we ask the dispatcher whether a held arrow should
        // fire one more `ValueChanged` (initial 250 ms delay, then
        // 30 ms repeat). Events drained through `hero.apply_event` so
        // a Transform field that's been incrementing flows through
        // the same commit path as a Enter/blur — the EditorCommand
        // pipeline drain below picks it up.
        if let Some(hero) = hero_screen.as_mut() {
            let tick_events: Vec<WidgetEvent> =
                ph2d_editor::dispatch_tick(hero_arena, &mut hero.store, Self::timestamp_ns())
                    .to_vec();
            for e in tick_events {
                let _ = hero.apply_event(e);
            }
        }

        // M7 per-frame GC step. Cheap (p99 ≤ 10µs target per the M7
        // gate test) — keeps the Luau heap from accumulating between
        // future scripted ticks. Errors here are non-fatal; just log.
        if let Some(host) = script
            && let Err(e) = host.gc_step()
        {
            eprintln!("M7 gc_step error: {e}");
        }

        // Apply coalesced resize once per frame.
        if let Some(size) = self.pending_resize.take() {
            surface.resize(size);
            // Layout + every offscreen RT in the pipeline must follow
            // surface size. M14.5: game_rt, tonemap output, vello
            // intermediate — all three; then the compositor's bind
            // group must be rebuilt against the new texture views.
            *layout = EditorLayout::new(size.width as f32, size.height as f32);
            let dim = (size.width, size.height);
            game_rt.ensure_size(surface.gpu(), dim);
            tonemap.ensure_size(surface.gpu(), dim);
            tonemap.rebind_game_view(
                surface.gpu(),
                game_rt
                    .texture()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
            vello_pass.ensure_size(surface.gpu(), dim);
            compositor.rebind(
                surface.gpu(),
                tonemap
                    .output_texture()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                vello_pass
                    .intermediate_texture()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
            self.handler.on_resize(size, host.scale_factor());
            self.title_dirty = true;
        }

        // Drive fixed-step accumulator.
        let now = Instant::now();
        let wall_dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        // M14.4g: feed EWMA frame-time using the same `wall_dt` —
        // single source of truth for "how long did the last frame
        // take". α=0.1 smooths over jitter while still tracking
        // sustained changes in ~10 frames.
        let frame_ms_now = (wall_dt * 1000.0) as f32;
        const ALPHA: f32 = 0.1;
        self.frame_ms_ewma = ALPHA * frame_ms_now + (1.0 - ALPHA) * self.frame_ms_ewma;
        let report = self.fixed_step.advance(wall_dt);
        if report.dropped_secs > 0.0 {
            eprintln!(
                "warn: dropped {:.3}s of sim time (max_substeps cap)",
                report.dropped_secs
            );
        }
        panic::set_frame_id(self.fixed_step.tick_count());

        // Sim tick: bouncing motion. Single substep per frame for the
        // M5 demo (we don't yet honor the FixedStep substep count for
        // gameplay — that lands in M10 with the physics integrator).
        let dt = self.fixed_step.fixed_dt() as f32;
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
        // `propagate_transforms` walks the `ChildOf` tree once, and
        // the closure spawns one mirror entity per sim entity in
        // PresentWorld carrying `(SimRef, GlobalTransform)` plus an
        // optional `RenderInstance` for sprite-bearing entities.
        //
        // HR-3: WorklistBuf reuses its capacity across frames so this
        // hot path is zero-alloc after warm-up
        // (`tests/propagate_no_alloc.rs`).
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

        // Sprite-layer clear color = backdrop visible in the canvas
        // area through the transparent regions of `vello_rt`. Live
        // editor mode wants a static neutral surface so it doesn't
        // pulse rainbow under the chrome.
        //
        // Why 0.047 instead of the theme-canonical 0.012:
        // pre-M14.5 the chrome AA edges were rendered against a
        // backdrop of `Bg1` painted by Vello as sRGB byte ~12,
        // which the legacy wgpu blitter sampled as 12/255 ≈ 0.047
        // *treated as linear* (the documented vello-blitter gamma
        // confusion in `vello_pass.rs`). Anti-aliased chrome edges
        // in `ph2d-tokens` are calibrated against that 0.047
        // backdrop. Setting `game_rt` clear to the theme's true
        // linear 0.012 would make the AA halos contrast strongly
        // against the now-much-darker dst — that's exactly the
        // "pixelated borders" regression seen in M14.5 round 2.
        // Match the legacy backdrop value here; the chrome edges
        // composite identically.
        let (r, g, b) = if hero_live.is_some() {
            (0.047, 0.047, 0.055)
        } else {
            let t = self.fixed_step.tick_count() as f64 * self.fixed_step.fixed_dt();
            (
                (t.sin() * 0.05 + 0.05).clamp(0.0, 1.0),
                ((t + 2.094).sin() * 0.05 + 0.05).clamp(0.0, 1.0),
                ((t + 4.188).sin() * 0.05 + 0.05).clamp(0.0, 1.0),
            )
        };

        let window_size = surface.size();
        // M11: build the widget scene up-front (no GPU work yet — just
        // VectorScene encoding). Done outside acquire_frame so an
        // Occluded/Timeout doesn't waste the encoder.
        let viewport = EditorRect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        );
        vector_scene.reset();
        let mut paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };

        // Default editor mode: AppGfx owns a HeroScreen with a
        // retained WidgetStore (ADR-0024). Paint reads + writes its
        // hit_index each frame; pointer/key events are forwarded to
        // it from window_event handlers via `hero_screen.handle_*`.
        // `hero_screen` is `None` only under `PH2D_M5_DEMO=1`.
        if let Some(hero) = hero_screen.as_mut() {
            // M14.4a: if live-bridge enabled, rebuild HierarchySnapshot
            // from SimWorld + push into HeroScreen BEFORE paint. The
            // snapshot's DFS visit order = hierarchy panel display
            // order. `paint_hero_screen` reads `live_hierarchy_entries`
            // via thread-local in `hierarchy::set_live_entries`.
            if let Some(live) = hero_live.as_mut() {
                build_hierarchy_snapshot(
                    sim.world(),
                    &mut live.walk_state,
                    &mut live.walk_scratch,
                    &mut live.snapshot,
                );
                let (ordered, entries) = live.bridge.sync_from_snapshot(&live.snapshot);
                hero.sync_from_hierarchy(&ordered, entries);
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
                .query::<&ph2d_ecs::SimRef>()
                .iter(present.world_mut())
                .count() as u32;
            let fps = if self.frame_ms_ewma > 0.001 {
                1000.0 / self.frame_ms_ewma
            } else {
                0.0
            };
            // M14.7 polish (10.1): raw fps = inverse of pure
            // CPU/command-encode time. Floored at 1 ms (1000 fps) so
            // a startup-edge measurement of 0 doesn't blow up to
            // `inf`; real workloads stabilize within a few frames.
            let raw_fps = 1000.0 / self.frame_cpu_ms_ewma.max(0.001);
            hero.stats = ph2d_editor::BottomHudStats {
                fps,
                frame_ms: self.frame_ms_ewma,
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
            ph2d_editor::set_live_component_count(component_count);
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
            hero.gizmo_view = hero.gizmo_selection.and_then(|bits| {
                let sim_entity = ph2d_ecs::Entity::from_bits(bits);
                let sprite = sim.world().get::<Sprite>(sim_entity)?;
                // Look up the present entity that mirrors this sim
                // entity via `SimRef`. We can't reuse the sim
                // `Entity` directly because entity ids are
                // per-`World` (ADR-0021).
                let mut q = present
                    .world_mut()
                    .query::<(&ph2d_ecs::SimRef, &ph2d_ecs::GlobalTransform)>();
                let gt = q.iter(present.world()).find_map(|(sref, gt)| {
                    if sref.0 == sim_entity {
                        Some(*gt)
                    } else {
                        None
                    }
                })?;
                // Decompose the affine matrix the same way the
                // extract path does — column lengths for scale,
                // atan2(col0.y, col0.x) for rotation. Keeps gizmo
                // math in lockstep with the render path so any
                // future change has one canonical place.
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
                Some(ph2d_editor::GizmoView {
                    bbox_min_world: [p.x - half_w, p.y - half_h],
                    bbox_max_world: [p.x + half_w, p.y + half_h],
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
                    cursor_screen: Some(self.last_pointer),
                })
            });
            // M14.5 inspector phase (6.4/§9): publish a per-frame
            // snapshot of the selected sprite so `paint_inspector` can
            // surface the Render Source section + Reimport button
            // without crossing the ADR-0021 boundary into SimWorld.
            hero.inspector_sprite = hero.gizmo_selection.and_then(|bits| {
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
            hero.inspector_transform = hero.gizmo_selection.and_then(|bits| {
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
            hero.inspector_visibility = hero.gizmo_selection.and_then(|bits| {
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
            hero.inspector_name = hero.gizmo_selection.and_then(|bits| {
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
            // ─────────────────────────────────────────────────────────
            // Wave 2.5 PR 11.8 closeout — consolidated bus drain.
            // ─────────────────────────────────────────────────────────
            //
            // Previously, each of the 18 EditorAction variants had its
            // own filter-and-replace block (one per drain site, ~20 LOC
            // of "drain, capture this variant, push others back" each).
            // Now we drain the bus ONCE at the top of this section,
            // categorize every variant into per-kind locals, and the
            // dispatch sites further down just read `if let Some(x) = X`.
            //
            // First-wins for most variants (matches the old
            // `found.is_none()` short-circuit). Latest-wins for
            // `InspectorNameEdit` (preserves the pre-bus Option
            // coalescing that drained at most one SetComponent per
            // frame). `Bgremoval` is NOT categorized here — it keeps
            // a separate filter-and-replace at its original site so
            // its `bgremoval_active` gate runs AFTER any same-frame
            // `ActivateBgRemoval` fires (1-frame defer edge case).
            //
            // The `undo_image_edit` / `activate_bgremoval` flag-style
            // variants collapse to a `bool` (idempotent — multiple
            // pushes in one frame = one dispatch).
            let mut activate_bgremoval = false;
            let mut visibility_toggle_row: Option<NodeId> = None;
            let mut reparent_intent: Option<ph2d_editor::screens::hero::HierReparentIntent> = None;
            let mut duplicate_row: Option<NodeId> = None;
            let mut add_child_row: Option<NodeId> = None;
            let mut reset_transform_row: Option<NodeId> = None;
            let mut delete_row: Option<NodeId> = None;
            let mut hierarchy_row_click: Option<NodeId> = None;
            let mut rename_seed_row: Option<NodeId> = None;
            let mut rename_commit: Option<(NodeId, String)> = None;
            let mut view_focus_kind: Option<ph2d_editor::ViewFocusKind> = None;
            let mut reimport_entity: Option<u64> = None;
            let mut trim_entity: Option<u64> = None;
            let mut make_square_entity: Option<u64> = None;
            let mut undo_image_edit = false;
            let mut transform_edit: Option<ph2d_editor::InspectorTransformInfo> = None;
            let mut visibility_edit: Option<ph2d_editor::InspectorVisibilityInfo> = None;
            let mut sprite_source_change: Option<(u64, RequestedSpriteStrategy)> = None;
            let mut name_edit: Option<ph2d_editor::InspectorNameInfo> = None;
            let mut bgremoval_leftover: Vec<ph2d_editor::action_bus::EditorAction> = Vec::new();
            for action in hero.bus.drain() {
                use ph2d_editor::action_bus::EditorAction;
                match action {
                    EditorAction::ActivateBgRemoval => activate_bgremoval = true,
                    EditorAction::UndoImageEdit => undo_image_edit = true,
                    EditorAction::HierToggleVisibility { row } => {
                        visibility_toggle_row.get_or_insert(row);
                    }
                    EditorAction::HierReparent(intent) => {
                        reparent_intent.get_or_insert(intent);
                    }
                    EditorAction::HierDuplicate { row } => {
                        duplicate_row.get_or_insert(row);
                    }
                    EditorAction::HierAddChild { row } => {
                        add_child_row.get_or_insert(row);
                    }
                    EditorAction::HierResetTransform { row } => {
                        reset_transform_row.get_or_insert(row);
                    }
                    EditorAction::HierDelete { row } => {
                        delete_row.get_or_insert(row);
                    }
                    EditorAction::HierRowClick { row } => {
                        hierarchy_row_click.get_or_insert(row);
                    }
                    EditorAction::HierRenameSeed { row } => {
                        rename_seed_row.get_or_insert(row);
                    }
                    EditorAction::HierRenameCommit { row, new_name } if rename_commit.is_none() => {
                        rename_commit = Some((row, new_name));
                    }
                    EditorAction::SetViewFocus { kind } => {
                        view_focus_kind.get_or_insert(kind);
                    }
                    EditorAction::Reimport { entity_bits } => {
                        reimport_entity.get_or_insert(entity_bits);
                    }
                    EditorAction::Trim { entity_bits } => {
                        trim_entity.get_or_insert(entity_bits);
                    }
                    EditorAction::MakeSquare { entity_bits } => {
                        make_square_entity.get_or_insert(entity_bits);
                    }
                    EditorAction::InspectorTransformEdit(info) => {
                        transform_edit.get_or_insert(info);
                    }
                    EditorAction::InspectorVisibilityEdit(info) => {
                        visibility_edit.get_or_insert(info);
                    }
                    EditorAction::InspectorSpriteSourceChange {
                        entity_bits,
                        strategy,
                    } => {
                        sprite_source_change.get_or_insert((entity_bits, strategy));
                    }
                    EditorAction::InspectorNameEdit(info) => {
                        // Latest-wins (Option-coalesce parity).
                        name_edit = Some(info);
                    }
                    // Bgremoval falls through to its own filter-and-
                    // replace at the image-edit drain site so the
                    // `bgremoval_active` gate runs AFTER any same-frame
                    // ActivateBgRemoval fires.
                    other @ EditorAction::Bgremoval { .. } => bgremoval_leftover.push(other),
                    // EditorAction is `#[non_exhaustive]`. A future
                    // variant landing in `ph2d-editor` shouldn't break
                    // the shell — drop it silently here until a
                    // dispatch site is wired up.
                    _ => {}
                }
            }
            for a in bgremoval_leftover {
                hero.bus.push(a);
            }
            // Drain the `EditorAction::ActivateBgRemoval` intent raised
            // by clicking the Bg Removal pill. The hero can't reach
            // `gfx.tools` so the activation round-trips via the bus.
            // Same force-refresh of the snapshot push state as the
            // Digit3 shortcut below so the next snapshot push fires
            // against the current selection.
            if activate_bgremoval && tools.set_active(&ph2d_editor::ToolId::new("bgremoval")) {
                self.last_bgremoval_pushed_entity = None;
                self.title_dirty = true;
                toasts.push(Toast::info("Tool → Bg Removal"));
            }
            // Snapshot push for the active BgRemovalTool. The tool needs
            // the current selection's RGBA so its 160×160 preview shows
            // the live sprite — pushed once per (tool-active + new
            // selection) tuple. `last_bgremoval_pushed_entity` is
            // reset to `None` whenever the user activates BgRemoval
            // via Digit3 (force-refresh) AND tracked across selection
            // changes (push when it drifts). Pulls source pixels via
            // the same Atlas-vs-Individual branch the trim_transparency
            // drain uses below.
            let bgremoval_is_active = tools
                .active()
                .map(|t| t.id() == ph2d_editor::ToolId::new("bgremoval"))
                .unwrap_or(false);
            if bgremoval_is_active
                && let Some(bits) = hero.gizmo_selection
                && self.last_bgremoval_pushed_entity != Some(bits)
            {
                let entity = ph2d_ecs::Entity::from_bits(bits);
                let snap =
                    sim.world()
                        .get::<Sprite>(entity)
                        .and_then(|sprite| match sprite.source {
                            ph2d_render::SpriteSource::Atlas { key } => {
                                let aid = atlas_asset_map.get(&key)?;
                                let asset = asset_db.get(aid)?;
                                match &*asset {
                                    ph2d_asset::Asset::ImageRgba8 {
                                        width,
                                        height,
                                        pixels,
                                    } => Some((*width, *height, pixels.clone())),
                                    _ => None,
                                }
                            }
                            ph2d_render::SpriteSource::Individual { texture_id } => renderer
                                .readback_individual(texture_id)
                                .ok()
                                .map(|(w, h, pix)| (w, h, pix.into())),
                        });
                if let Some((w, h, rgba)) = snap
                    && let Some(tool) = tools.active_mut()
                    && let Some(bg) = tool
                        .as_any_mut()
                        .downcast_mut::<ph2d_editor::tools::bgremoval::BgRemovalTool>()
                {
                    bg.set_source_snapshot(rgba.to_vec(), w, h);
                    self.last_bgremoval_pushed_entity = Some(bits);
                }
            }
            paint_hero_screen(hero, viewport, vector_scene, paint_ctx.text);
            // M14.4b.bis: drain pending camera-reset request from
            // the VIEW button (legacy "Zero" mode — kept around for
            // shells that still raise it).
            if hero.camera_reset_pending {
                hero.camera_reset_pending = false;
                *camera = Camera2d::default();
                toasts.push(Toast::info("View → Zero (camera reset)"));
                self.title_dirty = true;
            }
            // M14.7 polish: drain pending view-focus intent (F/Home
            // key OR VIEW button click). Per `ViewFocusKind`:
            //   - `Selected`: pan to gizmo_selection or (0,0).
            //   - `Camera`: pan to (0,0) until camera-object exists.
            //   - `All`: pan + zoom to fit all sprites.
            // `view_focus_kind` pre-populated by the consolidated drain.
            if let Some(kind) = view_focus_kind
                && hero_intents::drain_view_focus(
                    kind,
                    hero.gizmo_selection,
                    present,
                    camera,
                    window_size,
                    toasts,
                )
            {
                self.title_dirty = true;
            }
            // M14.6A: drain pending hierarchy visibility toggle —
            // resolve row NodeId → ECS Entity via the bridge, flip
            // the `Visibility` component on SimWorld.
            // `visibility_toggle_row` pre-populated by the consolidated drain.
            if let Some(row_id) = visibility_toggle_row
                && let Some(live) = hero_live.as_ref()
                && let Some(entity_bits) = live.bridge.entity_for(row_id)
            {
                let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                let sim_w = sim.world_mut();
                if let Ok(mut entry) = sim_w.get_entity_mut(entity) {
                    let was_hidden = entry
                        .get::<ph2d_ecs::Visibility>()
                        .is_some_and(|v| v.hidden);
                    entry.insert(ph2d_ecs::Visibility {
                        hidden: !was_hidden,
                    });
                }
            }
            // M14.6B: drain pending hierarchy reparent intent —
            // translate dragged + new_parent NodeIds via the bridge,
            // then either `insert(ChildOf(p))` or remove the
            // `ChildOf` component for a root-level drop. With M14.7
            // polish (14.3 continuation) we also honor `intent.before`
            // to position the dragged entity at a specific slot in
            // the new parent's `Children` list — bevy_ecs 0.18
            // `Children` preserves insertion order, so we rebuild the
            // ordering by re-inserting every relevant child's
            // ChildOf in the desired sequence.
            // `reparent_intent` pre-populated by the consolidated drain.
            if let Some(intent) = reparent_intent
                && let Some(live) = hero_live.as_ref()
            {
                hero_intents::drain_reparent(intent, live, sim);
            }
            // M14.6 F: drain per-row Hierarchy context-menu actions.
            // Each is a `HierDuplicate/AddChild/ResetTransform/Delete`
            // bus variant — bridge resolves row → Entity, then we
            // apply the corresponding ECS mutation. Order is
            // intentional: Delete last, so a (degenerate) frame that
            // queues "duplicate then delete" leaves the duplicate in
            // place and removes the original. The next snapshot rebuild
            // picks up the result automatically.
            // `duplicate_row` / `add_child_row` / `reset_transform_row`
            // / `delete_row` pre-populated by the consolidated drain.
            if let Some(row) = duplicate_row
                && let Some(live) = hero_live.as_ref()
                && let Some(entity_bits) = live.bridge.entity_for(row)
            {
                let src = ph2d_ecs::Entity::from_bits(entity_bits);
                let sim_w = sim.world_mut();
                let transform = sim_w.get::<Transform>(src).copied();
                let sprite = sim_w.get::<Sprite>(src).copied();
                let name = sim_w.get::<Name>(src).map(|n| n.as_str().to_owned());
                let parent = sim_w.get::<ph2d_ecs::ChildOf>(src).map(|c| c.parent());
                let copy_name = name
                    .map(|n| format!("{n}_copy"))
                    .unwrap_or_else(|| "copy".to_string());
                let mut builder = sim_w.spawn_empty();
                if let Some(t) = transform {
                    builder.insert(t);
                }
                if let Some(s) = sprite {
                    builder.insert(s);
                }
                builder.insert(Name::new(copy_name));
                if let Some(p) = parent {
                    builder.insert(ph2d_ecs::ChildOf(p));
                }
                toasts.push(Toast::success("Duplicated entity"));
                self.title_dirty = true;
            }
            if let Some(row) = add_child_row
                && let Some(live) = hero_live.as_ref()
                && let Some(parent_bits) = live.bridge.entity_for(row)
            {
                let parent = ph2d_ecs::Entity::from_bits(parent_bits);
                sim.world_mut().spawn((
                    Transform::IDENTITY,
                    Name::new("Child"),
                    ph2d_ecs::ChildOf(parent),
                ));
                toasts.push(Toast::success("Added child entity"));
                self.title_dirty = true;
            }
            if let Some(row) = reset_transform_row
                && let Some(live) = hero_live.as_ref()
                && let Some(entity_bits) = live.bridge.entity_for(row)
            {
                let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) {
                    *t = Transform::IDENTITY;
                    toasts.push(Toast::info("Transform reset"));
                    self.title_dirty = true;
                }
            }
            if let Some(row) = delete_row
                && let Some(live) = hero_live.as_ref()
                && let Some(entity_bits) = live.bridge.entity_for(row)
            {
                // bevy_ecs 0.18 `ChildOf` cascade: despawning a parent
                // takes its descendants with it. No manual recursion
                // here (see `transform_hierarchy.rs::despawn_root_
                // cascades_via_child_of`).
                let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                sim.world_mut().despawn(entity);
                // Clear gizmo selection if it pointed at the deleted
                // entity — the bbox lookup would otherwise dangle for
                // a frame until the next snapshot rebuilds.
                if hero.gizmo_selection == Some(entity_bits) {
                    hero.gizmo_selection = None;
                }
                toasts.push(Toast::warning("Deleted entity"));
                self.title_dirty = true;
            }
            // M14.6 D: drain pending hierarchy-row click → sync
            // `gizmo_selection` to whichever entity the user just
            // picked in the hierarchy panel. Inverse of the M14.7 A
            // canvas-pick path (canvas → label sync runs further down
            // when we publish gizmo_view).
            // `hierarchy_row_click` pre-populated by the consolidated drain.
            if let Some(row) = hierarchy_row_click
                && let Some(live) = hero_live.as_ref()
                && let Some(entity_bits) = live.bridge.entity_for(row)
            {
                hero.gizmo_selection = Some(entity_bits);
            }
            // M14.7 polish: one-shot seed of the rename TextInput
            // when rename mode opens. `HierRenameSeed` is pushed by
            // hero on the open path (right-click Rename / long-press)
            // and drained here exactly once — so subsequent Backspace
            // edits that empty the buffer don't get clobbered back
            // to the original name on the next frame.
            // `rename_seed_row` pre-populated by the consolidated drain.
            if let Some(row) = rename_seed_row
                && let Some(live) = hero_live.as_ref()
                && let Some(entity_bits) = live.bridge.entity_for(row)
            {
                let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                let value = sim
                    .world()
                    .get::<Name>(entity)
                    .map(|n| n.as_str().to_owned())
                    .unwrap_or_default();
                if let Some(ph2d_editor::interaction::InteractiveState::TextInput {
                    text,
                    caret,
                    selection_anchor,
                    ..
                }) = hero
                    .store
                    .get_mut(ph2d_editor::screens::hero::ids::HIER_RENAME_INPUT)
                {
                    let len = value.len();
                    *text = value;
                    *caret = len;
                    *selection_anchor = Some(0); // select all
                }
            }
            // Drain a finalized rename commit (Enter pressed in
            // rename input). Write the new Name component on the
            // entity; toast confirms.
            // `rename_commit` pre-populated by the consolidated drain
            // (owned String moved out of the EditorAction at the match).
            if let Some((row, new_name)) = rename_commit
                && let Some(live) = hero_live.as_ref()
                && let Some(entity_bits) = live.bridge.entity_for(row)
            {
                let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                let sim_w = sim.world_mut();
                if let Ok(mut entry) = sim_w.get_entity_mut(entity) {
                    entry.insert(Name::new(new_name.clone()));
                    toasts.push(Toast::success(format!("Renamed to {new_name}")));
                    self.title_dirty = true;
                }
                // Clear the rename TextInput buffer for next session.
                if let Some(ph2d_editor::interaction::InteractiveState::TextInput {
                    text,
                    caret,
                    selection_anchor,
                    ..
                }) = hero
                    .store
                    .get_mut(ph2d_editor::screens::hero::ids::HIER_RENAME_INPUT)
                {
                    text.clear();
                    *caret = 0;
                    *selection_anchor = None;
                }
            }
            // M14.5 inspector phase (6.4): drain Reimport intent →
            // re-decode the atlas source's pixel dimensions at the
            // current `project.pixels_per_meter` and write the new
            // world size back to the Sprite component. The texture
            // itself is unchanged; only `Sprite.size` is recomputed.
            // `reimport_entity` pre-populated by the consolidated drain.
            if let Some(entity_bits) = reimport_entity {
                let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                let px_per_m = hero.project.pixels_per_meter.max(EPS_PIXELS_PER_METER);
                let new_size = sim.world().get::<Sprite>(entity).and_then(|sprite| {
                    let ph2d_render::SpriteSource::Atlas { key } = sprite.source else {
                        return None;
                    };
                    let aid = atlas_asset_map.get(&key)?;
                    let asset = asset_db.get(aid)?;
                    match &*asset {
                        ph2d_asset::Asset::ImageRgba8 { width, height, .. } => {
                            Some([*width as f32 / px_per_m, *height as f32 / px_per_m])
                        }
                        _ => None,
                    }
                });
                if let Some(size) = new_size {
                    let sim_w = sim.world_mut();
                    if let Some(mut sprite) = sim_w.get_mut::<Sprite>(entity) {
                        sprite.size = size;
                        toasts.push(Toast::success(format!(
                            "Reimported at {:.0} px/m → {:.3} × {:.3} m",
                            px_per_m, size[0], size[1]
                        )));
                        self.title_dirty = true;
                    }
                } else {
                    toasts.push(Toast::error("Reimport unavailable for this source"));
                    self.title_dirty = true;
                }
            }
            // M14.A: drain Inspector Transform commit → push
            // `EditorCommand::SetComponent` to the editor queue, then
            // apply. **First end-to-end consumer** of the editor
            // command pipeline (every prior `pending_*` field mutated
            // SimWorld directly). When MCP / Luau / multi-agent edits
            // arrive in M14.B+ they share this same code path —
            // governance, audit, conflict resolution all live one
            // level up from the producer.
            // `transform_edit` pre-populated by the consolidated drain.
            if let Some(info) = transform_edit {
                let t = Transform {
                    translation: Vec2::new(info.translation[0], info.translation[1]),
                    rotation: info.rotation_rad,
                    scale: Vec2::new(info.scale[0], info.scale[1]),
                };
                match postcard::to_allocvec(&t) {
                    Ok(data) => {
                        let push_res = editor_queue.push(EditorCommand::SetComponent {
                            entity: info.entity_bits,
                            type_id: *transform_type_id,
                            data,
                        });
                        if let Err(e) = push_res {
                            toasts.push(Toast::error(format!("Editor queue full: {e}")));
                            self.title_dirty = true;
                        } else if let Err(e) =
                            apply_editor_commands(sim.world_mut(), editor_queue, component_registry)
                        {
                            toasts.push(Toast::error(format!("Transform commit failed: {e}")));
                            self.title_dirty = true;
                        }
                    }
                    Err(e) => {
                        toasts.push(Toast::error(format!("Transform encode failed: {e}")));
                        self.title_dirty = true;
                    }
                }
            }
            // M14.D: drain Inspector Visibility commit → same
            // EditorCommandQueue path as Transform. We always write
            // an explicit `Visibility { hidden: ... }` (rather than
            // removing the component when `visible == true`) so the
            // round-trip is unambiguous and the audit log captures
            // both directions of the toggle.
            // `visibility_edit` pre-populated by the consolidated drain.
            if let Some(info) = visibility_edit {
                let v = Visibility {
                    hidden: !info.visible,
                };
                match postcard::to_allocvec(&v) {
                    Ok(data) => {
                        let push_res = editor_queue.push(EditorCommand::SetComponent {
                            entity: info.entity_bits,
                            type_id: *visibility_type_id,
                            data,
                        });
                        if let Err(e) = push_res {
                            toasts.push(Toast::error(format!("Editor queue full: {e}")));
                            self.title_dirty = true;
                        } else if let Err(e) =
                            apply_editor_commands(sim.world_mut(), editor_queue, component_registry)
                        {
                            toasts.push(Toast::error(format!("Visibility commit failed: {e}")));
                            self.title_dirty = true;
                        }
                    }
                    Err(e) => {
                        toasts.push(Toast::error(format!("Visibility encode failed: {e}")));
                        self.title_dirty = true;
                    }
                }
            }
            // M14.E: drain Inspector Name commit → push a
            // `Name(string)` postcard via `EditorCommand::SetComponent`
            // and apply. The consolidated drain captures latest-wins
            // (mirrors the pre-bus Option coalesce: even if the user
            // types fast, we apply once per frame with the latest text).
            if let Some(info) = name_edit {
                let n = ph2d_ecs::Name(info.name.clone());
                match postcard::to_allocvec(&n) {
                    Ok(data) => {
                        let push_res = editor_queue.push(EditorCommand::SetComponent {
                            entity: info.entity_bits,
                            type_id: *name_type_id,
                            data,
                        });
                        if let Err(e) = push_res {
                            toasts.push(Toast::error(format!("Editor queue full: {e}")));
                            self.title_dirty = true;
                        } else if let Err(e) =
                            apply_editor_commands(sim.world_mut(), editor_queue, component_registry)
                        {
                            toasts.push(Toast::error(format!("Name commit failed: {e}")));
                            self.title_dirty = true;
                        }
                    }
                    Err(e) => {
                        toasts.push(Toast::error(format!("Name encode failed: {e}")));
                        self.title_dirty = true;
                    }
                }
            }
            // M14.C: drain Render Source Strategy switch. Atlas →
            // Individual works (re-decode source pixels +
            // `acquire_individual` for the renderer, then a
            // canonical `EditorCommand::SetComponent` for the
            // updated `Sprite` — audit fix #8 puts this on the same
            // pipeline as Transform / Visibility / Name).
            // Individual → Atlas and any HandPacked transition
            // surface a toast — atlas re-insert + hand-packed asset
            // picker land in M14.C+.
            // `sprite_source_change` pre-populated by the consolidated drain.
            if let Some((entity_bits, requested)) = sprite_source_change {
                let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                let current_sprite = sim.world().get::<Sprite>(entity).copied();
                // Audit fix #7 helper: when a swap is rejected (toast
                // path), demote the clicked Strategy button's stored
                // state back to Normal so it doesn't keep painting
                // Pressed/Hovered alongside the still-active button.
                let reject_visual_reset =
                    |hero: &mut HeroScreen, clicked: RequestedSpriteStrategy| {
                        let id = match clicked {
                            RequestedSpriteStrategy::Atlas => {
                                ph2d_editor::screens::hero::ids::INSP_RENDER_STRATEGY_ATLAS
                            }
                            RequestedSpriteStrategy::Individual => {
                                ph2d_editor::screens::hero::ids::INSP_RENDER_STRATEGY_INDIVIDUAL
                            }
                            RequestedSpriteStrategy::HandPacked => {
                                ph2d_editor::screens::hero::ids::INSP_RENDER_STRATEGY_HANDPACKED
                            }
                        };
                        if let Some(ph2d_editor::InteractiveState::Button { state }) =
                            hero.store.get_mut(id)
                        {
                            *state = ph2d_editor::widget::ButtonState::Normal;
                        }
                    };
                match (current_sprite.map(|s| s.source), requested) {
                    (
                        Some(ph2d_render::SpriteSource::Atlas { key }),
                        RequestedSpriteStrategy::Individual,
                    ) => {
                        let decoded = atlas_asset_map.get(&key).and_then(|aid| {
                            asset_db.get(aid).and_then(|asset| match &*asset {
                                ph2d_asset::Asset::ImageRgba8 {
                                    width,
                                    height,
                                    pixels,
                                } => Some((*width, *height, pixels.clone())),
                                _ => None,
                            })
                        });
                        match decoded {
                            Some((w, h, pixels)) => {
                                match renderer.acquire_individual(w, h, &pixels) {
                                    Ok(texture_id) => {
                                        // Audit fix #8: route the Sprite mutation
                                        // through `EditorCommand::SetComponent`
                                        // so MCP / Luau / audit-log consumers
                                        // see the same flow as Transform / Name.
                                        // The renderer-side `acquire_individual`
                                        // already happened; what we encode is
                                        // the updated `Sprite` (size + tint
                                        // preserved from the snapshot).
                                        let mut updated =
                                            current_sprite.unwrap_or(ph2d_render::Sprite::atlas(
                                                0,
                                                [1.0, 1.0],
                                                [1.0, 1.0, 1.0, 1.0],
                                            ));
                                        updated.source =
                                            ph2d_render::SpriteSource::Individual { texture_id };
                                        match postcard::to_allocvec(&updated) {
                                            Ok(data) => {
                                                let push_res = editor_queue.push(
                                                    EditorCommand::SetComponent {
                                                        entity: entity_bits,
                                                        type_id: *sprite_type_id,
                                                        data,
                                                    },
                                                );
                                                if let Err(e) = push_res {
                                                    toasts.push(Toast::error(format!(
                                                        "Editor queue full: {e}"
                                                    )));
                                                    self.title_dirty = true;
                                                } else if let Err(e) = apply_editor_commands(
                                                    sim.world_mut(),
                                                    editor_queue,
                                                    component_registry,
                                                ) {
                                                    toasts.push(Toast::error(format!(
                                                        "Strategy commit failed: {e}"
                                                    )));
                                                    self.title_dirty = true;
                                                } else {
                                                    toasts.push(Toast::success(format!(
                                                        "Strategy → Individual (texture {})",
                                                        texture_id
                                                    )));
                                                    self.title_dirty = true;
                                                }
                                            }
                                            Err(e) => {
                                                toasts.push(Toast::error(format!(
                                                    "Sprite encode failed: {e}"
                                                )));
                                                self.title_dirty = true;
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        toasts.push(Toast::error(format!(
                                            "Individual acquire failed: {err}"
                                        )));
                                        self.title_dirty = true;
                                        reject_visual_reset(hero, requested);
                                    }
                                }
                            }
                            None => {
                                toasts.push(Toast::error(
                                    "Cannot promote to Individual — source asset missing",
                                ));
                                self.title_dirty = true;
                                reject_visual_reset(hero, requested);
                            }
                        }
                    }
                    (
                        Some(ph2d_render::SpriteSource::Atlas { .. }),
                        RequestedSpriteStrategy::Atlas,
                    )
                    | (
                        Some(ph2d_render::SpriteSource::Individual { .. }),
                        RequestedSpriteStrategy::Individual,
                    ) => {
                        // No-op: requested matches current. The
                        // `apply_event` guard already short-circuits
                        // identical clicks, but keep this branch
                        // explicit so an out-of-band publish (script,
                        // future MCP) doesn't accidentally bounce.
                    }
                    (Some(_), RequestedSpriteStrategy::Atlas) => {
                        toasts.push(Toast::info(
                            "Individual → Atlas swap is M14.C+ (atlas re-insert path)",
                        ));
                        self.title_dirty = true;
                        reject_visual_reset(hero, requested);
                    }
                    (_, RequestedSpriteStrategy::HandPacked) => {
                        toasts.push(Toast::info(
                            "Hand-packed strategy needs an atlas asset — M14.C+ asset picker",
                        ));
                        self.title_dirty = true;
                        reject_visual_reset(hero, requested);
                    }
                    (None, _) => {
                        // Entity vanished between commit and drain
                        // (despawn, hierarchy delete) — silent no-op.
                    }
                }
            }
            // ImageToolsV1: drain Trim Transparency request — read the
            // sprite's atlas-source RGBA pixels, run the trim algorithm,
            // and (if any transparent border was found) re-source the
            // sprite to a fresh `IndividualTextureStore` entry at the
            // trimmed dimensions. Atlas-shared sprites cannot be edited
            // in-place (would corrupt every sibling sharing the same
            // key); we materialise the trim result as a NEW individual
            // texture and repoint only this entity. Individual-source
            // sprites would need a GPU readback to fetch their current
            // pixels — unsupported in V1; surface a toast and bail.
            //
            // World-position preservation: after the crop, the entity's
            // `Transform.translation` is shifted by
            // `ph2d_editor::image_edit::recenter_after_crop` so the *visual* center
            // of the surviving opaque content stays put even when it
            // lived off-center inside the original frame. The shift
            // happens in pure-CPU pixel math (Y-flip handled inside
            // `recenter_after_crop`); HR-5-deterministic.
            // `trim_entity` pre-populated by the consolidated drain
            // (EditorAction::Trim from `pending_trim_transparency`).
            if let Some(entity_bits) = trim_entity
                && hero_intents::drain_trim_transparency(
                    entity_bits,
                    hero.project.pixels_per_meter,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                    toasts,
                    image_edit_undo,
                )
            {
                self.title_dirty = true;
            }
            // Make Square drain — parallel to Trim Transparency. Pads
            // the source image with transparent pixels on the shorter
            // axis so the result is square (width == height), then
            // repoints the sprite to a fresh IndividualTextureStore
            // entry and updates Sprite::size at the current px/m.
            //
            // Audit fixes applied: M1 (cap output dim against
            // device.max_texture_dimension_2d BEFORE acquire — was
            // deferred device-loss); M2 (sub-pixel recenter via
            // recenter_after_pad for odd-diff parity with Trim — was
            // accumulating 0.5 px drift across Trim↔Square cycles);
            // C1 (release OLD individual texture id after a successful
            // re-acquire — was leaking GPU memory on repeated edits).
            // `make_square_entity` pre-populated by the consolidated drain.
            if let Some(entity_bits) = make_square_entity
                && hero_intents::drain_make_square(
                    entity_bits,
                    hero.project.pixels_per_meter,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                    toasts,
                    image_edit_undo,
                )
            {
                self.title_dirty = true;
            }
            // Bg Removal drain — parallel to Trim Transparency, but
            // dimensions are preserved (the algorithm only mutates
            // alpha + optionally despills RGB, never crops). No pivot
            // reproject for that reason. Reads source RGBA via the
            // same Atlas / Individual branch the Trim drain uses,
            // calls `BgRemovalTool::run_full_resolution` at the
            // sprite's native size, and swaps to a fresh Individual
            // texture so the on-canvas sprite picks up the new alpha.
            //
            // Gate on BgRemoval being the active tool — the user is
            // expected to apply WHILE the tool is active (the Apply
            // Toggle lives in its panel). If they switch tools in
            // the same frame, leave the `Bgremoval` variant on the
            // bus so the next activation can complete the round-trip
            // (the filter predicate below only matches when
            // `bgremoval_active`, so the variant is pushed back as
            // a leftover otherwise).
            //
            // The consolidated drain at the top of this section
            // intentionally pushed every `Bgremoval` variant BACK
            // onto the bus (`bgremoval_leftover`). We pick it up
            // here, where the `bgremoval_active` gate runs AFTER
            // any same-frame `ActivateBgRemoval` has already fired
            // — preserving the 1-frame-no-defer contract from the
            // pre-Wave-2.5 `pending_bgremoval` field.
            let bgremoval_id = ph2d_editor::ToolId::new("bgremoval");
            let bgremoval_active = tools
                .active()
                .map(|t| t.id() == bgremoval_id)
                .unwrap_or(false);
            let bgremoval_entity = {
                let mut found: Option<u64> = None;
                let leftovers: Vec<ph2d_editor::action_bus::EditorAction> = hero
                    .bus
                    .drain()
                    .filter_map(|a| match a {
                        ph2d_editor::action_bus::EditorAction::Bgremoval { entity_bits }
                            if bgremoval_active && found.is_none() =>
                        {
                            found = Some(entity_bits);
                            None
                        }
                        other => Some(other),
                    })
                    .collect();
                for a in leftovers {
                    hero.bus.push(a);
                }
                found
            };
            if let Some(entity_bits) = bgremoval_entity {
                let bg = tools
                    .active_mut()
                    .and_then(|t| {
                        t.as_any_mut()
                            .downcast_mut::<ph2d_editor::tools::bgremoval::BgRemovalTool>()
                    })
                    .expect("bgremoval_active gate guarantees a BgRemovalTool");
                if hero_intents::drain_bgremoval(
                    entity_bits,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                    toasts,
                    image_edit_undo,
                    bg,
                    &mut self.last_bgremoval_pushed_entity,
                ) {
                    self.title_dirty = true;
                }
            }
            // Image-edit undo drain. Cmd+Z (or TOOL_UNDO click)
            // pushes `EditorAction::UndoImageEdit` onto the bus; the
            // shell owns the snapshot. Single-level: each new edit
            // overwrites the slot (releasing the previous pre-source),
            // so undo restores at most the MOST RECENT Trim / Make
            // Square / Bg Removal.
            //
            // `undo_image_edit` pre-populated by the consolidated drain
            // (bool — multiple Undo pushes in one frame = one dispatch,
            // mirroring the old flag's edge-triggered semantics).
            if undo_image_edit
                && hero_intents::drain_undo_image_edit(image_edit_undo, sim, renderer, toasts)
            {
                self.title_dirty = true;
            }
            // Publish whether a snapshot is currently stored so the UI
            // can dim the TOOL_UNDO chip when there's nothing to undo.
            hero.has_undoable_image_edit = image_edit_undo.is_some();
            // M14.4c: drain pending import request → open native
            // file picker, import every selected image (PNG/WEBP/
            // JPEG), spawn a sprite per image at the camera center.
            if hero.import_requested {
                hero.import_requested = false;
                let picked = rfd::FileDialog::new()
                    .add_filter("Image (PNG / WEBP / JPEG)", &["png", "webp", "jpg", "jpeg"])
                    .pick_files();
                let pixels_per_meter = hero.project.pixels_per_meter;
                if let Some(paths) = picked {
                    for path in paths {
                        match import_image_at_camera(
                            sim,
                            &mut *renderer,
                            asset_db,
                            camera,
                            *next_import_cell,
                            &path,
                            pixels_per_meter,
                            atlas_asset_map,
                        ) {
                            Ok(spawned_label) => {
                                // Monotonic increment — the Skyline atlas
                                // grows up to 4096²; no slot reuse cycle
                                // (the old `% 8 + 8` math was for the
                                // M5 grid placeholder).
                                *next_import_cell = next_import_cell.saturating_add(1);
                                toasts.push(Toast::success(format!("Imported {spawned_label}")));
                                self.title_dirty = true;
                            }
                            Err(e) => {
                                eprintln!("M14.4c import failed: {e}");
                                toasts.push(Toast::error(format!("Import failed: {e}")));
                                self.title_dirty = true;
                            }
                        }
                    }
                }
            }
            // Active tool's floating panel — same paint as fixture
            // mode below. Was missing in live mode (PH2D_HERO_LIVE=1),
            // so the BgRemoval / Brush / Move panels never showed
            // even when their tool was active (Enio's "Painel
            // BGRemoval não apareceu" report, 2026-05-16). Painted
            // AFTER `paint_hero_screen` so the panel sits on top of
            // canvas / gizmo / grid overlays, and BEFORE toasts so
            // notifications still cover it.
            if !zen.is_active()
                && let Some(active) = tools.active()
            {
                let panel = active.build_panel();
                panel.paint(vector_scene, &mut paint_ctx);
            }
            toasts.paint(vector_scene, &mut paint_ctx);
            // Drain frame-local arena AFTER the dispatch + paint pass
            // so any events emitted earlier this frame are still alive
            // for downstream consumers — wired in Phase A+ (currently
            // events are logged, not acted on).
            hero_arena.reset();
        } else {
            layout.paint(vector_scene, &mut paint_ctx);

            // Tool palette in the CREATE zone (top-right). Always-visible
            // chips; click switches active tool. Hidden in Zen mode by
            // virtue of `tool_palette_rects` returning empty there.
            let palette_rects = layout.tool_palette_rects(tools.tools().len());
            let active_id = tools.active().map(|t| t.id());
            let palette_icons: Vec<(EditorRect, &str, bool)> = palette_rects
                .iter()
                .zip(tools.tools().iter())
                .map(|(r, tool)| {
                    let is_active = active_id.as_ref() == Some(&tool.id());
                    (*r, tool.label(), is_active)
                })
                .collect();
            ph2d_editor::paint_tool_palette_icons(
                paint_ctx.text,
                vector_scene,
                &palette_icons,
                paint_ctx.theme,
            );

            if !zen.is_active()
                && let Some(active) = tools.active()
            {
                // Active tool's panel — built fresh each frame; cheap.
                let panel = active.build_panel();
                panel.paint(vector_scene, &mut paint_ctx);
            }
            toasts.paint(vector_scene, &mut paint_ctx);
        }

        // M14.7 polish (10.1 fix): `surface.acquire_frame()` blocks
        // until the next swap-chain texture is ready — under
        // `PresentMode::Fifo` (the wgpu default + the macOS default)
        // that wait IS the vsync interval, ~16.7 ms at 60 Hz.
        // Including it in the raw-fps measurement caps the reading
        // at the refresh rate, which is exactly what we DON'T want
        // ("Unity shows 2000 fps"). Pause the clock around the
        // acquire, then resume for the actual encode + submit work.
        let work_before_acquire = cpu_start.elapsed();
        match surface.acquire_frame() {
            Ok(frame) => {
                let after_acquire = Instant::now();
                // M14.5 — viewport / RT pipeline. Four GPU submissions
                // each frame, all independent.
                //
                // Pass 1: sprite (+ future light/particle/material)
                //   target: `game_rt` (Rgba16Float HDR offscreen)
                //   ↳ clear color is opaque so the canvas reads as a
                //   single tinted surface beneath sprites + grid.
                renderer.render(
                    game_rt.view(),
                    present,
                    camera,
                    window_size,
                    wgpu::Color { r, g, b, a: 1.0 },
                );
                // Pass 2: AgX tonemap
                //   target: `tonemap.output_view()` (Bgra8UnormSrgb LDR)
                tonemap.run(surface.gpu());
                // Pass 3: Vello chrome
                //   target: `vello_pass.intermediate_view()`
                //   ↳ TRANSPARENT clear so any pixel the editor scene
                //   doesn't paint stays α=0 and the compositor reveals
                //   `game_rt_ldr` through it.
                if let Err(e) = vello_pass.render_to_intermediate(
                    surface.gpu(),
                    vector_scene.inner(),
                    (window_size.width, window_size.height),
                    VelloColor::TRANSPARENT,
                ) {
                    eprintln!("M14.5 vello_pass.render_to_intermediate error: {e}");
                }
                // Pass 4: compositor
                //   reads: tonemap output + vello intermediate
                //   target: swap chain
                compositor.run(surface.gpu(), frame.view());
                // FrameTarget presents on Drop.
                let work_after_acquire = after_acquire.elapsed();
                let cpu_total = work_before_acquire + work_after_acquire;
                let cpu_ms_now = cpu_total.as_secs_f64() * 1000.0;
                const ALPHA_CPU: f32 = 0.1;
                self.frame_cpu_ms_ewma =
                    ALPHA_CPU * (cpu_ms_now as f32) + (1.0 - ALPHA_CPU) * self.frame_cpu_ms_ewma;
            }
            Err(AcquireError::AwaitingReconfigure) => {
                surface.reconfigure_after_lost();
            }
            Err(AcquireError::Occluded) => {}
            Err(AcquireError::Timeout) => {}
            Err(AcquireError::Other(s)) => {
                eprintln!("acquire_frame other error: {s}");
            }
        }

        // Window title carries editor state. Refresh only when state
        // actually changes — winit set_title triggers a platform call.
        if self.title_dirty {
            let tool_label = tools.active().map(|t| t.label()).unwrap_or("none");
            let title = format!(
                "PH2D — M5+M6+M7+M11+M12 demo | sprites={SPRITE_COUNT} | atlas={} ({} assets) \
                 | script={} | theme={:?} | zen={} | toasts={} | tool={}",
                if *atlas_is_real { "PNG" } else { "dummy" },
                asset_db.len_assets(),
                if script.is_some() { "ok" } else { "off" },
                theme,
                if zen.is_active() { "on" } else { "off" },
                toasts.len(),
                tool_label,
            );
            host.window().set_title(&title);
            self.title_dirty = false;
        }

        host.request_redraw();
    }
}

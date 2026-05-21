//! Per-frame render orchestration.
//!
//! Wave 3.1 stage C — `App::render_frame`'s body lifted verbatim from
//! `main.rs` into this sibling. Wave 3.2 stage A splits the lifted
//! body further into per-phase siblings, each implemented as an
//! `impl crate::App` block on a sibling file (split-impl pattern,
//! same as Wave 3.1 used for the initial lift).
//!
//! Phases (called by `run_render_frame` in order):
//!  - `present.rs` — paint + 4 GPU passes + title refresh.
//!  - (more phases land as Wave 3.2 progresses.)

mod bgremoval_preview;
mod hierarchy;
mod image_edit;
mod inspector_commits;
mod present;
mod sim_extract;
mod snapshots;

use crate::*;

use ph2d_editor::interaction::WidgetEvent;
use ph2d_editor::paint::PaintCtx;
use ph2d_editor::zones::Rect as EditorRect;
use ph2d_editor::{Layout as EditorLayout, RequestedSpriteStrategy, Toast, paint_hero_screen};
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
            atlas_is_real: _,
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

        // Sim tick + extract — extracted to sibling `sim_extract.rs`
        // (Wave 3.2 stage A). Runs the bouncing-motion sim tick and
        // the ADR-0021 / ADR-0025 propagate-transforms + sprite
        // emit pass.
        // Demo bouncing-motion integrates ONCE per render frame, so it
        // must use the real wall-clock delta — not the fixed timestep —
        // or its speed scales with the frame rate. That was invisible
        // under vsync (~60 fps) but the non-blocking `Immediate` present
        // mode (stutter fix, 2026-05-21) uncaps the loop to hundreds of
        // fps, which made the sprites race + jitter. `wall_dt` makes the
        // motion frame-rate-independent (real-time, smooth at any fps);
        // clamped so a hitch / debugger pause can't teleport a sprite.
        // (The proper fixed-step substep integration lands with the M10
        // gameplay sim; this is the M5 demo's stop-gap.)
        let dt = (wall_dt as f32).min(1.0 / 30.0);
        // While the Background-Removal tool is active on a selection,
        // suppress that sprite from the sprite pass — its live preview
        // overlay (drawn later, on top of the Vello scene) stands in for
        // it, so the removed (transparent) regions reveal the canvas
        // backdrop rather than the untouched original underneath.
        let bgremoval_preview_entity: Option<u64> = if tools
            .active()
            .map(|t| t.id() == ph2d_editor::ToolId::new("bgremoval"))
            .unwrap_or(false)
        {
            hero_screen.as_ref().and_then(|h| h.gizmo.selection)
        } else {
            None
        };
        sim_extract::run(
            dt,
            sim,
            present,
            renderer,
            prop_state,
            worklist,
            bgremoval_preview_entity,
        );

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
            // Snapshot publication phase — extracted to sibling
            // `snapshots.rs` as a free fn taking explicit refs (Wave
            // 3.2 stage A). Reads PresentWorld + SimWorld + AssetDb,
            // writes onto the HeroScreen (live_hierarchy, grid_view,
            // stats, gizmo_view, inspector_*) so the paint pass
            // honors the HR-8 / ADR-0021 boundary.
            snapshots::publish(
                hero,
                hero_live,
                sim,
                present,
                camera,
                asset_db,
                atlas_asset_map,
                window_size,
                self.last_pointer,
                self.frame_ms_ewma,
                self.frame_cpu_ms_ewma,
            );
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
            let mut real_size_entity: Option<u64> = None;
            let mut undo_image_edit = false;
            let mut transform_edit: Option<ph2d_editor::InspectorTransformInfo> = None;
            let mut visibility_edit: Option<ph2d_editor::InspectorVisibilityInfo> = None;
            let mut sprite_source_change: Option<(u64, RequestedSpriteStrategy)> = None;
            let mut name_edit: Option<ph2d_editor::InspectorNameInfo> = None;
            let mut bgremoval_leftover: Vec<ph2d_editor::action_bus::EditorAction> = Vec::new();
            let mut bgremoval_ui_edits: Vec<ph2d_editor::tools::bgremoval::BgRemovalUiEdit> =
                Vec::new();
            let mut bgremoval_cancel = false;
            for action in hero.bus.drain() {
                use ph2d_editor::action_bus::EditorAction;
                match action {
                    EditorAction::ActivateBgRemoval => activate_bgremoval = true,
                    EditorAction::BgremovalUiEdit(edit) => bgremoval_ui_edits.push(edit),
                    EditorAction::BgremovalCancel => bgremoval_cancel = true,
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
                    EditorAction::RealSize { entity_bits } => {
                        real_size_entity.get_or_insert(entity_bits);
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
                    EditorAction::SetImageFilter { mode } => {
                        // Single global image-filter toggle. Rebuilds the
                        // atlas + individual samplers and their bind groups
                        // so EVERY sprite samples with the new mode; no
                        // texture re-upload. The Vello BG-Removal preview
                        // reads `hero.project.image_filter` directly (set by
                        // the editor before this action), so both stay in
                        // sync.
                        renderer.set_filter_mode(mode);
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
            // Cancel Background Removal: deactivate the tool by switching
            // back to the default (first-registered) tool. The preview
            // is dropped below (visibility gate sets it None when the
            // tool is no longer active), the sprite un-suppresses, and
            // the Inspector returns.
            if bgremoval_cancel
                && let Some(default_id) = tools.tools().first().map(|t| t.id())
                && tools.set_active(&default_id)
            {
                self.last_bgremoval_pushed_entity = None;
                self.bgremoval_preview = None;
                self.title_dirty = true;
            }
            // Bg Removal panel ⟷ tool bridge + on-canvas live preview
            // — extracted to sibling `bgremoval_preview.rs` (HR-18 LOC).
            let bgremoval_apply_committed = bgremoval_preview::dispatch(
                hero,
                tools,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                camera,
                window_size,
                vector_scene,
                bgremoval_ui_edits,
                &mut self.last_bgremoval_pushed_entity,
                &mut self.bgremoval_preview,
            );
            paint_hero_screen(hero, viewport, vector_scene, paint_ctx.text);
            // Hierarchy intent dispatch phase — camera reset +
            // view-focus + 9 hierarchy intents (visibility_toggle /
            // reparent / duplicate / add_child / reset_transform /
            // delete / row_click / rename_seed / rename_commit).
            // Extracted to sibling `hierarchy.rs` as a free fn (Wave
            // 3.2 stage A).
            if hierarchy::dispatch(
                view_focus_kind,
                visibility_toggle_row,
                reparent_intent,
                duplicate_row,
                add_child_row,
                reset_transform_row,
                delete_row,
                hierarchy_row_click,
                rename_seed_row,
                rename_commit,
                hero,
                hero_live,
                sim,
                present,
                camera,
                toasts,
                window_size,
            ) {
                self.title_dirty = true;
            }
            // Inspector commits phase — Transform / Visibility / Name
            // / Sprite source-strategy + Reimport. Extracted to sibling
            // `inspector_commits.rs` as a free fn (Wave 3.2 stage A).
            if inspector_commits::dispatch(
                reimport_entity,
                transform_edit,
                visibility_edit,
                name_edit,
                sprite_source_change,
                hero,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                toasts,
                editor_queue,
                component_registry,
                *transform_type_id,
                *visibility_type_id,
                *name_type_id,
                *sprite_type_id,
            ) {
                self.title_dirty = true;
            }
            // Image-edit drain phase + file-picker import — extracted
            // to sibling `image_edit.rs` as a free fn (Wave 3.2 stage A).
            // Returns whether any drain pushed a toast.
            if image_edit::dispatch(
                trim_entity,
                make_square_entity,
                real_size_entity,
                undo_image_edit,
                hero,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                toasts,
                image_edit_undo,
                tools,
                camera,
                next_import_cell,
                &mut self.last_bgremoval_pushed_entity,
            ) {
                self.title_dirty = true;
            }
            // Apply teardown — runs AFTER the bake above (which needs
            // the BgRemovalTool still active to read the result). Now
            // that the committed alpha lives in the sprite texture,
            // deactivate the tool exactly like Cancel: the panel hides,
            // the sprite un-suppresses, the Inspector returns, and the
            // on-canvas preview overlay stops re-rendering on top of the
            // freshly baked sprite (that double-draw was the ghost edge
            // outline that appeared only while the image stayed selected).
            if bgremoval_apply_committed
                && let Some(default_id) = tools.tools().first().map(|t| t.id())
                && tools.set_active(&default_id)
            {
                self.last_bgremoval_pushed_entity = None;
                self.bgremoval_preview = None;
                self.title_dirty = true;
            }
            // Legacy `FloatingPanel` Procreate-style paint was retired
            // here (2026-05-17). The pink/magenta tab-strip + Accent
            // toggle decoration was inconsistent with the canonical
            // dark-glass surface used by Inspector / Hierarchy /
            // Widget Gallery. `Tool::build_panel()` still exists for
            // event dispatch but the visual is dropped; per-tool
            // chrome rewires through the new panel style in a
            // follow-up wave (BgRemoval especially needs its preview
            // panel re-painted; Move/Brush were stubs anyway).
            let _ = tools;
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

            // Legacy `FloatingPanel` paint retired (2026-05-17). Same
            // rationale as the live-mode branch above. Tool palette
            // chrome above remains because it's the click entrypoint
            // to switch tools; the per-tool panel itself is gone.
            toasts.paint(vector_scene, &mut paint_ctx);
        }

        // Paint + present + title — extracted to `present.rs` sibling
        // method (Wave 3.2 stage A). Re-acquires self.gfx + self.host
        // refs inside; values needed are passed explicitly.
        self.run_present_phase(cpu_start, r, g, b);
    }
}

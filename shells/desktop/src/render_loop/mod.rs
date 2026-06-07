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
//!
// ph2d-loc-cap: frame orchestrator — heavy phases already extracted to
// siblings (present/image_edit/sim_extract/snapshots/bgremoval_preview/
// hierarchy); residual is the frame skeleton + EditorAction intent drain.
// FOLLOW-UP: extract the intent-drain match to a `intents.rs` sibling to
// drop back under the cap (2026-05-21: +SetPresentMode/RealSize tipped it).

mod bgremoval_preview;
mod color_equalization_bridge;
mod cooked_texture_bridge;
mod equalize_sizes_bridge;
mod hierarchy;
mod image_edit;
mod inspector_commits;
mod inspector_ordering;
mod inspector_visibility;
mod motion_smoke;
mod padding_bridge;
pub(crate) mod painter_bridge;
// `pub(crate)`: `apply_layer_reparent` is called from `input_dispatch` (outside
// render_loop) to route the W3.T3.8 layer drag-reparent through the allowlisted
// bridge-queries module instead of downcasting in central dispatch.
pub(crate) mod painter_bridge_queries;
// W15.3 GPU fluid drive (ADR-0049). Feature-gated — default build excludes it.
#[cfg(feature = "fluid")]
pub(crate) mod painter_fluid_bridge;
mod painter_gpu_flatten;
pub(crate) mod painter_gpu_preview;
mod present;
mod sim_extract;
mod snapshots;
mod upscale_bridge;
mod vector_graph_bridge;
mod vector_inspector_bridge;
mod vector_pen_bridge;
mod vector_pencil_bridge;
pub(crate) mod vector_scene;
mod vector_selection_bridge;
mod vector_shape_bridge;

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

        // P4 (ADR-0061): drain any finished background LLM-vector generation and
        // commit it to the live document. Cheap non-blocking try_recv; runs here
        // in the clean-borrow region before the `gfx` split below.
        self.poll_llm_vector();

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
            sort_scratch,
            sort_inputs,
            hero_live,
            next_import_cell,
            atlas_asset_map,
            logical_texture_map,
            component_registry,
            editor_queue,
            transform_type_id,
            visibility_type_id,
            name_type_id,
            sprite_type_id,
            image_edit_undo,
            // ADR-0054 W0.T6: registries held but not yet consumed
            // inside the render loop — W1 wires Open/Save user paths
            // through `imageio_importers.find_for(...)`.
            imageio_importers: _,
            imageio_exporters: _,
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
            // Size every offscreen RT to the surface's CLAMPED size, not the
            // raw winit size. `surface.resize` clamps each dim to ≥1, and
            // `game_rt.ensure_size` REJECTS a 0 dim (keeps its old size). On a
            // transient 0-dimension frame (minimize/restore) the raw size
            // would diverge: game_rt stays old while the W3 §8 clip/mask
            // stencil sizes to `surface.size()` → a render pass pairing the
            // game_rt color attachment with a differently-sized stencil =
            // wgpu validation panic. Using the clamped size keeps color +
            // stencil extents equal every frame (audit MEDIUM fix).
            let clamped = surface.size();
            let dim = (clamped.width, clamped.height);
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
        // **W15.1 (ADR-0040-amendment-2):** per-frame heartbeat on the ACTIVE tool,
        // with the real frame delta. Drives the watercolor live wet-on-wet diffusion
        // (ADR-0049 / ADR-0077 D11) so the wash keeps blooming + drying after pen-up;
        // a no-op default for every other tool, so this costs nothing elsewhere.
        if let Some(t) = tools.active_mut() {
            t.on_tick(frame_ms_now);
        }
        // **W15.3 (ADR-0049, feature `fluid`):** drive the painter's live wet field
        // on the GPU (else the CPU `on_tick` above already stepped it). No-op
        // without an active fluid stroke; absent in the default build.
        #[cfg(feature = "fluid")]
        painter_fluid_bridge::drive_fluid_gpu(tools, surface.gpu());
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
        // Lens F (2026-05-26): the Background-Removal live preview no
        // longer suppresses the sprite + paints a Vello overlay on
        // top; instead it injects a synthetic `PreviewOverride` that
        // swaps the entity's `RenderInstance.texture_id` for a
        // transient `IndividualTextureStore` slot owning the preview
        // pixels. Same `Rgba8UnormSrgb` + sprite shader + premul
        // blend as Apply → byte-for-byte parity. The GPU slot is
        // populated by `bgremoval_preview::dispatch` LATER in the
        // frame, so reading `self.bgremoval_preview_gpu` here picks
        // up last frame's upload (1-frame lag is invisible — the
        // preview is a continuous animation).
        let bgremoval_preview_override: Option<sim_extract::PreviewOverride> = self
            .bgremoval_preview_gpu
            .map(|gpu| sim_extract::PreviewOverride {
                entity_bits: gpu.entity_bits,
                texture_id: gpu.texture_id,
                // Byte-space premul upload + Apply uses the same flag.
                premultiplied: true,
            });
        // W3 Painter sprite-suppression: same single `preview_override` slot.
        // The base layer composite REPLACES the source sprite in-place (no
        // overlay duplication) so its opacity/representation affects the whole
        // image. Painter and BgRemoval are never active simultaneously (one
        // active tool), so `.or()` picks whichever is live.
        let painter_preview_override: Option<sim_extract::PreviewOverride> = self
            .painter_preview_gpu
            .map(|gpu| sim_extract::PreviewOverride {
                entity_bits: gpu.entity_bits,
                texture_id: gpu.texture_id,
                premultiplied: true,
            });
        let preview_override = painter_preview_override.or(bgremoval_preview_override);
        // W2.T3 visual smoke (PH2D_MOTION_SMOKE=1): the Motion vertical owns the
        // canvas this frame — cook grid→transform→clone and publish its
        // RenderInstances instead of the M5 demo sim. Debug-only; off by default.
        if motion_smoke::enabled() {
            motion_smoke::run(present, renderer);
        } else {
            // Project px/m for `Sprite::resolve_anchor` (intrinsic-px
            // `offset` → local meters). `None` only under the M5 demo /
            // headless, whose sprites use the centered/offset defaults so
            // the value is inert; fall back to the canonical default.
            let ppm = hero_screen
                .as_ref()
                .map(|h| h.project.pixels_per_meter)
                .unwrap_or(ph2d_editor::project::DEFAULT_PIXELS_PER_METER);
            // W3.T3.11: project-default sampling for all-Inherit sprites,
            // from the project image filter (PixelArt → Nearest, Smooth →
            // Linear); repeat defaults to clamp (Disabled).
            let default_filter = match hero_screen
                .as_ref()
                .map(|h| h.project.image_filter)
                .unwrap_or(ph2d_render::ImageFilterMode::Smooth)
            {
                ph2d_render::ImageFilterMode::PixelArt => ph2d_ecs::FilterMode::Nearest,
                ph2d_render::ImageFilterMode::Smooth => ph2d_ecs::FilterMode::Linear,
            };
            // W2.T4 cooked-texture loader: resolve + decode + upload every
            // `SpriteSource::CookedTexture` sprite's KTX2 (for the device tier,
            // descending the fallback ladder) BEFORE extract reads back the
            // cached `texture_id`. Idempotent + cheap after the first upload.
            cooked_texture_bridge::ensure_uploaded(sim, renderer, asset_db, logical_texture_map);
            sim_extract::run(
                dt,
                sim,
                present,
                renderer,
                prop_state,
                worklist,
                sort_scratch,
                sort_inputs,
                preview_override,
                ppm,
                camera.cull_mask,
                default_filter,
                ph2d_ecs::RepeatMode::Disabled,
            );
        }

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
                renderer,
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
            // `ActivateTool { tool_id: "bgremoval" }` fires (1-frame
            // defer edge case).
            //
            // Audit 2026-05-26 F1: 6 flags hardcoded per-tool (`activate_bgremoval`
            // etc.) substituídas por uma única option `pending_image_tool_activation`.
            // O drain único abaixo usa `installed_registry().cluster("image_tools")`
            // + `Tool::label()` para dispatch data-driven. Painter + os 5 image-tools
            // pré-existentes flow pelo mesmo canal — anti-padrão Image Tools Bugs
            // §2.b fechado neste ponto da render loop.
            let mut pending_image_tool_activation: Option<&'static str> = None;
            let mut visibility_toggle_row: Option<NodeId> = None;
            let mut lock_toggle_row: Option<NodeId> = None;
            let mut group_toggle_row: Option<NodeId> = None;
            let mut reparent_intent: Option<ph2d_editor::screens::hero::HierReparentIntent> = None;
            let mut duplicate_row: Option<NodeId> = None;
            let mut add_child_row: Option<NodeId> = None;
            let mut reset_transform_row: Option<NodeId> = None;
            let mut delete_row: Option<NodeId> = None;
            // Enio 2026-05-27: right-click → Merge Sprites in Hierarchy.
            // Carries the clicked row's `NodeId` (the merged sprite
            // adopts that row's parent for Hierarchy placement); the
            // drain reads the full multi-selection at apply time.
            let mut merge_sprites_row: Option<NodeId> = None;
            let mut hierarchy_row_click: Option<NodeId> = None;
            let mut hierarchy_select_intent: Option<hierarchy::HierarchySelectIntent> = None;
            let mut rename_seed_row: Option<NodeId> = None;
            let mut rename_commit: Option<(NodeId, String)> = None;
            let mut view_focus_kind: Option<ph2d_editor::ViewFocusKind> = None;
            let mut reimport_entity: Option<u64> = None;
            // Fase 0e: per-sprite tools collect a Vec<u64> instead of
            // Option<u64> so a multi-select OneShotImageOp broadcast
            // applies the bake to every selected sprite (legacy
            // single-select still works — the Vec just carries one
            // entry). image_edit::dispatch iterates each Vec.
            let mut trim_entities: Vec<u64> = Vec::new();
            let mut make_square_entities: Vec<u64> = Vec::new();
            let mut real_size_entities: Vec<u64> = Vec::new();
            let mut rasterize_entities: Vec<u64> = Vec::new();
            let mut undo_image_edit = false;
            let mut transform_edit: Option<ph2d_editor::InspectorTransformInfo> = None;
            let mut visibility_edit: Option<ph2d_editor::InspectorVisibilityInfo> = None;
            let mut sprite_source_change: Option<(u64, RequestedSpriteStrategy)> = None;
            // Sprite field edits (flip/region/sheet/tint/…) — a Vec so a
            // bulk edit that touches several fields in one frame all apply.
            let mut sprite_edits: Vec<(u64, ph2d_editor::SpriteFieldEdit)> = Vec::new();
            // §7 ordering edits (W3) — optional-component edits, fanned out
            // to the selection like sprite edits.
            let mut ordering_edits: Vec<(u64, ph2d_editor::OrderingFieldEdit)> = Vec::new();
            let mut sampling_edits: Vec<(u64, ph2d_editor::SamplingFieldEdit)> = Vec::new();
            let mut blend_edits: Vec<(u64, ph2d_editor::BlendFieldEdit)> = Vec::new();
            let mut visibility_section_edits: Vec<(u64, ph2d_editor::VisibilityFieldEdit)> =
                Vec::new();
            let mut name_edit: Option<ph2d_editor::InspectorNameInfo> = None;
            let mut bgremoval_leftover: Vec<ph2d_editor::action_bus::EditorAction> = Vec::new();
            // Painter Apply leftover — same shape as bgremoval (drained
            // back into the bus so `image_edit::dispatch`'s
            // `painter_active` gate runs AFTER any same-frame
            // ActivateTool resolution). Day-7 ship.
            let mut painter_leftover: Vec<ph2d_editor::action_bus::EditorAction> = Vec::new();
            // BulkSelect (T2.0): the live selection (primary + extras),
            // captured before the drain so an Inspector sprite edit can
            // fan out to every selected sprite. Only allocated for a
            // MULTI-selection; single-select takes the empty path and the
            // edit's own `entity_bits` (no per-frame alloc — audit D-5).
            let inspector_selection: Vec<u64> = if hero.gizmo.selected_len() > 1 {
                hero.gizmo.iter_selected().collect()
            } else {
                Vec::new()
            };
            for action in hero.bus.drain() {
                use ph2d_editor::action_bus::EditorAction;
                match action {
                    // ADR-0040 TG-A: generic activation. Per-tool flags
                    // preserve the existing mode_on gating / activation
                    // side effects after the drain.
                    // ADR-0040 TG-A: generic activation. Audit F1 (2026-05-26):
                    // data-driven via cluster lookup no drain abaixo; sem
                    // per-tool flag flooding.
                    EditorAction::ActivateTool { tool_id } => {
                        pending_image_tool_activation = Some(tool_id);
                    }
                    // ADR-0040 TG-B: generic panel→tool channel. Route the
                    // event to the active tool's `handle_panel_event` —
                    // semantic mapping (slider id → typed UI edit) lives on
                    // the tool, not here.
                    EditorAction::ToolPanelEvent(ev) => {
                        if let Some(t) = tools.active_mut() {
                            t.handle_panel_event(ev);
                        }
                    }
                    // ADR-0040 TG-B/TG-C: generic "cancel the active modal
                    // tool". Switch back to the default tool and tear down
                    // any image-tool shell-side preview caches. Bg Removal +
                    // Padding panels both raise this; the bgremoval cleanup
                    // is a no-op when padding (or any non-bgremoval tool)
                    // was active. Padding's shell-side state is purely
                    // tool-internal (no shell-cached preview), so no
                    // padding-specific cleanup is needed here.
                    EditorAction::CancelActiveTool => {
                        // Audit H7 — destructive-deactivate warn (mirror of the
                        // Painter unflushed-strokes warn below). Toggling the Pen
                        // pill off while a path is mid-author runs
                        // `on_deactivate → reset_path`, silently discarding the
                        // in-progress vertices. Surface the loss via toast BEFORE
                        // set_active fires the deactivate path. Downcast lives in
                        // the allowlisted bridge; this dispatch stays downcast-free.
                        if vector_pen_bridge::pen_has_in_progress_path(tools) {
                            toasts.push(Toast::warning(
                                "Vector Pen: in-progress path discarded. Close \
                                 the path or press Esc to cancel deliberately."
                                    .to_string(),
                            ));
                        }
                        if vector_pencil_bridge::pencil_has_in_progress_stroke(tools) {
                            toasts.push(Toast::warning(
                                "Vector Pencil: in-progress stroke discarded. \
                                 Finish the stroke or press Esc to cancel deliberately."
                                    .to_string(),
                            ));
                        }
                        if vector_shape_bridge::shape_has_in_progress_shape(tools) {
                            toasts.push(Toast::warning(
                                "Vector Shape: in-progress shape discarded. \
                                 Finish the drag or press Esc to cancel deliberately."
                                    .to_string(),
                            ));
                        }
                        if let Some(default_id) = tools.default_tool_id()
                            && tools.set_active(&default_id)
                        {
                            self.last_bgremoval_pushed_entity = None;
                            self.bgremoval_preview = None;
                            self.title_dirty = true;
                        }
                    }
                    EditorAction::UndoImageEdit => undo_image_edit = true,
                    EditorAction::HierToggleVisibility { row } => {
                        visibility_toggle_row.get_or_insert(row);
                    }
                    EditorAction::HierToggleLock { row } => {
                        lock_toggle_row.get_or_insert(row);
                    }
                    EditorAction::HierToggleGroup { row } => {
                        group_toggle_row.get_or_insert(row);
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
                    EditorAction::HierMergeSprites { row } => {
                        merge_sprites_row.get_or_insert(row);
                    }
                    EditorAction::HierRowClick { row } => {
                        hierarchy_row_click.get_or_insert(row);
                    }
                    // Fase 0e: multi-select-aware hierarchy click +
                    // shift-range. Collect into a single latest-wins
                    // intent — the dispatch resolves row → entity_bits
                    // and applies the matching `GizmoStateGroup`
                    // mutation. Range overrides Row when both arrive
                    // in the same frame (the user can only be in one
                    // selection-gesture at a time).
                    EditorAction::HierSelectRow { row, modifier }
                        if !matches!(
                            hierarchy_select_intent,
                            Some(hierarchy::HierarchySelectIntent::Range { .. })
                        ) =>
                    {
                        hierarchy_select_intent =
                            Some(hierarchy::HierarchySelectIntent::Row { row, modifier });
                    }
                    EditorAction::HierRangeSelect { row } => {
                        hierarchy_select_intent =
                            Some(hierarchy::HierarchySelectIntent::Range { row });
                    }
                    // Fase 0e: canvas-side select via the bus (reserved
                    // for callers that don't have direct hero access —
                    // input_dispatch.rs:435 mutates hero.gizmo directly
                    // because it already holds the borrow).
                    EditorAction::SelectSprite {
                        entity_bits,
                        modifier,
                    } => match modifier {
                        ph2d_editor::action_bus::SelectModifier::Replace => {
                            hero.gizmo.replace_selection(Some(entity_bits));
                        }
                        ph2d_editor::action_bus::SelectModifier::Add => {
                            hero.gizmo.add_to_selection(entity_bits);
                        }
                        ph2d_editor::action_bus::SelectModifier::Toggle => {
                            hero.gizmo.toggle_in_selection(entity_bits);
                        }
                    },
                    EditorAction::ClearSelection => {
                        hero.gizmo.clear_all_selection();
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
                    // ADR-0040 TG-A: generic one-shot image-op dispatch.
                    // Trim/MakeSquare/RealSize collect into per-tool Option<u64>
                    // for the existing per-tool drain functions; bgremoval bake
                    // is deferred via leftover (must run AFTER ActivateTool
                    // has switched the tool active, image_edit.rs:184 picks it up).
                    oneshot @ EditorAction::OneShotImageOp {
                        tool_id,
                        entity_bits,
                    } => match tool_id {
                        "trim_transparency" => {
                            trim_entities.push(entity_bits);
                        }
                        "make_square" => {
                            make_square_entities.push(entity_bits);
                        }
                        "real_size" => {
                            real_size_entities.push(entity_bits);
                        }
                        "rasterize" => {
                            rasterize_entities.push(entity_bits);
                        }
                        "bgremoval" => {
                            bgremoval_leftover.push(oneshot);
                        }
                        "painter" => {
                            painter_leftover.push(oneshot);
                        }
                        _ => {}
                    },
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
                    EditorAction::InspectorSpriteEdit { entity_bits, edit } => {
                        // BulkSelect: apply to EVERY selected sprite, not
                        // just the dispatching (primary) entity. The Vec
                        // includes the primary first; single-select pushes
                        // one. Fall back to the edit's own entity if the
                        // selection snapshot is empty (stale dispatch).
                        if inspector_selection.is_empty() {
                            sprite_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                sprite_edits.push((t, edit));
                            }
                        }
                    }
                    EditorAction::InspectorOrderingEdit { entity_bits, edit } => {
                        // BulkSelect fan-out, same shape as the sprite edit.
                        if inspector_selection.is_empty() {
                            ordering_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                ordering_edits.push((t, edit));
                            }
                        }
                    }
                    EditorAction::InspectorSamplingEdit { entity_bits, edit } => {
                        if inspector_selection.is_empty() {
                            sampling_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                sampling_edits.push((t, edit));
                            }
                        }
                    }
                    EditorAction::InspectorBlendEdit { entity_bits, edit } => {
                        if inspector_selection.is_empty() {
                            blend_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                blend_edits.push((t, edit));
                            }
                        }
                    }
                    EditorAction::InspectorVisibilitySectionEdit { entity_bits, edit } => {
                        // BulkSelect fan-out, same shape as the sampling edit.
                        if inspector_selection.is_empty() {
                            visibility_section_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                visibility_section_edits.push((t, edit));
                            }
                        }
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
                    EditorAction::SetPresentMode { vsync } => {
                        // Config → Display toggle. VSync (Fifo) = smooth
                        // hardware-paced motion; Immediate = non-blocking
                        // (no mouse-stutter). Reconfigures the swap chain
                        // in place. Both modes are available on this
                        // backend (boot log confirms); Fifo is the
                        // universal fallback.
                        surface.set_present_mode(if vsync {
                            wgpu::PresentMode::Fifo
                        } else {
                            wgpu::PresentMode::Immediate
                        });
                    }
                    EditorAction::SetApiKey(key) => {
                        // P4 (ADR-0061): persist the Anthropic key to the user
                        // config dir; the LLM-vector feature resolves it lazily
                        // on the next generation. Empty input clears it.
                        let cleared = key.trim().is_empty();
                        match crate::llm_vector::save_api_key(&key) {
                            Ok(()) => {
                                toasts.push(Toast::info(
                                    if cleared {
                                        "Anthropic API key cleared."
                                    } else {
                                        "Anthropic API key saved."
                                    }
                                    .to_string(),
                                ));
                            }
                            Err(e) => {
                                toasts.push(Toast::warning(format!("Failed to save API key: {e}")));
                            }
                        }
                    }
                    EditorAction::GenerateVectorFromPrompt(prompt) => {
                        // P4 (ADR-0061): run the LLM generation off-thread; the
                        // per-frame poll_llm_vector commits the result. `llm_vector`
                        // is a disjoint App field (not gfx) so it's reachable here.
                        let started = self.llm_vector.submit(prompt);
                        toasts.push(Toast::info(
                            if started {
                                "Generating a vector shape…"
                            } else {
                                "A vector generation is already in progress."
                            }
                            .to_string(),
                        ));
                    }
                    // (Bgremoval bake leftover handled inside the
                    // `OneShotImageOp` arm above — defers to the
                    // image_edit drain site so `bgremoval_active` is
                    // observed AFTER any same-frame ActivateTool fires.)
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
            for a in painter_leftover {
                hero.bus.push(a);
            }
            // Drain the `EditorAction::ActivateTool { tool_id: "bgremoval" }`
            // intent raised by clicking the Bg Removal pill. The hero can't reach
            // `gfx.tools` so the activation round-trips via the bus.
            // Same force-refresh of the snapshot push state as the
            // Digit3 shortcut below so the next snapshot push fires
            // against the current selection.
            // Data-driven activation of any stateful image-tool (audit F1
            // 2026-05-26 — substitui 6 drain blocks hardcoded per-tool).
            // Gated on `mode_on`: image tools are only reachable while Image
            // Tools toggle is on (the pills only exist then; the Digit3
            // shortcut must also respect the mode). The reconcile below is
            // the safety net, but gating here avoids a 1-frame
            // activate→deactivate flicker + a spurious toast.
            //
            // Cluster lookup via `installed_registry()` resolve o handler kind
            // (Stateful vs OneShot) e o label canônico (`Tool::label()`); zero
            // hardcoded id no dispatch. Tools dropped via fan-out drop-crate
            // (incluindo Painter T1.1) flow pelo mesmo canal automaticamente.
            //
            // Legacy débito: `last_bgremoval_pushed_entity = None` reset é
            // bgremoval-specific shell cache. Em T-N.X (refactor cache-per-tool
            // map) substituído por `HashMap<ToolId, ShellCache>` ou hook em
            // `Tool::on_activate` (ADR-0041). Por hoje, mantido inline.
            if let Some(tool_id) = pending_image_tool_activation.take() {
                // Look up the activating tool's cluster + Stateful gate.
                // W1.T1.7 generalization: was "image_tools" only; now also
                // accepts "vector_tools" (Pen tool ship). When a third
                // cluster appears, add it here OR extract a generic
                // `find_activatable_stateful_tool` helper.
                let activating_cluster: Option<&'static str> = ph2d_editor::installed_registry()
                    .and_then(|reg| {
                        ["image_tools", "vector_tools"]
                            .into_iter()
                            .find(|&cluster_name| {
                                reg.cluster(cluster_name).iter().any(|m| {
                                    m.id == tool_id
                                        && matches!(
                                            m.handler,
                                            ph2d_tool_registry::ToolHandler::Stateful { .. }
                                        )
                                })
                            })
                    });
                // Per-cluster activation gate. "image_tools" requires
                // the IMG mode toggle; "vector_tools" has no toggle in
                // W1 so always-on (Pen pill is direct-activate).
                let gate_on = match activating_cluster {
                    Some("image_tools") => hero.image_edit.mode_on,
                    Some("vector_tools") => true,
                    _ => false,
                };
                if gate_on && tools.set_active(&ph2d_editor::ToolId::new(tool_id)) {
                    self.title_dirty = true;
                    if tool_id == "bgremoval" {
                        self.last_bgremoval_pushed_entity = None;
                    }
                    if let Some(active) = tools.active() {
                        toasts.push(Toast::info(format!("Tool · {}", active.label())));
                    }
                    // (R4: Pen activation no longer needs a sprite —
                    // network IS the asset, world-coords throughout.)
                }
            }
            // Image Tools OFF is AUTHORITATIVE over the active tool. The
            // TopBar Image Tools toggle (`image_edit.mode_on`) and the
            // ToolRegistry's active tool are otherwise decoupled: a
            // stateful image tool (Bg Removal / Padding) activated while
            // the mode was on stays active — panel + on-canvas preview and
            // all — after the mode is toggled off, since nothing
            // deactivated it. Reconcile here every frame, BEFORE the
            // panel/preview bridges run: when the mode is off, no
            // image-edit tool may remain active, so switch back to the
            // default tool and drop the Bg-Removal preview. This is the
            // single invariant that makes "Image Tools off ⟹ every image
            // tool off & inaccessible" hold no matter how the tool became
            // active (toggle-off, a stale path, the Digit3 shortcut).
            if !hero.image_edit.mode_on {
                let active_is_image_tool = tools
                    .active()
                    .map(|t| crate::is_image_edit_tool(&t.id()))
                    .unwrap_or(false);
                // **Audit T1.6 R7 L1-1 — destructive-deactivate warn.**
                // Painter is registered in the `image_tools` cluster
                // (for TopBar pill placement) but unlike bgremoval /
                // padding / etc., Painter is a workhorse stateful tool:
                // its `Tool::on_deactivate` → `RasterEditTool::deactivate`
                // wipes `canvas_rgba` to `Vec::new()`. A user mid-paint
                // who toggles Image Tools OFF previously lost every
                // unflushed stroke silently — no warn, no toast, no
                // recovery. Surface the loss via toast BEFORE the
                // set_active fires the destructive deactivate path.
                if active_is_image_tool
                    && tools
                        .active()
                        .map(|t| t.id().0.as_str() == "painter")
                        .unwrap_or(false)
                {
                    // Tool-concrete downcast lives in the allowlisted
                    // painter_bridge (this central dispatch stays downcast-free
                    // per the arch-gate).
                    let lost_painting =
                        painter_bridge_queries::painter_has_unflushed_strokes(tools);
                    if lost_painting {
                        toasts.push(Toast::warning(
                            "Painter: unflushed strokes were discarded when \
                             Image Tools toggled off. Use Apply before \
                             switching the mode."
                                .to_string(),
                        ));
                    }
                }
                if active_is_image_tool
                    && let Some(default_id) = tools.default_tool_id()
                    && tools.set_active(&default_id)
                {
                    self.bgremoval_preview = None;
                    self.last_bgremoval_pushed_entity = None;
                    self.title_dirty = true;
                }
            }
            // Reconcile Image Tools pill ButtonState ↔ active tool. Each pill
            // whose manifest id matches `tools.active()` is forced to Pressed;
            // pills holding a stale Pressed (tool no longer active) drop back
            // to Normal. Hovered/click-transient states are preserved (we only
            // touch the Normal↔Pressed transitions).
            //
            // Data-driven via `installed_registry().cluster("image_tools")` —
            // zero hardcoded tool id (anti-padrão Image Tools Bugs §2.b
            // fechado em T1.2). New tools dropped via fan-out drop-crate
            // inherit the highlight wiring automatically.
            {
                let active_id_string: Option<String> = tools.active().map(|t| t.id().0.clone());
                if let Some(reg) = ph2d_editor::installed_registry() {
                    // W1.T1.7 R3: iterate both image_tools (existing)
                    // AND vector_tools (Pen pill ship) so the Pressed-
                    // highlight reconcile picks up the Pen pill when
                    // the Vector Pen tool activates. Each pill's
                    // NodeId is computed via `hash_node_id(manifest.id)`
                    // — for Pen this matches `TOPBAR_VECTOR_PEN` only
                    // because `TOPBAR_VECTOR_PEN = hash_node_id("vector_pen")`
                    // (image-action pill convention; see ids.rs).
                    for cluster_name in ["image_tools", "vector_tools"] {
                        for manifest in reg.cluster(cluster_name) {
                            let pill_id = ph2d_tool_registry::hash_node_id(manifest.id);
                            let should_press = active_id_string.as_deref() == Some(manifest.id);
                            if let Some(ph2d_editor::InteractiveState::Button { state }) =
                                hero.store.get_mut(pill_id)
                            {
                                use ph2d_editor::widget::ButtonState;
                                match (*state, should_press) {
                                    (ButtonState::Normal, true) => *state = ButtonState::Pressed,
                                    (ButtonState::Pressed, false) => *state = ButtonState::Normal,
                                    _ => {} // preserve Hovered + already-consistent
                                }
                            }
                        }
                    }
                }
            }
            // Padding panel ⟷ tool bridge — publishes the snapshot, draws
            // the live (non-destructive) canvas-bounds preview, and returns
            // the (selection, spec, pivot mode) to bake on Apply. Panel
            // events themselves are routed earlier in the frame via
            // `EditorAction::ToolPanelEvent` → `Tool::handle_panel_event`
            // (ADR-0040 TG-C). Sibling `padding_bridge.rs`.
            let padding_apply =
                padding_bridge::dispatch(hero, tools, sim, camera, window_size, vector_scene);
            // Bg Removal panel ⟷ tool bridge + on-canvas live preview
            // — extracted to sibling `bgremoval_preview.rs` (HR-18 LOC).
            // Panel events now flow through `EditorAction::ToolPanelEvent`
            // (drained above into `handle_panel_event` → `apply_ui_edit`);
            // the canvas-preview cache is gated on `BgRemovalTool::take_params_dirty`
            // instead of a per-frame edits vector (ADR-0040 TG-B).
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
                &mut self.last_bgremoval_pushed_entity,
                &mut self.bgremoval_preview,
                &mut self.bgremoval_preview_gpu,
                toasts,
            );
            // Color Equalization panel ⟷ tool bridge: drives panel
            // visibility, refreshes the tool's source bitmap when the
            // primary changes, publishes the snapshot the panel paints,
            // and returns the multi-selection on Apply for the bake.
            let color_equalization_apply = color_equalization_bridge::dispatch(
                hero,
                tools,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                camera,
                window_size,
                vector_scene,
                &mut self.last_color_equalization_pushed_entity,
                &mut self.color_equalization_previews,
                toasts,
            );
            // Equalize Sizes panel ⟷ tool bridge — multi-sprite, no
            // per-frame on-canvas preview (the visual effect is the
            // Apply bake; an interim transform-only preview is future
            // work). Returns the full `iter_selected()` on Apply for
            // the cross-sprite `run_full_resolution_multi` bake.
            let equalize_sizes_apply = equalize_sizes_bridge::dispatch(hero, tools);
            // Upscale panel ⟷ tool bridge — sabor 3 with on-canvas
            // live preview (algo + scale apply each frame the user
            // moves the slider). Mirror of `color_equalization_bridge`.
            let upscale_apply = upscale_bridge::dispatch(
                hero,
                tools,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                camera,
                window_size,
                vector_scene,
                &mut self.last_upscale_pushed_entity,
                &mut self.upscale_preview,
            );
            // Painter panel ⟷ tool bridge (W1 T1.5) — source push +
            // current_preview drain + pending_commit capture; on-canvas
            // overlay paints the canvas RGBA over the sprite footprint.
            // Sidebar Procreate-style lands in W2 (ph2d-panel-painter).
            let painter_apply_committed = painter_bridge::dispatch(
                hero,
                tools,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                camera,
                window_size,
                vector_scene,
                &mut self.last_painter_pushed_entity,
                &mut self.painter_preview,
                &mut self.painter_preview_gpu,
                &mut self.painter_gpu_preview,
                &mut self.painter_commit_requested,
                &mut self.painter_undo_requested,
                &mut self.painter_redo_requested,
                toasts,
            );
            // Vector Pen tool ⟷ shell bridge. Per-frame world-space
            // render of committed scene paths + in-progress overlay. The
            // network IS the asset (ADR-0056 §1.1); world coords come from
            // `camera.screen_to_world` in `vector_pen_input.rs`.
            // ADR-0076 (Rank 10): per-asset placement affines (the `Transform`
            // overlay) so a gizmo-moved vector renders at its new position.
            // Identity for un-moved assets ⇒ bit-identical to the old path.
            let vector_placements = vector_scene::placements(sim, &self.vector_scene_entities);
            vector_pen_bridge::dispatch(
                tools,
                camera,
                window_size,
                self.last_pointer,
                &mut self.committed_vector_pen_paths,
                &vector_placements,
                vector_scene,
            );
            // Vector Pencil ⟷ shell bridge — drains the committed freehand
            // stroke into the shared scene + paints the in-progress sample
            // overlay. Committed paths render (fill + stroke) via the pen
            // bridge's `draw_vector_network`; the pencil bridge owns the live
            // overlay + the drain only.
            vector_pencil_bridge::dispatch(
                tools,
                camera,
                window_size,
                &mut self.committed_vector_pen_paths,
                vector_scene,
            );
            // Vector Shape ⟷ shell bridge — drains the committed shape + paints
            // the in-progress preview. Committed paths (closed fills + spiral
            // strokes) render via the pen bridge's canonical `draw_vector_network`.
            vector_shape_bridge::dispatch(
                tools,
                camera,
                window_size,
                &mut self.committed_vector_pen_paths,
                vector_scene,
            );
            // ADR-0076: if a vector's scene entity was deleted externally (Delete /
            // hierarchy row), drop its asset too — otherwise the shape renders but
            // is gone from the ECS (no gizmo, no pick = orphaned + unselectable).
            // Runs BEFORE reconcile (which would no-op on matching lengths).
            // Snapshot the pre-delete scene first (once, only on a real delete
            // frame) so Ctrl+Z restores the deleted shape.
            if vector_scene::will_prune(
                sim,
                &self.committed_vector_pen_paths,
                &self.vector_scene_entities,
            ) {
                crate::input_dispatch::vector_undo::checkpoint(
                    &mut self.vector_undo_stack,
                    &mut self.vector_redo_stack,
                    &self.committed_vector_pen_paths,
                );
            }
            vector_scene::prune_deleted(
                sim,
                &mut self.committed_vector_pen_paths,
                &mut self.vector_scene_entities,
            );
            // ADR-0076: re-sync the per-asset scene entities AFTER every commit
            // bridge (pen/pencil/shape append; Esc-clear truncates). Spawns
            // Transform+Name+VectorSceneRef for new assets (→ hierarchy + gizmo),
            // despawns removed. O(delta); usually a no-op.
            vector_scene::reconcile(
                sim,
                &mut self.vector_scene_entities,
                &self.committed_vector_pen_paths,
            );
            // ADR-0076: bridge the gizmo/hierarchy selection ⟷ the vector tools'
            // network selection, so a shape selected anywhere is selected
            // everywhere (gizmo box + fill/color + hierarchy highlight). Runs
            // BEFORE the selection-overlay + inspector bridges so they paint /
            // apply against the unified selection this same frame.
            vector_scene::sync_object_selection(
                &mut hero.gizmo.selection,
                &mut hero.gizmo.extra_selection,
                &mut self.vector_selection,
                &self.vector_scene_entities,
                &mut self.last_synced_gizmo_set,
                &mut self.last_synced_vec_networks,
            );
            // Vector Select / Direct-Select ⟷ shell overlay (T2.3). Pure
            // feedback (never mutates the scene): selected-network outlines +
            // vertex dots + the live marquee rect, read from the shared
            // `vector_selection` + the active Select tool's marquee.
            vector_selection_bridge::dispatch(
                tools,
                hero.theme,
                camera,
                window_size,
                &self.committed_vector_pen_paths,
                &vector_placements,
                &self.vector_selection,
                vector_scene,
            );
            // Vector Geometry-Graph smoke (W3 T3.1): show the param-slider panel
            // under PH2D_VECTOR_GRAPH=1, then cook `vector.source` from those
            // sliders and draw the network into the shared scene — the node-graph
            // PRODUCER path the W2 tool-direct render doesn't cover. Off → no-op
            // (normal app untouched, mirror of `motion_smoke`).
            let vgraph_visible = vector_graph_bridge::enabled();
            hero.panel_visibility.insert("vector_graph", vgraph_visible);
            // `surface.gpu()` (&GpuContext, disjoint from `vector_scene`) feeds the
            // ADR-0065 GPU SDF draft during a slider drag.
            vector_graph_bridge::dispatch(
                vgraph_visible,
                camera,
                window_size,
                vector_scene,
                surface.gpu(),
            );
            // Vector Inspector (W2.T2.4): panel visibility + fill-swatch picker
            // read-back + publish (`hero`/`tools` still in scope from above).
            vector_inspector_bridge::dispatch(
                hero,
                tools,
                &mut self.vector_fill_color,
                &mut self.committed_vector_pen_paths,
                &self.vector_selection,
                &mut self.vector_undo_stack,
                &mut self.vector_redo_stack,
            );
            // Drain the right-click point-type menu choice (chrome parked a 0..=3
            // index in `pending_vector_point_type`) → apply EAGER to the selected
            // vertices via the Direct tool (re-derives tangents on the spot).
            if let Some(pt_idx) = hero.take_pending_vector_point_type() {
                let kind = match pt_idx {
                    0 => ph2d_vector_doc::VertexKind::Free,
                    1 => ph2d_vector_doc::VertexKind::Mirror,
                    2 => ph2d_vector_doc::VertexKind::Aligned,
                    _ => ph2d_vector_doc::VertexKind::Auto,
                };
                if let Some(direct) = tools.active_mut().and_then(|t| {
                    t.as_any_mut()
                        .downcast_mut::<ph2d_tool_vector_direct::VectorDirectTool>()
                }) {
                    crate::input_dispatch::vector_undo::checkpoint(
                        &mut self.vector_undo_stack,
                        &mut self.vector_redo_stack,
                        &self.committed_vector_pen_paths,
                    );
                    direct.set_selected_vertex_kind(
                        &mut self.committed_vector_pen_paths,
                        &self.vector_selection,
                        kind,
                    );
                }
            }
            // Onda 2C: clear the gizmo hit_map BEFORE paint_hero_screen
            // runs. `paint_hero_screen` now paints BOTH the primary gizmo
            // AND the multi-selection extras + global gizmo (the latter
            // via `paint_sprite_gizmo_keyed`, which populates `hit_map` —
            // those entries drive the dispatcher's group-transform routing
            // in `on_mouse_input`). Painting them inside `paint_hero_screen`
            // (before the floating panels) keeps gizmos BELOW the panels
            // both visually and in hit-test (z-order fix 2026-05-31).
            // ADR-0076: the vertex-edit / authoring vector tools (Direct / Pen /
            // Pencil / Shape) must NOT show the object-transform gizmo. Its painted
            // box + handles overlay the shape, and those tools' `vector_*_world`
            // reject any click over a `hit_index` widget — so the gizmo's hit-rects
            // would block EVERY vertex/handle grab + canvas click (Enio: "Direct
            // não move pontos/handles"). The gizmo belongs to object selection
            // (Select) + the arrow/Move tools. Suppress the painted view here; the
            // selection stays armed (hierarchy highlight) — just no box/handles.
            let suppress_gizmo = tools
                .active()
                .map(|t| {
                    let id = t.id();
                    id == ph2d_editor::ToolId::new("vector_direct")
                        || id == ph2d_editor::ToolId::new("vector_pen")
                        || id == ph2d_editor::ToolId::new("vector_pencil")
                        || id == ph2d_editor::ToolId::new("vector_shape")
                })
                .unwrap_or(false);
            if suppress_gizmo {
                hero.gizmo.view = None;
                hero.gizmo.extra_views.clear();
                hero.gizmo.global_view = None;
            }
            hero.gizmo.gizmo_hit_map.clear();
            paint_hero_screen(hero, viewport, vector_scene, paint_ctx.text);
            // Fase 0f: overlay the active rubber-band rect on top of
            // everything (panels, gizmo, hero chrome). Pure shell
            // concern — coords stay in screen space so the rect
            // doesn't shift if the camera pans mid-drag. Semi-
            // transparent fill + 4 thin border rects (no stroke API
            // on VectorScene yet; the 4-fills idiom matches the rest
            // of the shell's overlay painters).
            if let Some(rb) = self.rubber_band {
                let (ax, ay) = rb.anchor_screen;
                let (cx, cy) = rb.current_screen;
                let x0 = ax.min(cx) as f64;
                let y0 = ay.min(cy) as f64;
                let x1 = ax.max(cx) as f64;
                let y1 = ay.max(cy) as f64;
                use ph2d_vector::{Color, Rect as VRect};
                // Selection accent — design tokens use OKLCH; the
                // sRGB approximation here is the canonical Selection
                // color from `ColorToken::Selection` (~#3a8ee6 @ 25%
                // fill, 100% border) baked at boot. Keeping it inline
                // avoids threading the theme into render_loop just
                // for one overlay; a follow-up can swap to a token
                // lookup if the rubber-band needs theme parity.
                let fill = Color::new([0.23, 0.56, 0.90, 0.18]);
                let border = Color::new([0.23, 0.56, 0.90, 1.0]);
                vector_scene.fill_rect(VRect::new(x0, y0, x1, y1), fill);
                vector_scene.fill_rect(VRect::new(x0, y0, x1, y0 + 1.0), border);
                vector_scene.fill_rect(VRect::new(x0, y1 - 1.0, x1, y1), border);
                vector_scene.fill_rect(VRect::new(x0, y0, x0 + 1.0, y1), border);
                vector_scene.fill_rect(VRect::new(x1 - 1.0, y0, x1, y1), border);
            }
            // Hierarchy intent dispatch phase — camera reset +
            // view-focus + 9 hierarchy intents (visibility_toggle /
            // reparent / duplicate / add_child / reset_transform /
            // delete / row_click / rename_seed / rename_commit).
            // Extracted to sibling `hierarchy.rs` as a free fn (Wave
            // 3.2 stage A).
            if hierarchy::dispatch(
                view_focus_kind,
                visibility_toggle_row,
                lock_toggle_row,
                group_toggle_row,
                reparent_intent,
                duplicate_row,
                add_child_row,
                reset_transform_row,
                delete_row,
                hierarchy_row_click,
                hierarchy_select_intent,
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
                &sprite_edits,
                &ordering_edits,
                &sampling_edits,
                &blend_edits,
                &visibility_section_edits,
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
            // Merge Sprites (Enio 2026-05-27, Hierarchy right-click).
            // Drains BEFORE `image_edit::dispatch` so a same-frame
            // image-edit on one of the originals (extremely unlikely
            // path but documented) doesn't race with the despawn. The
            // multi-selection comes from `hero.gizmo` — primary first,
            // extras after. Right-clicked row resolves to the "primary
            // anchor" the merged sprite parents under.
            if let Some(row) = merge_sprites_row
                && let Some(live) = hero_live.as_ref()
                && let Some(primary_bits) = live.bridge.entity_for(row)
            {
                let in_selection = hero.gizmo.is_selected(primary_bits);
                let selected_count = hero.gizmo.iter_selected().count();
                // Audit B-M3: if the user right-clicked OUTSIDE the
                // multi-selection (and they already had 2+ sprites
                // selected), the previous behaviour silently fell back
                // to "single-entity merge → <2 warning" which read as
                // "select 2+ first" — misleading. Steer them to the
                // actual fix.
                if !in_selection && selected_count >= 2 {
                    toasts.push(ph2d_editor::Toast::warning(
                        "Merge Sprites: right-click on one of the selected sprites",
                    ));
                    self.title_dirty = true;
                } else {
                    let to_merge: Vec<u64> = if in_selection {
                        hero.gizmo.iter_selected().collect()
                    } else {
                        vec![primary_bits]
                    };
                    if hero_intents::drain_merge_sprites(
                        to_merge.clone(),
                        primary_bits,
                        hero.project.pixels_per_meter,
                        sim,
                        renderer,
                        asset_db,
                        atlas_asset_map,
                        toasts,
                    ) {
                        self.title_dirty = true;
                    }
                    // Clear merged entity_bits from gizmo so the global
                    // gizmo doesn't paint over vanished entities
                    // (mirror of Delete).
                    for bits in &to_merge {
                        if hero.gizmo.selection == Some(*bits) {
                            hero.gizmo.selection = None;
                        }
                        hero.gizmo.extra_selection.retain(|b| b != bits);
                    }
                    // Audit B-H2: promote the freshly-spawned merged
                    // entity to the selection so the user's next
                    // action (Move, Apply tool, etc.) operates on the
                    // merged result — matches Photoshop / Figma "after
                    // merge, the merged layer IS the selection".
                    if let Some(result) = hero_intents::take_last_merge_result() {
                        hero.gizmo.replace_selection(Some(result.new_entity_bits));
                    } else if hero.gizmo.selection.is_none()
                        && !hero.gizmo.extra_selection.is_empty()
                    {
                        // Merge bailed before spawning — promote oldest
                        // surviving extra (mirror of Delete's
                        // headless-cleanup path).
                        hero.gizmo.selection = Some(hero.gizmo.extra_selection.remove(0));
                    }
                }
            }
            // Image-edit drain phase + file-picker import — extracted
            // to sibling `image_edit.rs` as a free fn (Wave 3.2 stage A).
            // Returns whether any drain pushed a toast.
            // `padding_apply` carries a `Vec<u64>` (not `Copy`) — capture
            // the Apply-fired flag here so the teardown below can run
            // after the dispatch consumes the value.
            let padding_apply_fired = padding_apply.is_some();
            if image_edit::dispatch(
                trim_entities,
                make_square_entities,
                real_size_entities,
                rasterize_entities,
                padding_apply,
                color_equalization_apply.clone(),
                equalize_sizes_apply.clone(),
                upscale_apply.clone(),
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
                &mut self.last_painter_pushed_entity,
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
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.last_bgremoval_pushed_entity = None;
                self.bgremoval_preview = None;
                self.title_dirty = true;
            }
            // Padding Apply teardown — deactivate the tool so the panel
            // hides + the Inspector returns, exactly like Bg Removal.
            if padding_apply_fired
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.title_dirty = true;
            }
            // Color Equalization Apply teardown — deactivate the tool
            // (panel hides, sprite returns to its un-edited live state
            // visually, multi-selection preserved). Mirror of Padding.
            if color_equalization_apply.is_some()
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.last_color_equalization_pushed_entity = None;
                self.title_dirty = true;
            }
            // Equalize Sizes Apply teardown — same shape as Padding /
            // Color EQ: bake just ran, so switch back to the default
            // tool. The panel auto-hides because its `panel_visible`
            // gate keys off `tools.active().id() == "equalize_sizes"`
            // and the bridge clears the published snapshot on the
            // next frame.
            if equalize_sizes_apply.is_some()
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.title_dirty = true;
            }
            // Upscale Apply teardown — mirror of Color EQ. Clear the
            // preview cache + push-tracker so re-activating starts
            // fresh against the new (post-bake) source.
            if upscale_apply.is_some()
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.last_upscale_pushed_entity = None;
                self.upscale_preview = None;
                self.title_dirty = true;
            }
            // Painter Apply teardown (W1 T1.5) — same shape as BgR /
            // Upscale: deactivate the tool so the chrome returns to its
            // pre-painting state, and clear the preview/push-tracker so
            // re-activating starts fresh against the freshly-baked sprite.
            if painter_apply_committed
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.last_painter_pushed_entity = None;
                self.painter_preview = None;
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

            // Tool palette in the CREATE zone (top-right). Hidden in Zen
            // mode by virtue of `tool_palette_rects` returning empty.
            // This branch is the legacy no-hero (demo) path, so there is
            // no Image Tools mode → `mode_on = false`. Map slots through
            // the SAME `palette_visible_tool_indices` the click hit-test
            // uses so the two never drift (image tools filtered out when
            // off — no icon, no hit zone).
            let visible = crate::palette_visible_tool_indices(tools, false);
            let palette_rects = layout.tool_palette_rects(visible.len());
            let active_id = tools.active().map(|t| t.id());
            let palette_icons: Vec<(EditorRect, &str, bool)> = palette_rects
                .iter()
                .zip(visible.iter())
                .map(|(r, &i)| {
                    let tool = &tools.tools()[i];
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

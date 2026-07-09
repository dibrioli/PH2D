//! `App` + `AppGfx` + `ImageEditSnapshot` + `HeroLive` struct
//! definitions extracted from main.rs (Wave 3.2 stage B).
//!
//! Re-exported by main.rs at the crate root so other modules
//! (`render_loop`, `input_handlers`, `init`) can keep their
//! `crate::App` / `crate::AppGfx` paths unchanged. All fields stay
//! `pub(crate)` — these are shell-internal aggregates, not a public
//! API surface.
//
// ph2d-loc-cap: AppGfx + App are the shell's two top-level aggregates —
// each new editor subsystem (renderer / asset / tool / per-tool preview
// state, and now the W2.T4 cooked-texture LogicalTextureMap) adds one
// field + its doc here, so this definitions file accretes past 600 LOC by
// design. Already over on origin/main (608) before this field; splitting
// App/AppGfx into per-subsystem sub-structs is a tracked decomposition
// refactor, not mid-feature work.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use bumpalo::Bump;
use ph2d_asset::{AssetDb, AssetId, LogicalTextureMap};
use ph2d_core::{FixedStep, Playhead};
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, HierarchySnapshot, HierarchyWalkState,
};
use ph2d_ecs::{PresentWorld, SimWorld, TransformPropagationState, WorklistBuf};
use ph2d_editor::{HeroScreen, Layout as EditorLayout, NodeId, ToastQueue, ToolRegistry, ZenMode};
use ph2d_gpu::SurfaceContext;
use ph2d_host::WindowSize;
use ph2d_imageio::{ExporterRegistry, ImporterRegistry};
use ph2d_input::InputState;
use ph2d_render::{Camera2d, Compositor, GameRt, SpriteRenderer, Tonemap, VelloPass};
use ph2d_script::ScriptHost;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use winit::keyboard::ModifiersState;
use winit::window::Window;

use crate::hero_bridge;
use crate::winit_host::{LoggingHandler, WinitHost};

/// Holds every initialized-after-`resumed` resource. Bundling them into
/// a single `Option<AppGfx>` lets us destructure into per-field `&mut`
/// borrows in `render_frame()` — split-borrowing through a method
/// chain on individual `Option<...>` fields would be awkward.
pub(crate) struct AppGfx {
    pub(crate) surface: SurfaceContext,
    pub(crate) renderer: SpriteRenderer,
    pub(crate) sim: SimWorld,
    pub(crate) present: PresentWorld,
    pub(crate) camera: Camera2d,
    /// M6 — set when PNG fixtures loaded successfully; held so the
    /// AssetDb keeps `Arc<Asset>` alive for hot-reload follow-ups.
    pub(crate) asset_db: AssetDb,
    /// KTX2 Fase 2 (W2.T4): `LogicalTextureId → BTreeMap<TierIndex, AssetId>`
    /// — which cooked KTX2 artifact (in `asset_db`) backs a tier-agnostic
    /// logical texture for each platform tier. The cooked-texture loader
    /// pass (`cooked_texture_bridge`) reads this and the device tier to
    /// resolve and upload a `SpriteSource::CookedTexture` sprite's pixels.
    /// Empty until a cooked texture is loaded (e.g. the `PH2D_KTX2_SMOKE`
    /// harness).
    pub(crate) logical_texture_map: LogicalTextureMap,
    /// M6 — true when the atlas was composed from real PNG files (vs the
    /// procedural dummy fallback). Surfaced in window title.
    pub(crate) atlas_is_real: bool,
    /// M7 — Luau VM with placeholder script loaded. Per-frame gc_step
    /// keeps the GC budget warm; set/get bindings ready for follow-up
    /// gameplay work.
    pub(crate) script: Option<ScriptHost>,
    /// M12 editor data layer + M11 widget paint pass.
    pub(crate) theme: Theme,
    pub(crate) zen: ZenMode,
    pub(crate) toasts: ToastQueue,
    /// Registered editor tools. Keys 1/2 switch active tool; the
    /// active tool's `build_panel()` is painted each frame as the
    /// FloatingPanel that shows in the bottom-center of the canvas.
    pub(crate) tools: ToolRegistry,
    /// 4-zone editor layout (ADR-0023 §3). Sized from window each
    /// resize; the M11 paint pass walks this to draw zone backdrops.
    pub(crate) layout: EditorLayout,
    /// M14.5: offscreen HDR (Rgba16Float) render target for the game
    /// world. Sprite + future light/particle/material passes write
    /// here; the tonemap pass reads here and writes to LDR. Recreated
    /// on resize.
    pub(crate) game_rt: GameRt,
    /// M14.5: AgX tonemap pass — owns its own LDR output texture
    /// (`game_rt_ldr`). Sampled by the compositor as the "game layer"
    /// input. Identity LUT by default; swap in real AgX via
    /// `set_lut`.
    pub(crate) tonemap: Tonemap,
    /// M14.5: compositor that composes `tonemap.output_view()` (game)
    /// and `vello_pass.intermediate_view()` (UI chrome) onto the swap
    /// chain. Replaces the old `vello_pass.blitter` direct-to-surface
    /// blit so chrome and game live in fully isolated RTs.
    pub(crate) compositor: Compositor,
    /// Vello pipeline + intermediate texture for the widget paint
    /// pass. In M14.5 the intermediate is sampled by `compositor`
    /// (not blitted directly to the surface).
    pub(crate) vello_pass: VelloPass,
    /// Reused [`VectorScene`] — encoded fresh each frame; allocations
    /// pool inside Vello so this is cheap.
    pub(crate) vector_scene: VectorScene,
    /// ADR-0108 Fase 0: cena vetorial NOVA (modelo editor-first). Renderizada
    /// pela pipeline `ph2d-vec-render` no canvas, sob a mesma `vector_scene`
    /// Vello compartilhada. Fase 0 = cena-demo (prova o seam ponta-a-ponta);
    /// Fase 1 = dirigida pelas ferramentas de desenho.
    pub(crate) vec_scene: ph2d_vec_scene::VecScene,
    /// Motion Nodes runtime state (M0.T8): document + transport + persistent
    /// `Cook` + node registry + reused instance buffer. Cooked per frame by
    /// `render_loop::motion_bridge` while the `motion` tool is active. Mirror of
    /// `vec_scene` — document ≠ tool (ADR-0040).
    pub(crate) motion: crate::motion_state::MotionState,
    /// parley font + layout context (heavy state). Threaded through
    /// `PaintCtx` so future text passes don't re-load fonts.
    pub(crate) text_system: TextSystem,
    /// Hero screen (`02-editor-main` mockup) — populated by default;
    /// `None` only when `PH2D_M5_DEMO=1` selects the legacy
    /// 1000-sprite perf demo. Owns the [`WidgetStore`] + [`HitIndex`]
    /// so input pipeline (ADR-0024) can route pointer/key events
    /// through `dispatch_*`.
    pub(crate) hero_screen: Option<HeroScreen>,
    /// Per-frame arena for [`WidgetEvent`]s emitted by the hero
    /// dispatcher. Reset at end-of-frame.
    pub(crate) hero_arena: Bump,
    /// OS clipboard handle — used by Cmd+C/V/X. `None` when the OS
    /// rejected our request (rare; we just no-op those keys then).
    pub(crate) clipboard: Option<arboard::Clipboard>,
    /// M14.1 — cached `QueryState` pair for hierarchical transform
    /// propagation. Built once after `populate_sim`; used every frame
    /// inside the extract phase. The only way to iterate `&World`
    /// from inside `extract!`.
    pub(crate) prop_state: TransformPropagationState,
    /// M14.1 — pre-allocated DFS worklist for `propagate_transforms`.
    /// Capacity sized to `WorklistBuf::DEFAULT_CAPACITY` (8 192
    /// entities) — comfortably above `SPRITE_COUNT = 1000`. HR-3
    /// zero-alloc verified by `crates/ph2d-ecs/tests/propagate_no_alloc.rs`.
    pub(crate) worklist: WorklistBuf,
    /// W3.T3.8 reusable scratch + per-frame input buffer for the
    /// canonical sorting pipeline; threaded like `worklist` so the sort
    /// stays allocation-free after warm-up (HR-3).
    pub(crate) sort_scratch: ph2d_ecs::sort_key::SortScratch,
    pub(crate) sort_inputs: Vec<ph2d_ecs::sort_key::SortInput>,
    /// M14.4a live-bridge state. Present in the default editor mode
    /// (i.e. always unless `PH2D_M5_DEMO=1` switched to the legacy
    /// untouched.
    pub(crate) hero_live: Option<HeroLive>,
    /// M14.4c+M14.4d: next free atlas key for imported images.
    /// Starts at `FIRST_IMPORT_KEY` (= 16) so it sits past the
    /// demo's seeded HSV tile keys (0..15). With the Skyline atlas
    /// the key space is effectively unbounded (the underlying packer
    /// runs out of pixel space before `u32` does), so we just
    /// increment monotonically — no cycling, no overwrite. When
    /// the atlas does run out of room the regrow path
    /// (`insert_atlas_sprite_with_regrow`) uses `atlas_asset_map` to
    /// recover each existing region's source bytes from `asset_db`.
    pub(crate) next_import_cell: u32,
    /// M14.7 polish: atlas-key → AssetId map kept in sync with each
    /// import. Drives the regrow callback so doubling the atlas
    /// texture preserves every previously-imported sprite. BTreeMap
    /// per HR-5 / ADR-0022.
    pub(crate) atlas_asset_map: BTreeMap<u32, AssetId>,
    /// M14.A: editor → SimWorld mutation pipeline. Populated at boot
    /// with the canonical Transform / Name / Visibility / RootOrder
    /// type registrations via `register_ecs_components`; future crates
    /// (`ph2d-render` for Sprite, `ph2d-script` for LuauScript) will
    /// extend it as their components join the live inspector.
    ///
    /// First real consumer is the Inspector's Transform editor: each
    /// commit pushes `EditorCommand::SetComponent` to
    /// [`Self::editor_queue`], which `apply_editor_commands` drains
    /// once per frame to write back to SimWorld via this registry.
    pub(crate) component_registry: ComponentRegistry,
    /// Editor command queue (Arc<Mutex<…>>-backed for multi-producer
    /// access). The Inspector's commit path is the only producer
    /// today; the shell drains and applies once per frame after the
    /// hero `apply_event` pass.
    pub(crate) editor_queue: EditorCommandQueue,
    /// Cached stable type id for `ph2d::ecs::Transform`. Lookup is
    /// blake3-of-name → first 8 bytes; cached so the per-commit push
    /// path doesn't re-hash. The matching registry entry was added
    /// via `register_ecs_components`.
    pub(crate) transform_type_id: u64,
    /// M14.D: same as `transform_type_id` for the `ph2d::ecs::Visibility`
    /// component. Cached at boot so the Inspector visibility checkbox
    /// commit doesn't re-hash on every toggle.
    pub(crate) visibility_type_id: u64,
    /// M14.E: same cache for `ph2d::ecs::Name`. Used when draining
    /// `pending_name_edit` from the Inspector's editable name field.
    pub(crate) name_type_id: u64,
    /// M14.C audit fix #8: cache for `ph2d::render::Sprite` so the
    /// Strategy switch commit goes through the canonical
    /// `EditorCommand::SetComponent` pipeline (parity with Transform /
    /// Visibility / Name). Loaded into the registry via
    /// `register_render_components` at boot.
    pub(crate) sprite_type_id: u64,
    /// Single-level image-edit undo — one TRANSACTION (covers a whole
    /// multi-sprite Apply) per slot. Cmd+Z (or TOOL_UNDO click) restores
    /// every sprite the transaction touched. The full editor undo system
    /// (a multi-transaction stack rooted in `EditorCommandQueue`) is
    /// M14.x scope.
    ///
    /// **Cross-sprite contract** (was the §1 audit CRITICAL): each
    /// per-sprite drain appends ONE `ImageEditSnapshot` to the in-flight
    /// `pending_entries` vec the orchestrator owns; once the multi-sprite
    /// loop completes, the orchestrator commits a single
    /// `ImageEditTransaction` here, releasing the previous transaction's
    /// pre-edit individual textures (if any). Undo pops the whole vec —
    /// restoring N sprites in one keypress — instead of just the last
    /// sprite the old single-slot design retained.
    pub(crate) image_edit_undo: Option<ImageEditTransaction>,
    /// ADR-0054 W0.T6 — image I/O importer registry populated at boot
    /// by `ph2d_imageio_registry_init::register_all_importers`. Held on
    /// `AppGfx` so future Open-file paths (W1+) can `find_for` a
    /// recognized format and decode straight to `DecodedImage`. Empty
    /// outside the format crates' coverage (W0: PNG only).
    #[allow(
        dead_code,
        reason = "W0.T6 stages the registry; W1+ wires Open into find_for"
    )]
    pub(crate) imageio_importers: ImporterRegistry,
    /// ADR-0054 W0.T6 — image I/O exporter registry populated at boot
    /// by `ph2d_imageio_registry_init::register_all_exporters`. Held
    /// for future Save-as paths (W1+).
    #[allow(
        dead_code,
        reason = "W0.T6 stages the registry; W1+ wires Save into find_for"
    )]
    pub(crate) imageio_exporters: ExporterRegistry,
}

/// One Apply pass — covers `entries.len()` sprites (1 for a single-sprite
/// tool like Trim, N for a multi-sprite Apply like Color EQ over a
/// selection). Restored atomically by `drain_undo_image_edit`.
pub(crate) struct ImageEditTransaction {
    /// One entry per sprite the Apply pass mutated. Drained in reverse
    /// on undo (conventional — irrelevant here since each entry targets
    /// a distinct entity, but keeps the restore order deterministic).
    pub(crate) entries: Vec<ImageEditSnapshot>,
    /// Toast label for the transaction (`"Color EQ"`, `"Bg Removal"`,
    /// `"Padding"`, …). One label per transaction even when N sprites
    /// were touched — the user sees ONE undo toast.
    pub(crate) label: &'static str,
}

/// Pre-edit snapshot of ONE sprite that an image-edit action mutated.
/// Multiple `ImageEditSnapshot`s aggregate into an
/// [`ImageEditTransaction`] for multi-sprite Apply (one entry per
/// affected sprite). Populated by the 8 image-edit drainers
/// (`drain_trim_transparency` / `drain_make_square` / `drain_rasterize`
/// / `drain_padding` / `drain_color_equalization` / `drain_upscale` /
/// `drain_equalize_sizes` / `drain_bgremoval`), consumed by
/// `drain_undo_image_edit`.
pub(crate) struct ImageEditSnapshot {
    /// Bevy entity bits of the sprite the edit targeted.
    pub(crate) entity_bits: u64,
    /// `Sprite.source` before the edit. When this is
    /// `Individual { texture_id }`, the texture is **retained**
    /// (refcount + 1 vs the natural acquire-by-the-edit path) so the
    /// undo restore can repoint without re-uploading pixels. The
    /// drainer that captured the snapshot is responsible for the
    /// matching `acquire`/refcount bump.
    pub(crate) pre_source: ph2d_render::SpriteSource,
    /// `Sprite.size` before the edit (world meters).
    pub(crate) pre_size: [f32; 2],
    /// `Transform.translation` before the edit (world meters).
    pub(crate) pre_translation: [f32; 2],
    /// `Sprite.premultiplied` before the edit. BG-Removal Apply sets it
    /// `true` (premultiplied bake, fringe fix); undo must restore the
    /// pre-edit value so the original straight-alpha source renders
    /// straight again. Trim / Make-Square leave it `false`.
    pub(crate) pre_premultiplied: bool,
    /// `Sprite.anchor` (pivot offset) before the edit. Padding's Keep
    /// mode rebases the anchor to keep content + pivot world-fixed under
    /// an asymmetric resize; undo restores the pre-edit value. `[0,0]`
    /// for edits that don't touch the pivot (and every sprite until the
    /// TOOL_PIVOT tool moves a pivot).
    pub(crate) pre_anchor: [f32; 2],
    /// The new individual texture id that the edit acquired. Released
    /// on undo so the now-orphaned post-edit texture doesn't leak.
    pub(crate) post_individual_id: u32,
    /// Human-readable label for the toast: "Trim" / "Make square".
    pub(crate) label: &'static str,
}

/// Per-frame state owned by the live editor bridge (ADR-0025 M14.4a).
pub(crate) struct HeroLive {
    pub(crate) bridge: hero_bridge::EntityNodeMap,
    pub(crate) walk_state: HierarchyWalkState,
    /// Scratch buffer for `build_hierarchy_snapshot`'s DFS stack.
    /// Preserved across frames so HR-3 zero-alloc invariant holds.
    pub(crate) walk_scratch: Vec<(ph2d_ecs::Entity, u8, Option<ph2d_ecs::Entity>)>,
    /// Reused per-frame snapshot. `build_hierarchy_snapshot` clears
    /// the inner Vec without releasing capacity.
    pub(crate) snapshot: HierarchySnapshot,
}

pub(crate) struct App {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) host: Option<WinitHost>,
    pub(crate) gfx: Option<AppGfx>,
    pub(crate) handler: LoggingHandler,
    pub(crate) fixed_step: FixedStep,
    /// Engine-wide timeline cursor. Advanced once per fixed tick; every
    /// animatable system samples it for the current frame (the general
    /// timeline's live scalar side). See `render_loop::timeline_smoke`.
    pub(crate) playhead: Playhead,
    /// The app-general timeline document + selection/history/flags (W1). The
    /// `render_loop::timeline_bridge` drains `timeline_intents` into it, then
    /// applies it to the scene at the Playhead. Empty until the panel (W2) or a
    /// programmatic bind adds tracks.
    pub(crate) timeline: ph2d_timeline::TimelineState,
    /// Pending timeline commands (from the panel / auto-key), drained once per
    /// frame by `render_loop::timeline_bridge`.
    pub(crate) timeline_intents: Vec<ph2d_timeline::TimelineIntent>,
    /// A queued transport jump (go-to-start/end, frame step, typed time) wants
    /// the dope sheet panned to wherever the playhead lands. It cannot ask the
    /// panel straight away: the action bus is drained AFTER `timeline_bridge`
    /// has already run, so the intent applies next frame — asking now would pan
    /// to the playhead's OLD position. The flag rides across to the apply.
    pub(crate) timeline_reveal_after_apply: bool,
    /// Reused view-snapshot buffer, rebuilt from `timeline` + `playhead` each
    /// frame and published to the timeline panel (`set_current_timeline`).
    pub(crate) timeline_view: ph2d_timeline::TimelineViewSnapshot,
    /// Set by the `K` key: on the next frame, insert a keyframe at the playhead
    /// on every track bound to the selected sprite (capturing its current pose).
    pub(crate) timeline_insert_key: bool,
    /// Whether a gizmo drag was in flight last frame — used to bracket an
    /// AutoKey drag into a SINGLE undo step (begin on the opening frame, commit
    /// when it ends) instead of one step per recorded frame.
    pub(crate) timeline_drag_active: bool,
    pub(crate) last_frame: Instant,
    pub(crate) pending_resize: Option<WindowSize>,
    /// FLUID-DRAG present-mode override: the configured mode saved while a live resize streams (the
    /// drag runs in the backend's best non-blocking mode; restored after the stream settles).
    pub(crate) resize_saved_present_mode: Option<wgpu::PresentMode>,
    /// Quiet-frame countdown before the saved present mode is restored (see `RESIZE_SETTLE_FRAMES`).
    pub(crate) resize_settle_frames: u32,
    pub(crate) modifiers: ModifiersState,
    pub(crate) last_pointer: (f32, f32),
    /// Set to `Some(node_id)` when the user pressed inside a draggable
    /// widget (Slider) — subsequent pointer-move events continue to
    /// fire SetValue until pointer-up clears this. None for click-only
    /// widgets (Toggle, RadioGroup, ColorSwatch).
    pub(crate) dragging: Option<NodeId>,
    pub(crate) title_dirty: bool,
    /// gilrs context (M8). `None` if init failed (e.g. Linux without
    /// /dev/input read perms in CI sandboxes — we degrade gracefully
    /// instead of crashing the renderer).
    pub(crate) gilrs: Option<gilrs::Gilrs>,
    /// Audio subsystem (Phase 2.1). `None` if no output device / unsupported
    /// format — the editor runs silent (degrade-gracefully, like `gilrs`).
    /// Holds the control-side `AudioEngine` + the live cpal output stream.
    pub(crate) audio: Option<crate::audio::AudioSystem>,
    /// Audio Editor waveform selection drag: the anchor frame while the primary
    /// button is held over the overlay waveform (`None` = not selecting).
    #[cfg(feature = "panel-audio-editor")]
    pub(crate) audio_sel_drag: Option<u64>,
    /// Input snapshot pumped by the gilrs adapter each frame.
    pub(crate) input: InputState,
    /// M14.4b.bis: middle-button camera pan state.
    /// `Some(anchor)` while a middle-drag is in progress; subsequent
    /// `CursorMoved` events feed `Camera2d::pan_screen_delta`.
    pub(crate) pan_anchor: Option<(f32, f32)>,
    /// Motion Nodes M0.T1: the pointer button currently held, tracked so
    /// `CursorMoved` forwards the REAL button to editor-core (winit's Move
    /// carries no button — the shell used to hardcode `Primary`, which lost
    /// middle/right identity for graph pan/box-select). `Some` between Down
    /// and Up; `None` when no button is held.
    pub(crate) held_button: Option<ph2d_host::PointerButton>,
    /// `true` while the primary button is held with the BgRemoval
    /// eyedropper armed — drives multi-colour sampling on `CursorMoved`.
    /// Set on Primary Down (when over the sprite + armed), cleared on
    /// Primary Up. Separate from `dragging` (panel-slider drag).
    pub(crate) eyedropper_dragging: bool,
    /// M14.4e: most recent cursor position (in physical px, top-left
    /// origin). Cached on every `CursorMoved` so `DroppedFile` can
    /// project the drop point to world coords — winit's `DroppedFile`
    /// event itself carries no position.
    pub(crate) last_cursor: (f32, f32),
    /// M14.4e: paths buffered between `HoveredFile` and `DroppedFile`
    /// (winit emits one `HoveredFile` per file when multiple are
    /// dragged together). Cleared on `HoveredFileCancelled` or after
    /// `DroppedFile` is handled.
    pub(crate) hovered_files: Vec<std::path::PathBuf>,
    /// M14.7 polish (7.3 fix): paths queued by `DroppedFile` events.
    /// winit fires `DroppedFile` once per file when the user drops
    /// multiple, but if any one event is lost (e.g. another window
    /// event consumes the loop iteration) sprites silently go missing.
    /// We buffer here and drain at the start of `render_frame` so
    /// every path that reached us imports atomically, regardless of
    /// event interleaving.
    pub(crate) pending_drops: Vec<std::path::PathBuf>,
    /// M14.4g Telemetry Phase A: EWMA-smoothed frame time pushed to
    /// the hero's `BottomHudStats` each frame so the status bar shows
    /// real fps/ms instead of the M5 placeholder strings. α=0.1 —
    /// canonical "smooth without dormant" value used by RTSS / Unity
    /// stats / Tracy.
    pub(crate) frame_ms_ewma: f32,
    /// M14.7 polish (10.1): EWMA-smoothed "raw" frame work time —
    /// measured from start of `render_frame` to end of
    /// `queue.submit`, excluding the vsync wait. Same α as
    /// `frame_ms_ewma`. Surfaced as `BottomHudStats.raw_fps` so the
    /// status bar shows hardware capacity alongside the synced fps.
    pub(crate) frame_cpu_ms_ewma: f32,
    /// M14.7 polish (19.3): overlap-cycle state for the canvas-pick
    /// path. Each Primary Down at the same world position increments
    /// `cycle_count`; on every ODD count we advance `cycle_idx` so
    /// the user can step DOWN the stack at a fixed cursor location.
    /// Even counts leave the selection alone (allowing drag without
    /// re-cycling). Reset when the click moves > 4 px in world space
    /// or the hit list shape changes.
    pub(crate) cycle_pick_world: Option<[f32; 2]>,
    pub(crate) cycle_pick_hits: Vec<u64>,
    pub(crate) cycle_pick_idx: usize,
    pub(crate) cycle_pick_count: u32,
    /// `entity_bits` of the sprite whose RGBA was last pushed into
    /// the active `BgRemovalTool` snapshot. `None` until the first
    /// push. Reset to `None` whenever the user activates BgRemoval
    /// via Digit3 so the very next frame re-pushes against the
    /// current selection. The snapshot loop reads this every frame
    /// while BgRemoval is the active tool to skip redundant
    /// pushes (the thumbnail rebuild + preview rerun are work).
    pub(crate) last_bgremoval_pushed_entity: Option<u64>,
    /// Mirror of `last_bgremoval_pushed_entity` for the Color
    /// Equalization tool. `Some(bits)` is the entity whose source
    /// RGBA was last pushed into the active `ColorEqualizationTool`
    /// preview cache. When the primary changes (or the tool just
    /// activated) the bridge re-pushes so the panel's live thumbnail
    /// reflects the current selection. Reset to `None` on tool
    /// deactivate.
    pub(crate) last_color_equalization_pushed_entity: Option<u64>,
    /// On-canvas live previews for the Color Equalization tool — one
    /// entry per CURRENTLY-SELECTED sprite (mirrors the multi-select
    /// Apply set). The preview overlay paints each entry over its
    /// sprite's footprint so the user sees the edited result on EVERY
    /// selected sprite when the slider is released. Keyed by
    /// `entity_bits`; entries for sprites that left the selection get
    /// pruned on the next rebuild. Cleared on Apply + deactivate.
    pub(crate) color_equalization_previews:
        std::collections::BTreeMap<u64, ColorEqualizationPreview>,
    /// Mirror of `last_color_equalization_pushed_entity` for the
    /// Upscale tool. Reset to `None` on tool deactivate so the next
    /// activation re-pushes the source against the current selection.
    pub(crate) last_upscale_pushed_entity: Option<u64>,
    /// On-canvas live preview for the Upscale tool — same shape as
    /// `color_equalization_preview`. Drawn over the primary sprite's
    /// footprint while the tool is active so the user sees the
    /// algorithm + scale apply in real time. Cleared on Apply +
    /// deactivate.
    pub(crate) upscale_preview: Option<UpscalePreview>,
    /// Cached full-resolution Background-Removal preview for the active
    /// sprite. Recomputed only when params change / source changes /
    /// the tool (re)activates; the per-frame canvas overlay reuses the
    /// `Arc` buffer (no pixel copy). `None` when the tool is inactive
    /// or no source is loaded. NOT a destructive edit — the sprite's
    /// own texture is untouched; the overlay just paints on top, so
    /// Apply (which re-reads the original source) and undo stay correct.
    pub(crate) bgremoval_preview: Option<BgremovalPreview>,
    /// Transient GPU texture backing the on-canvas BgRemoval preview
    /// (Lens F, 2026-05-26). The bridge uploads the premultiplied
    /// preview RGBA into an `IndividualTextureStore` slot once per
    /// `Arc` swap; `sim_extract` injects a synthetic `RenderInstance`
    /// pointing at this texture in place of the suppressed source
    /// sprite, so the live preview goes through the SAME sprite
    /// pipeline (Rgba8UnormSrgb + sprite.wgsl + premul blend) as Apply
    /// — eliminating the gamma/blend-space halo that the Vello overlay
    /// path introduced. `None` when the preview cache is `None` (tool
    /// inactive, no source, or post-Apply).
    pub(crate) bgremoval_preview_gpu: Option<BgremovalPreviewGpu>,
    /// Last entity whose source RGBA was pushed into the active
    /// `PainterTool`. Reset to `None` on tool deactivate so the next
    /// activation re-pushes against the current selection. Same shape
    /// as `last_bgremoval_pushed_entity` — Wave 10 driver helpers
    /// (`ph2d_tool_runtime::drive_source_push`) consume it directly.
    pub(crate) last_painter_pushed_entity: Option<u64>,
    /// Coalescing (perf): latest canvas pointer position buffered while a restore-based painter stroke
    /// is open (Curve/Circle/Polygon/Line/Anchored/DragDot — `PainterTool::coalesces_canvas_motion`).
    /// The frame loop flushes ONE Move per frame from this instead of re-stamping the whole shape on
    /// every raw winit `CursorMoved` between frames — the FPS-drop / "Raw rises" fix
    /// (`HANDOFF_per_layer_color_perf_artifacts` §1.R). `None` = nothing pending.
    pub(crate) pending_painter_move: Option<(f32, f32)>,
    /// Diagnostics (HUD): raw winit `CursorMoved` events since the last frame (input rate).
    pub(crate) input_events_this_frame: u32,
    /// Diagnostics (HUD): canvas pointer Moves delivered to the painter this frame (= whole-shape
    /// re-stamps). With coalescing this is ≤1 even when `input_events_this_frame` is high.
    pub(crate) paint_stamps_this_frame: u32,
    /// Diagnostics (HUD): last frame's painter preview produce/upload dispatch time (µs).
    pub(crate) last_dispatch_us: u64,
    /// Diagnostics (HUD): total canvas re-stamp time this frame (µs) — the once-per-frame coalesced
    /// flush PLUS every incremental per-event stamp (Space/Dots/Airbrush) since the last frame. Snapshot
    /// + reset into `last_paint_stamp_us` at the top of `run_render_frame`.
    pub(crate) paint_stamp_us_this_frame: u64,
    /// Diagnostics (HUD): the previous frame's total re-stamp time (µs) — snapshot of
    /// `paint_stamp_us_this_frame`; folded with the dispatch time into `paint_ms_ewma`.
    pub(crate) last_paint_stamp_us: u64,
    /// Diagnostics (HUD): EWMA of painter CPU per frame (stamp flush + dispatch), milliseconds.
    pub(crate) paint_ms_ewma: f32,
    /// Cached on-canvas live preview for the Painter tool — drained
    /// from `PainterTool::current_preview` (the layer composite) each
    /// frame the canvas was painted into. The CPU source of truth for
    /// [`Self::painter_preview_gpu`]. Cleared on Apply + deactivate.
    pub(crate) painter_preview: Option<PainterPreview>,
    /// GPU slot backing the Painter live preview (W3 sprite-suppression).
    /// Mirrors [`Self::bgremoval_preview_gpu`]: the bridge uploads the
    /// premultiplied composite into an `IndividualTextureStore` slot and
    /// the next frame's `sim_extract` emits a `PreviewOverride` pointing
    /// here — so the base layer's opacity/representation affects the WHOLE
    /// sprite in-place (no Vello overlay duplicating the image). `None`
    /// when the preview cache is `None`.
    pub(crate) painter_preview_gpu: Option<PainterPreviewGpu>,
    /// GPU slot backing the live preview of a sprite used as the brush **Shape** while it is NOT the
    /// selected sprite — so brush opacity/blend remote-control edits show on it in real time. A SECOND,
    /// independent slot/override from [`Self::painter_preview_gpu`] (the active sprite): the painter bridge
    /// composites the stashed Shape-source document into it. `None` when no non-selected Shape source.
    pub(crate) painter_shape_source_preview_gpu: Option<PainterPreviewGpu>,
    /// GPU live-preview session for the Painter (ADR-0045 Phase 3 step 2):
    /// holds the `LayerCompositor` + straight→premul blit. Lazily built on the
    /// first GPU-representable frame and reused for the tool session; drives the
    /// SAME `painter_preview_gpu` slot the CPU producer uses, so the GPU path is
    /// transparent to `sim_extract`'s `PreviewOverride`.
    pub(crate) painter_gpu_preview:
        Option<crate::render_loop::painter_gpu_preview::PainterGpuPreview>,
    /// Transient flag set by the Cmd/Ctrl+Enter keybind in
    /// `input_handlers::handle_editor_key` to Apply (bake the layer composite
    /// into the sprite) WITHOUT switching tools. Consumed (taken) by
    /// `painter_bridge::dispatch` — the only site allowed to downcast to
    /// `PainterTool` — which calls `request_commit()`. The deactivate-commit
    /// path (tool switch) is separate via `drive_pending_commit`. No
    /// concrete-tool downcast in the keyboard handler (keeps
    /// `architecture_no_downcast_to_concrete_tool_in_shell` green).
    pub(crate) painter_commit_requested: bool,
    /// Transient flags set by the Cmd+Z / Cmd+Shift+Z keybind (only while the
    /// Painter tool is active) to undo/redo the last structural layer edit.
    /// Consumed (taken) by `painter_bridge::dispatch` — the downcast-allowed
    /// site — which calls `PainterTool::undo_last` / `redo_last` (those mark
    /// the preview dirty so the bridge re-composites). No concrete-tool
    /// downcast in the keyboard handler.
    pub(crate) painter_undo_requested: bool,
    pub(crate) painter_redo_requested: bool,
    /// ADR-0108 cutover: the Pen of the Vector drawing tool. Operates on the
    /// document scene `AppGfx.vec_scene` (document ≠ tool); driven by the shell
    /// input hooks while the `vector` tool is active, and styled each frame from
    /// the tool's palette via `render_loop::vector_bridge`.
    pub(crate) vec_pen: ph2d_vec_edit::PenTool,
    /// ADR-0108 Fase 1: drag-to-size shape drawing (Rectangle / Ellipse /
    /// Polygon). Sibling of `vec_pen`; the shell routes canvas input to one or
    /// the other by `vec_draw_mode` (mirrored from the tool by `vector_bridge`).
    pub(crate) vec_shape: ph2d_vec_edit::ShapeTool,
    /// The tool's current draw-mode + shape parameters, mirrored each frame from
    /// the `VectorTool` (the input dispatch can't downcast — that lives in the
    /// allowlisted bridge). Decides pen vs shape routing + sizes the shapes.
    pub(crate) vec_draw_config: ph2d_tool_vector::VectorDrawConfig,
    /// ADR-0108 Fase 1: node box-select marquee (screen-space `(start, current)`,
    /// same `(f32, f32)` as `last_pointer`) — Shift+drag on empty canvas while the
    /// Vector tool is active. `None` when idle; on release drives `box_select`.
    pub(crate) vec_marquee: Option<((f32, f32), (f32, f32))>,
    /// ADR-0108: undo/redo by snapshot of `vec_scene` (Ctrl+Z / Ctrl+Shift+Z).
    pub(crate) vec_history: ph2d_vec_edit::History,
    /// Gradient group: the gradient handle currently being DRAGGED on-canvas —
    /// a multi-point point OR a linear/radial endpoint (`None` = not dragging).
    pub(crate) vec_grad_drag: Option<ph2d_vec_render::GradHandle>,
    /// The selected gradient handle (drives the overlay highlight + the Remove-
    /// point / Influence / Jitter targets, via [`GradHandle::point`]). `None` = none.
    pub(crate) vec_grad_selected: Option<ph2d_vec_render::GradHandle>,
    /// In-app path clipboard for Vector Ctrl+C/X/V — a clone of the copied path
    /// (geometry + style, id-less). `None` until the first copy/cut.
    pub(crate) vec_clipboard: Option<ph2d_vec_scene::VecClip>,
    /// Snap + grid settings of the Vector tool (edited by the panel's Snap section).
    pub(crate) vec_snap: crate::vec_snap::VecSnapSettings,
    /// Snap targets of the CURRENT gesture — collected once at Down (the scene's
    /// shape doesn't change mid-drag; only the dragged thing, which is excluded).
    pub(crate) vec_snap_targets: ph2d_vec_edit::SnapTargets,
    /// Smart guides to draw this frame (cleared at Up / when nothing snapped).
    pub(crate) vec_snap_guides: Vec<ph2d_vec_render::Guide>,
    /// `VecPathId` → entidade ECS que o representa na Hierarquia (ADR-0110). O
    /// invariante "um path ⟺ uma entidade" é mantido por `vec_entities::sync`.
    pub(crate) vec_entities: crate::vec_entities::VecEntityMap,
    /// Espelho da última sincronia de seleção canvas ↔ Hierarquia (ADR-0110): diz
    /// **quem** mudou neste frame, e por isso quem manda. Ver `sync_selection`.
    pub(crate) vec_sel: crate::vec_selection::VecSelSync,
    /// TOOL_PIVOT: world-space center of the selected sprite's CONTENT
    /// bbox (non-transparent pixels), computed once (lazily, on the
    /// first CTRL-held move) per MovePivot drag and reused as a snap
    /// candidate. `None` between drags / before the first readback.
    /// Cleared when the drag ends. Cached on `App` (not in
    /// `GizmoDragState`) so the pixel readback stays a shell concern.
    pub(crate) pivot_content_center: Option<[f32; 2]>,
    /// Fase 0f: canvas rubber-band box select. `Some` while the user
    /// is left-dragging on empty canvas (no sprite hit, no gizmo
    /// handle hit). Anchored at the Down point in screen coords —
    /// SCREEN, not world, so the rect stays put if the user pans the
    /// camera mid-drag (Photoshop/Figma convention). On Up the rect
    /// is converted to world coords and intersected against every
    /// sprite's bbox via `pick_sprites_in_world_rect`.
    pub(crate) rubber_band: Option<RubberBandState>,
    /// Onda 2 polish: click-vs-drag distinguisher. `Some((bits, down_pos))`
    /// while the user Down'd on a sprite that is part of the multi-
    /// selection (smart-click preservation). If the cursor stays within
    /// a few px of `down_pos` until Up — a click, not a drag — the Up
    /// handler `replace_selection(Some(bits))` to collapse the multi to
    /// just the clicked sprite (Enio: "se há multiplas sprites
    /// selecionas e eu clicar com botão esquerdo em uma delas, todas as
    /// outras devem ser desselecionadas"). If the cursor moves past
    /// the threshold first, this gets cleared and the open Translate
    /// drag becomes a group translate as before (Onda 1).
    pub(crate) pending_single_replace: Option<(u64, (f32, f32))>,
    /// Onda 1 + 2C.4: per-sprite snapshot captured at PointerDown when
    /// a gizmo drag opens with `selected_len > 1`. Covers EVERY
    /// selected sprite EXCEPT the drag's own primary (whose snapshot
    /// lives on `GizmoDragState::start_transform`). `advance_gizmo_drag`
    /// applies the primary's transform delta to each entry — Translate
    /// adds the same world delta to each translation; Local Scale /
    /// Rotate (target = PrimaryIndividual or ExtraIndividual) applies
    /// the factor / angle around each sprite's OWN pivot; Global Scale
    /// / Rotate (target = Global) applies around the shared global
    /// pivot. Cleared on PointerUp (and on every drag-open that opens
    /// a single-sprite drag).
    pub(crate) group_drag_starts: Vec<GroupDragSnapshot>,
}

/// Onda 2C.4: per-sprite start snapshot for group transforms. Captured
/// once at PointerDown — `advance_gizmo_drag` references it without
/// re-reading PresentWorld each frame (avoids compounding mutations).
#[derive(Copy, Clone, Debug)]
pub(crate) struct GroupDragSnapshot {
    pub(crate) entity_bits: u64,
    pub(crate) start_transform: ph2d_editor::TransformSnapshot,
    /// World transform of this entity's parent chain (Enio 2026-05-26
    /// fix: writes into the entity's LOCAL Transform must compensate
    /// for ancestor rotation/scale, or group-drags on children of
    /// rotated parents move along the local axis instead of world).
    /// Consumido em `gizmo_drag.rs` para Translate (world delta → local
    /// via inverse parent) e Global rotate/scale (new world translation
    /// → local). Local rotate/scale aditivo continua funcionando sob
    /// composição sem precisar de parent_world.
    pub(crate) parent_world: ph2d_editor::TransformSnapshot,
}

/// Fase 0f: canvas rubber-band box-select state. Cleared on Up.
#[derive(Copy, Clone, Debug)]
pub(crate) struct RubberBandState {
    /// Screen-space anchor (cursor position at Down), physical px.
    pub(crate) anchor_screen: (f32, f32),
    /// Screen-space current cursor (updated each CursorMoved while
    /// the rubber-band is active).
    pub(crate) current_screen: (f32, f32),
    /// `true` when Shift was held at Down → resolve adds to the
    /// existing selection. `false` → resolve replaces the selection
    /// (clearing first).
    pub(crate) add_mode: bool,
}

/// Cached on-canvas preview bitmap for the Background-Removal tool.
///
/// Wave 10 / Etapa 3 STATUS: all three image-tool previews (BgR, CEQ,
/// Upscale) are now aliases of the generic `ph2d_tool_runtime::PreviewCache`
/// — same shape (entity_bits + Arc<Vec<u8>> rgba + width + height). The
/// type aliases are kept so existing call sites that say
/// `app_state::BgremovalPreview` / `ColorEqualizationPreview` /
/// `UpscalePreview` still resolve; new code should prefer
/// `ph2d_tool_runtime::PreviewCache` directly via the `drive_*` helpers.
pub(crate) type BgremovalPreview = ph2d_tool_runtime::PreviewCache;

/// GPU-side companion to [`BgremovalPreview`] (Lens F, 2026-05-26).
///
/// Owns the transient `IndividualTextureStore` slot that backs a tool's
/// on-canvas live preview. **Tool-agnostic** — shared by BgRemoval and the
/// Painter (see [`PainterPreviewGpu`]); the fields carry no BgR-specific
/// state. The CPU-side preview cache (`PreviewCache`) is
/// the source of truth (Arc-shared with the tool's `current_preview`);
/// the bridge replays it onto this texture whenever the `Arc` buffer
/// is swapped. `arc_ptr` is `Arc::as_ptr` of the last uploaded buffer
/// — comparing it against the live cache's pointer detects a fresh
/// preview without hashing pixels.
#[derive(Copy, Clone, Debug)]
pub(crate) struct BgremovalPreviewGpu {
    /// Renderer-assigned id (the slot in `IndividualTextureStore`).
    pub(crate) texture_id: u32,
    /// Source-pixel width of the texture currently uploaded. Used to
    /// detect resize → `replace_pixels` will rebuild the entry.
    pub(crate) width: u32,
    /// Source-pixel height of the texture currently uploaded.
    pub(crate) height: u32,
    /// `Arc::as_ptr` of the `rgba` buffer most recently uploaded,
    /// stored as `usize` so the struct stays `Send + Sync` without an
    /// unsafe impl — it's an identity token, never dereferenced. A
    /// different value in the live cache means the tool produced a
    /// new preview frame (or selection drifted) → re-upload.
    pub(crate) arc_token: usize,
    /// Entity whose source produced the uploaded pixels. Used as a
    /// belt-and-suspenders check alongside `arc_token` so a
    /// coincidental token reuse can't paint the wrong sprite.
    pub(crate) entity_bits: u64,
}

/// Cached on-canvas live preview bitmap for the Painter tool (W1 T1.5).
/// Same generic `ph2d_tool_runtime::PreviewCache` shape as BgR / CEQ /
/// Upscale — `drive_source_push` + `drive_preview_cache` consume directly.
pub(crate) type PainterPreview = ph2d_tool_runtime::PreviewCache;

/// GPU preview slot for the Painter — same tool-agnostic shape as
/// [`BgremovalPreviewGpu`] (texture_id + dims + arc_token + entity_bits).
/// Aliased so the Painter bridge reads as its own type while sharing the
/// one implementation (W3 sprite-suppression).
pub(crate) type PainterPreviewGpu = BgremovalPreviewGpu;

/// Cached on-canvas live preview bitmap for the Color Equalization
/// tool. Wave 10 / Etapa 3: now uniformized with BgR + Upscale as
/// the generic `ph2d_tool_runtime::PreviewCache`. CEQ still holds
/// a `BTreeMap<u64, ColorEqualizationPreview>` (one entry per
/// selected sprite — CEQ paints per-sprite previews), driven by
/// the new `drive_multi_preview_cache` helper that replaced the
/// hand-written loop + downcast.
pub(crate) type ColorEqualizationPreview = ph2d_tool_runtime::PreviewCache;

/// Cached on-canvas live preview bitmap for the Upscale tool.
///
/// Wave 10 / Etapa 2: re-exported as the generic
/// `ph2d_tool_runtime::PreviewCache` (same shape as BgRemoval —
/// entity_bits + Arc<Vec<u8>> rgba + width + height). The
/// `UpscalePreview` alias is kept so existing call sites still
/// resolve; new code uses `PreviewCache` directly via the
/// `drive_*` helpers in `ph2d-tool-runtime`.
pub(crate) type UpscalePreview = ph2d_tool_runtime::PreviewCache;

/// True for any tool that belongs to the **Image Tools** group — i.e.
/// whose manifest is registered in the `"image_tools"` cluster (Bg
/// Removal, Padding, Trim Transparency, Make Square, Real Size, and
/// EVERY future image tool). Data-driven on purpose: there is no
/// hardcoded id list to keep in sync — add a tool to the `image_tools`
/// cluster in its manifest and this predicate (and everything gated on
/// it) picks it up automatically.
///
/// `installed_registry()` is `Some` in the real editor (installed at
/// boot); the `None` fallback (pre-registry boot / isolated tests)
/// reports `false`, matching the legacy "no gating" behavior there.
pub(crate) fn is_image_edit_tool(id: &ph2d_editor::ToolId) -> bool {
    // `m.id` is the manifest's `&'static str` id; `id` is the editor's
    // `ToolId` newtype (what `Tool::id()` returns). Bridge the two via
    // `ToolId::new`.
    ph2d_editor::installed_registry()
        .map(|reg| {
            reg.cluster("image_tools")
                .iter()
                .any(|m| ph2d_editor::ToolId::new(m.id) == *id)
        })
        .unwrap_or(false)
}

/// Indices into `tools.tools()` that are visible in the top-right tool
/// palette right now. Brush / Move are always present; the image-edit
/// tools appear ONLY while Image Tools mode is on — off, they're fully
/// gone (no icon painted, no hit zone), the absolute rule for "Image
/// Tools off ⟹ image tools inaccessible". Paint AND hit-test must both
/// map palette slots through this so their indices can't drift.
pub(crate) fn palette_visible_tool_indices(
    tools: &ph2d_editor::ToolRegistry,
    image_tools_mode_on: bool,
) -> Vec<usize> {
    tools
        .tools()
        .iter()
        .enumerate()
        .filter(|(_, t)| image_tools_mode_on || !is_image_edit_tool(&t.id()))
        .map(|(i, _)| i)
        .collect()
}

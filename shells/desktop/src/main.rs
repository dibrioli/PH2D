#![forbid(unsafe_code)]
// ph2d-loc-cap: 928 LOC post-Wave-3.1 stages A+B+C. Wave 3.1
// extracted ~1680 LOC (was 2607 pre-wave): atlas_loader.rs +
// sim_populate.rs (stage B); render_loop.rs (stage C — the
// 1582-LOC `App::render_frame` body lifted to a sibling that
// still carries its own ph2d-loc-cap exception). What remains
// here: struct App + AppGfx + ImageEditSnapshot + HeroLive defs
// (~280 LOC), several `impl App` input-handler methods (~400
// LOC: handle_dropped_files, handle_editor_key,
// dispatch_panel_pointer), the ApplicationHandler trait impl
// (~80 LOC), tests + main(). Further reduction to ≤ 600 needs
// extracting App/AppGfx structs + impl App methods to siblings
// (`app_state.rs` + `app_input.rs`) — Wave 3.2 scope.

//! Desktop shell — winit 0.30 + wgpu + ECS + sprite render + M6+M7+M12.
//!
//! Run with: `cargo run -p ph2d-host-desktop`
//!
//! Layered subsystems (each gated to keep the demo bootable even if
//! one fails — never crash the shell over an integration-demo issue):
//! - **M5** SpriteRenderer + 1000-sprite Vogel spiral with bouncing motion
//! - **M6** AssetDb loads 16 real PNG files from `assets/sprites/` (auto-
//!   generated on first launch) and composes them into a 256×256
//!   RGBA8 atlas. Falls back to procedural dummy if anything fails.
//! - **M7** ScriptHost with placeholder Luau script; per-frame gc_step
//!   keeps the GC budget warm for future script-driven gameplay.
//! - **M12** editor data layer: `ZenMode` (Tab toggle), `ToastQueue`
//!   (T key adds info toast), theme switch (M key flips Dark↔Light),
//!   and a `FloatingPanel` (Procreate-style selection demo). Visible
//!   via window title since Vello widget paint requires sharing the
//!   wgpu Surface with the sprite pipeline (see `integration.rs`).
//!
//! M8 add-on (gamepad path):
//! - gilrs adapter (`gilrs_adapter` module) pumps gamepad events into
//!   [`ph2d_input::InputState`] each frame, BEFORE sim tick.
//! - Connection / button / axis events log to the terminal at the
//!   `[Nms]` timestamp prefix used by the rest of the shell.
//! - Axis logs filter dead-zone jitter (only `|value| > 0.25`).
//! - Pencil events are wired through the abstraction but not produced
//!   by this shell (iPad shell will, in M9+).
//!
//! Out of scope here:
//! - **M8 → ScriptHost**: `InputState` lives on `App`, but routing into
//!   `ph2d.input` Luau snapshot is a follow-up.
//! - **M11** Vello text/widget overlay — needs surface-sharing pass.

mod atlas_loader;
mod cursor_pos;
mod forwarding;
mod hero_bridge;
mod hero_intents;
mod image_import;
mod init;
mod input_dispatch;
mod input_log;
mod integration;
mod keymap;
mod render_loop;
mod sim_populate;
mod theme;
mod winit_host;

use cursor_pos::live_cursor_in_window;
// forwarding::* moved to input_dispatch.rs (PR 9b).
use image_import::import_image_at_camera;
use input_log::log_input_event;
// keymap::winit_to_editor_keycode moved to input_dispatch.rs (PR 9b).
// theme::parse_theme_env moved to init.rs (PR 9c).
use winit_host::{LoggingHandler, WinitHost};

use bumpalo::Bump;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::{FixedStep, Vec2, install_panic_hook, panic};
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommand, EditorCommandQueue, HierarchySnapshot, HierarchyWalkState,
    apply_editor_commands, build_hierarchy_snapshot,
};
use ph2d_ecs::{
    Component, PresentWorld, SimComponent, SimWorld, Transform, TransformPropagationState,
    WorklistBuf,
};
use ph2d_editor::paint::Paint;
use ph2d_editor::zones::Rect as EditorRect;
use ph2d_editor::{
    HeroScreen, Layout as EditorLayout, PanelControl, PanelEvent, Toast, ToastQueue, ToolRegistry,
    ZenMode,
};
use std::collections::BTreeMap;
// NodeId surfaces in our `dragging` field; re-exported by ph2d-editor.
use ph2d_editor::NodeId;
use ph2d_gpu::{AcquireError, SurfaceContext};
use ph2d_host::{HostHandler, Lifecycle, Modifiers, PlatformHost, WindowSize};
use ph2d_input::InputState;
use ph2d_render::{Camera2d, Compositor, GameRt, SpriteRenderer, Tonemap, VelloPass};
use ph2d_script::ScriptHost;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Color as VelloColor, VectorScene};
use std::sync::Arc;
use std::time::Instant;

mod gilrs_adapter;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState};
use winit::window::{Window, WindowId};

pub(crate) const SPRITE_COUNT: u32 = 1000;
/// Half-extent of the bouncing world in meters. Camera default has
/// `height_world = 10`, so [-5, 5] in Y is exactly the visible region;
/// X depends on aspect (narrower than visible at 4:3+).
pub(crate) const WORLD_HALF: f32 = 5.0;

// ADR-0025: `Position(Vec2)` removed — `Transform` from `ph2d_ecs` is
// the canonical pose component now. `Velocity` stays local to the
// demo because no other crate consumes it yet (M14.2+ may promote it).
#[derive(Component, Copy, Clone, Debug)]
pub(crate) struct Velocity(pub(crate) Vec2);
impl SimComponent for Velocity {}

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
    /// Single-level undo for image-edit actions (Trim Transparency,
    /// Make Square). Captures the pre-edit Sprite source / size /
    /// Transform translation so Cmd+Z (or TOOL_UNDO click) restores
    /// the previous state. The full editor undo system is M14.x scope;
    /// image-edits are the only path that ships undo in this milestone.
    ///
    /// Single-level by design: each new image-edit overwrites the
    /// snapshot (releasing the now-orphaned individual texture if the
    /// PRE-edit state was on one). Future M14.x replaces this with a
    /// proper command stack rooted in `EditorCommandQueue`.
    pub(crate) image_edit_undo: Option<ImageEditSnapshot>,
}

/// Pre-edit snapshot of a sprite that an image-edit action mutated.
/// Owned by [`AppGfx::image_edit_undo`]; populated by the Trim / Make
/// Square drainers, consumed by [`AppGfx::undo_image_edit`].
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
    /// The new individual texture id that the edit acquired. Released
    /// on undo so the now-orphaned post-edit texture doesn't leak.
    pub(crate) post_individual_id: u32,
    /// Human-readable label for the toast: "Trim" / "Make square".
    pub(crate) label: &'static str,
}

/// Per-frame state owned by the live editor bridge (ADR-0025 M14.4a).
struct HeroLive {
    bridge: hero_bridge::EntityNodeMap,
    walk_state: HierarchyWalkState,
    /// Scratch buffer for `build_hierarchy_snapshot`'s DFS stack.
    /// Preserved across frames so HR-3 zero-alloc invariant holds.
    walk_scratch: Vec<(ph2d_ecs::Entity, u8, Option<ph2d_ecs::Entity>)>,
    /// Reused per-frame snapshot. `build_hierarchy_snapshot` clears
    /// the inner Vec without releasing capacity.
    snapshot: HierarchySnapshot,
}

pub(crate) struct App {
    window: Option<Arc<Window>>,
    host: Option<WinitHost>,
    gfx: Option<AppGfx>,
    handler: LoggingHandler,
    fixed_step: FixedStep,
    last_frame: Instant,
    pending_resize: Option<WindowSize>,
    modifiers: ModifiersState,
    last_pointer: (f32, f32),
    /// Set to `Some(node_id)` when the user pressed inside a draggable
    /// widget (Slider) — subsequent pointer-move events continue to
    /// fire SetValue until pointer-up clears this. None for click-only
    /// widgets (Toggle, RadioGroup, ColorSwatch).
    dragging: Option<NodeId>,
    title_dirty: bool,
    /// gilrs context (M8). `None` if init failed (e.g. Linux without
    /// /dev/input read perms in CI sandboxes — we degrade gracefully
    /// instead of crashing the renderer).
    gilrs: Option<gilrs::Gilrs>,
    /// Input snapshot pumped by the gilrs adapter each frame.
    input: InputState,
    /// M14.4b.bis: middle-button camera pan state.
    /// `Some(anchor)` while a middle-drag is in progress; subsequent
    /// `CursorMoved` events feed `Camera2d::pan_screen_delta`.
    pan_anchor: Option<(f32, f32)>,
    /// M14.4e: most recent cursor position (in physical px, top-left
    /// origin). Cached on every `CursorMoved` so `DroppedFile` can
    /// project the drop point to world coords — winit's `DroppedFile`
    /// event itself carries no position.
    last_cursor: (f32, f32),
    /// M14.4e: paths buffered between `HoveredFile` and `DroppedFile`
    /// (winit emits one `HoveredFile` per file when multiple are
    /// dragged together). Cleared on `HoveredFileCancelled` or after
    /// `DroppedFile` is handled.
    hovered_files: Vec<std::path::PathBuf>,
    /// M14.7 polish (7.3 fix): paths queued by `DroppedFile` events.
    /// winit fires `DroppedFile` once per file when the user drops
    /// multiple, but if any one event is lost (e.g. another window
    /// event consumes the loop iteration) sprites silently go missing.
    /// We buffer here and drain at the start of `render_frame` so
    /// every path that reached us imports atomically, regardless of
    /// event interleaving.
    pending_drops: Vec<std::path::PathBuf>,
    /// M14.4g Telemetry Phase A: EWMA-smoothed frame time pushed to
    /// the hero's `BottomHudStats` each frame so the status bar shows
    /// real fps/ms instead of the M5 placeholder strings. α=0.1 —
    /// canonical "smooth without dormant" value used by RTSS / Unity
    /// stats / Tracy.
    frame_ms_ewma: f32,
    /// M14.7 polish (10.1): EWMA-smoothed "raw" frame work time —
    /// measured from start of `render_frame` to end of
    /// `queue.submit`, excluding the vsync wait. Same α as
    /// `frame_ms_ewma`. Surfaced as `BottomHudStats.raw_fps` so the
    /// status bar shows hardware capacity alongside the synced fps.
    frame_cpu_ms_ewma: f32,
    /// M14.7 polish (19.3): overlap-cycle state for the canvas-pick
    /// path. Each Primary Down at the same world position increments
    /// `cycle_count`; on every ODD count we advance `cycle_idx` so
    /// the user can step DOWN the stack at a fixed cursor location.
    /// Even counts leave the selection alone (allowing drag without
    /// re-cycling). Reset when the click moves > 4 px in world space
    /// or the hit list shape changes.
    cycle_pick_world: Option<[f32; 2]>,
    cycle_pick_hits: Vec<u64>,
    cycle_pick_idx: usize,
    cycle_pick_count: u32,
    /// `entity_bits` of the sprite whose RGBA was last pushed into
    /// the active `BgRemovalTool` snapshot. `None` until the first
    /// push. Reset to `None` whenever the user activates BgRemoval
    /// via Digit3 so the very next frame re-pushes against the
    /// current selection. The snapshot loop reads this every frame
    /// while BgRemoval is the active tool to skip redundant
    /// pushes (the thumbnail rebuild + preview rerun are work).
    last_bgremoval_pushed_entity: Option<u64>,
}

impl App {
    fn new() -> Self {
        let gilrs = match gilrs::Gilrs::new() {
            Ok(g) => {
                let pads: Vec<String> = g
                    .gamepads()
                    .map(|(id, pad)| format!("[{:?}] {}", id, pad.name()))
                    .collect();
                if pads.is_empty() {
                    println!("gilrs: initialized; no gamepads connected yet");
                } else {
                    println!("gilrs: detected {} gamepad(s):", pads.len());
                    for p in &pads {
                        println!("  {p}");
                    }
                }
                Some(g)
            }
            Err(e) => {
                eprintln!("gilrs init failed (continuing without gamepad): {e}");
                None
            }
        };
        Self {
            window: None,
            host: None,
            gfx: None,
            handler: LoggingHandler::new(),
            fixed_step: FixedStep::default(),
            last_frame: Instant::now(),
            pending_resize: None,
            modifiers: ModifiersState::default(),
            last_pointer: (0.0, 0.0),
            dragging: None,
            title_dirty: true,
            gilrs,
            input: InputState::new(),
            pan_anchor: None,
            last_cursor: (0.0, 0.0),
            hovered_files: Vec::new(),
            pending_drops: Vec::new(),
            frame_cpu_ms_ewma: 1.0, // optimistic baseline; reseeds on
            // the first frame's measurement
            cycle_pick_world: None,
            cycle_pick_hits: Vec::new(),
            cycle_pick_idx: 0,
            cycle_pick_count: 0,
            last_bgremoval_pushed_entity: None,
            frame_ms_ewma: 16.7, // ~60 Hz baseline so the first
                                 // frame's status bar doesn't display
                                 // a wild value while the EWMA seeds.
        }
    }

    /// Pump every queued gilrs event into the [`InputState`] and log
    /// the salient ones. Press / release / axis-change all logged at
    /// elapsed-ms timestamps so behavior is auditable from the
    /// terminal without an explicit debug overlay.
    fn pump_gamepad(&mut self) {
        let Some(g) = self.gilrs.as_mut() else {
            return;
        };
        // begin_frame snapshots last-frame held buttons so
        // pressed()/released() return correct edge-trigger values.
        self.input.begin_frame();
        while let Some(gilrs::Event { event, time: _, .. }) = g.next_event() {
            match event {
                gilrs::EventType::Connected => {
                    println!("[{:>6}ms] gamepad connected", self.handler.elapsed_ms());
                }
                gilrs::EventType::Disconnected => {
                    println!("[{:>6}ms] gamepad disconnected", self.handler.elapsed_ms());
                }
                _ => {
                    if let Some(translated) = gilrs_adapter::translate(event) {
                        self.input.apply_event(translated);
                        log_input_event(self.handler.elapsed_ms(), &translated);
                    }
                }
            }
        }
    }

    fn convert_modifiers(state: ModifiersState) -> Modifiers {
        Modifiers {
            shift: state.shift_key(),
            ctrl: state.control_key(),
            alt: state.alt_key(),
            meta: state.super_key(),
        }
    }

    fn timestamp_ns() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    }

    /// M14.4e: process files the user dropped onto the window.
    /// Each call handles exactly the paths in this drop event (winit
    /// fires DroppedFile once per path).
    ///
    /// - Image extensions go through `import_image_at_camera` with
    ///   the drop point converted to world coords via
    ///   `Camera2d::screen_to_world(last_cursor, surface_size)`.
    /// - Non-image files raise a "Skipped" toast each — non-fatal.
    /// - Sprite spawn position = the cursor's world coordinate; the
    ///   sprite quad is center-anchored (see sprite.wgsl) so the
    ///   image visually centers on the cursor.
    /// - Batch drops (multiple paths in one DroppedFile sequence)
    ///   stack at the same point — user can fan them out via the
    ///   M14.7 gizmo after spawn.
    fn handle_dropped_files(&mut self, paths: &[std::path::PathBuf]) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return;
        };
        let pixels_per_meter = hero.project.pixels_per_meter;
        let win = gfx.surface.size();
        // macOS-only: winit 0.30 does NOT emit `CursorMoved` during
        // external file drag operations (see
        // winit-0.30.13/src/platform_impl/macos/window_delegate.rs —
        // `draggingEntered:` doesn't extract `draggingLocation`, no
        // `draggingUpdated:` is implemented at all). So `last_cursor`
        // is whatever it was BEFORE the drag started, not where the
        // file was actually dropped. Query the live cursor from
        // CoreGraphics (`macos_cursor_query`) to override; fall back
        // to `last_cursor` on other platforms.
        let cursor_px = self
            .host
            .as_ref()
            .and_then(|h| live_cursor_in_window(h.window()))
            .unwrap_or(self.last_cursor);
        let drop_world_raw = gfx.camera.screen_to_world(cursor_px, win);
        // Grid-snap apply (drag-drop site). When snap is enabled in
        // `grid_snap_state`, align the drop position to the active
        // grid before spawning so a multi-sprite drop forms a tidy
        // grid rather than scattering at sub-pixel offsets. No-op
        // when snap_enabled = false or active kind has no snap target.
        let drop_world: [f32; 2] = if let Some(hero) = gfx.hero_screen.as_mut() {
            // Drag-drop: sprite hasn't been imported yet, so half-size
            // is unknown. Pass [0.0, 0.0] — Corner-family modes
            // degenerate to point-Intersection snap in that case.
            hero.grid_snap_state.snap_world(drop_world_raw, [0.0, 0.0])
        } else {
            drop_world_raw
        };
        for path in paths {
            if !ph2d_asset::is_supported_image_extension(path) {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("(unnamed)");
                gfx.toasts
                    .push(Toast::warning(format!("Skipped non-image: {name}")));
                self.title_dirty = true;
                continue;
            }
            // The import helper currently anchors the spawn at
            // `camera.center`. Temporarily move the camera so the
            // sprite Transform lands at the cursor world point;
            // restore afterward so pan/zoom state is preserved.
            // Cleaner alternative: `import_image_at_world(pos, ...)`
            // — defer to M14.5 when the import path is touched.
            let saved_center = gfx.camera.center;
            gfx.camera.center = drop_world;
            let next_key = gfx.next_import_cell;
            let result = import_image_at_camera(
                &mut gfx.sim,
                &mut gfx.renderer,
                &gfx.asset_db,
                &gfx.camera,
                next_key,
                path,
                pixels_per_meter,
                &mut gfx.atlas_asset_map,
            );
            gfx.camera.center = saved_center;
            match result {
                Ok(label) => {
                    gfx.next_import_cell = gfx.next_import_cell.saturating_add(1);
                    gfx.toasts.push(Toast::success(format!("Imported {label}")));
                    self.title_dirty = true;
                }
                Err(e) => {
                    eprintln!("M14.4e drop failed: {e}");
                    gfx.toasts.push(Toast::error(format!("Drop failed: {e}")));
                    self.title_dirty = true;
                }
            }
        }
    }

    /// M12 demo control router.
    ///   Tab — toggle ZenMode (debounced 30 frames)
    ///   M   — flip theme Dark↔Light
    ///   T   — push info toast
    ///   1   — activate Brush tool
    ///   2   — activate Move tool
    fn handle_editor_key(&mut self, code: KeyCode) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        match code {
            KeyCode::Tab if gfx.zen.try_toggle() => {
                let msg = if gfx.zen.is_active() {
                    "Zen mode ON (zones collapsed)"
                } else {
                    "Zen mode OFF (zones restored)"
                };
                gfx.toasts.push(Toast::info(msg));
                self.title_dirty = true;
            }
            KeyCode::KeyM => {
                gfx.theme = gfx.theme.next();
                gfx.toasts
                    .push(Toast::info(format!("Theme → {}", gfx.theme.id())));
                self.title_dirty = true;
            }
            KeyCode::KeyT => {
                gfx.toasts.push(Toast::info("Toast key (T) pressed"));
                self.title_dirty = true;
            }
            // Cmd+Z / Ctrl+Z — image-edit undo (Trim, Make Square,
            // Bg Removal). Single-level by design; broader editor
            // undo is M14.x. Wave 2.5 PR 11.8b3: bus migration (was
            // `hero.pending_undo_image_edit = true`).
            KeyCode::KeyZ if self.modifiers.super_key() || self.modifiers.control_key() => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.bus
                        .push(ph2d_editor::action_bus::EditorAction::UndoImageEdit);
                }
            }
            KeyCode::Digit1 if gfx.tools.set_active(&ph2d_editor::ToolId::new("brush")) => {
                gfx.toasts.push(Toast::info("Tool → Brush"));
                self.title_dirty = true;
            }
            KeyCode::Digit2 if gfx.tools.set_active(&ph2d_editor::ToolId::new("move")) => {
                gfx.toasts.push(Toast::info("Tool → Move"));
                self.title_dirty = true;
            }
            KeyCode::Digit3 if gfx.tools.set_active(&ph2d_editor::ToolId::new("bgremoval")) => {
                gfx.toasts.push(Toast::info("Tool → Bg Removal"));
                self.title_dirty = true;
                // Force a fresh snapshot push on the next frame so the
                // newly-active tool sees the current selection's RGBA
                // (the snapshot-push loop below tracks last-pushed
                // entity — invalidating it here re-triggers).
                self.last_bgremoval_pushed_entity = None;
            }
            // M14.7 polish: F / Home = frame the currently selected
            // sprite. Falls back to (0, 0) when nothing is selected
            // (Blender / Maya "frame view" semantics). Raises a
            // pending intent on the hero — the render_frame drain
            // resolves the selection and updates `gfx.camera`.
            KeyCode::Home | KeyCode::KeyF => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    // Wave 2.5 PR 11.8d: bus migration (was
                    // `hero.pending_view_focus = Some(...)`).
                    hero.bus
                        .push(ph2d_editor::action_bus::EditorAction::SetViewFocus {
                            kind: ph2d_editor::ViewFocusKind::Selected,
                        });
                } else {
                    // No hero panel — fall back to legacy "reset
                    // camera" so the non-editor demo mode still has
                    // a way to recover from a bad pan/zoom.
                    gfx.camera = Camera2d::default();
                    gfx.toasts.push(Toast::info("Camera → reset"));
                }
                self.title_dirty = true;
            }
            // M14.4b: toggle grid visibility. The context-menu entry
            // promises "Show Grid · G" — this is the shortcut. Affects
            // only the hero's grid_visible flag; grid_view publishing
            // by the host continues regardless.
            KeyCode::KeyG => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.grid_visible = !hero.grid_visible;
                    let msg = if hero.grid_visible {
                        "Grid → on"
                    } else {
                        "Grid → off"
                    };
                    gfx.toasts.push(Toast::info(msg));
                    self.title_dirty = true;
                }
            }
            _ => {}
        }
    }

    /// Hit-test the active tool's panel at `(px, py)` and dispatch a
    /// [`PanelEvent`] into the tool. `is_press` distinguishes the
    /// initial mouse-down (which may start a drag) from continued
    /// move-while-dragging (which only updates an in-progress slider).
    fn dispatch_panel_pointer(&mut self, px: f32, py: f32, is_press: bool) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        if gfx.zen.is_active() {
            return; // panels hidden
        }
        let Some(tool) = gfx.tools.active() else {
            return;
        };
        let panel = tool.build_panel();
        let viewport = EditorRect::new(
            0.0,
            0.0,
            gfx.surface.size().width as f32,
            gfx.surface.size().height as f32,
        );
        let widget_rects = panel.control_widget_rects(viewport);

        // Existing drag → re-emit SetValue against the same node. Done
        // even if pointer left the original cell (slider-style "live
        // drag" feel).
        if let Some(dragging_id) = self.dragging
            && let Some((idx, ctrl)) = panel
                .controls
                .iter()
                .enumerate()
                .find(|(_, c)| matches!(c, PanelControl::Slider(s) if s.id == dragging_id))
            && let Some(rect) = widget_rects.get(idx)
            && let PanelControl::Slider(_) = ctrl
        {
            let v = ((px - rect.x) / rect.w).clamp(0.0, 1.0) as f64;
            if let Some(active) = gfx.tools.active_mut() {
                active.handle_panel_event(PanelEvent::SetValue(dragging_id, v));
            }
            return;
        }

        if !is_press {
            return; // not a click and not a drag — nothing to do
        }

        // Find the cell containing (px, py).
        let Some((idx, _)) = widget_rects
            .iter()
            .enumerate()
            .find(|(_, r)| r.contains(px, py))
        else {
            return;
        };
        let ctrl = &panel.controls[idx];
        let rect = widget_rects[idx];

        let event = match ctrl {
            PanelControl::Slider(s) => {
                self.dragging = Some(s.id);
                let v = ((px - rect.x) / rect.w).clamp(0.0, 1.0) as f64;
                Some(PanelEvent::SetValue(s.id, v))
            }
            PanelControl::Toggle(t) => Some(PanelEvent::Toggle(t.id, !t.on)),
            PanelControl::RadioGroup(g) if !g.options.is_empty() => {
                // Horizontal split — pick option by which sub-rect
                // contains the pointer.
                let opt_w = rect.w / g.options.len() as f32;
                let opt_idx = (((px - rect.x) / opt_w) as usize).min(g.options.len() - 1);
                Some(PanelEvent::SelectOption(
                    g.id,
                    g.options[opt_idx].value.clone(),
                ))
            }
            PanelControl::ColorSwatch(s) => Some(PanelEvent::Click(s.id)),
            PanelControl::Action(_) | PanelControl::RadioGroup(_) => None,
        };

        if let Some(event) = event
            && let Some(active) = gfx.tools.active_mut()
        {
            active.handle_panel_event(event);
            self.title_dirty = true;
            // Drain BgRemoval's Apply Toggle: when on, the Tool sets
            // `pending_apply = true` inside `handle_panel_event`.
            // Push `EditorAction::Bgremoval { entity_bits }` so the
            // per-frame drain in `render_frame` runs the algorithm
            // at full resolution against the live sprite. Wave 2.5
            // PR 11.8b3: bus migration (was `hero.pending_bgremoval
            // = Some(bits)`).
            if let Some(bg) = active
                .as_any_mut()
                .downcast_mut::<ph2d_editor::tools::bgremoval::BgRemovalTool>()
                && bg.take_pending_apply()
                && let Some(hero) = gfx.hero_screen.as_mut()
                && let Some(bits) = hero.gizmo_selection
            {
                hero.bus
                    .push(ph2d_editor::action_bus::EditorAction::Bgremoval { entity_bits: bits });
            }
        }
    }

    /// Snapshot `InputState` into the ScriptHost's `ph2d.input` table
    /// so Luau can read held buttons / axis values via the canonical
    /// `gamepad.held.<button>` and `gamepad.axis.<axis>` keys.
    /// Cleared and rebuilt every frame — keys absent from this frame
    /// resolve to `nil` on the Luau side (per the M8 ph2d_input
    /// resolves test).
    fn push_input_to_script(&self) {
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let Some(host) = gfx.script.as_ref() else {
            return;
        };
        host.clear_input();
        for button in self.input.gamepad.iter_held() {
            let key = format!("gamepad.held.{}", button.as_lua_key());
            host.provide_input(&key, 1.0);
        }
        for (axis, value) in self.input.gamepad.iter_axes() {
            let key = format!("gamepad.axis.{}", axis.as_lua_key());
            host.provide_input(&key, value as f64);
        }
    }

    /// Per-frame render orchestration — body lifted to
    /// [`crate::render_loop`] (Wave 3.1 stage C). See its module docs
    /// for the rationale + the split-impl pattern.
    fn render_frame(&mut self) {
        self.run_render_frame();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // PR 9c of the convention-by-discovery migration: subsystem
        // boot pipeline lives in `init::build_initial_state`. This
        // method now only wires the produced state into `self` and
        // fires lifecycle hooks. See
        // `docs/Migracao/2026-05-convention-by-discovery.md`.
        let (window, host, gfx) = init::build_initial_state(&self.handler, event_loop);
        let size = gfx.surface.size();
        let scale = host.scale_factor();
        self.window = Some(window);
        self.host = Some(host);
        self.gfx = Some(gfx);
        self.handler.on_lifecycle(Lifecycle::Foreground);
        self.handler.on_resize(size, scale);
        self.title_dirty = true;
        if let Some(host_ref) = self.host.as_ref() {
            host_ref.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // PR 9b of the convention-by-discovery migration: per-arm
        // handlers live in `input_dispatch::App::on_<arm>`. This
        // method is a pure dispatch table — adding a new arm is one
        // line here + one method there. See
        // `docs/Migracao/2026-05-convention-by-discovery.md`.
        match event {
            WindowEvent::CloseRequested => self.on_close_request(event_loop),
            WindowEvent::Resized(size) => self.on_resized(size),
            WindowEvent::HoveredFile(path) => self.on_hovered_file(path),
            WindowEvent::HoveredFileCancelled => self.on_hovered_file_cancelled(),
            WindowEvent::DroppedFile(path) => self.on_dropped_file(path),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.on_scale_factor_changed(scale_factor)
            }
            WindowEvent::ModifiersChanged(mods) => self.on_modifiers_changed(mods),
            WindowEvent::Ime(winit::event::Ime::Commit(text)) => self.on_ime_commit(text),
            WindowEvent::CursorMoved { position, .. } => self.on_cursor_moved(position),
            WindowEvent::MouseWheel { delta, .. } => self.on_mouse_wheel(delta),
            WindowEvent::MouseInput { state, button, .. } => self.on_mouse_input(state, button),
            WindowEvent::KeyboardInput { event, .. } => self.on_keyboard_input(event),
            WindowEvent::RedrawRequested => self.render_frame(),
            _ => {}
        }
    }
}

/// Floor for `pixels_per_meter` used inside the import math; below
/// this a single sprite would span kilometers and break camera math.
/// The UI clamps to a higher floor (`MIN_PIXELS_PER_METER = 1.0` in
/// `ph2d_editor::project`) but defense-in-depth here keeps the shell
/// safe even if a future config path skips that clamp.
pub(crate) const EPS_PIXELS_PER_METER: f32 = 0.01;

/// When the image-edit undo slot is being overwritten by a new edit,
/// release the previous snapshot's pre-edit Individual texture (if
/// any) so the now-orphaned texture doesn't leak. Atlas-backed
/// pre-sources don't need release — they share the texture via the
/// asset_db. No-op when the slot is empty.
pub(crate) fn drop_undo_pre_source_if_individual(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<ImageEditSnapshot>,
) {
    if let Some(prev) = slot.take()
        && let ph2d_render::SpriteSource::Individual { texture_id } = prev.pre_source
    {
        renderer.individual_mut().release(texture_id);
    }
}

/// Floor for the world-space side length of an imported sprite.
/// Guarantees the quad is selectable even if the user picks an
/// absurd `pixels_per_meter` value combined with a 1-pixel image.
pub(crate) const MIN_SPRITE_SIZE: f32 = 0.001;

/// Query the live cursor position relative to `window` in physical
/// pixels (top-left origin). Returns `None` if the platform path
/// fails; callers fall back to a cached value.
///
/// Existence rationale: winit 0.30 on macOS does not emit
/// `CursorMoved` during external file drag operations, so by the
/// time `DroppedFile` fires the cached cursor is stale. We bypass
/// the event stream by asking CoreGraphics for the live cursor
/// directly. Other platforms reach here only as a no-op stub.
/// Resolve the editor theme from a name (typically read from the
/// `PH2D_THEME` env var), falling back to [`Theme::Forge`] for
/// missing/invalid values. Recognised names match `Theme::id()`
/// (`forge`, `workshop`, `sunstone`, `blueprint`).
fn main() {
    install_panic_hook();
    let event_loop = EventLoop::new().expect("create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    println!("PH2D desktop shell starting (close window or Cmd+Q to exit)…");
    event_loop.run_app(&mut app).expect("event loop crashed");
    println!("PH2D desktop shell exited cleanly.");
}

#[cfg(test)]
mod theme_env_tests {
    use super::*;
    use crate::theme::resolve_theme;

    #[test]
    fn unset_defaults_to_forge() {
        assert_eq!(resolve_theme(None), Theme::Forge);
    }

    #[test]
    fn known_names_resolve() {
        assert_eq!(resolve_theme(Some("workshop")), Theme::Workshop);
        assert_eq!(resolve_theme(Some("sunstone")), Theme::Sunstone);
        assert_eq!(resolve_theme(Some("blueprint")), Theme::Blueprint);
        assert_eq!(resolve_theme(Some("forge")), Theme::Forge);
    }

    #[test]
    fn unknown_falls_back_to_default() {
        assert_eq!(resolve_theme(Some("dracula")), Theme::Forge);
    }
}

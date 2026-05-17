#![forbid(unsafe_code)]
// ph2d-loc-cap: 2602 LOC post-Wave-2.5. The Action Bus migration
// retired all 20 `pending_X` fields (PR 11.8b/c/d) and collapsed
// the 18 filter-and-replace drains into a single consolidated
// `for action in hero.bus.drain()` match at the top of the editor
// dispatch section. Saved ~300 LOC; the file still carries
// non-editor concerns (winit event loop, GPU bootstrap, Asset/
// ScriptHost wiring, file picker, integration demo) so the 600
// cap can't be reached without splitting those into sibling
// modules. Deferred to a follow-up refactor PR (Wave 3).

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
    Component, Name, PresentWorld, SimComponent, SimRef, SimWorld, Transform,
    TransformPropagationState, Visibility, WorklistBuf, propagate_transforms,
};
use ph2d_editor::paint::{Paint, PaintCtx};
use ph2d_editor::zones::Rect as EditorRect;
use ph2d_editor::{
    HeroScreen, Layout as EditorLayout, PanelControl, PanelEvent, RequestedSpriteStrategy, Toast,
    ToastQueue, ToolRegistry, WidgetEvent, ZenMode, paint_hero_screen,
};
use std::collections::BTreeMap;
// NodeId surfaces in our `dragging` field; re-exported by ph2d-editor.
use ph2d_editor::NodeId;
use ph2d_gpu::{AcquireError, SurfaceContext};
use ph2d_host::{HostHandler, Lifecycle, Modifiers, PlatformHost, WindowSize};
use ph2d_input::InputState;
use ph2d_render::{
    Camera2d, Compositor, GameRt, RenderInstance, Sprite, SpriteRenderer, Tonemap, VelloPass,
};
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

struct App {
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

    fn render_frame(&mut self) {
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

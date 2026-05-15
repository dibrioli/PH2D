#![forbid(unsafe_code)]
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

mod cursor_pos;
mod hero_bridge;
mod integration;
mod keymap;
mod theme;

use cursor_pos::live_cursor_in_window;
use keymap::winit_to_editor_keycode;
use theme::parse_theme_env;

use bumpalo::Bump;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::{FixedStep, Vec2, install_panic_hook, panic};
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommand, EditorCommandQueue, HierarchySnapshot, HierarchyWalkState,
    apply_editor_commands, build_hierarchy_snapshot, register_ecs_components, stable_type_id,
};
use ph2d_ecs::{
    Component, Name, PresentWorld, SimComponent, SimRef, SimWorld, Transform,
    TransformPropagationState, Visibility, WorklistBuf, propagate_transforms,
};
use ph2d_editor::paint::{Paint, PaintCtx};
use ph2d_editor::zones::Rect as EditorRect;
use ph2d_editor::{
    BrushTool, HeroScreen, Layout as EditorLayout, MoveTool, PanelControl, PanelEvent,
    RequestedSpriteStrategy, Toast, ToastQueue, ToolRegistry, WidgetEvent, ZenMode,
    paint_hero_screen,
};
use std::collections::BTreeMap;
// NodeId surfaces in our `dragging` field; re-exported by ph2d-editor.
use ph2d_editor::NodeId;
use ph2d_gpu::{AcquireError, GpuContext, SurfaceContext};
use ph2d_host::{
    CloseAction, HostHandler, KeyEvent, KeyKind, Lifecycle, Modifiers, PlatformHost, PointerEvent,
    PointerKind, PointerSource, WindowSize,
};
use ph2d_input::{Event as InputEvent, InputState};
use ph2d_render::{
    Camera2d, Compositor, GameRt, RenderInstance, Sprite, SpriteRenderer, TextureAtlas, Tonemap,
    VelloPass,
};
use ph2d_script::ScriptHost;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Color as VelloColor, VectorScene};
use std::cell::Cell;
use std::sync::Arc;
use std::time::Instant;

mod gilrs_adapter;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent as WinitKeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Window, WindowId};

const SPRITE_COUNT: u32 = 1000;
/// Half-extent of the bouncing world in meters. Camera default has
/// `height_world = 10`, so [-5, 5] in Y is exactly the visible region;
/// X depends on aspect (narrower than visible at 4:3+).
const WORLD_HALF: f32 = 5.0;

// ADR-0025: `Position(Vec2)` removed — `Transform` from `ph2d_ecs` is
// the canonical pose component now. `Velocity` stays local to the
// demo because no other crate consumes it yet (M14.2+ may promote it).
#[derive(Component, Copy, Clone, Debug)]
struct Velocity(Vec2);
impl SimComponent for Velocity {}

struct WinitHost {
    window: Arc<Window>,
    scale: Cell<f32>,
}

impl WinitHost {
    fn new(window: Arc<Window>) -> Self {
        let scale = window.scale_factor() as f32;
        Self {
            window,
            scale: Cell::new(scale),
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }
}

impl PlatformHost for WinitHost {
    fn request_redraw(&self) {
        self.window.request_redraw();
    }
    fn window_size(&self) -> WindowSize {
        let size = self.window.inner_size();
        WindowSize::new(size.width, size.height)
    }
    fn scale_factor(&self) -> f32 {
        self.scale.get()
    }
}

struct LoggingHandler {
    started_at: Instant,
}
impl LoggingHandler {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
    fn elapsed_ms(&self) -> u128 {
        self.started_at.elapsed().as_millis()
    }
}
impl HostHandler for LoggingHandler {
    fn on_resize(&mut self, size: WindowSize, scale_factor: f32) {
        println!(
            "[{:>6}ms] resize: {}x{} @ {:.2}x scale",
            self.elapsed_ms(),
            size.width,
            size.height,
            scale_factor
        );
    }
    fn on_lifecycle(&mut self, kind: Lifecycle) {
        println!("[{:>6}ms] lifecycle: {:?}", self.elapsed_ms(), kind);
    }
    fn on_pointer(&mut self, event: PointerEvent) {
        if matches!(event.kind, PointerKind::Down | PointerKind::Up) {
            println!(
                "[{:>6}ms] pointer {:?} {:?} ({:.0}, {:.0}) p={:.2}",
                self.elapsed_ms(),
                event.source,
                event.kind,
                event.x,
                event.y,
                event.pressure
            );
        }
    }
    fn on_key(&mut self, event: KeyEvent) {
        println!(
            "[{:>6}ms] key {:?} keycode={} mods={:?}",
            self.elapsed_ms(),
            event.kind,
            event.keycode,
            event.modifiers
        );
    }
    fn on_close_request(&mut self) -> CloseAction {
        println!("[{:>6}ms] close requested → Close", self.elapsed_ms());
        CloseAction::Close
    }
}

/// Holds every initialized-after-`resumed` resource. Bundling them into
/// a single `Option<AppGfx>` lets us destructure into per-field `&mut`
/// borrows in `render_frame()` — split-borrowing through a method
/// chain on individual `Option<...>` fields would be awkward.
struct AppGfx {
    surface: SurfaceContext,
    renderer: SpriteRenderer,
    sim: SimWorld,
    present: PresentWorld,
    camera: Camera2d,
    /// M6 — set when PNG fixtures loaded successfully; held so the
    /// AssetDb keeps `Arc<Asset>` alive for hot-reload follow-ups.
    asset_db: AssetDb,
    /// M6 — true when the atlas was composed from real PNG files (vs the
    /// procedural dummy fallback). Surfaced in window title.
    atlas_is_real: bool,
    /// M7 — Luau VM with placeholder script loaded. Per-frame gc_step
    /// keeps the GC budget warm; set/get bindings ready for follow-up
    /// gameplay work.
    script: Option<ScriptHost>,
    /// M12 editor data layer + M11 widget paint pass.
    theme: Theme,
    zen: ZenMode,
    toasts: ToastQueue,
    /// Registered editor tools. Keys 1/2 switch active tool; the
    /// active tool's `build_panel()` is painted each frame as the
    /// FloatingPanel that shows in the bottom-center of the canvas.
    tools: ToolRegistry,
    /// 4-zone editor layout (ADR-0023 §3). Sized from window each
    /// resize; the M11 paint pass walks this to draw zone backdrops.
    layout: EditorLayout,
    /// M14.5: offscreen HDR (Rgba16Float) render target for the game
    /// world. Sprite + future light/particle/material passes write
    /// here; the tonemap pass reads here and writes to LDR. Recreated
    /// on resize.
    game_rt: GameRt,
    /// M14.5: AgX tonemap pass — owns its own LDR output texture
    /// (`game_rt_ldr`). Sampled by the compositor as the "game layer"
    /// input. Identity LUT by default; swap in real AgX via
    /// `set_lut`.
    tonemap: Tonemap,
    /// M14.5: compositor that composes `tonemap.output_view()` (game)
    /// and `vello_pass.intermediate_view()` (UI chrome) onto the swap
    /// chain. Replaces the old `vello_pass.blitter` direct-to-surface
    /// blit so chrome and game live in fully isolated RTs.
    compositor: Compositor,
    /// Vello pipeline + intermediate texture for the widget paint
    /// pass. In M14.5 the intermediate is sampled by `compositor`
    /// (not blitted directly to the surface).
    vello_pass: VelloPass,
    /// Reused [`VectorScene`] — encoded fresh each frame; allocations
    /// pool inside Vello so this is cheap.
    vector_scene: VectorScene,
    /// parley font + layout context (heavy state). Threaded through
    /// `PaintCtx` so future text passes don't re-load fonts.
    text_system: TextSystem,
    /// Hero screen (`02-editor-main` mockup) — populated by default;
    /// `None` only when `PH2D_M5_DEMO=1` selects the legacy
    /// 1000-sprite perf demo. Owns the [`WidgetStore`] + [`HitIndex`]
    /// so input pipeline (ADR-0024) can route pointer/key events
    /// through `dispatch_*`.
    hero_screen: Option<HeroScreen>,
    /// Per-frame arena for [`WidgetEvent`]s emitted by the hero
    /// dispatcher. Reset at end-of-frame.
    hero_arena: Bump,
    /// OS clipboard handle — used by Cmd+C/V/X. `None` when the OS
    /// rejected our request (rare; we just no-op those keys then).
    clipboard: Option<arboard::Clipboard>,
    /// M14.1 — cached `QueryState` pair for hierarchical transform
    /// propagation. Built once after `populate_sim`; used every frame
    /// inside the extract phase. The only way to iterate `&World`
    /// from inside `extract!`.
    prop_state: TransformPropagationState,
    /// M14.1 — pre-allocated DFS worklist for `propagate_transforms`.
    /// Capacity sized to `WorklistBuf::DEFAULT_CAPACITY` (8 192
    /// entities) — comfortably above `SPRITE_COUNT = 1000`. HR-3
    /// zero-alloc verified by `crates/ph2d-ecs/tests/propagate_no_alloc.rs`.
    worklist: WorklistBuf,
    /// M14.4a live-bridge state. Present in the default editor mode
    /// (i.e. always unless `PH2D_M5_DEMO=1` switched to the legacy
    /// untouched.
    hero_live: Option<HeroLive>,
    /// M14.4c+M14.4d: next free atlas key for imported images.
    /// Starts at `FIRST_IMPORT_KEY` (= 16) so it sits past the
    /// demo's seeded HSV tile keys (0..15). With the Skyline atlas
    /// the key space is effectively unbounded (the underlying packer
    /// runs out of pixel space before `u32` does), so we just
    /// increment monotonically — no cycling, no overwrite. When
    /// the atlas does run out of room the regrow path
    /// (`insert_atlas_sprite_with_regrow`) uses `atlas_asset_map` to
    /// recover each existing region's source bytes from `asset_db`.
    next_import_cell: u32,
    /// M14.7 polish: atlas-key → AssetId map kept in sync with each
    /// import. Drives the regrow callback so doubling the atlas
    /// texture preserves every previously-imported sprite. BTreeMap
    /// per HR-5 / ADR-0022.
    atlas_asset_map: BTreeMap<u32, AssetId>,
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
    component_registry: ComponentRegistry,
    /// Editor command queue (Arc<Mutex<…>>-backed for multi-producer
    /// access). The Inspector's commit path is the only producer
    /// today; the shell drains and applies once per frame after the
    /// hero `apply_event` pass.
    editor_queue: EditorCommandQueue,
    /// Cached stable type id for `ph2d::ecs::Transform`. Lookup is
    /// blake3-of-name → first 8 bytes; cached so the per-commit push
    /// path doesn't re-hash. The matching registry entry was added
    /// via `register_ecs_components`.
    transform_type_id: u64,
    /// M14.D: same as `transform_type_id` for the `ph2d::ecs::Visibility`
    /// component. Cached at boot so the Inspector visibility checkbox
    /// commit doesn't re-hash on every toggle.
    visibility_type_id: u64,
    /// M14.E: same cache for `ph2d::ecs::Name`. Used when draining
    /// `pending_name_edit` from the Inspector's editable name field.
    name_type_id: u64,
    /// M14.C audit fix #8: cache for `ph2d::render::Sprite` so the
    /// Strategy switch commit goes through the canonical
    /// `EditorCommand::SetComponent` pipeline (parity with Transform /
    /// Visibility / Name). Loaded into the registry via
    /// `register_render_components` at boot.
    sprite_type_id: u64,
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
        let drop_world = gfx.camera.screen_to_world(cursor_px, win);
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
            KeyCode::Digit1 if gfx.tools.set_active(&ph2d_editor::ToolId::new("brush")) => {
                gfx.toasts.push(Toast::info("Tool → Brush"));
                self.title_dirty = true;
            }
            KeyCode::Digit2 if gfx.tools.set_active(&ph2d_editor::ToolId::new("move")) => {
                gfx.toasts.push(Toast::info("Tool → Move"));
                self.title_dirty = true;
            }
            // M14.7 polish: F / Home = frame the currently selected
            // sprite. Falls back to (0, 0) when nothing is selected
            // (Blender / Maya "frame view" semantics). Raises a
            // pending intent on the hero — the render_frame drain
            // resolves the selection and updates `gfx.camera`.
            KeyCode::Home | KeyCode::KeyF => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.pending_view_focus = Some(ph2d_editor::ViewFocusKind::Selected);
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

    /// Compose the M6 atlas from real PNG files. Generates fixtures on first
    /// launch; subsequent launches reuse the on-disk files. Any failure
    /// bubbles a String — caller falls back to the dummy atlas.
    fn try_load_real_atlas(
        gpu: &GpuContext,
        asset_db: &AssetDb,
        dir: &std::path::Path,
    ) -> Result<TextureAtlas, String> {
        let created = integration::ensure_demo_assets_exist(dir)
            .map_err(|e| format!("ensure_demo_assets_exist({}): {e}", dir.display()))?;
        if created > 0 {
            println!(
                "M6: generated {created} demo PNG fixtures in {}",
                dir.display()
            );
        }
        let ids = integration::load_demo_assets(asset_db, dir)?;
        let composed = integration::compose_atlas_rgba(asset_db, &ids)?;
        // M14.4d retrofit: the demo atlas used to be a single 256×256
        // texture mirroring the `compose_atlas_rgba` layout. With the
        // Skyline packer it's a dynamic atlas seeded by 16 inserts of
        // the per-tile slices — keys 0..16 reproduce the same
        // (col, row) ordering the dummy HSV path uses, so the rest of
        // the demo (which addresses tiles by `i % 16`) needs no
        // change.
        let mut atlas = TextureAtlas::new(gpu, ph2d_render::ATLAS_DEFAULT_SIZE_PX);
        let tile_px = ph2d_render::DEMO_TILE_PX;
        let composed_px = integration::ATLAS_PX;
        for i in 0..ph2d_render::DEMO_TILE_COUNT {
            let col = i % integration::ATLAS_GRID;
            let row = i / integration::ATLAS_GRID;
            let mut tile = Vec::with_capacity((tile_px * tile_px * 4) as usize);
            for ty in 0..tile_px {
                let src_y = row * tile_px + ty;
                let src_x = col * tile_px;
                let row_start = ((src_y * composed_px + src_x) * 4) as usize;
                let row_end = row_start + (tile_px * 4) as usize;
                tile.extend_from_slice(&composed[row_start..row_end]);
            }
            atlas
                .insert(gpu, i, tile_px, tile_px, &tile)
                .map_err(|e| format!("demo tile {i}: {e}"))?;
        }
        Ok(atlas)
    }

    /// Spawn `SPRITE_COUNT` sprites on a Vogel (golden-angle) spiral
    /// with pseudo-random velocities derived from index — fully
    /// deterministic, no PRNG dep.
    fn populate_sim(sim: &mut SimWorld) {
        for i in 0..SPRITE_COUNT {
            let f = i as f32;
            let angle = f * 2.399_963_2; // golden angle (rad)
            let r = (f / SPRITE_COUNT as f32).sqrt() * (WORLD_HALF - 0.5);
            let pos = Vec2::new(r * angle.cos(), r * angle.sin());
            // Velocity in m/s; both axes seeded by independent index hashes
            // so motion isn't correlated with the spiral pattern.
            let vx = ((f * 12.9898).sin() * 43758.547).fract() * 3.0 - 1.5;
            let vy = ((f * 78.233).sin() * 12345.678).fract() * 3.0 - 1.5;
            sim.world_mut().spawn((
                Transform::from_translation(pos),
                Velocity(Vec2::new(vx, vy)),
                Sprite::atlas(i % 16, [0.18, 0.18], [1.0, 1.0, 1.0, 1.0]),
            ));
        }
    }

    /// Spawn 8 named entities in a depth-2 tree — used in the default
    /// editor mode. `Name` components let the hierarchy panel display
    /// readable rows ("sprite_001" through "sprite_008") instead of a
    /// wall of `Entity_…` hex strings.
    fn populate_sim_live(sim: &mut SimWorld) {
        // 2 roots × 3 children = 8 entities arranged as a depth-2
        // tree. Lets the user exercise the M14.6C hierarchy
        // expand/collapse chevron (which only appears on parents)
        // and M14.6A hide/show propagation as soon as the demo
        // boots — without that, every row is a root and there's
        // nothing to collapse. Layout: roots on Y=+0.5/-0.5 with
        // their 3 children fanned out on the same row, 0.5 m apart.
        let mut sprite_idx = 0u32;
        let world = sim.world_mut();
        for group in 0..2 {
            let y_root = if group == 0 { 0.5 } else { -0.5 };
            let root_entity = world
                .spawn((
                    Transform::from_translation(Vec2::new(-1.5, y_root)),
                    Sprite::atlas(sprite_idx % 16, [0.4, 0.4], [1.0, 1.0, 1.0, 1.0]),
                    Name::new(format!("group_{:02}", group + 1)),
                ))
                .id();
            sprite_idx += 1;
            for child in 0..3 {
                let f = child as f32;
                let vx = ((sprite_idx as f32 * 12.9898).sin() * 43758.547).fract() * 1.0 - 0.5;
                world.spawn((
                    Transform::from_translation(Vec2::new(-0.5 + f, y_root)),
                    Velocity(Vec2::new(vx, 0.0)),
                    Sprite::atlas(sprite_idx % 16, [0.4, 0.4], [1.0, 1.0, 1.0, 1.0]),
                    Name::new(format!("sprite_{:03}", sprite_idx)),
                    ph2d_ecs::ChildOf(root_entity),
                ));
                sprite_idx += 1;
            }
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
            if let Some(kind) = hero.pending_view_focus.take() {
                let label = match kind {
                    ph2d_editor::ViewFocusKind::Selected => {
                        let target = hero.gizmo_selection.and_then(|bits| {
                            ph2d_render::selection_bbox_world(present.world_mut(), bits)
                        });
                        if let Some(bbox) = target {
                            let ([cx, cy], _) = bbox.center_half();
                            camera.center = [cx, cy];
                            "View → Selected"
                        } else {
                            camera.center = [0.0, 0.0];
                            "View → Selected (no selection → origin)"
                        }
                    }
                    ph2d_editor::ViewFocusKind::Camera => {
                        // No camera-object yet — frame the origin.
                        camera.center = [0.0, 0.0];
                        "View → Camera (origin)"
                    }
                    ph2d_editor::ViewFocusKind::All => {
                        // Walk PresentWorld for every sprite's bbox
                        // and fit camera around the union. 10% pad
                        // so handles + the bbox stroke have room.
                        let mut q = present
                            .world_mut()
                            .query::<(&ph2d_ecs::GlobalTransform, &ph2d_render::RenderInstance)>();
                        let mut min_x = f32::INFINITY;
                        let mut min_y = f32::INFINITY;
                        let mut max_x = f32::NEG_INFINITY;
                        let mut max_y = f32::NEG_INFINITY;
                        let mut count = 0u32;
                        for (gt, ri) in q.iter(present.world()) {
                            let p = gt.translation();
                            let hw = ri.size[0] * 0.5;
                            let hh = ri.size[1] * 0.5;
                            min_x = min_x.min(p.x - hw);
                            min_y = min_y.min(p.y - hh);
                            max_x = max_x.max(p.x + hw);
                            max_y = max_y.max(p.y + hh);
                            count += 1;
                        }
                        if count > 0 {
                            let cx = (min_x + max_x) * 0.5;
                            let cy = (min_y + max_y) * 0.5;
                            let span_x = max_x - min_x;
                            let span_y = max_y - min_y;
                            // Height needed = max of (span_y,
                            // span_x/aspect) × 1.1 for padding.
                            let aspect =
                                (window_size.width as f32) / (window_size.height.max(1) as f32);
                            let need_h = span_y.max(span_x / aspect.max(1e-3));
                            camera.center = [cx, cy];
                            camera.height_world = (need_h * 1.1).max(0.5);
                            "View → All"
                        } else {
                            *camera = Camera2d::default();
                            "View → All (empty scene → reset)"
                        }
                    }
                };
                toasts.push(Toast::info(label));
                self.title_dirty = true;
            }
            // M14.6A: drain pending hierarchy visibility toggle —
            // resolve row NodeId → ECS Entity via the bridge, flip
            // the `Visibility` component on SimWorld.
            if let Some(row_id) = hero.pending_visibility_toggle.take()
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
            if let Some(intent) = hero.pending_reparent.take()
                && let Some(live) = hero_live.as_ref()
                && let Some(dragged_bits) = live.bridge.entity_for(intent.dragged)
            {
                let dragged = ph2d_ecs::Entity::from_bits(dragged_bits);
                // M14.7 polish (14.3 fix): the dispatcher computes
                // `new_parent` via `store.hierarchy_parent_of(target)`,
                // which only knows about fixture-mode DnD mutations.
                // In LIVE mode the parent comes from the ECS snapshot,
                // so "Before(t)" arrives with `new_parent = None` even
                // when t is a child row — drops then turned into root
                // promotions. Fallback: when the intent has a `before`
                // target but no resolved parent, inherit the target's
                // actual `ChildOf` parent from SimWorld.
                let new_parent_entity = if let Some(parent_node) = intent.new_parent
                    && let Some(parent_bits) = live.bridge.entity_for(parent_node)
                {
                    Some(ph2d_ecs::Entity::from_bits(parent_bits))
                } else if let Some(before_node) = intent.before
                    && let Some(target_bits) = live.bridge.entity_for(before_node)
                {
                    let target = ph2d_ecs::Entity::from_bits(target_bits);
                    sim.world()
                        .get::<ph2d_ecs::ChildOf>(target)
                        .map(|c| c.parent())
                } else if let Some(after_node) = intent.after
                    && let Some(target_bits) = live.bridge.entity_for(after_node)
                {
                    // "After t" inherits t's parent (or root when t
                    // itself is root).
                    let target = ph2d_ecs::Entity::from_bits(target_bits);
                    sim.world()
                        .get::<ph2d_ecs::ChildOf>(target)
                        .map(|c| c.parent())
                } else {
                    None
                };
                let sim_w = sim.world_mut();
                // Cycle guard: refuse to make dragged a child of
                // itself or any of its descendants. Walk up
                // new_parent's ancestors looking for `dragged`; if
                // found, skip the mutation. Without this, a user
                // dropping a parent inside one of its own children
                // would corrupt the hierarchy (infinite loop in any
                // ancestor walk).
                let would_cycle = new_parent_entity.is_some_and(|np| {
                    let mut current = Some(np);
                    while let Some(c) = current {
                        if c == dragged {
                            return true;
                        }
                        current = sim_w.get::<ph2d_ecs::ChildOf>(c).map(|c| c.parent());
                    }
                    false
                });
                if !would_cycle {
                    // Step 1: pick the new ChildOf relation. We need
                    // this BEFORE rebuilding the child order so
                    // step 2 sees the up-to-date Children list.
                    if let Ok(mut entry) = sim_w.get_entity_mut(dragged) {
                        match new_parent_entity {
                            Some(p) => {
                                entry.insert(ph2d_ecs::ChildOf(p));
                            }
                            None => {
                                entry.remove::<ph2d_ecs::ChildOf>();
                            }
                        }
                    }
                    // M14.7 polish: root drops need an explicit
                    // `RootOrder` so `build_hierarchy_snapshot` sorts
                    // them at the user-chosen slot instead of falling
                    // back to `Entity::to_bits()`. Walk the current
                    // root list with the dragged entity inserted at
                    // the desired position (before / after a target,
                    // or at the end), then assign sequential indices.
                    // Non-root drops skip this — child order is
                    // handled by step 2's ChildOf re-insert pass.
                    if new_parent_entity.is_none() {
                        let mut roots: Vec<ph2d_ecs::Entity> = {
                            let mut q = sim_w.query_filtered::<ph2d_ecs::Entity, (
                                ph2d_ecs::With<Transform>,
                                ph2d_ecs::Without<ph2d_ecs::ChildOf>,
                            )>();
                            let mut acc: Vec<(ph2d_ecs::Entity, u32)> = Vec::new();
                            for entity in q.iter(sim_w) {
                                if entity == dragged {
                                    continue;
                                }
                                let order = sim_w
                                    .get::<ph2d_ecs::RootOrder>(entity)
                                    .map(|r| r.0)
                                    .unwrap_or(u32::MAX);
                                acc.push((entity, order));
                            }
                            acc.sort_unstable_by(|a, b| {
                                a.1.cmp(&b.1)
                                    .then_with(|| a.0.to_bits().cmp(&b.0.to_bits()))
                            });
                            acc.into_iter().map(|(e, _)| e).collect()
                        };
                        let before_target = intent
                            .before
                            .and_then(|n| live.bridge.entity_for(n))
                            .map(ph2d_ecs::Entity::from_bits);
                        let after_target = intent
                            .after
                            .and_then(|n| live.bridge.entity_for(n))
                            .map(ph2d_ecs::Entity::from_bits);
                        let insert_at = if let Some(b) = before_target {
                            roots.iter().position(|e| *e == b).unwrap_or(roots.len())
                        } else if let Some(a) = after_target {
                            roots
                                .iter()
                                .position(|e| *e == a)
                                .map(|i| i + 1)
                                .unwrap_or(roots.len())
                        } else {
                            roots.len()
                        };
                        roots.insert(insert_at.min(roots.len()), dragged);
                        for (idx, e) in roots.iter().enumerate() {
                            if let Ok(mut entry) = sim_w.get_entity_mut(*e) {
                                entry.insert(ph2d_ecs::RootOrder(idx as u32));
                            }
                        }
                    }
                    // Step 2: enforce sibling order. When dropping
                    // "before t" (intent.before = Some(target)), the
                    // dragged entity should sit RIGHT BEFORE target
                    // in the parent's `Children`. We walk the parent's
                    // current children, build the desired order by
                    // inserting `dragged` at the slot just ahead of
                    // `target`, then re-apply ChildOf to each child in
                    // sequence — every fresh `insert(ChildOf)` pushes
                    // the entity to the END of Children, so the loop
                    // ends with the children laid out in our intended
                    // order. Skipped when:
                    //   - new_parent is None (root drop; root order is
                    //     decided by `Entity::to_bits` sorting in
                    //     `build_hierarchy_snapshot`, not by us).
                    //   - intent.before is None ("append at end" is
                    //     what the bare insert already did).
                    // Pick the sibling pivot for the reorder:
                    //   - Before(t): insert dragged just before t
                    //   - After(t):  insert dragged just after t
                    // Either resolves to a target NodeId in
                    // `intent.before` / `intent.after`; from there we
                    // map to the ECS entity and pick the insert side.
                    let target_kind: Option<(ph2d_ecs::Entity, bool /* place_before */)> =
                        if let Some(before_node) = intent.before
                            && let Some(b) = live.bridge.entity_for(before_node)
                        {
                            Some((ph2d_ecs::Entity::from_bits(b), true))
                        } else if let Some(after_node) = intent.after
                            && let Some(a) = live.bridge.entity_for(after_node)
                        {
                            Some((ph2d_ecs::Entity::from_bits(a), false))
                        } else {
                            None
                        };
                    if let (Some(parent), Some((target, place_before))) =
                        (new_parent_entity, target_kind)
                    {
                        // Snapshot current Children (excluding the
                        // dragged entity — we'll re-insert it at the
                        // chosen slot). `Children::iter` returns
                        // entities in insertion order.
                        let current: Vec<ph2d_ecs::Entity> = sim_w
                            .get::<bevy_ecs::hierarchy::Children>(parent)
                            .map(|c| c.iter().copied().filter(|e| *e != dragged).collect())
                            .unwrap_or_default();
                        // Build new desired order:
                        //   place_before = true  → [..target) + dragged + [target..end)
                        //   place_before = false → [..target] + dragged + [target+1..end)
                        let mut desired: Vec<ph2d_ecs::Entity> =
                            Vec::with_capacity(current.len() + 1);
                        let mut inserted = false;
                        for &c in &current {
                            if !inserted && c == target && place_before {
                                desired.push(dragged);
                                inserted = true;
                            }
                            desired.push(c);
                            if !inserted && c == target && !place_before {
                                desired.push(dragged);
                                inserted = true;
                            }
                        }
                        if !inserted {
                            // Target wasn't actually a child of parent
                            // (concurrent mutation or stale snapshot)
                            // → append as fallback so we don't lose
                            // the dragged entity from the list.
                            desired.push(dragged);
                        }
                        // Re-insert ChildOf for each entity in order.
                        // Every insert moves the entity to the END of
                        // `Children`, so iterating `desired` in
                        // sequence rebuilds the list in that exact
                        // order.
                        for &child in &desired {
                            if let Ok(mut entry) = sim_w.get_entity_mut(child) {
                                entry.remove::<ph2d_ecs::ChildOf>();
                                entry.insert(ph2d_ecs::ChildOf(parent));
                            }
                        }
                    }
                }
            }
            // M14.6 F: drain per-row Hierarchy context-menu actions.
            // Each is a `Some(row_id)` — bridge resolves to Entity,
            // then we apply the corresponding ECS mutation. Order is
            // intentional: Delete last, so a (degenerate) frame that
            // queues "duplicate then delete" leaves the duplicate in
            // place and removes the original. The next snapshot rebuild
            // picks up the result automatically.
            if let Some(row) = hero.pending_duplicate.take()
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
            if let Some(row) = hero.pending_add_child.take()
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
            if let Some(row) = hero.pending_reset_transform.take()
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
            if let Some(row) = hero.pending_delete.take()
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
            if let Some(row) = hero.pending_hierarchy_row_click.take()
                && let Some(live) = hero_live.as_ref()
                && let Some(entity_bits) = live.bridge.entity_for(row)
            {
                hero.gizmo_selection = Some(entity_bits);
            }
            // M14.7 polish: one-shot seed of the rename TextInput
            // when rename mode opens. `pending_rename_seed` is set by
            // hero on the open path (right-click Rename / long-press)
            // and taken here exactly once — so subsequent Backspace
            // edits that empty the buffer don't get clobbered back
            // to the original name on the next frame.
            if let Some(row) = hero.pending_rename_seed.take()
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
            if let Some((row, new_name)) = hero.pending_rename_commit.take()
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
            // M14.5 inspector phase (6.4): drain pending reimport →
            // re-decode the atlas source's pixel dimensions at the
            // current `project.pixels_per_meter` and write the new
            // world size back to the Sprite component. The texture
            // itself is unchanged; only `Sprite.size` is recomputed.
            if let Some(entity_bits) = hero.pending_reimport.take() {
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
            if let Some(info) = hero.pending_transform_edit.take() {
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
            if let Some(info) = hero.pending_visibility_edit.take() {
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
            // and apply. Coalesced via the Option: even if the user
            // types fast, we drain once per frame with the latest text.
            if let Some(info) = hero.pending_name_edit.take() {
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
            if let Some((entity_bits, requested)) = hero.pending_sprite_source_change.take() {
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
                            *state = ph2d_editor::ButtonState::Normal;
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
            // `ph2d_editor::recenter_after_crop` so the *visual* center
            // of the surviving opaque content stays put even when it
            // lived off-center inside the original frame. The shift
            // happens in pure-CPU pixel math (Y-flip handled inside
            // `recenter_after_crop`); HR-5-deterministic.
            if let Some(entity_bits) = hero.pending_trim_transparency.take() {
                let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                let px_per_m = hero.project.pixels_per_meter.max(EPS_PIXELS_PER_METER);
                // Snapshot `Sprite` (for size + source) and `Transform`
                // (for translation) in one read pass so we can drop the
                // immutable borrow before the mutable `world_mut()` later.
                // Individual-source sprites go through a GPU readback —
                // safe here because Trim is a one-shot user action
                // (HR-3 only applies inside render/physics/audio hot
                // paths, not user-initiated edit actions).
                let snapshot = {
                    let world = sim.world();
                    world.get::<Sprite>(entity).and_then(|sprite| {
                        let old_size_world = sprite.size;
                        let old_translation = world
                            .get::<ph2d_ecs::Transform>(entity)
                            .map(|t| [t.translation.x, t.translation.y])
                            .unwrap_or([0.0, 0.0]);
                        match sprite.source {
                            ph2d_render::SpriteSource::Atlas { key } => {
                                let aid = atlas_asset_map.get(&key)?;
                                let asset = asset_db.get(aid)?;
                                match &*asset {
                                    ph2d_asset::Asset::ImageRgba8 {
                                        width,
                                        height,
                                        pixels,
                                    } => Some((
                                        *width,
                                        *height,
                                        pixels.clone(),
                                        old_size_world,
                                        old_translation,
                                    )),
                                    _ => None,
                                }
                            }
                            ph2d_render::SpriteSource::Individual { texture_id } => {
                                match renderer.readback_individual(texture_id) {
                                    Ok((w, h, pixels)) => {
                                        Some((w, h, pixels.into(), old_size_world, old_translation))
                                    }
                                    Err(_) => None,
                                }
                            }
                        }
                    })
                };
                match snapshot {
                    None => {
                        toasts.push(Toast::error("Trim unavailable for this sprite"));
                        self.title_dirty = true;
                    }
                    Some((width, height, pixels, old_size_world, old_translation)) => {
                        let result = ph2d_editor::trim_transparency(&pixels, width, height, 0);
                        if !result.trimmed {
                            toasts.push(Toast::info("Nothing to trim"));
                            self.title_dirty = true;
                        } else {
                            match renderer.acquire_individual(
                                result.width,
                                result.height,
                                &result.pixels,
                            ) {
                                Err(err) => {
                                    toasts.push(Toast::error(format!("Trim failed: {err}")));
                                    self.title_dirty = true;
                                }
                                Ok(texture_id) => {
                                    let new_size = [
                                        result.width as f32 / px_per_m,
                                        result.height as f32 / px_per_m,
                                    ];
                                    // Pivot-preserving recenter (F1).
                                    // Shifts the entity's translation so
                                    // the cropped sprite's new center
                                    // aligns with where the surviving
                                    // content visually sat.
                                    let new_translation = ph2d_editor::recenter_after_crop(
                                        old_translation,
                                        old_size_world,
                                        [width, height],
                                        ph2d_editor::PixelBounds::from_trim(result.bounds.clone()),
                                    );
                                    let sim_w = sim.world_mut();
                                    if let Some(mut sprite) = sim_w.get_mut::<Sprite>(entity) {
                                        sprite.source =
                                            ph2d_render::SpriteSource::Individual { texture_id };
                                        sprite.size = new_size;
                                    }
                                    if let Some(mut transform) =
                                        sim_w.get_mut::<ph2d_ecs::Transform>(entity)
                                    {
                                        transform.translation.x = new_translation[0];
                                        transform.translation.y = new_translation[1];
                                    }
                                    toasts.push(Toast::success(format!(
                                        "Trimmed → {} × {} px",
                                        result.width, result.height
                                    )));
                                    self.title_dirty = true;
                                }
                            }
                        }
                    }
                }
            }
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
        let attrs = Window::default_attributes()
            .with_title("PH2D — editor")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("create_window must succeed"),
        );
        // Enable IME so dead-key + composition sequences (PT-BR
        // accents `á`, `ç`, `ñ`, …) reach us via `WindowEvent::Ime`.
        // Without this the macOS text-input service swallows the
        // dead-key keystroke and `KeyEvent::text` arrives empty.
        window.set_ime_allowed(true);
        let host = WinitHost::new(window.clone());
        let size = host.window_size();
        let scale = host.scale_factor();

        let instance = GpuContext::default_instance();
        let raw_surface = instance
            .create_surface(window.clone())
            .expect("create_surface");
        let gpu = GpuContext::new(instance, Some(&raw_surface)).expect("GpuContext::new");
        let surface = SurfaceContext::new(gpu, raw_surface, size).expect("SurfaceContext::new");

        // M6: try to compose the atlas from real PNG files on disk.
        // Auto-generates 16 procedural fixtures on first launch so the
        // demo is self-contained (no committed binary fixtures). Any
        // failure logs and falls back to the M5 procedural dummy —
        // the shell must boot regardless of asset-pipeline issues.
        let asset_db = AssetDb::new();
        let assets_dir = integration::demo_assets_dir();
        let (atlas, atlas_is_real) =
            match Self::try_load_real_atlas(surface.gpu(), &asset_db, &assets_dir) {
                Ok(atlas) => {
                    println!(
                        "[{:>6}ms] M6: real atlas composed from {} ({} assets cached)",
                        self.handler.elapsed_ms(),
                        assets_dir.display(),
                        asset_db.len_assets()
                    );
                    (atlas, true)
                }
                Err(e) => {
                    eprintln!(
                        "[{:>6}ms] M6 fallback to dummy atlas: {e}",
                        self.handler.elapsed_ms()
                    );
                    (TextureAtlas::dummy(surface.gpu()), false)
                }
            };
        // M14.5: sprite pipeline now targets the offscreen HDR game RT
        // (Rgba16Float) instead of the swap chain. The tonemap +
        // compositor passes carry pixels through to the surface.
        let renderer = SpriteRenderer::new(
            surface.gpu().clone(),
            GameRt::FORMAT,
            atlas,
            SPRITE_COUNT.next_power_of_two(),
        );

        // Mode gate inverted 2026-05-14: hero live is the **default**
        // user-facing experience. `PH2D_M5_DEMO=1` opts into the legacy
        // M5 perf-validation demo (1000-sprite Vogel spiral, no editor
        // chrome) — kept reachable for HR-4 frame-budget validation,
        // 100k-sprite stress tests, and future bench work without
        // forcing users through it on first launch.
        let m5_demo_enabled = std::env::var("PH2D_M5_DEMO").as_deref() == Ok("1");
        let hero_live_enabled = !m5_demo_enabled;
        let mut sim = SimWorld::new();
        if hero_live_enabled {
            // M14.4a: spawn a small named set so the hierarchy
            // panel renders readable rows. The 1000-sprite Vogel
            // spiral demo is fixture-only; live mode is for the
            // editor's hierarchy/inspector pipeline.
            Self::populate_sim_live(&mut sim);
            println!(
                "[{:>6}ms] live hero mode (8 named entities; \
                 hierarchy panel binds to ECS)",
                self.handler.elapsed_ms()
            );
        } else {
            Self::populate_sim(&mut sim);
            println!(
                "[{:>6}ms] M5 demo mode (PH2D_M5_DEMO=1; 1000-sprite \
                 Vogel spiral, no editor chrome)",
                self.handler.elapsed_ms()
            );
        }
        let present = PresentWorld::new();
        // ADR-0025 M14.1: build the cached propagation queries AFTER
        // populate_sim so bevy_ecs has already seen the Transform
        // archetype. QueryState::new on `&mut World` is fine here
        // (one-shot at boot); inside the extract phase the queries
        // iterate via `&World` only.
        let prop_state = TransformPropagationState::new(sim.world_mut());
        let worklist = WorklistBuf::new();
        let hero_live = if hero_live_enabled {
            let walk_state = HierarchyWalkState::new(sim.world_mut());
            Some(HeroLive {
                bridge: hero_bridge::EntityNodeMap::new(),
                walk_state,
                walk_scratch: Vec::with_capacity(64),
                snapshot: HierarchySnapshot::new(),
            })
        } else {
            None
        };
        let camera = Camera2d::default();

        // M7: ScriptHost. Failure here is also non-fatal (script is
        // a placeholder; full sim-driving lands in M12+ editor panel).
        let script = match integration::init_script_host() {
            Ok(host) => {
                println!(
                    "[{:>6}ms] M7: ScriptHost initialized (placeholder script loaded)",
                    self.handler.elapsed_ms()
                );
                Some(host)
            }
            Err(e) => {
                eprintln!(
                    "[{:>6}ms] M7 ScriptHost failed: {e} — continuing without scripting",
                    self.handler.elapsed_ms()
                );
                None
            }
        };

        // M12 + M11: editor data layer + Vello widget paint pass.
        // ZenMode/ToastQueue/ToolRegistry model state, Layout computes
        // the 4 zones, VelloPass renders all widgets onto the surface
        // AFTER the sprite pass.
        let theme = parse_theme_env();
        eprintln!("[ph2d] theme = {}", theme.id());
        let zen = ZenMode::new();
        let mut toasts = ToastQueue::new();
        toasts.push(Toast::success("Editor data layer wired (M12)"));
        toasts.push(Toast::info("Press 1=Brush, 2=Move, Tab=Zen"));
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(BrushTool::default()));
        tools.register(Box::new(MoveTool::default()));
        let layout = EditorLayout::new(size.width as f32, size.height as f32);
        let vello_pass =
            match VelloPass::new(surface.gpu(), surface.format(), (size.width, size.height)) {
                Ok(p) => {
                    println!(
                        "[{:>6}ms] M11: VelloPass initialized ({}×{} intermediate)",
                        self.handler.elapsed_ms(),
                        size.width,
                        size.height
                    );
                    p
                }
                Err(e) => {
                    // Pass init failure is fatal here — the demo's whole
                    // point is showing the editor over the canvas.
                    panic!("VelloPass::new failed: {e}");
                }
            };

        // M14.5: viewport / RT pipeline construction. game_rt → tonemap
        // → compositor (which also reads vello_pass intermediate). The
        // sample views are extracted here at boot; rebound on resize
        // alongside game_rt/tonemap output recreation.
        let game_rt = GameRt::new(surface.gpu(), (size.width, size.height));
        let tonemap = Tonemap::new(
            surface.gpu(),
            game_rt
                .texture()
                .create_view(&wgpu::TextureViewDescriptor::default()),
            (size.width, size.height),
        );
        let compositor = Compositor::new(
            surface.gpu(),
            surface.format(),
            tonemap
                .output_texture()
                .create_view(&wgpu::TextureViewDescriptor::default()),
            vello_pass
                .intermediate_texture()
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        println!(
            "[{:>6}ms] M14.5: RT pipeline ready (game_rt Rgba16Float HDR + AgX tonemap + compositor)",
            self.handler.elapsed_ms()
        );
        let vector_scene = VectorScene::new();
        let text_system = TextSystem::new();

        // Hero screen (TopBar / LeftRail / Hierarchy / Inspector /
        // BottomHUD) is always-on in the default mode and disabled
        // in the M5 demo path. The legacy `PH2D_HERO_SCREEN=1` env
        // var is kept as a no-op alias — anyone with it in their
        // shell rc still gets the editor instead of an error.
        let hero_screen_enabled = hero_live_enabled;
        let hero_screen = if hero_screen_enabled {
            Some(HeroScreen::new(NodeId(1)).theme(theme))
        } else {
            None
        };

        self.window = Some(window);
        self.host = Some(host);
        self.gfx = Some(AppGfx {
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
            hero_arena: Bump::with_capacity(4096),
            clipboard: arboard::Clipboard::new()
                .map_err(|e| eprintln!("[ph2d] clipboard init failed: {e}"))
                .ok(),
            prop_state,
            worklist,
            hero_live,
            next_import_cell: ph2d_render::FIRST_IMPORT_KEY,
            atlas_asset_map: BTreeMap::new(),
            component_registry: {
                let mut reg = ComponentRegistry::new();
                register_ecs_components(&mut reg);
                // M14.C audit fix #8: register Sprite alongside the
                // ecs components so the Strategy switch can flow
                // through `EditorCommand::SetComponent` instead of
                // direct world mutation.
                ph2d_render::register_render_components(&mut reg);
                reg
            },
            editor_queue: EditorCommandQueue::new(),
            transform_type_id: stable_type_id("ph2d::ecs::Transform"),
            visibility_type_id: stable_type_id("ph2d::ecs::Visibility"),
            name_type_id: stable_type_id("ph2d::ecs::Name"),
            sprite_type_id: stable_type_id("ph2d::render::Sprite"),
        });
        self.handler.on_lifecycle(Lifecycle::Foreground);
        self.handler.on_resize(size, scale);
        self.title_dirty = true;
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => match self.handler.on_close_request() {
                CloseAction::Close => {
                    self.handler.on_lifecycle(Lifecycle::WillTerminate);
                    event_loop.exit();
                }
                CloseAction::Cancel => {}
            },

            WindowEvent::Resized(size) => {
                self.pending_resize = Some(WindowSize::new(size.width, size.height));
            }

            // M14.4e drag-and-drop. winit emits one HoveredFile per
            // path when multiple files are dragged together. We
            // buffer paths into `self.hovered_files` and push them to
            // the hero (for the overlay) on every HoveredFile event.
            WindowEvent::HoveredFile(path) => {
                self.hovered_files.push(path.clone());
                if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    hero.dragging_files = Some((self.hovered_files.clone(), self.last_cursor));
                }
                self.handler.on_file_hover(&self.hovered_files);
                if let Some(host) = self.host.as_ref() {
                    host.request_redraw();
                }
            }
            WindowEvent::HoveredFileCancelled => {
                self.hovered_files.clear();
                if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    hero.dragging_files = None;
                }
                self.handler.on_file_hover_cancel();
                if let Some(host) = self.host.as_ref() {
                    host.request_redraw();
                }
            }
            WindowEvent::DroppedFile(path) => {
                // M14.7 polish (7.3 fix): winit fires `DroppedFile`
                // once PER FILE on macOS but the events arrive across
                // multiple loop iterations. Importing inline on each
                // event was racy — some imports silently dropped when
                // an event came in mid-render. Buffer the path here;
                // `render_frame` drains `pending_drops` atomically.
                self.pending_drops.push(path);
                if let Some(host) = self.host.as_ref() {
                    host.request_redraw();
                }
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(host) = &self.host {
                    host.scale.set(scale_factor as f32);
                    if let Some(gfx) = self.gfx.as_ref() {
                        self.pending_resize = Some(gfx.surface.size());
                    }
                }
            }

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
                // M14.A: push the Shift state to the hero's WidgetStore
                // so `dispatch_pointer` Move can scale the NumberInput
                // drag delta correctly (Shift = fine adjustment). The
                // ph2d-host `PointerEvent` schema doesn't carry
                // modifiers natively — the store cache is the canonical
                // bridge for now.
                if let Some(gfx) = self.gfx.as_mut()
                    && let Some(hero) = gfx.hero_screen.as_mut()
                {
                    hero.store.set_shift_held(self.modifiers.shift_key());
                }
            }

            // IME composition commits — PT-BR / Spanish / French
            // accent dead-key sequences arrive here on macOS, NOT
            // in `KeyEvent::text` (the system text-input service
            // swallows the dead-key keystroke and emits the
            // composed char via `Ime::Commit`).
            WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                for ch in text.chars() {
                    if !ch.is_control() {
                        forward_text_to_hero(self.gfx.as_mut(), ch);
                    }
                }
                // `Preedit` (in-progress composition) is ignored for
                // now — no visible preedit caret yet. Future:
                // render the preedit text in italics at the caret.
            }

            WindowEvent::CursorMoved { position, .. } => {
                let prev = self.last_pointer;
                self.last_pointer = (position.x as f32, position.y as f32);
                // M14.4e: cache the latest cursor for DroppedFile —
                // winit's DroppedFile carries no position, so we
                // project the most-recently-seen cursor to world.
                self.last_cursor = self.last_pointer;
                // M14.4b.bis: middle-drag camera pan. Applied BEFORE
                // pointer forwarding so widgets receive the move
                // event but the camera also follows.
                if let Some(anchor) = self.pan_anchor
                    && let Some(gfx) = self.gfx.as_mut()
                {
                    let dx = self.last_pointer.0 - anchor.0;
                    let dy = self.last_pointer.1 - anchor.1;
                    let size = gfx.surface.size();
                    gfx.camera
                        .pan_screen_delta(dx, dy, size.width as f32, size.height as f32);
                    self.pan_anchor = Some(self.last_pointer);
                    let _ = prev; // silence unused warning when feature shifts
                }
                let evt = PointerEvent {
                    x: self.last_pointer.0,
                    y: self.last_pointer.1,
                    pressure: 1.0,
                    kind: PointerKind::Move,
                    source: PointerSource::Mouse,
                    button: ph2d_host::PointerButton::Primary,
                    timestamp_ns: Self::timestamp_ns(),
                };
                self.handler.on_pointer(evt);
                forward_to_hero(self.gfx.as_mut(), evt);
                // M14.7 C: advance the gizmo drag if one is open. We
                // update the cursor on the snapshot, derive the new
                // Transform via the pure math in `compute_gizmo_
                // transform`, and write it back to SimWorld. The next
                // frame's extract+paint mirror the change visually.
                if let Some(gfx) = self.gfx.as_mut()
                    && let Some(hero) = gfx.hero_screen.as_mut()
                    && let Some(mut drag) = hero.gizmo_drag
                {
                    drag.cursor_screen = (self.last_pointer.0, self.last_pointer.1);
                    hero.gizmo_drag = Some(drag);
                    let window_size = gfx.surface.size();
                    let cam = ph2d_editor::GizmoCamera {
                        center: gfx.camera.center,
                        height_world: gfx.camera.height_world,
                        window_w: window_size.width as f32,
                        window_h: window_size.height as f32,
                    };
                    // M14.7 D: sample winit's tracked modifier state
                    // (updated on ModifiersChanged). Shift / Ctrl /
                    // Alt feed AR lock + snap + mirror-anchor. On
                    // macOS we treat Cmd as Ctrl (industry convention
                    // for snap-to-grid).
                    let mods = ph2d_editor::GizmoModifiers {
                        shift: self.modifiers.shift_key(),
                        ctrl: self.modifiers.control_key() || self.modifiers.super_key(),
                        alt: self.modifiers.alt_key(),
                    };
                    let snap = ph2d_editor::GizmoSnap {
                        move_meters: hero.project.snap_move_meters,
                        rotate_deg: hero.project.snap_rotate_deg,
                    };
                    let new_t = ph2d_editor::compute_gizmo_transform(&drag, &cam, mods, snap);
                    let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
                    if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(entity) {
                        t.translation =
                            ph2d_core::Vec2::new(new_t.translation[0], new_t.translation[1]);
                        t.rotation = new_t.rotation;
                        t.scale = ph2d_core::Vec2::new(new_t.scale[0], new_t.scale[1]);
                    }
                }
                // Drag-in-progress: forward pointer to active tool
                // panel hit-test → updates slider value continuously.
                if self.dragging.is_some() {
                    self.dispatch_panel_pointer(self.last_pointer.0, self.last_pointer.1, false);
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x * 16.0, y * 16.0),
                    winit::event::MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
                };
                // M14.4b.bis: wheel over the canvas zooms the camera.
                // Wheel over a hero panel keeps the existing
                // panel-scroll behavior (forward to hero).
                let over_panel = cursor_over_hero_panel(
                    self.gfx.as_ref(),
                    self.last_pointer.0,
                    self.last_pointer.1,
                );
                if !over_panel && let Some(gfx) = self.gfx.as_mut() {
                    // Wheel up (positive dy) zooms IN (smaller height_world).
                    let factor = 0.9_f32.powf(dy / 16.0);
                    gfx.camera.zoom(factor);
                } else {
                    let evt = ph2d_host::WheelEvent {
                        x: self.last_pointer.0,
                        y: self.last_pointer.1,
                        delta_x: dx,
                        delta_y: dy,
                        modifiers: Self::convert_modifiers(self.modifiers),
                        timestamp_ns: Self::timestamp_ns(),
                    };
                    forward_wheel_to_hero(self.gfx.as_mut(), evt);
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let kind = match state {
                    ElementState::Pressed => PointerKind::Down,
                    ElementState::Released => PointerKind::Up,
                };
                let mapped_button = match button {
                    winit::event::MouseButton::Left => ph2d_host::PointerButton::Primary,
                    winit::event::MouseButton::Right => ph2d_host::PointerButton::Secondary,
                    winit::event::MouseButton::Middle => ph2d_host::PointerButton::Middle,
                    _ => ph2d_host::PointerButton::Primary,
                };
                let evt = PointerEvent {
                    x: self.last_pointer.0,
                    y: self.last_pointer.1,
                    pressure: 1.0,
                    kind,
                    source: PointerSource::Mouse,
                    button: mapped_button,
                    timestamp_ns: Self::timestamp_ns(),
                };
                self.handler.on_pointer(evt);
                forward_to_hero(self.gfx.as_mut(), evt);
                // M14.7 C: gizmo drag begin/end. A Primary Down that
                // lands on a gizmo handle starts a drag (snapshot
                // Transform + cursor world pos); Up clears it. Move
                // handling lives in CursorMoved so every motion event
                // gets the live cursor.
                if mapped_button == ph2d_host::PointerButton::Primary
                    && let Some(gfx) = self.gfx.as_mut()
                    && let Some(hero) = gfx.hero_screen.as_mut()
                {
                    match kind {
                        PointerKind::Down => {
                            let hit_id = hero.hit_index.hit(evt.x, evt.y);
                            let gizmo_kind = hit_id.and_then(ph2d_editor::gizmo_kind_for_id);
                            // M14.7 polish (19.3 fix): only the SPECIFIC
                            // handles (corner / edge / rotate) bypass
                            // canvas picking. `BBOX_INTERIOR` resolves
                            // to `Translate`, which previously absorbed
                            // every click inside the current selection's
                            // rect — that blocked the overlap-cycle path
                            // when a back sprite was selected and the
                            // user clicked the front sprite's visible
                            // area. The bbox_interior click now falls
                            // through to picking, which selects the
                            // topmost sprite first AND starts a translate
                            // drag on it via the canvas-pick branch.
                            let is_specific_handle = matches!(
                                gizmo_kind,
                                Some(ph2d_editor::GizmoDragKind::ScaleCorner { .. })
                                    | Some(ph2d_editor::GizmoDragKind::ScaleEdge { .. })
                                    | Some(ph2d_editor::GizmoDragKind::Rotate)
                            );
                            if is_specific_handle
                                && let Some(gkind) = gizmo_kind
                                && let Some(entity_bits) = hero.gizmo_selection
                            {
                                // Snapshot the entity's Transform + the
                                // bbox pivot (center) for the math.
                                let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                                let window_size = gfx.surface.size();
                                let start_world =
                                    gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                                if let Some(t) = gfx.sim.world().get::<Transform>(entity) {
                                    // Pivot defaults to the entity
                                    // translation (sprite center).
                                    // M14.7 D will swap this when Alt
                                    // is held.
                                    let pivot = [t.translation.x, t.translation.y];
                                    let snap = ph2d_editor::TransformSnapshot {
                                        translation: [t.translation.x, t.translation.y],
                                        rotation: t.rotation,
                                        scale: [t.scale.x, t.scale.y],
                                    };
                                    hero.gizmo_drag = Some(ph2d_editor::GizmoDragState {
                                        kind: gkind,
                                        entity_bits,
                                        start_screen: (evt.x, evt.y),
                                        cursor_screen: (evt.x, evt.y),
                                        start_transform: snap,
                                        pivot_world: pivot,
                                        start_cursor_world: start_world,
                                    });
                                }
                            } else if hero.store.panel_at(evt.x, evt.y).is_none()
                                && hero.store.context_menu().is_none()
                                && (
                                    // No widget under cursor: clean
                                    // canvas click.
                                    hit_id.is_none()
                                    // Gizmo bbox interior or pivot:
                                    // user clicked inside the
                                    // selection's footprint. Still
                                    // a canvas click — picking may
                                    // promote a different sprite to
                                    // the selection (overlap-cycle).
                                    || matches!(
                                        gizmo_kind,
                                        Some(ph2d_editor::GizmoDragKind::Translate),
                                    )
                                    || hit_id == Some(ph2d_editor::gizmo::ids::GIZMO_PIVOT)
                                )
                            {
                                // Canvas pick (M14.7 A) — but only when:
                                // 1. The click did NOT begin a gizmo drag.
                                // 2. The cursor is outside chrome panels.
                                // 3. NO context menu is currently open
                                //    (clicking on a menu item must not
                                //    also hijack as a canvas click → was
                                //    the M14.6 F regression user reported).
                                // 4. No widget id was hit at all
                                //    (search field, menu item, etc. all
                                //    register hit rects in HitIndex; if
                                //    any of those resolves, the dispatch
                                //    consumes the click and canvas pick
                                //    must stay out of the way).
                                let window_size = gfx.surface.size();
                                let world_pos =
                                    gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                                // M14.7 polish (19.3): alternate-click
                                // overlap cycling — anchored by the
                                // SAME HIT SET, not the same pixel.
                                // Touch + stylus inputs can't reliably
                                // hit the same world point twice; tying
                                // the cycle to the entity set instead
                                // lets the user tap anywhere inside the
                                // overlap region and still walk the
                                // stack. Reset triggers:
                                //   - hit list shape changed (cursor
                                //     moved to a different overlap)
                                //   - hit list empty (= empty canvas
                                //     click → deselect)
                                let hits = ph2d_render::pick_sprites_at_world(
                                    gfx.present.world_mut(),
                                    world_pos,
                                );
                                let same_list = !hits.is_empty() && hits == self.cycle_pick_hits;
                                if !same_list {
                                    // Fresh click — reset the cycle.
                                    self.cycle_pick_world = Some(world_pos);
                                    self.cycle_pick_hits = hits.clone();
                                    self.cycle_pick_idx = 0;
                                    self.cycle_pick_count = 1;
                                } else {
                                    // Same overlap stack as before —
                                    // increment counter; advance idx on
                                    // every odd count past the first.
                                    self.cycle_pick_count = self.cycle_pick_count.saturating_add(1);
                                    if self.cycle_pick_count.is_multiple_of(2) {
                                        // Even count → selection stays.
                                    } else if !hits.is_empty() {
                                        self.cycle_pick_idx =
                                            (self.cycle_pick_idx + 1) % hits.len();
                                    }
                                }
                                let picked = if hits.is_empty() {
                                    None
                                } else {
                                    hits.get(self.cycle_pick_idx).copied()
                                };
                                hero.gizmo_selection = picked;
                                // M14.7 polish: same-click select +
                                // drag. Without this the user has to
                                // click twice — once to select, once
                                // to grab the bbox-interior handle —
                                // because the gizmo isn't painted (so
                                // its handles aren't in hit_index) on
                                // the frame of the initial pick. We
                                // start a Translate drag using the
                                // picked sprite's Transform snapshot;
                                // if the user releases without moving,
                                // the math returns delta=0 → Transform
                                // unchanged → visually identical to a
                                // pure selection click.
                                if let Some(bits) = picked {
                                    let entity = ph2d_ecs::Entity::from_bits(bits);
                                    if let Some(t) = gfx.sim.world().get::<Transform>(entity) {
                                        let snap_t = ph2d_editor::TransformSnapshot {
                                            translation: [t.translation.x, t.translation.y],
                                            rotation: t.rotation,
                                            scale: [t.scale.x, t.scale.y],
                                        };
                                        let pivot = [t.translation.x, t.translation.y];
                                        hero.gizmo_drag = Some(ph2d_editor::GizmoDragState {
                                            kind: ph2d_editor::GizmoDragKind::Translate,
                                            entity_bits: bits,
                                            start_screen: (evt.x, evt.y),
                                            cursor_screen: (evt.x, evt.y),
                                            start_transform: snap_t,
                                            pivot_world: pivot,
                                            start_cursor_world: world_pos,
                                        });
                                    }
                                }
                                // M14.6 D: canvas-pick → hierarchy sync.
                                // Resolve entity bits → bridge NodeId →
                                // live entry → update `hero.selection`
                                // so the hierarchy row paints as selected
                                // (the panel matches by name label).
                                if let Some(live) = gfx.hero_live.as_ref()
                                    && let Some(bits) = picked
                                    && let Some(node_id) = live.bridge.node_for(bits)
                                    && let Some(entries) = hero.live_hierarchy_entries.as_ref()
                                    && let Some(entry) = entries.get(&node_id)
                                {
                                    hero.selection = Some(ph2d_editor::HeroSelection {
                                        label: entry.name.clone(),
                                        kind: entry
                                            .badge
                                            .clone()
                                            .unwrap_or_else(|| "ENT".to_string()),
                                        world_pos: (0.0, 0.0),
                                    });
                                } else if picked.is_none() {
                                    hero.selection = None;
                                }
                                self.title_dirty = true;
                            }
                        }
                        PointerKind::Up => {
                            // Drop the drag — Transform is already
                            // committed up to the latest Move position.
                            hero.gizmo_drag = None;
                        }
                        _ => {}
                    }
                }
                // M14.4b.bis: middle button = camera pan anchor.
                // Tracked here so CursorMoved can drive the pan.
                if button == winit::event::MouseButton::Middle {
                    match state {
                        ElementState::Pressed => {
                            self.pan_anchor = Some(self.last_pointer);
                        }
                        ElementState::Released => {
                            self.pan_anchor = None;
                        }
                    }
                }
                match state {
                    ElementState::Pressed => {
                        // Mirror-sidebar chip takes precedence over the
                        // panel hit-test (different zone, no overlap).
                        let mut consumed = false;
                        if let Some(gfx) = self.gfx.as_mut()
                            && !gfx.zen.is_active()
                            && let Some(btn) = gfx.layout.mirror_button_rect()
                            && btn.contains(self.last_pointer.0, self.last_pointer.1)
                        {
                            gfx.layout.mirror_sidebar();
                            gfx.toasts.push(Toast::info(format!(
                                "Sidebar → {:?}",
                                gfx.layout.sidebar_side
                            )));
                            self.title_dirty = true;
                            consumed = true;
                        }
                        // Tool palette icon click — switch active tool.
                        if !consumed
                            && let Some(gfx) = self.gfx.as_mut()
                            && !gfx.zen.is_active()
                        {
                            let palette = gfx.layout.tool_palette_rects(gfx.tools.tools().len());
                            let hit_idx = palette
                                .iter()
                                .position(|r| r.contains(self.last_pointer.0, self.last_pointer.1));
                            if let Some(idx) = hit_idx {
                                let tool_id = gfx.tools.tools()[idx].id();
                                let tool_label = gfx.tools.tools()[idx].label().to_string();
                                if gfx.tools.set_active(&tool_id) {
                                    gfx.toasts.push(Toast::info(format!("Tool → {tool_label}")));
                                    self.title_dirty = true;
                                }
                                consumed = true;
                            }
                        }
                        if !consumed {
                            // Mouse down — start hit-test against active panel.
                            self.dispatch_panel_pointer(
                                self.last_pointer.0,
                                self.last_pointer.1,
                                true,
                            );
                        }
                    }
                    ElementState::Released => {
                        // End any drag-in-progress.
                        self.dragging = None;
                    }
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    WinitKeyEvent {
                        physical_key,
                        state,
                        repeat,
                        ref text,
                        ..
                    },
                ..
            } => {
                let keycode = match physical_key {
                    winit::keyboard::PhysicalKey::Code(code) => code as u32,
                    winit::keyboard::PhysicalKey::Unidentified(_) => 0,
                };
                let kind = match (state, repeat) {
                    (ElementState::Pressed, false) => KeyKind::Down,
                    (ElementState::Pressed, true) => KeyKind::Repeat,
                    (ElementState::Released, _) => KeyKind::Up,
                };
                self.handler.on_key(KeyEvent {
                    keycode,
                    modifiers: Self::convert_modifiers(self.modifiers),
                    kind,
                    timestamp_ns: Self::timestamp_ns(),
                });

                // Hero pipeline (ADR-0024): translate winit's
                // physical KeyCode into the editor's KEY_* constants
                // and route to the focused widget.
                if state == ElementState::Pressed
                    && let PhysicalKey::Code(code) = physical_key
                    && let Some(editor_keycode) = winit_to_editor_keycode(code)
                {
                    forward_key_to_hero(
                        self.gfx.as_mut(),
                        KeyEvent {
                            keycode: editor_keycode,
                            modifiers: Self::convert_modifiers(self.modifiers),
                            kind,
                            timestamp_ns: Self::timestamp_ns(),
                        },
                    );
                }
                // Printable text from this key event (winit already
                // resolved layout + dead-keys + shift). Send each
                // char through the text-input dispatcher so focused
                // TextInput/NumberInput/Combobox buffers update.
                // EXCEPT for ' ' coming from the physical Space key
                // and 'a'/'A' coming from KeyA with Cmd/Ctrl held —
                // those are inserted by the dispatch's key handler
                // directly (SPACE bug: winit's first text-event for
                // SPACE on macOS arrived empty; Cmd+A: text-event
                // would replace the just-selected range with 'a').
                if state == ElementState::Pressed
                    && let Some(s) = text.as_ref()
                {
                    let is_space_key = matches!(physical_key, PhysicalKey::Code(KeyCode::Space));
                    let cmd_held = self.modifiers.super_key() || self.modifiers.control_key();
                    for ch in s.chars() {
                        if ch.is_control() {
                            continue;
                        }
                        if is_space_key && ch == ' ' {
                            continue;
                        }
                        // Cmd/Ctrl chord with a letter: skip the
                        // text-event so Cmd+A's select-all isn't
                        // overwritten by 'a' insertion.
                        if cmd_held && ch.is_ascii_alphabetic() {
                            continue;
                        }
                        forward_text_to_hero(self.gfx.as_mut(), ch);
                    }
                }

                // M12 demo controls (only on key Down, no repeat).
                if matches!((state, repeat), (ElementState::Pressed, false))
                    && let PhysicalKey::Code(code) = physical_key
                {
                    self.handle_editor_key(code);
                }
            }

            WindowEvent::RedrawRequested => {
                self.render_frame();
            }

            _ => {}
        }
    }
}

fn log_input_event(elapsed_ms: u128, event: &InputEvent) {
    match event {
        InputEvent::GamepadButtonDown(b) => {
            println!(
                "[{:>6}ms] gamepad button down: {}",
                elapsed_ms,
                b.as_lua_key()
            );
        }
        InputEvent::GamepadButtonUp(b) => {
            println!(
                "[{:>6}ms] gamepad button up:   {}",
                elapsed_ms,
                b.as_lua_key()
            );
        }
        InputEvent::GamepadAxis { axis, value } => {
            // Spam-prone: log only when |value| > 0.25 to skip
            // dead-zone jitter.
            if value.abs() > 0.25 {
                println!(
                    "[{:>6}ms] gamepad axis {} = {:+.2}",
                    elapsed_ms,
                    axis.as_lua_key(),
                    value
                );
            }
        }
        InputEvent::Pencil(_) => {
            // No iPad shell yet; pencil events can't originate here.
        }
    }
}

/// Forward a pointer event to the hero screen's interaction
/// dispatcher when the hero is active. Drains emitted
/// [`WidgetEvent`]s into `HeroScreen::apply_event` (consumed events
/// drive hero-level state mutations) and logs unconsumed ones to
/// stderr for the developer to verify wiring.
fn forward_to_hero(gfx: Option<&mut AppGfx>, event: PointerEvent) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    // Snapshot events before applying — apply_event may mutate hero,
    // but the events slice itself lives in the arena (immutable view).
    // Threads the live TextSystem so click→caret on text widgets
    // snaps to the nearest glyph boundary (real measurement) instead
    // of the dispatch's char-count heuristic.
    let snapshot: Vec<WidgetEvent> = hero
        .handle_pointer_with_text(event, &mut gfx.text_system, &gfx.hero_arena)
        .to_vec();
    for e in snapshot {
        // Eyedropper pick — read the rendered pixel at the click
        // position from vello_pass's intermediate texture and apply
        // it to the picker. Only the host can do this (the dispatch
        // has no GPU access); intercept before `apply_event`.
        if let WidgetEvent::EyedropperPick { parent, px, py } = e {
            if let Some([r, g, b, a]) = gfx.vello_pass.read_pixel(gfx.surface.gpu(), px, py) {
                hero.store
                    .set_blender_value(parent, ph2d_tokens::ColorValue::from_rgba8(r, g, b, a));
            }
            continue;
        }
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled event: {e:?}");
        }
    }
}

/// Forward a translated [`KeyEvent`] (with editor-canonical
/// `keycode` from [`winit_to_editor_keycode`]) into the hero
/// dispatcher so focused widgets see Tab/Enter/Backspace/arrows etc.
/// Also drains any clipboard copy/paste requests the dispatcher set
/// for this key event and bridges to `arboard`.
fn forward_key_to_hero(gfx: Option<&mut AppGfx>, event: KeyEvent) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let snapshot: Vec<WidgetEvent> = hero.handle_key(event, &gfx.hero_arena).to_vec();
    for e in snapshot {
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled key event: {e:?}");
        }
    }
    // Drain clipboard requests set by Cmd+C / Cmd+X / Cmd+V.
    if let Some(text) = hero.store.take_clipboard_copy()
        && let Some(cb) = gfx.clipboard.as_mut()
        && let Err(err) = cb.set_text(text)
    {
        eprintln!("[ph2d] clipboard set_text failed: {err}");
    }
    if let Some(target) = hero.store.take_clipboard_paste_request() {
        let text = gfx
            .clipboard
            .as_mut()
            .and_then(|cb| cb.get_text().ok())
            .unwrap_or_default();
        if !text.is_empty()
            && ph2d_editor::interaction::apply_clipboard_paste(&mut hero.store, target, &text)
        {
            // Mimic the TextChanged path so sliders/links update.
            let _ = hero.apply_event(WidgetEvent::TextChanged(target));
        }
    }
}

/// Forward a wheel / trackpad scroll into the hero dispatcher.
/// Routes to whichever panel registered its rect under the cursor.
fn forward_wheel_to_hero(gfx: Option<&mut AppGfx>, event: ph2d_host::WheelEvent) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let _ = hero.handle_wheel(event, &gfx.hero_arena);
}

/// M14.4c+M14.4d retrofit: full pipeline for importing one image
/// file into the running demo.
///
/// 1. Read bytes from disk.
/// 2. `AssetDb::insert_image_bytes` (auto-detects PNG/WEBP/JPEG,
///    hashes blake3 per HR-6).
/// 3. `SpriteRenderer::insert_atlas_sprite` packs the native-
///    resolution RGBA into the atlas via the Skyline rect packer
///    — no resize, no aspect-ratio squash.
/// 4. Spawn `(Transform at camera center, Sprite { atlas_index:
///    cell_idx, size: source_pixels / pixels_per_meter }, Name)`.
///
/// Returns the human-readable label that was assigned to the
/// spawned entity (so the host can show it in a toast).
// Helper has 8 args because the import path is a top-level fixture
// orchestrator that needs full access to sim/renderer/asset_db plus
// the camera + per-import inputs. Splitting into a struct would just
// move the noise — every call site already destructures `AppGfx`.
#[allow(clippy::too_many_arguments)]
fn import_image_at_camera(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    camera: &Camera2d,
    cell_idx: u32,
    path: &std::path::Path,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let asset_id = asset_db
        .insert_image_bytes(&bytes)
        .map_err(|e| format!("decode {}: {e}", path.display()))?;
    let decoded = asset_db
        .get(&asset_id)
        .ok_or_else(|| format!("asset {asset_id} missing after insert"))?;
    let (width, height, pixels) = match &*decoded {
        ph2d_asset::Asset::ImageRgba8 {
            width,
            height,
            pixels,
        } => (*width, *height, pixels.clone()),
        _ => return Err(format!("{asset_id} is not ImageRgba8 after decode")),
    };
    // M14.7 polish: pack via the regrow path so a 2nd / 3rd 4K
    // import doubles the atlas instead of failing with AtlasFull.
    // The closure recovers each existing region's source bytes from
    // `asset_db` via `atlas_asset_map[key] → AssetId`. Track this
    // import's mapping BEFORE the insert so a regrow triggered by
    // it sees the new key.
    atlas_asset_map.insert(cell_idx, asset_id);
    let fetch_pixels = |key: u32| -> Option<Vec<u8>> {
        let aid = atlas_asset_map.get(&key)?;
        let asset = asset_db.get(aid)?;
        match &*asset {
            // `pixels` is `Arc<[u8]>`; the regrow callback wants an
            // owned `Vec<u8>` because the underlying packer may
            // outlive the asset borrow.
            ph2d_asset::Asset::ImageRgba8 { pixels, .. } => Some(pixels.to_vec()),
            _ => None,
        }
    };
    if let Err(e) =
        renderer.insert_atlas_sprite_with_regrow(cell_idx, width, height, &pixels, fetch_pixels)
    {
        // On failure, roll back the map insert so the next import
        // doesn't see a dangling key → nonexistent atlas region.
        atlas_asset_map.remove(&cell_idx);
        return Err(format!("atlas insert {}: {e}", path.display()));
    }

    let base_label = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| format!("imported_{cell_idx}"));
    // M14.7 polish: enforce unique entity names. Without this, the
    // same image imported N times produced N rows all sharing the
    // same `Name`, and the M14.6 D selection-by-label sync would
    // highlight every row when one of the duplicates was canvas-
    // picked. We scan the SimWorld for an existing match and append
    // `_{cell_idx}` (monotonic, never repeats) on collision.
    let label = {
        let world = sim.world_mut();
        let mut q = world.query::<&Name>();
        let collides = q
            .iter(world)
            .any(|n: &Name| n.as_str() == base_label.as_str());
        if collides {
            format!("{base_label}_{cell_idx}")
        } else {
            base_label
        }
    };
    let spawn_pos = Vec2::new(camera.center[0], camera.center[1]);
    // Sprite world size = source pixels / pixels_per_meter. With the
    // Skyline atlas the source bytes are stored at full resolution,
    // so the world quad's aspect ratio matches the file exactly
    // (a 256×128 PNG renders as a 2:1 rect in world space).
    let safe_px_per_m = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let world_w = (width as f32 / safe_px_per_m).max(crate::MIN_SPRITE_SIZE);
    let world_h = (height as f32 / safe_px_per_m).max(crate::MIN_SPRITE_SIZE);
    sim.world_mut().spawn((
        Transform::from_translation(spawn_pos),
        Sprite::atlas(cell_idx, [world_w, world_h], [1.0, 1.0, 1.0, 1.0]),
        Name::new(label.clone()),
    ));
    Ok(label)
}

/// Floor for `pixels_per_meter` used inside the import math; below
/// this a single sprite would span kilometers and break camera math.
/// The UI clamps to a higher floor (`MIN_PIXELS_PER_METER = 1.0` in
/// `ph2d_editor::project`) but defense-in-depth here keeps the shell
/// safe even if a future config path skips that clamp.
const EPS_PIXELS_PER_METER: f32 = 0.01;

/// Floor for the world-space side length of an imported sprite.
/// Guarantees the quad is selectable even if the user picks an
/// absurd `pixels_per_meter` value combined with a 1-pixel image.
const MIN_SPRITE_SIZE: f32 = 0.001;

/// Query the live cursor position relative to `window` in physical
/// pixels (top-left origin). Returns `None` if the platform path
/// fails; callers fall back to a cached value.
///
/// Existence rationale: winit 0.30 on macOS does not emit
/// `CursorMoved` during external file drag operations, so by the
/// time `DroppedFile` fires the cached cursor is stale. We bypass
/// the event stream by asking CoreGraphics for the live cursor
/// directly. Other platforms reach here only as a no-op stub.
/// M14.4b.bis: true when `(x, y)` lies inside either the Inspector
/// or Hierarchy panel rect published by the most-recent
/// `paint_hero_screen` pass. Used to decide whether a mouse-wheel
/// event should zoom the camera (over canvas) or scroll a panel
/// (over a panel).
///
/// Returns false when no hero is active — the demo's fixture mode
/// shows raw sprites with no panels, so the whole window is "canvas"
/// and wheel zooms the camera.
fn cursor_over_hero_panel(gfx: Option<&AppGfx>, x: f32, y: f32) -> bool {
    let Some(gfx) = gfx else { return false };
    let Some(hero) = gfx.hero_screen.as_ref() else {
        return false;
    };
    use ph2d_editor::screens::hero::ids::{GAL_PANEL, HIER_PANEL, INSP_PANEL};
    let inside = |panel_id| {
        hero.store
            .panel_rect(panel_id)
            .map(|r| r.contains(x, y))
            .unwrap_or(false)
    };
    // GAL_PANEL is the floating Widget Gallery — must intercept the
    // wheel here so it scrolls the panel body instead of zooming the
    // camera underneath. `panel_rect(GAL_PANEL)` is only published
    // when the gallery is visible, so this check returns false in
    // its default closed state.
    inside(INSP_PANEL) || inside(HIER_PANEL) || inside(GAL_PANEL)
}

/// Forward a single printable character into the hero text-input
/// dispatcher (focused TextInput/NumberInput/Combobox buffer).
fn forward_text_to_hero(gfx: Option<&mut AppGfx>, ch: char) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let snapshot: Vec<WidgetEvent> = hero.handle_text_input(ch, &gfx.hero_arena).to_vec();
    for e in snapshot {
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled text-input event: {e:?}");
        }
    }
}

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

//! `App` + `AppGfx` + `ImageEditSnapshot` + `HeroLive` struct
//! definitions extracted from main.rs (Wave 3.2 stage B).
//!
//! Re-exported by main.rs at the crate root so other modules
//! (`render_loop`, `input_handlers`, `init`) can keep their
//! `crate::App` / `crate::AppGfx` paths unchanged. All fields stay
//! `pub(crate)` — these are shell-internal aggregates, not a public
//! API surface.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use bumpalo::Bump;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::FixedStep;
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, HierarchySnapshot, HierarchyWalkState,
};
use ph2d_ecs::{PresentWorld, SimWorld, TransformPropagationState, WorklistBuf};
use ph2d_editor::{HeroScreen, Layout as EditorLayout, NodeId, ToastQueue, ToolRegistry, ZenMode};
use ph2d_gpu::SurfaceContext;
use ph2d_host::WindowSize;
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
    pub(crate) last_frame: Instant,
    pub(crate) pending_resize: Option<WindowSize>,
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
    /// Input snapshot pumped by the gilrs adapter each frame.
    pub(crate) input: InputState,
    /// M14.4b.bis: middle-button camera pan state.
    /// `Some(anchor)` while a middle-drag is in progress; subsequent
    /// `CursorMoved` events feed `Camera2d::pan_screen_delta`.
    pub(crate) pan_anchor: Option<(f32, f32)>,
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
    /// On-canvas live preview for the Color Equalization tool — drawn
    /// over the primary sprite's footprint while the tool is active so
    /// the user sees CLAHE + adjusts apply in real time. Cleared on
    /// Apply (the bake replaces the texture) + on deactivate.
    pub(crate) color_equalization_preview: Option<ColorEqualizationPreview>,
    /// Cached full-resolution Background-Removal preview for the active
    /// sprite. Recomputed only when params change / source changes /
    /// the tool (re)activates; the per-frame canvas overlay reuses the
    /// `Arc` buffer (no pixel copy). `None` when the tool is inactive
    /// or no source is loaded. NOT a destructive edit — the sprite's
    /// own texture is untouched; the overlay just paints on top, so
    /// Apply (which re-reads the original source) and undo stay correct.
    pub(crate) bgremoval_preview: Option<BgremovalPreview>,
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
pub(crate) struct BgremovalPreview {
    /// Sprite the preview belongs to — invalidates when selection moves.
    pub(crate) entity_bits: u64,
    /// Straight-alpha RGBA8, `width * height * 4`. `Arc` so the
    /// per-frame `VectorScene::draw_image_rgba` shares it cheaply.
    pub(crate) rgba: std::sync::Arc<Vec<u8>>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

/// Cached on-canvas live preview bitmap for the Color Equalization
/// tool — mirror of `BgremovalPreview`. Updated by
/// `color_equalization_bridge::dispatch` when the tool flags
/// `take_params_dirty()` true (a slider/chip/toggle just moved). The
/// per-frame overlay paints this RGBA on top of the primary sprite's
/// footprint while the tool is active, so the user sees the CLAHE +
/// adjusts apply in real time. Cleared on Apply (the bake takes over)
/// + on tool deactivate. Straight-alpha (Color EQ doesn't touch alpha).
pub(crate) struct ColorEqualizationPreview {
    pub(crate) entity_bits: u64,
    pub(crate) rgba: std::sync::Arc<Vec<u8>>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

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

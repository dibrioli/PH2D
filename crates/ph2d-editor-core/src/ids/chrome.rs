use super::*;

pub const TOPBAR_THEME: NodeId = hash_node_id("topbar_theme");
pub const TOPBAR_SAVE: NodeId = hash_node_id("topbar_save");
pub const TOPBAR_PROJECT: NodeId = hash_node_id("topbar_project");
pub const TOPBAR_PLAY_TOGGLE: NodeId = hash_node_id("topbar_play_toggle");
pub const TOPBAR_PLAY_BUTTON: NodeId = hash_node_id("topbar_play_button");
pub const TOPBAR_RIGHT_LAYERS: NodeId = hash_node_id("topbar_right_layers");
pub const TOPBAR_RIGHT_ASSETS: NodeId = hash_node_id("topbar_right_assets");
pub const TOPBAR_RIGHT_SCRIPT: NodeId = hash_node_id("topbar_right_script");
pub const TOPBAR_PAUSE: NodeId = hash_node_id("topbar_pause");
pub const TOPBAR_RESET: NodeId = hash_node_id("topbar_reset");
pub const TOPBAR_SAVE_AS: NodeId = hash_node_id("topbar_save_as");
pub const TOPBAR_OPEN: NodeId = hash_node_id("topbar_open");
/// Settings cluster (gear icon) — opens the SettingsMenu context menu
/// with project-level toggles (pixels-per-meter presets, future
/// global config). Added M14.4d retrofit.
pub const TOPBAR_SETTINGS: NodeId = hash_node_id("topbar_settings");
/// Frosted-glass agrupador backdrops behind each topbar cluster
/// group. Painted before the chips so clicks on chips win; clicks
/// on the empty backdrop space land here.
pub const TOPBAR_LEFT_BACKDROP: NodeId = hash_node_id("topbar_left_backdrop");
pub const TOPBAR_RIGHT_BACKDROP: NodeId = hash_node_id("topbar_right_backdrop");
pub const TOPBAR_IMAGE_TOOLS_BACKDROP: NodeId = hash_node_id("topbar_image_tools_backdrop");
/// Image Tools cluster — toggle entry-point for the image-editing
/// action row (Trim Transparency in V1; BG Removal / Equalize / etc.
/// to follow). Click flips the TopBar between Edit mode and
/// ImageTools mode; the state lives on
/// [`crate::screens::HeroScreen::image_tools_mode`].
pub const TOPBAR_IMAGE_TOOLS: NodeId = hash_node_id("topbar_image_tools");
/// Vector Pen tool cluster — TopBar single-pill that activates the
/// Vector Module Pen tool (W1.T1.7). Click pushes
/// `EditorAction::ActivateTool { tool_id: "vector_pen" }`; shell
/// drain in `render_loop::mod` calls
/// `tools.set_active(&ToolId::new("vector_pen"))`. MVP placement
/// alongside the other right-side single pills; W2+ may move into a
/// dedicated "vector_tools" mode toggle (parallel to Image Tools).
///
/// **Hash key = `hash_node_id("vector_pen")`** (NOT
/// `"topbar_vector_pen"`) to match the **image-action pill
/// convention** (`IMAGE_ACTION_BGREMOVAL = hash_node_id("bgremoval")`,
/// etc.). This lets the Pressed-highlight reconcile loop in
/// `shells/desktop/src/render_loop/mod.rs` discover the pill via
/// `hash_node_id(manifest.id)` lookup — same code path bgremoval uses.
pub const TOPBAR_VECTOR_PEN: NodeId = hash_node_id("vector_pen");
/// Vector Pencil pill (W2 T2.1) — same `hash_node_id(manifest.id)`
/// convention as the Pen, so the `vector_tools` reconcile loop highlights it.
pub const TOPBAR_VECTOR_PENCIL: NodeId = hash_node_id("vector_pencil");
/// Vector Shape pill (W2 T2.2) — same `hash_node_id(manifest.id)` convention.
pub const TOPBAR_VECTOR_SHAPE: NodeId = hash_node_id("vector_shape");
/// Vector Select pill (W2 T2.3) — same `hash_node_id(manifest.id)` convention.
pub const TOPBAR_VECTOR_SELECT: NodeId = hash_node_id("vector_select");
/// Vector Direct-Select pill (W2 T2.3) — same `hash_node_id(manifest.id)` convention.
pub const TOPBAR_VECTOR_DIRECT: NodeId = hash_node_id("vector_direct");
/// Vector Geometry-Graph panel (W3 T3.1) — docked panel that places the
/// `vector.source` node + drives its 8 params (sliders) and renders the cooked
/// `VectorNetwork` live. Outer-rect id for `z_order`; the 8 param sliders below.
pub const VGRAPH_PANEL: NodeId = hash_node_id("vgraph.panel");
pub const VGRAPH_KIND: NodeId = hash_node_id("vgraph.kind");
pub const VGRAPH_WIDTH: NodeId = hash_node_id("vgraph.width");
pub const VGRAPH_HEIGHT: NodeId = hash_node_id("vgraph.height");
pub const VGRAPH_SIDES: NodeId = hash_node_id("vgraph.sides");
pub const VGRAPH_INNER_RATIO: NodeId = hash_node_id("vgraph.inner_ratio");
pub const VGRAPH_TURNS: NodeId = hash_node_id("vgraph.turns");
pub const VGRAPH_SAMPLES_PER_TURN: NodeId = hash_node_id("vgraph.samples_per_turn");
pub const VGRAPH_ROTATION: NodeId = hash_node_id("vgraph.rotation");
/// Vector Inspector panel (W2 T2.4) — minimal right-docked panel hosting the
/// fill swatch (+ future vertex/node params). Outer-rect id for `z_order`.
pub const VECTOR_INSPECTOR_PANEL: NodeId = hash_node_id("vector_inspector.panel");
/// Vector Inspector close (X) button.
pub const VECTOR_INSPECTOR_CLOSE: NodeId = hash_node_id("vector_inspector.close");
/// Vector Inspector fill-color swatch — a picker swatch (opens the Blender
/// picker on Down via `is_picker_swatch`); the shell read-back applies the
/// picked color to the selected regions.
pub const VECTOR_INSPECTOR_FILL_SWATCH: NodeId = hash_node_id("vector_inspector.fill_swatch");
/// Vector Inspector Shape-kind picker (W2 §4.2) — the on-screen replacement for
/// the interim hotkeys 1-5. A vertical 5-option segmented control shown only
/// while the `vector_shape` tool is active. `_KIND` is the group/label id; the
/// five `_SHAPE_*` ids are the per-option Button hit-targets (the generic
/// RadioGroup dispatch is not wired yet, so each option is a Button — its
/// `Click` routes through the shell bridge to `VectorShapeTool::set_kind`).
pub const VECTOR_INSPECTOR_SHAPE_KIND: NodeId = hash_node_id("vector_inspector.shape_kind");
pub const VECTOR_INSPECTOR_SHAPE_RECT: NodeId = hash_node_id("vector_inspector.shape.rect");
pub const VECTOR_INSPECTOR_SHAPE_ELLIPSE: NodeId = hash_node_id("vector_inspector.shape.ellipse");
pub const VECTOR_INSPECTOR_SHAPE_POLYGON: NodeId = hash_node_id("vector_inspector.shape.polygon");
pub const VECTOR_INSPECTOR_SHAPE_STAR: NodeId = hash_node_id("vector_inspector.shape.star");
pub const VECTOR_INSPECTOR_SHAPE_SPIRAL: NodeId = hash_node_id("vector_inspector.shape.spiral");
/// Widget Gallery cluster — toggles the floating reference panel
/// that showcases every canonical widget (Inputs / Slider /
/// Switches / Lists / Vector / Status / Color / Actions / Identity /
/// Card). Peripheral agents open this from the live app as the
/// single in-app source of truth for UI decoration. Visibility lives
/// in `HeroScreen::panel_visibility` (keyed `"widget_gallery"`) after
/// ADR-0029 Phase C.3; persistent rect lives on
/// `ph2d_panel_widget_gallery::WidgetGalleryState::rect`.
pub const TOPBAR_WIDGET_GALLERY: NodeId = hash_node_id("topbar_widget_gallery");
/// Grid Settings cluster — opens the floating Grid Settings panel
/// (grid-snap subsystem). Toggles
/// `HeroScreen::panel_visibility["grid_snap"]` via the typed
/// `GridSnapPanel::apply_event` after ADR-0029 Phase C.4.
pub const TOPBAR_GRID_SETTINGS: NodeId = hash_node_id("topbar_grid_settings");

/// Image Tools action — Trim Transparency pill. Lives in the action
/// row that replaces the right-side TopBar clusters when
/// `image_tools_mode` is on. Click is no-op for now — wiring to the
/// `ph2d_tool_trim_transparency::trim_transparency()` on a selected sprite
/// requires the live asset model (out of scope for this PR).
// Wave 2 PR 11.4: the three Image Tools action pills are now derived
// from the `image_tools` cluster in the runtime registry. To make
// hand-written click dispatch (`id == ids::IMAGE_ACTION_*`) work
// against registry-derived pills, each chrome const hashes the SAME
// slug as the matching tool manifest's `id` field. The
// `chrome_manifest_coverage` integration test pins this contract.
pub const IMAGE_ACTION_TRIM: NodeId = hash_node_id("trim_transparency");

/// Image Tools action — Make Square pill. Sibling of `IMAGE_ACTION_TRIM`,
/// pads the selected sprite with transparent pixels on the shorter axis
/// so width == height. Click raises `pending_make_square` on `HeroScreen`;
/// host drains, runs the algorithm, replaces sprite pixels + reprojects
/// pivot. Algorithm in crate `ph2d-tool-make-square` (ADR-0040 T1.5).
pub const IMAGE_ACTION_MAKE_SQUARE: NodeId = hash_node_id("make_square");

/// Image Tools action — Background Removal pill. Unlike `IMAGE_ACTION_TRIM`
/// and `IMAGE_ACTION_MAKE_SQUARE` (one-shot algorithms), this one
/// ACTIVATES the stateful `BgRemovalTool` so its floating panel opens
/// at the BottomCenter with a live 160×160 preview. Click raises
/// `pending_activate_bgremoval` on `HeroScreen`; host drains via
/// `tools.set_active(ToolId::new("bgremoval"))` and force-refreshes
/// the snapshot push.
pub const IMAGE_ACTION_BGREMOVAL: NodeId = hash_node_id("bgremoval");

/// Image Tools action — Real Size pill. One-shot like Trim / Make Square:
/// resets the selected sprite's `Transform.scale` to 1:1 (preserving flip
/// sign). Click raises `EditorAction::OneShotImageOp { tool_id: "real_size" }`; the shell drain mutates
/// the ECS `Transform`. Algorithm in `ph2d-tool-real-size`.
pub const IMAGE_ACTION_REAL_SIZE: NodeId = hash_node_id("real_size");

/// Image Tools action — Padding pill. Unlike the one-shots, this ACTIVATES
/// the stateful Padding tool (panel with 4 signed per-edge fields + Apply;
/// the directional-expand gizmo edge-drag is a v2). Click raises
/// `EditorAction::ActivateTool { tool_id: "padding" }`; the shell sets the tool active.
/// Condenses the legacy Image Padding + Directional Expand.
pub const IMAGE_ACTION_PADDING: NodeId = hash_node_id("padding");

/// Color Equalization panel marker NodeId. Right-docked in the
/// Inspector geometry slot while the `color_equalization` tool is
/// active. Hash matches `ph2d_tool_color_equalization::ids::CEQ_PANEL`
/// (the tool crate owns the canonical const for its own widgets;
/// editor-core mirrors it here so `paint_hero_screen`'s z_order
/// fallback can walk the panel without a circular dep on the tool
/// crate). Same hash key (`"panel.color_equalization"`), same
/// resolved id.
pub const CEQ_PANEL: NodeId = hash_node_id("panel.color_equalization");

/// Equalize Sizes panel marker NodeId. Mirror of `CEQ_PANEL` for the
/// multi-sprite size-normalization tool. Hash matches
/// `ph2d_tool_equalize_sizes::ids::EQS_PANEL` and
/// `ph2d_panel_equalize_sizes::EqualizeSizesPanel::NODE_ID` — keeping
/// the dispatcher's `panel_at` lookup, the typed panel registry, and
/// `paint_hero_screen`'s z_order fallback consistent.
pub const EQS_PANEL: NodeId = hash_node_id("panel.equalize_sizes");

/// Image Tools action — Color Equalization pill. Stateful tool: opens
/// the right-docked panel with 5 slider+chip rows (clip limit, tile
/// grid size, brightness, contrast, saturation), an Auto-WB toggle,
/// and Cancel/Apply. Pipeline (CPU, zero-deps): CLAHE (Zuiderveld
/// 1994), then brightness/contrast/saturation in linear sRGB, then
/// optional Gray-World auto-WB. Click raises `EditorAction::ActivateTool
/// { tool_id: "color_equalization" }`; Apply pushes one
/// `EditorAction::OneShotImageOp { tool_id: "color_equalization",
/// entity_bits }` per selected sprite, and the shell drain reads each
/// sprite's source then bakes via the tool's `run_full_resolution`.
pub const IMAGE_ACTION_COLOR_EQUALIZATION: NodeId = hash_node_id("color_equalization");

/// Image Tools action — Equalize Sizes pill. Stateful, multi-sprite:
/// opens the right-docked panel with target-mode radio (Max/Fixed/Grid),
/// per-mode chips / slider, Upscale-if-smaller + algorithm radio,
/// Rasterize-after toggle, Cancel/Apply. Click raises
/// `EditorAction::ActivateTool { tool_id: "equalize_sizes" }`; Apply
/// arms the tool's latch which the bridge drains into a single
/// `run_full_resolution_multi` over `hero.gizmo.iter_selected()` (the
/// only cross-sprite Image Tool — Max/Grid modes compute global
/// targets, so per-sprite `OneShotImageOp` broadcast won't work here).
pub const IMAGE_ACTION_EQUALIZE_SIZES: NodeId = hash_node_id("equalize_sizes");

/// Image Tools action — Rasterize pill. One-shot: bakes the sprite's
/// active Transform (scale + rotation + flip) into the source pixel
/// buffer and resets `Transform.scale = (1,1)` / `rotation = 0`. Click
/// raises one `EditorAction::OneShotImageOp { tool_id: "rasterize",
/// entity_bits }` per selected sprite; the shell drain calls
/// `ph2d_tool_rasterize::rasterize`, commits via
/// `texture_edit::commit_edited_texture`, then writes the identity
/// Transform.
pub const IMAGE_ACTION_RASTERIZE: NodeId = hash_node_id("rasterize");

/// Upscale panel marker NodeId. Right-docked in the Inspector geometry
/// slot while the `upscale` tool is active. Hash matches
/// `ph2d_tool_upscale::tool::ids` namespace and
/// `ph2d_panel_upscale::UpscalePanel::NODE_ID` — keeping the
/// dispatcher's `panel_at` lookup, the typed panel registry, and
/// `paint_hero_screen`'s z_order fallback consistent.
pub const UPS_PANEL: NodeId = hash_node_id("panel.upscale");

/// Image Tools action — Upscale pill. Stateful, sabor 3: opens the
/// right-docked panel with a 3-way algorithm radio (Lanczos3 / Nearest
/// / xBR), a 1×–16× scale slider paired with a number chip, and
/// Cancel / Apply. Click raises `EditorAction::ActivateTool { tool_id:
/// "upscale" }`; Apply arms the tool's latch, and the bridge drains
/// it per-sprite via `UpscaleTool::run_full_resolution`.
pub const IMAGE_ACTION_UPSCALE: NodeId = hash_node_id("upscale");

/// Image Tools action — Painter pill. Stateful workhorse (sucessor do
/// Procreate, cascata W0 ratificada 2026-05-26 — ADRs 0043..0053). Click
/// raises `EditorAction::ActivateTool { tool_id: "painter" }`; activation
/// installs the takeover model (suprime chrome PH2D normal + instala
/// chrome Procreate-style + sidebar — vide ADR-0043 §1.1).
pub const IMAGE_ACTION_PAINTER: NodeId = hash_node_id("painter");

/// Padding panel widget NodeIds (typed `ph2d-panel-padding`, right-docked
/// in the Inspector slot while the `padding` tool is active). Four signed
/// per-edge rows — each a bipolar Slider (`PAD_*`) linked in real time to
/// a px-valued NumberInput chip (`PAD_*_NUM`) — plus a pivot-mode toggle
/// with Apply / Cancel. The directional-expand gizmo (v2) will add
/// canvas-edge handles, not panel ids.
pub const PAD_PANEL: NodeId = hash_node_id("pad_panel");
pub const PAD_TOP: NodeId = hash_node_id("pad_top");
pub const PAD_RIGHT: NodeId = hash_node_id("pad_right");
pub const PAD_BOTTOM: NodeId = hash_node_id("pad_bottom");
pub const PAD_LEFT: NodeId = hash_node_id("pad_left");
/// Per-edge px chips paired with the `PAD_*` sliders (real-time link).
pub const PAD_TOP_NUM: NodeId = hash_node_id("pad_top_num");
pub const PAD_RIGHT_NUM: NodeId = hash_node_id("pad_right_num");
pub const PAD_BOTTOM_NUM: NodeId = hash_node_id("pad_bottom_num");
pub const PAD_LEFT_NUM: NodeId = hash_node_id("pad_left_num");
/// Pivot-mode toggle: ON = recenter (recalculate translation so the
/// original content stays world-fixed); OFF = keep the pivot unchanged.
pub const PAD_PIVOT_RECENTER: NodeId = hash_node_id("pad_pivot_recenter");
pub const PAD_APPLY: NodeId = hash_node_id("pad_apply");
pub const PAD_CANCEL: NodeId = hash_node_id("pad_cancel");
/// Reset-all button — returns every per-edge padding back to 0 and
/// the pivot-recenter toggle to its default. Also fires from the
/// tool's `on_activate`.
pub const PAD_RESET: NodeId = hash_node_id("pad_reset");

pub const HIERARCHY_ADD: NodeId = hash_node_id("hierarchy_add");

pub const TOOL_TRANSLATE: NodeId = hash_node_id("tool_translate");
pub const TOOL_ROTATE: NodeId = hash_node_id("tool_rotate");
pub const TOOL_SCALE: NodeId = hash_node_id("tool_scale");
pub const TOOL_PIVOT: NodeId = hash_node_id("tool_pivot");
pub const TOOL_SPACE: NodeId = hash_node_id("tool_space");
pub const TOOL_PROJECTION: NodeId = hash_node_id("tool_projection");
pub const TOOL_HOME: NodeId = hash_node_id("tool_home");
pub const TOOL_UNDO: NodeId = hash_node_id("tool_undo");
pub const TOOL_REDO: NodeId = hash_node_id("tool_redo");
/// Show/Hide toggles for the side panels — top of the left rail.
/// `Pressed` state == panel currently visible.
pub const RAIL_SHOW_INSPECTOR: NodeId = hash_node_id("rail_show_inspector");
pub const RAIL_SHOW_HIERARCHY: NodeId = hash_node_id("rail_show_hierarchy");
/// Frosted-glass backdrop of the side rail. Painted before the
/// chips so chip clicks win; clicks on the rail's empty space
/// (between chips, around dividers) land here.
pub const RAIL_BACKDROP: NodeId = hash_node_id("rail_backdrop");

/// Painter sidebar panel container — the typed `ph2d-panel-painter-sidebar`
/// outer rect. Right-docked (same geometry slot as the Inspector) and
/// only visible while the `painter` tool is active. W2.T2.1 plan §5.
pub const PAINTER_SIDEBAR_PANEL: NodeId = hash_node_id("painter_sidebar_panel");
/// Size slider (size_px) no Painter sidebar (normalizado 0..1).
pub const PAINTER_SIDEBAR_SIZE_SLIDER: NodeId = hash_node_id("painter_sidebar.size_slider");
/// Chip numeric do size_slider (link via store.link_slider_number).
pub const PAINTER_SIDEBAR_SIZE_CHIP: NodeId = hash_node_id("painter_sidebar.size_chip");
/// Opacity slider no Painter sidebar (normalizado 0..1).
pub const PAINTER_SIDEBAR_OPACITY_SLIDER: NodeId = hash_node_id("painter_sidebar.opacity_slider");
/// Chip numeric do opacity_slider.
pub const PAINTER_SIDEBAR_OPACITY_CHIP: NodeId = hash_node_id("painter_sidebar.opacity_chip");
/// Botão Undo (sidebar). 2-finger gesture também dispara.
pub const PAINTER_SIDEBAR_UNDO_BUTTON: NodeId = hash_node_id("painter_sidebar.undo_button");
/// Botão Redo (sidebar). 3-finger gesture também dispara.
pub const PAINTER_SIDEBAR_REDO_BUTTON: NodeId = hash_node_id("painter_sidebar.redo_button");
/// Modifier square (centro da sidebar) — W2 default eyedropper-while-held.
pub const PAINTER_SIDEBAR_MODIFIER_SQUARE: NodeId = hash_node_id("painter_sidebar.modifier_square");
/// Close (X) button do Painter sidebar — routes pra `CancelActiveTool`
/// (canon BgRemoval/Padding). Deactivates Painter tool quando clicado.
pub const PAINTER_SIDEBAR_CLOSE: NodeId = hash_node_id("painter_sidebar.close");
/// Floating color thumb painted in the canvas top-right while the
/// Painter tool is active (W2.T2.3). Clicking it opens the shared
/// `INSP_BLENDER_PICKER` seeded with the Painter's active color; the
/// chosen color is applied back via `PainterUiEdit::SetColorSrgb`.
pub const PAINTER_COLOR_THUMB: NodeId = hash_node_id("painter.color_thumb");

/// Painter layers panel container — the typed `ph2d-panel-painter-layers`
/// outer rect. Right-docked (same geometry slot as the Inspector, mirror
/// do `PAINTER_SIDEBAR_PANEL`) and only visible while the `painter` tool
/// is active. W3.T3.4 plan §6 / design 02_layers.md §2.3.
pub const PAINTER_LAYERS_PANEL: NodeId = hash_node_id("painter_layers_panel");
/// Close (X) button do Painter layers panel — routes pra `CancelActiveTool`
/// (canon BgRemoval/Painter sidebar). Deactivates Painter tool quando clicado.
pub const PAINTER_LAYERS_CLOSE: NodeId = hash_node_id("painter_layers.close");
/// "+ Layer" button no rodapé do body do Painter layers panel. Click →
/// `PainterTool::add_raster_layer` (cria + ativa uma raster transparente no
/// topo). W3.T3.4 UI-plumbing.
pub const PAINTER_LAYERS_ADD: NodeId = hash_node_id("painter_layers.add");
/// Header "Duplicate" icon-button — Click → `PainterTool::duplicate_layer`
/// (clones the active raster + its pixels above itself). W3.T3.5 header batch.
pub const PAINTER_LAYERS_DUPLICATE: NodeId = hash_node_id("painter_layers.duplicate");
/// Header "Delete" icon-button — Click → `PainterTool::delete_layer` (removes
/// the active layer + its subtree/mask; the base sprite is not removable).
pub const PAINTER_LAYERS_DELETE: NodeId = hash_node_id("painter_layers.delete");
/// Header "Group" icon-button — Click → `PainterTool::add_group` (empty group
/// at the top; the user nests layers into it). W3.T3.7.
pub const PAINTER_LAYERS_GROUP: NodeId = hash_node_id("painter_layers.group");
/// Modifier toolbar "Mask" — Click → `PainterTool::add_mask_to_active` (creates
/// a grayscale mask on the active raster + makes it the edit target). W3.T3.5.
pub const PAINTER_LAYERS_MASK: NodeId = hash_node_id("painter_layers.mask");
/// Modifier toolbar "Clip" toggle — flips the active layer's clipping-mask
/// modifier (§2.8). W3.T3.6.
pub const PAINTER_LAYERS_CLIP: NodeId = hash_node_id("painter_layers.clip");
/// Modifier toolbar "Lock" toggle — flips the active layer's alpha-lock
/// modifier (§2.10). W3.T3.7.
pub const PAINTER_LAYERS_ALPHA_LOCK: NodeId = hash_node_id("painter_layers.alpha_lock");
/// Modifier toolbar "Ref" toggle — flips the active layer's reference modifier
/// (§2.9, exclusive). W3.T3.7.
pub const PAINTER_LAYERS_REFERENCE: NodeId = hash_node_id("painter_layers.reference");
/// Action toolbar "+ Adj" — creates a non-destructive adjustment layer (W4
/// T4.3; HSB for the Day-4 smoke, full 24-kind menu lands with T4.15).
pub const PAINTER_LAYERS_ADD_ADJUSTMENT: NodeId = hash_node_id("painter_layers.add_adjustment");
/// Dock-mode toggle no header do Painter **layers** panel — alterna o slot
/// docado de volta pra brush-settings (mostra "Brush"). Enio escolheu o modo
/// C = toggle (um slot, dois painéis). Estado vive no `PainterTool`
/// (`dock_shows_layers`); o bridge lê e computa a visibilidade.
pub const PAINTER_LAYERS_TOGGLE_DOCK: NodeId = hash_node_id("painter_layers.toggle_dock");
/// Dock-mode toggle no header do Painter **sidebar** (brush) panel — alterna o
/// slot docado pra layers (mostra "Layers"). Mirror simétrico de
/// [`PAINTER_LAYERS_TOGGLE_DOCK`].
pub const PAINTER_SIDEBAR_TOGGLE_DOCK: NodeId = hash_node_id("painter_sidebar.toggle_dock");
/// "Apply" CTA (shared by both painter panels) — commits the live layer
/// composite into the active sprite. Routes `PanelEvent::Click` →
/// `PainterTool::request_commit` (the bridge bakes via `run_full` next frame).
/// Without it the only way to commit was the invisible Cmd/Ctrl+Enter shortcut.
pub const PAINTER_APPLY: NodeId = hash_node_id("painter.apply");

/// Per-layer-row interactive widget kind, used to derive a stable, collision-
/// safe [`NodeId`] for each control painted on a Painter layers-panel row via
/// [`painter_layer_widget_id`]. The id is hash-derived (FNV) from the layer's
/// runtime id + the kind tag, so the panel (paint/event) and the tool
/// (`handle_panel_event`) agree on the id without sharing any per-row id table
/// — the decoder simply iterates `layers × kinds`, recomputes the id, and
/// matches. Mirror in spirit of the `hier_*_companion` per-row ids, but
/// hash-based (no companion-bit dispatcher allowlist needed, since the panel
/// registers these in the `WidgetStore` itself during `paint`).
///
/// `layer_id` is passed as a raw `u64` (the `LayerId`/`RtLayerId` newtype's
/// inner value) so this stays in `ph2d-editor-core` without a dependency edge
/// to `ph2d-tool-painter` (which would be a cycle).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PainterLayerWidget {
    /// The row body — click selects (activates) the layer.
    Row,
    /// The eye toggle — click flips the layer's visibility.
    Visibility,
    /// The opacity slider (stores `0..1`).
    Opacity,
    /// The opacity numeric chip paired with [`Self::Opacity`].
    OpacityChip,
    /// The blend-mode dropdown chip (opens the blend popover).
    Blend,
    /// The move-up (↑) reorder button — moves the layer toward the front/top.
    MoveUp,
    /// The move-down (↓) reorder button — moves the layer toward the back.
    MoveDown,
    /// (Mask rows only) the Invert toggle — flips the mask's `inverted` flag
    /// (§2.7); the compositor already honors it (`1 - value`).
    MaskInvert,
    /// (Mask rows only) the Apply button — destructively bakes the mask into
    /// the parent layer's alpha and removes the mask (§2.7).
    MaskApply,
    /// (Adjustment rows only) generic slider for the adjustment's Nth slider
    /// param (W4 T4.3+). The kind decides what each slot means
    /// (`ph2d_painter_brush::adjustments::adjustment_slider_params`); 6 slots
    /// cover the slider-heavy kinds (bespoke controls — Curves etc. — are
    /// separate). Indexed instead of per-param-named so adding a kind needs no
    /// new widget id.
    AdjParam0,
    AdjParam1,
    AdjParam2,
    AdjParam3,
    AdjParam4,
    AdjParam5,
}

impl PainterLayerWidget {
    /// Stable tag woven into the hashed id string. Changing a tag changes
    /// every derived id for that kind — keep stable.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Visibility => "vis",
            Self::Opacity => "opacity",
            Self::OpacityChip => "opacity_chip",
            Self::Blend => "blend",
            Self::MoveUp => "move_up",
            Self::MoveDown => "move_down",
            Self::MaskInvert => "mask_invert",
            Self::MaskApply => "mask_apply",
            Self::AdjParam0 => "adj_param0",
            Self::AdjParam1 => "adj_param1",
            Self::AdjParam2 => "adj_param2",
            Self::AdjParam3 => "adj_param3",
            Self::AdjParam4 => "adj_param4",
            Self::AdjParam5 => "adj_param5",
        }
    }

    /// All kinds, in a fixed order — the decoder iterates this.
    pub const ALL: [PainterLayerWidget; 15] = [
        Self::Row,
        Self::Visibility,
        Self::Opacity,
        Self::OpacityChip,
        Self::Blend,
        Self::MoveUp,
        Self::MoveDown,
        Self::MaskInvert,
        Self::MaskApply,
        Self::AdjParam0,
        Self::AdjParam1,
        Self::AdjParam2,
        Self::AdjParam3,
        Self::AdjParam4,
        Self::AdjParam5,
    ];
}

/// Runtime FNV-1a 64-bit over `s`, byte-identical to the `const fn`
/// [`ph2d_tool_registry::hash_node_id`] (which only accepts `&'static str`).
/// Needed because the per-row layer ids below are derived from a runtime
/// `format!` (the layer id is only known at runtime). Kept here, private to
/// the additive per-row helpers, so the hashing stays consistent with the rest
/// of the id space (same offset basis / prime / `NodeId(0)` bump).
fn fnv_node_id_runtime(s: &str) -> NodeId {
    const FNV_OFFSET_BASIS_64: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME_64: u64 = 0x0000_0100_0000_01b3;
    let mut hash: u64 = FNV_OFFSET_BASIS_64;
    for &b in s.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    if hash == 0 {
        hash = 1; // reserve NodeId(0) = a11y root, mirror of hash_node_id
    }
    NodeId(hash)
}

/// Derive the stable [`NodeId`] for the `kind` control on the Painter
/// layers-panel row whose layer has runtime id `layer_id`. FNV-hashed from
/// `"painter_layer.<kind>.<layer_id>"`. Runtime `format!` is acceptable here:
/// the layers panel is not a hot path (≤8 layers, repainted per frame like the
/// sidebar formats "NN px"). See [`PainterLayerWidget`].
#[must_use]
pub fn painter_layer_widget_id(layer_id: u64, kind: PainterLayerWidget) -> NodeId {
    fnv_node_id_runtime(&format!("painter_layer.{}.{}", kind.tag(), layer_id))
}

/// Derive the stable [`NodeId`] for blend-mode option `mode` (the
/// [`BlendMode`](ph2d_painter_brush) wire discriminant, `0..MAX_BLEND_MODES`)
/// in the open blend dropdown popover of the row whose layer has runtime id
/// `layer_id`. Only the single open popover's options are ever hit-registered,
/// so the `format!` cost is bounded.
#[must_use]
pub fn painter_layer_blend_option_id(layer_id: u64, mode: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_layer.blendopt.{layer_id}.{mode}"))
}

/// Derive the stable [`NodeId`] for the `index`-th kind option in the open
/// "+ Adjustment" kind-picker popover (W4 T4.15). The index is the position in
/// `AdjustmentKind::ALL` (the wire value the panel forwards back to the tool's
/// `add_adjustment_layer`). Fixed (not per-layer) like the toolbar buttons, but
/// derived so the panel paint/event and the popover stay in sync without a table.
/// Only the open popover's options are hit-registered, so the `format!` is bounded.
#[must_use]
pub fn painter_adjustment_kind_option_id(index: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_layers.adjkind.{index}"))
}

/// Background-Removal panel container — the typed `ph2d-panel-bgremoval`
/// outer rect. Right-docked (same geometry slot as the Inspector) and
/// only visible while the `bgremoval` tool is active.
pub const BGR_PANEL: NodeId = hash_node_id("bgr_panel");
/// Mode segmented control — "Chroma" half.
pub const BGR_MODE_CHROMA: NodeId = hash_node_id("bgr_mode_chroma");
/// Mode segmented control — "Smart Cut" half.
pub const BGR_MODE_GRABCUT: NodeId = hash_node_id("bgr_mode_grabcut");
/// Tolerance slider (0..1 → ΔE 0..0.30 Oklab).
pub const BGR_TOLERANCE: NodeId = hash_node_id("bgr_tolerance");
/// Feather slider (0..1 → soft-band 0..0.20 Oklab).
pub const BGR_FEATHER: NodeId = hash_node_id("bgr_feather");
/// Refine slider (0..1 → guided-filter radius 0..100 px).
pub const BGR_REFINE: NodeId = hash_node_id("bgr_refine");
/// Grow/Shrink slider (bipolar; 0.5 = neutral, <0.5 erodes the matte to
/// eat residual background outline, >0.5 dilates it).
pub const BGR_GROW: NodeId = hash_node_id("bgr_grow");
/// Editable numeric chips (NumberInput) paired with the sliders above —
/// keyboard + drag-scrub edit the normalized 0..1 value.
pub const BGR_TOLERANCE_NUM: NodeId = hash_node_id("bgr_tolerance_num");
pub const BGR_FEATHER_NUM: NodeId = hash_node_id("bgr_feather_num");
pub const BGR_REFINE_NUM: NodeId = hash_node_id("bgr_refine_num");
pub const BGR_GROW_NUM: NodeId = hash_node_id("bgr_grow_num");
/// Apply button — commits the removal at full resolution.
pub const BGR_APPLY: NodeId = hash_node_id("bgr_apply");
/// Reset-all button — returns every param to its default in one click.
/// The tool also runs this from `on_activate` so reopening the panel
/// never inherits a previous session's slider/toggle state.
pub const BGR_RESET: NodeId = hash_node_id("bgr_reset");
/// Cancel button — abandons the preview and deactivates the tool
/// (returns to the Inspector).
pub const BGR_CANCEL: NodeId = hash_node_id("bgr_cancel");
/// Eyedropper toggle — when armed, click-drag over the sprite on the
/// canvas samples extra background colours into the swatch row below
/// the sliders. Right-click a swatch to delete it.
pub const BGR_EYEDROPPER: NodeId = hash_node_id("bgr_eyedropper");
/// Protection-brush toggle — when armed, click-drag over the sprite on
/// the canvas paints a freehand "keep" mask: every painted pixel is
/// forced foreground (never removed) in BOTH modes (Chroma force-keep /
/// Smart Cut `FgHard` trimap lock).
pub const BGR_PROTECT: NodeId = hash_node_id("bgr_protect");
/// Clear-protection button — wipes the painted protection mask.
pub const BGR_PROTECT_CLEAR: NodeId = hash_node_id("bgr_protect_clear");
/// "Add area" toggle — when armed, a single click on the canvas runs a
/// flood-fill from the clicked source pixel, expanding to every
/// 4-connected neighbour whose RGB is within a threshold of the seed,
/// and marks the connected region in the FORCE-REMOVE mask: those
/// pixels are forced to alpha=0 in the final compose, overriding both
/// the silhouette auto-protect AND the user's protect-brush mask.
/// Shown in the eyedropper row's slot ONLY when `auto_protect_subject`
/// is on (Pick Colors doesn't apply to the silhouette path, so the slot
/// is repurposed for this automatic destructive selector — Enio
/// 2026-05-26). Symmetric to the eyedropper: arm → single click → done.
pub const BGR_ADD_AREA: NodeId = hash_node_id("bgr_add_area");
/// Clear button for the force-remove mask — mirror of `BGR_PROTECT_CLEAR`.
pub const BGR_ADD_AREA_CLEAR: NodeId = hash_node_id("bgr_add_area_clear");
/// Show-mask toggle — shows/hides the on-canvas protection-mask overlay
/// tint (so the user can preview the clean result without the tint, or
/// turn it back on to keep painting).
pub const BGR_SHOW_MASK: NodeId = hash_node_id("bgr_show_mask");
/// Protection-brush size slider (0..1 → brush radius in source px) +
/// its editable numeric chip. Drives the canvas brush-size gizmo ring.
pub const BGR_BRUSH_SIZE: NodeId = hash_node_id("bgr_brush_size");
pub const BGR_BRUSH_SIZE_NUM: NodeId = hash_node_id("bgr_brush_size_num");
/// Protection-brush falloff profile — 4-option segmented control
/// (mirrors the Mode segmented group). Shapes the painted dab's
/// strength from centre (255) to edge (0): Constant = hard disc,
/// Smooth = smoothstep, Sphere = sqrt(1−d²), Sharp = concentrated peak.
pub const BGR_FALLOFF_SMOOTH: NodeId = hash_node_id("bgr_falloff_smooth");
pub const BGR_FALLOFF_SPHERE: NodeId = hash_node_id("bgr_falloff_sphere");
pub const BGR_FALLOFF_SHARP: NodeId = hash_node_id("bgr_falloff_sharp");
pub const BGR_FALLOFF_CONSTANT: NodeId = hash_node_id("bgr_falloff_constant");
/// Extra-colour swatch hit slots 0..11. Painted only when the
/// corresponding extra colour exists (a fixed pool, like the Blender
/// palette's `BLENDER_SWATCH_*`). Capacity matches
/// `ph2d_tool_bgremoval::params::MAX_EXTRA_BG_COLORS`. Right-clicking a
/// painted slot removes that colour.
pub const BGR_SWATCH_0: NodeId = hash_node_id("bgr_swatch_0");
pub const BGR_SWATCH_1: NodeId = hash_node_id("bgr_swatch_1");
pub const BGR_SWATCH_2: NodeId = hash_node_id("bgr_swatch_2");
pub const BGR_SWATCH_3: NodeId = hash_node_id("bgr_swatch_3");
pub const BGR_SWATCH_4: NodeId = hash_node_id("bgr_swatch_4");
pub const BGR_SWATCH_5: NodeId = hash_node_id("bgr_swatch_5");
pub const BGR_SWATCH_6: NodeId = hash_node_id("bgr_swatch_6");
pub const BGR_SWATCH_7: NodeId = hash_node_id("bgr_swatch_7");
pub const BGR_SWATCH_8: NodeId = hash_node_id("bgr_swatch_8");
pub const BGR_SWATCH_9: NodeId = hash_node_id("bgr_swatch_9");
pub const BGR_SWATCH_10: NodeId = hash_node_id("bgr_swatch_10");
pub const BGR_SWATCH_11: NodeId = hash_node_id("bgr_swatch_11");

/// Fixed-pool extra-colour swatch ids, indexed 0..11.
pub const BGR_SWATCHES: [NodeId; 12] = [
    BGR_SWATCH_0,
    BGR_SWATCH_1,
    BGR_SWATCH_2,
    BGR_SWATCH_3,
    BGR_SWATCH_4,
    BGR_SWATCH_5,
    BGR_SWATCH_6,
    BGR_SWATCH_7,
    BGR_SWATCH_8,
    BGR_SWATCH_9,
    BGR_SWATCH_10,
    BGR_SWATCH_11,
];

/// Recover the extra-colour swatch index `0..12` from a `NodeId` when
/// it matches one of the [`BGR_SWATCHES`] pool consts. Used by the
/// shell's right-click-delete dispatch to map a hit id → list index.
pub fn bgr_swatch_index(id: NodeId) -> Option<usize> {
    BGR_SWATCHES.iter().position(|&s| s == id)
}

/// Inspector panel container — used as the wheel-scroll key.
pub const INSP_PANEL: NodeId = hash_node_id("insp_panel");
/// Close (X) button at the top-right of the Inspector — toggles
/// `panel_visibility["inspector"]` same as the left-rail Inspector
/// pill. UI canon post-2026-05-24: every floating panel except
/// Hierarchy carries a close X.
pub const INSP_CLOSE: NodeId = hash_node_id("insp_close");
/// Drag handle at the top of the Inspector — click+drag moves the
/// panel. Registered as `BlenderHit { parent: INSP_PANEL, kind:
/// DragHandle }` so the existing picker-drag dispatch infra
/// (panel-agnostic on parent NodeId) drives it.
pub const INSP_DRAG_HANDLE: NodeId = hash_node_id("insp_drag_handle");
/// Resize gripper at the Inspector's bottom-right corner. Registered
/// as `BlenderHit { parent: INSP_PANEL, kind: ResizeHandle }`.
pub const INSP_RESIZE_HANDLE: NodeId = hash_node_id("insp_resize_handle");
/// Resize gripper at the Inspector's bottom-LEFT corner. Mirror of
/// [`INSP_RESIZE_HANDLE`]. Registered as
/// `BlenderHit { parent: INSP_PANEL, kind: ResizeHandleBl }`.
pub const INSP_RESIZE_HANDLE_BL: NodeId = hash_node_id("insp_resize_handle_bl");

#[cfg(test)]
mod tests {
    use super::*;

    /// W3 audit-2 B.2: [`fnv_node_id_runtime`] is a hand-copied twin of
    /// [`ph2d_tool_registry::hash_node_id`] (same offset basis / prime /
    /// `NodeId(0)` bump) used to derive the per-row painter widget ids. Editing
    /// one twin's basis or prime would silently diverge the runtime ids from the
    /// const id space, misrouting clicks. Pin the twin agreement + the
    /// empty-string FNV-1a-64 offset basis (the value `hash_node_id("")` also
    /// returns; `!= 0`, so it never shadows `NodeId(0)` = a11y root).
    #[test]
    fn fnv_node_id_runtime_agrees_with_hash_node_id() {
        assert_eq!(
            fnv_node_id_runtime("").0,
            0xcbf2_9ce4_8422_2325,
            "empty-string hash must equal the FNV-1a-64 offset basis",
        );
        assert_eq!(fnv_node_id_runtime("").0, hash_node_id("").0);

        for s in [
            "a",
            "painter_layer.row.0",
            "painter_layer.blend.42",
            "painter_layer.blendopt.7.3",
            "painter_layers_panel",
            "the quick brown fox",
            "x",
        ] {
            assert_eq!(
                fnv_node_id_runtime(s).0,
                hash_node_id(s).0,
                "fnv_node_id_runtime / hash_node_id diverged on {s:?}",
            );
        }
    }
}

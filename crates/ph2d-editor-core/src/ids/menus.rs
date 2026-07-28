use super::*;

// ── Context menu item ids ──────────────────────────────────────────────────
// The right-click context menu reuses these stable ids across both
// inspector and hierarchy. Click dispatch routes by id to the
// inspector's `apply_event`.
pub const CTX_MENU_CREATE_NOTE: NodeId = hash_node_id("ctx_menu_create_note");
pub const CTX_MENU_OUTLINE_NONE: NodeId = hash_node_id("ctx_menu_outline_none");
pub const CTX_MENU_OUTLINE_0: NodeId = hash_node_id("ctx_menu_outline_0");
pub const CTX_MENU_OUTLINE_1: NodeId = hash_node_id("ctx_menu_outline_1");
pub const CTX_MENU_OUTLINE_2: NodeId = hash_node_id("ctx_menu_outline_2");
pub const CTX_MENU_OUTLINE_3: NodeId = hash_node_id("ctx_menu_outline_3");
pub const CTX_MENU_OUTLINE_4: NodeId = hash_node_id("ctx_menu_outline_4");
// Theme selector menu items — opened by clicking TOPBAR_THEME.
pub const CTX_MENU_THEME_FORGE: NodeId = hash_node_id("ctx_menu_theme_forge");
pub const CTX_MENU_THEME_PAINT: NodeId = hash_node_id("ctx_menu_theme_paint");
pub const CTX_MENU_THEME_SUNSTONE: NodeId = hash_node_id("ctx_menu_theme_sunstone");
pub const CTX_MENU_THEME_BLUEPRINT: NodeId = hash_node_id("ctx_menu_theme_blueprint");
// Corner-radius scale presets — also exposed via the theme menu.
pub const CTX_MENU_RADIUS_SHARP: NodeId = hash_node_id("ctx_menu_radius_sharp");
pub const CTX_MENU_RADIUS_DEFAULT: NodeId = hash_node_id("ctx_menu_radius_default");
pub const CTX_MENU_RADIUS_ROUND: NodeId = hash_node_id("ctx_menu_radius_round");
// Rail-button size presets — also exposed via the theme menu
// (added 2026-05-24). Default is Small (the new canon); Large is the
// pre-2026-05-24 size; Medium is the halfway point.
pub const CTX_MENU_RAIL_SIZE_SMALL: NodeId = hash_node_id("ctx_menu_rail_size_small");
pub const CTX_MENU_RAIL_SIZE_MEDIUM: NodeId = hash_node_id("ctx_menu_rail_size_medium");
pub const CTX_MENU_RAIL_SIZE_LARGE: NodeId = hash_node_id("ctx_menu_rail_size_large");
/// "Mirror UI" entry in the theme menu — toggles
/// `HeroScreen::ui_mirrored`, which swaps Hierarchy ↔ Inspector
/// horizontally.
pub const CTX_MENU_MIRROR_UI: NodeId = hash_node_id("ctx_menu_mirror_ui");
/// "Show Statistics" entry in the theme menu — toggles
/// `HeroScreen::stats_visible`, which gates the bottom HUD.
pub const CTX_MENU_SHOW_STATS: NodeId = hash_node_id("ctx_menu_show_stats");
/// "Show Grid" entry in the theme menu — toggles
/// `HeroScreen::grid_visible`, which gates the world-space grid
/// overlay (ADR-0025 M14.4b).
pub const CTX_MENU_SHOW_GRID: NodeId = hash_node_id("ctx_menu_show_grid");
// Save-button context menu (Save / Save As).
pub const CTX_MENU_SAVE: NodeId = hash_node_id("ctx_menu_save");
pub const CTX_MENU_SAVE_AS: NodeId = hash_node_id("ctx_menu_save_as");
// Open-button context menu (Open Project / Import…).
pub const CTX_MENU_OPEN_PROJECT: NodeId = hash_node_id("ctx_menu_open_project");
pub const CTX_MENU_IMPORT: NodeId = hash_node_id("ctx_menu_import");

// Pixels-per-meter presets — opened from the Settings cluster (gear).
// Drives `HeroScreen.project.pixels_per_meter`. The values are the
// canonical presets surfaced as labels in `SettingsMenu`.
//
// Pre-PR-11.3 these sat at hand-picked integers 940..944 to dodge
// the SceneList popover rows (CTX_SCENE_ROW_*) — exactly the type
// of cross-cluster collision the M14.4d audit caught when an early
// draft reused 930..934. Now hash-derived; the regression test in
// `tests/architecture/node_id_collisions.rs` catches reuse mechanically.
pub const CTX_MENU_PPM_16: NodeId = hash_node_id("ctx_menu_ppm_16");
pub const CTX_MENU_PPM_32: NodeId = hash_node_id("ctx_menu_ppm_32");
pub const CTX_MENU_PPM_100: NodeId = hash_node_id("ctx_menu_ppm_100");
pub const CTX_MENU_PPM_256: NodeId = hash_node_id("ctx_menu_ppm_256");
pub const CTX_MENU_PPM_1024: NodeId = hash_node_id("ctx_menu_ppm_1024");

/// M14.7 polish (6.3): top-level Settings cascade entry that opens
/// the Pixels-per-meter submenu.
pub const CTX_MENU_SETTINGS_PPM: NodeId = hash_node_id("ctx_menu_settings_ppm");

/// Top-level Settings entry that opens the Display-unit submenu
/// (Meters / Pixels). Companion of `CTX_MENU_SETTINGS_PPM`.
pub const CTX_MENU_SETTINGS_UNIT: NodeId = hash_node_id("ctx_menu_settings_unit");
pub const CTX_MENU_UNIT_METERS: NodeId = hash_node_id("ctx_menu_unit_meters");
pub const CTX_MENU_UNIT_PIXELS: NodeId = hash_node_id("ctx_menu_unit_pixels");

/// Top-level Settings entry that opens the Image-filter submenu
/// (Pixel Art / Smooth). Companion of `CTX_MENU_SETTINGS_UNIT`.
/// Selecting a mode flips the app-wide `ImageFilterMode` — the single
/// sampler/quality applied to every sprite + the Vello preview.
pub const CTX_MENU_SETTINGS_FILTER: NodeId = hash_node_id("ctx_menu_settings_filter");
pub const CTX_MENU_FILTER_PIXELART: NodeId = hash_node_id("ctx_menu_filter_pixelart");
pub const CTX_MENU_FILTER_SMOOTH: NodeId = hash_node_id("ctx_menu_filter_smooth");
/// Top-level Settings entry that opens the Display submenu (present
/// mode). Selecting a mode switches the swap-chain present mode at
/// runtime: VSync (`Fifo`, smooth motion) vs Immediate (non-blocking,
/// no mouse-stutter — the M5-demo-continuous-render tradeoff).
pub const CTX_MENU_SETTINGS_DISPLAY: NodeId = hash_node_id("ctx_menu_settings_display");
pub const CTX_MENU_DISPLAY_VSYNC: NodeId = hash_node_id("ctx_menu_display_vsync");
pub const CTX_MENU_DISPLAY_IMMEDIATE: NodeId = hash_node_id("ctx_menu_display_immediate");
/// Top-level Settings entry that opens the Text rendering submenu —
/// toggle entre `Default` (histórico) e `Crisp Heavy` (ExtraBold +
/// snap-X + hint=false). Os presets intermediários (Crisp Light, Crisp)
/// foram removidos em 2026-05-25 por serem visualmente equivalentes
/// — vide `docs/UI_Fonts/2026-05-25-crisp-heavy-implementation.md`.
pub const CTX_MENU_SETTINGS_TEXT: NodeId = hash_node_id("ctx_menu_settings_text");
pub const CTX_MENU_TEXT_DEFAULT: NodeId = hash_node_id("ctx_menu_text_default");
pub const CTX_MENU_TEXT_CRISP_HEAVY: NodeId = hash_node_id("ctx_menu_text_crisp_heavy");
pub const CTX_MENU_TEXT_CRISP_HEAVY_PLUS: NodeId = hash_node_id("ctx_menu_text_crisp_heavy_plus");

/// Color-picker palette rename modal: the "Rename" button that commits the shared
/// `BLENDER_PALETTE_NAME` field to the active palette (the field doubles as the dialog input).
pub const CTX_MENU_PALETTE_RENAME: NodeId = hash_node_id("ctx_menu_palette_rename");

// New-image modal (Cmd/Ctrl+N): pick a square size + a background, then Create. The size + background
// buttons are radio groups (one selected); Create raises a `(size, bg)` request the shell services
// via `spawn_blank_canvas` (Enio 2026-06-24).
pub const CTX_MENU_NEW_IMAGE_CREATE: NodeId = hash_node_id("ctx_menu_new_image_create");
pub const CTX_MENU_NEW_IMAGE_BG_TRANSPARENT: NodeId =
    hash_node_id("ctx_menu_new_image_bg_transparent");
pub const CTX_MENU_NEW_IMAGE_BG_BLACK: NodeId = hash_node_id("ctx_menu_new_image_bg_black");
pub const CTX_MENU_NEW_IMAGE_BG_WHITE: NodeId = hash_node_id("ctx_menu_new_image_bg_white");
pub const CTX_MENU_NEW_IMAGE_SIZE_32: NodeId = hash_node_id("ctx_menu_new_image_size_32");
pub const CTX_MENU_NEW_IMAGE_SIZE_64: NodeId = hash_node_id("ctx_menu_new_image_size_64");
pub const CTX_MENU_NEW_IMAGE_SIZE_128: NodeId = hash_node_id("ctx_menu_new_image_size_128");
pub const CTX_MENU_NEW_IMAGE_SIZE_256: NodeId = hash_node_id("ctx_menu_new_image_size_256");
pub const CTX_MENU_NEW_IMAGE_SIZE_512: NodeId = hash_node_id("ctx_menu_new_image_size_512");
pub const CTX_MENU_NEW_IMAGE_SIZE_1024: NodeId = hash_node_id("ctx_menu_new_image_size_1024");
pub const CTX_MENU_NEW_IMAGE_SIZE_2048: NodeId = hash_node_id("ctx_menu_new_image_size_2048");
pub const CTX_MENU_NEW_IMAGE_SIZE_4096: NodeId = hash_node_id("ctx_menu_new_image_size_4096");
/// The 8 square-size choices, paired `(px, button id)` — one source for the modal paint, the
/// populate registration, and the click dispatch.
///
/// ⚠️ **A largura do modal é DERIVADA desta contagem** (`paint_new_image_dialog`), não um literal ao
/// lado dela: o `360.0` que ela tinha vinha com o comentário *"fits the 7 size buttons"*, então
/// acrescentar o oitavo teria espremido cada botão de 45,7 para 39,5 px em silêncio — e o próximo
/// tamanho espremeria de novo. Acrescentar uma linha aqui é a única edição necessária.
pub const CTX_MENU_NEW_IMAGE_SIZES: [(u32, NodeId); 8] = [
    (32, CTX_MENU_NEW_IMAGE_SIZE_32),
    (64, CTX_MENU_NEW_IMAGE_SIZE_64),
    (128, CTX_MENU_NEW_IMAGE_SIZE_128),
    (256, CTX_MENU_NEW_IMAGE_SIZE_256),
    (512, CTX_MENU_NEW_IMAGE_SIZE_512),
    (1024, CTX_MENU_NEW_IMAGE_SIZE_1024),
    (2048, CTX_MENU_NEW_IMAGE_SIZE_2048),
    // 4096² = 64 MB de RGBA por camada (Enio 2026-07-26, "para testes"): é o tamanho em que os custos
    // canvas-proporcionais do módulo ficam visíveis a olho nu, e por isso é o que as sondas de perf
    // usam. O atlas cresce sob demanda, então não há teto novo a declarar aqui.
    (4096, CTX_MENU_NEW_IMAGE_SIZE_4096),
];
/// The 3 background choices, paired `(bg wire u8, button id)` — `0` transparent, `1` black, `2` white.
pub const CTX_MENU_NEW_IMAGE_BGS: [(u8, NodeId); 3] = [
    (0, CTX_MENU_NEW_IMAGE_BG_TRANSPARENT),
    (1, CTX_MENU_NEW_IMAGE_BG_BLACK),
    (2, CTX_MENU_NEW_IMAGE_BG_WHITE),
];

/// Hierarchy row context menu: "Use as Brush **Grain**" — load the right-clicked sprite's pixels as the
/// brush Grain (texture) image (shown only for image/sprite rows; Enio 2026-06-24). The id keeps the
/// legacy `texture` slug for wire stability; the label is "Use as Brush Grain".
pub const CTX_MENU_HIER_USE_AS_BRUSH_TEXTURE: NodeId =
    hash_node_id("ctx_menu_hier_use_as_brush_texture");

/// Hierarchy row context menu: "Use as Brush **Shape**" — load the right-clicked sprite's pixels as the
/// brush Shape (silhouette tip) image (Enio 2026-06-25; the Shape slot replaces the falloff).
pub const CTX_MENU_HIER_USE_AS_BRUSH_SHAPE: NodeId =
    hash_node_id("ctx_menu_hier_use_as_brush_shape");

/// Hierarchy row context menu: "Use as **Watercolor Paper**" — install the right-clicked layer/group's
/// pixels as the watercolor paper (Grain slot, canvas-anchored), so the wash granulates against it
/// (`docs/Painter/10…` §5). A Group tags its composited children.
pub const CTX_MENU_HIER_USE_AS_PAPER: NodeId = hash_node_id("ctx_menu_hier_use_as_paper");

/// Hierarchy row context menu: "Use as **Granulation**" — like [`CTX_MENU_HIER_USE_AS_PAPER`] but as a
/// stronger mineral-settling map (pigment pools harder in the layer's valleys).
pub const CTX_MENU_HIER_USE_AS_GRANULATION: NodeId =
    hash_node_id("ctx_menu_hier_use_as_granulation");

// M14.6 F: per-row Hierarchy context menu entries. Triggered by a
// secondary (right-button) click on any hierarchy row in live mode;
// `ContextMenuKind::HierarchyRow { row }` carries the target row's
// NodeId so dispatch can attach the action to the right entity when
// any of these ids fires.
pub const CTX_MENU_HIER_DUPLICATE: NodeId = hash_node_id("ctx_menu_hier_duplicate");
pub const CTX_MENU_HIER_DELETE: NodeId = hash_node_id("ctx_menu_hier_delete");
pub const CTX_MENU_HIER_RESET_TRANSFORM: NodeId = hash_node_id("ctx_menu_hier_reset_transform");
pub const CTX_MENU_HIER_ADD_CHILD: NodeId = hash_node_id("ctx_menu_hier_add_child");
/// M14.7 polish: per-row "Rename..." entry. Opens inline rename
/// mode (the row's name turns into a TextInput).
pub const CTX_MENU_HIER_RENAME: NodeId = hash_node_id("ctx_menu_hier_rename");
/// Enio 2026-05-27: "Merge Sprites" entry — flattens the current
/// multi-selection (≥ 2 sprites) into a single new Individual-texture
/// sprite at the union bounding box, then despawns the originals.
/// Always shown in the HierarchyRow menu; the drain handler emits a
/// toast when fewer than 2 sprites are selected (silent no-op
/// otherwise feels broken).
pub const CTX_MENU_HIER_MERGE_SPRITES: NodeId = hash_node_id("ctx_menu_hier_merge_sprites");
// Painter brush Falloff curve point handle menu (secondary-click on a control
// point). Two HandleType options (Vector / Auto); the chrome handler maps a
// click to the HandleType wire u8 in `HeroScreen.pending_falloff_point_handle`.
pub const CTX_MENU_FALLOFF_HANDLE_VECTOR: NodeId = hash_node_id("ctx_menu_falloff_handle_vector");
pub const CTX_MENU_FALLOFF_HANDLE_AUTO: NodeId = hash_node_id("ctx_menu_falloff_handle_auto");
// On-canvas Curve / Free Hand editor point handle menu (secondary-click on a control point). Four handle
// kinds (Free / Aligned / Vector / Auto); the chrome handler maps a click to the wire u8 in
// `HeroScreen.pending_curve_point_handle` (`0 = Free`, `1 = Aligned`, `2 = Vector`, `3 = Auto`).
pub const CTX_MENU_CURVE_HANDLE_FREE: NodeId = hash_node_id("ctx_menu_curve_handle_free");
pub const CTX_MENU_CURVE_HANDLE_ALIGNED: NodeId = hash_node_id("ctx_menu_curve_handle_aligned");
pub const CTX_MENU_CURVE_HANDLE_SYMMETRIC: NodeId = hash_node_id("ctx_menu_curve_handle_symmetric");
pub const CTX_MENU_CURVE_HANDLE_VECTOR: NodeId = hash_node_id("ctx_menu_curve_handle_vector");
pub const CTX_MENU_CURVE_HANDLE_AUTO: NodeId = hash_node_id("ctx_menu_curve_handle_auto");

// On-canvas motion-path anchor handle types (ADR-0141), the vector Node trio.
pub const CTX_MENU_PATH_HANDLE_CORNER: NodeId = hash_node_id("ctx_menu_path_handle_corner");
pub const CTX_MENU_PATH_HANDLE_SMOOTH: NodeId = hash_node_id("ctx_menu_path_handle_smooth");
pub const CTX_MENU_PATH_HANDLE_SYMMETRIC: NodeId = hash_node_id("ctx_menu_path_handle_symmetric");
// Project-chip Scene List popover (search input + up to 8 result rows).
pub const CTX_SCENE_SEARCH: NodeId = hash_node_id("ctx_scene_search");
pub const CTX_SCENE_ROW_0: NodeId = hash_node_id("ctx_scene_row_0");
pub const CTX_SCENE_ROW_1: NodeId = hash_node_id("ctx_scene_row_1");
pub const CTX_SCENE_ROW_2: NodeId = hash_node_id("ctx_scene_row_2");
pub const CTX_SCENE_ROW_3: NodeId = hash_node_id("ctx_scene_row_3");
pub const CTX_SCENE_ROW_4: NodeId = hash_node_id("ctx_scene_row_4");
pub const CTX_SCENE_ROW_5: NodeId = hash_node_id("ctx_scene_row_5");
pub const CTX_SCENE_ROW_6: NodeId = hash_node_id("ctx_scene_row_6");
pub const CTX_SCENE_ROW_7: NodeId = hash_node_id("ctx_scene_row_7");
pub const CTX_SCENE_ROWS: [NodeId; 8] = [
    CTX_SCENE_ROW_0,
    CTX_SCENE_ROW_1,
    CTX_SCENE_ROW_2,
    CTX_SCENE_ROW_3,
    CTX_SCENE_ROW_4,
    CTX_SCENE_ROW_5,
    CTX_SCENE_ROW_6,
    CTX_SCENE_ROW_7,
];
/// Floating `BlenderColorPicker` parent id. The picker is painted
/// over the canvas (not inside the Inspector) — the historical
/// `INSP_` prefix is kept to avoid churning every side-table key.
pub const INSP_BLENDER_PICKER: NodeId = hash_node_id("insp_blender_picker");

// BlenderColorPicker sub-control hit ids — registered by
// `color_picker_demo::paint_blender_picker_demo` every frame,
// dispatched by `dispatch_pointer` into store mutations on
// `INSP_BLENDER_PICKER`.
pub const BLENDER_WHEEL: NodeId = hash_node_id("blender_wheel");
pub const BLENDER_VALUE_SLIDER: NodeId = hash_node_id("blender_value_slider");

// BlenderColorPicker extension hit ids (range 600-699).
// Channel sliders (4 rows: R/H, G/S, B/V, A).
pub const BLENDER_CHANNEL_0: NodeId = hash_node_id("blender_channel_0");
pub const BLENDER_CHANNEL_1: NodeId = hash_node_id("blender_channel_1");
pub const BLENDER_CHANNEL_2: NodeId = hash_node_id("blender_channel_2");
pub const BLENDER_CHANNEL_3: NodeId = hash_node_id("blender_channel_3");
// Hex `#RRGGBBAA` TextInput.
pub const BLENDER_HEX: NodeId = hash_node_id("blender_hex");
// Segmented toggle ids.
pub const BLENDER_INTERP_LINEAR: NodeId = hash_node_id("blender_interp_linear");
pub const BLENDER_INTERP_PERCEPTUAL: NodeId = hash_node_id("blender_interp_perceptual");
pub const BLENDER_CHANNEL_RGB: NodeId = hash_node_id("blender_channel_rgb");
pub const BLENDER_CHANNEL_HSV: NodeId = hash_node_id("blender_channel_hsv");
pub const BLENDER_CHANNEL_OKLCH: NodeId = hash_node_id("blender_channel_oklch");
// Channel value chips — interactive `NumberInput`s mirrored
// to the channel sliders. Display the current channel value
// (R/G/B/A or H/S/V/A depending on `channel_mode`) in 0..1.
pub const BLENDER_NUM_0: NodeId = hash_node_id("blender_num_0");
pub const BLENDER_NUM_1: NodeId = hash_node_id("blender_num_1");
pub const BLENDER_NUM_2: NodeId = hash_node_id("blender_num_2");
pub const BLENDER_NUM_3: NodeId = hash_node_id("blender_num_3");
// "+ swatch" button (appends current value to palette).
pub const BLENDER_ADD_SWATCH: NodeId = hash_node_id("blender_add_swatch");
// Palette Import / Export buttons (host file dialog → .gpl/.hex/.ase/.aco).
pub const BLENDER_IMPORT_PALETTE: NodeId = hash_node_id("blender_import_palette");
pub const BLENDER_EXPORT_PALETTE: NodeId = hash_node_id("blender_export_palette");
// Eyedropper button (enters pixel-pick mode).
pub const BLENDER_EYEDROPPER: NodeId = hash_node_id("blender_eyedropper");
// Drag handle bar at the top of the picker — drag to move.
pub const BLENDER_DRAG_HANDLE: NodeId = hash_node_id("blender_drag_handle");
// Palette swatch slots 0..26 — first 12 are the default palette,
// remaining 15 cover user "+ swatch" additions. Hard cap at 27 to
// keep registration static; `blender_palette_push` rejects beyond
// (and the painter hides the "+" tile when the palette is full).
pub const BLENDER_SWATCH_0: NodeId = hash_node_id("blender_swatch_0");
pub const BLENDER_SWATCH_1: NodeId = hash_node_id("blender_swatch_1");
pub const BLENDER_SWATCH_2: NodeId = hash_node_id("blender_swatch_2");
pub const BLENDER_SWATCH_3: NodeId = hash_node_id("blender_swatch_3");
pub const BLENDER_SWATCH_4: NodeId = hash_node_id("blender_swatch_4");
pub const BLENDER_SWATCH_5: NodeId = hash_node_id("blender_swatch_5");
pub const BLENDER_SWATCH_6: NodeId = hash_node_id("blender_swatch_6");
pub const BLENDER_SWATCH_7: NodeId = hash_node_id("blender_swatch_7");
pub const BLENDER_SWATCH_8: NodeId = hash_node_id("blender_swatch_8");
pub const BLENDER_SWATCH_9: NodeId = hash_node_id("blender_swatch_9");
pub const BLENDER_SWATCH_10: NodeId = hash_node_id("blender_swatch_10");
pub const BLENDER_SWATCH_11: NodeId = hash_node_id("blender_swatch_11");
pub const BLENDER_SWATCH_12: NodeId = hash_node_id("blender_swatch_12");
pub const BLENDER_SWATCH_13: NodeId = hash_node_id("blender_swatch_13");
pub const BLENDER_SWATCH_14: NodeId = hash_node_id("blender_swatch_14");
pub const BLENDER_SWATCH_15: NodeId = hash_node_id("blender_swatch_15");
pub const BLENDER_SWATCH_16: NodeId = hash_node_id("blender_swatch_16");
pub const BLENDER_SWATCH_17: NodeId = hash_node_id("blender_swatch_17");
pub const BLENDER_SWATCH_18: NodeId = hash_node_id("blender_swatch_18");
pub const BLENDER_SWATCH_19: NodeId = hash_node_id("blender_swatch_19");
pub const BLENDER_SWATCH_20: NodeId = hash_node_id("blender_swatch_20");
pub const BLENDER_SWATCH_21: NodeId = hash_node_id("blender_swatch_21");
pub const BLENDER_SWATCH_22: NodeId = hash_node_id("blender_swatch_22");
pub const BLENDER_SWATCH_23: NodeId = hash_node_id("blender_swatch_23");
pub const BLENDER_SWATCH_24: NodeId = hash_node_id("blender_swatch_24");
pub const BLENDER_SWATCH_25: NodeId = hash_node_id("blender_swatch_25");
pub const BLENDER_SWATCH_26: NodeId = hash_node_id("blender_swatch_26");
// Named-palette CRUD: tab strip (click → select; index = palette position), New, Delete.
pub const BLENDER_PALETTE_TABS: [NodeId; 8] = [
    hash_node_id("blender_palette_tab_0"),
    hash_node_id("blender_palette_tab_1"),
    hash_node_id("blender_palette_tab_2"),
    hash_node_id("blender_palette_tab_3"),
    hash_node_id("blender_palette_tab_4"),
    hash_node_id("blender_palette_tab_5"),
    hash_node_id("blender_palette_tab_6"),
    hash_node_id("blender_palette_tab_7"),
];
pub const BLENDER_NEW_PALETTE: NodeId = hash_node_id("blender_new_palette");
pub const BLENDER_DELETE_PALETTE: NodeId = hash_node_id("blender_delete_palette");
// Color Harmonies: the 7 scheme selector segments (index = `Harmony::ALL` position), the up-to-4
// derived partner swatches (click → set as value), and the "add all to palette" button.
pub const BLENDER_HARMONY_SCHEMES: [NodeId; 7] = [
    hash_node_id("blender_harmony_scheme_0"),
    hash_node_id("blender_harmony_scheme_1"),
    hash_node_id("blender_harmony_scheme_2"),
    hash_node_id("blender_harmony_scheme_3"),
    hash_node_id("blender_harmony_scheme_4"),
    hash_node_id("blender_harmony_scheme_5"),
    hash_node_id("blender_harmony_scheme_6"),
];
pub const BLENDER_HARMONY_SWATCHES: [NodeId; 4] = [
    hash_node_id("blender_harmony_swatch_0"),
    hash_node_id("blender_harmony_swatch_1"),
    hash_node_id("blender_harmony_swatch_2"),
    hash_node_id("blender_harmony_swatch_3"),
];
pub const BLENDER_HARMONY_ADD: NodeId = hash_node_id("blender_harmony_add");
// Active-palette rename field (a TextInput; Enter commits the new name).
pub const BLENDER_PALETTE_NAME: NodeId = hash_node_id("blender_palette_name");
// Palette dropdown chip (click → toggle the palette-select popover) + Rename (R) button.
pub const BLENDER_PALETTE_DROPDOWN: NodeId = hash_node_id("blender_palette_dropdown");
pub const BLENDER_RENAME_PALETTE: NodeId = hash_node_id("blender_rename_palette");
// Close (×) button at the top-right of the floating picker — dismisses it.
pub const BLENDER_CLOSE: NodeId = hash_node_id("blender_close");

/// Hierarchy panel container — wheel-scroll key.
pub const HIER_PANEL: NodeId = hash_node_id("hier_panel");
pub const HIER_PLAYER: NodeId = NodeId(400);
pub const HIER_SPRITE_IDLE: NodeId = NodeId(401);
pub const HIER_COLLIDER_BOX: NodeId = NodeId(402);
pub const HIER_SCRIPT_PLAYER: NodeId = NodeId(403);
pub const HIER_RIGIDBODY: NodeId = NodeId(404);
pub const HIER_TILEMAP_GROUND: NodeId = NodeId(405);
pub const HIER_TILEMAP_DECOR: NodeId = NodeId(406);
pub const HIER_SLIME_01: NodeId = NodeId(407);
pub const HIER_SLIME_02: NodeId = NodeId(408);
pub const HIER_TRIGGER_ZONE_A: NodeId = NodeId(409);
pub const HIER_AMBIENT_LIGHT: NodeId = NodeId(410);
pub const HIER_MAIN_CAMERA: NodeId = NodeId(411);

/// M14.6 E: search/filter TextInput in the Hierarchy header. Empty
/// query shows every row; non-empty case-insensitively filters by
/// `name.contains(query)` with ancestor-path preservation (a parent
/// stays visible if any descendant matches, so the user sees where
/// the hit lives in the tree).
pub const HIER_SEARCH: NodeId = hash_node_id("hier_search");

/// M14.7 polish: inline rename TextInput on a hierarchy row.
/// Painted only when `HeroScreen.hierarchy.rename_target_row` is `Some(id)`
/// — replaces the matching row's name label with an editable input.
pub const HIER_RENAME_INPUT: NodeId = hash_node_id("hier_rename_input");

/// M14.5 inspector phase (6.4/§9): "Reimport at current px/m" button
/// shown in the Render Source section when the selected sprite has a
/// live atlas-backed source. Click → `HeroScreen.pending_reimport`
/// gets set with the entity bits the host then drains to recompute
/// `Sprite.size` against the current `ProjectSettings.pixels_per_meter`.
pub const INSP_RENDER_SOURCE_REIMPORT: NodeId = hash_node_id("insp_render_source_reimport");

/// W2 Sprite Inspector v2: logical Flip H / Flip V checkboxes in the
/// Render Source section. Toggling dispatches an
/// `EditorAction::InspectorSpriteEdit` with `SpriteFieldEdit::FlipX/FlipY`.
pub const INSP_SPRITE_FLIP_X: NodeId = hash_node_id("insp_sprite_flip_x");
/// Vertical flip checkbox — see [`INSP_SPRITE_FLIP_X`].
pub const INSP_SPRITE_FLIP_Y: NodeId = hash_node_id("insp_sprite_flip_y");

/// M14.5 inspector phase: pixel-format segmented picker in the
/// Render Source section. Pressed = current choice, Normal = the
/// alternative. RGBA16 is `Disabled` until the asset crate gains
/// half-float / 16-bit-channel storage (currently only `ImageRgba8`).
/// Reimport reads the pressed button to decide the target format.
pub const INSP_RENDER_FORMAT_RGBA8: NodeId = hash_node_id("insp_render_format_rgba8");
pub const INSP_RENDER_FORMAT_RGBA16: NodeId = hash_node_id("insp_render_format_rgba16");

/// Map fixture entity name to canonical hierarchy `NodeId`. The
/// placeholder fixture currently exposes only "Scene Root"; the
/// other `HIER_*` ids are kept reserved for the pilot project's
/// real entities.
pub fn hierarchy_id(name: &str) -> Option<NodeId> {
    Some(match name {
        "Scene Root" => HIER_PLAYER,
        _ => return None,
    })
}

/// Map a hierarchy `NodeId` back to its fixture entity name. Inverse
/// of [`hierarchy_id`].
pub fn hierarchy_label_for_id(id: NodeId) -> Option<&'static str> {
    Some(match id {
        x if x == HIER_PLAYER => "Scene Root",
        _ => return None,
    })
}

/// Best-effort 3-letter "kind" badge for the selection tag.
/// Placeholder fixture has a single Scene Root; pilot replaces.
pub fn hierarchy_kind_for_label(_label: &str) -> &'static str {
    "ENT"
}

/// M14.6A: high-bit offset for the eye-toggle companion NodeId on
/// each hierarchy row. The row's primary NodeId is allocated by the
/// host bridge in the low 32-bit range
/// ([`shells/desktop/src/hero_bridge.rs`](shells/desktop/src/hero_bridge.rs)
/// uses `BASE_NODE_ID = 100_000`); the eye companion sits at
/// `row_id.0 | EYE_TOGGLE_BIT` so dispatch can recognize a click as
/// an eye-toggle without an explicit `BlenderHit` registration in
/// the `WidgetStore`.
pub const EYE_TOGGLE_BIT: u64 = 1u64 << 62;

/// M14.6C: high-bit offset for the expand/collapse chevron companion
/// NodeId on parent rows in the Hierarchy panel. Uses a different
/// bit from `EYE_TOGGLE_BIT` so the two companions stay
/// distinguishable in `apply_event`.
pub const EXPAND_TOGGLE_BIT: u64 = 1u64 << 61;
/// 2026-05-26: bit 60 for the lock-toggle companion (per row). Same
/// pattern as eye/expand — masked with `row_id.0` so dispatch can
/// recognize a click on the row's lock icon.
pub const LOCK_TOGGLE_BIT: u64 = 1u64 << 60;
/// 2026-05-26: bit 59 for the group-lock-toggle companion (per row).
pub const GROUP_TOGGLE_BIT: u64 = 1u64 << 59;
/// 2026-05-26: bit 58 for the entity-icon companion (per row). The
/// hierarchy row's leading icon (Sprite glyph) has its own hit so
/// double-click can be differentiated from a double-click on the row's
/// name (focus vs rename).
pub const ICON_COMPANION_BIT: u64 = 1u64 << 58;

/// Wave 2 PR 11.3 guard: companion detection only fires when the
/// un-masked id is below this threshold (i.e., looks like a real
/// hierarchy row id, allocated linearly from `BASE_NODE_ID = 100_000`
/// by `hero_bridge::EntityNodeMap`). Hash-derived chrome ids
/// (`hash_node_id(...)`) are uniformly distributed across 64 bits and
/// have a ~25% chance of accidentally setting bit 61 or 62; without
/// this threshold they'd be misrouted as row-companion clicks.
///
/// 2^32 (~4 G) is far above any realistic hierarchy size (fixture
/// has 12 rows; pilot project ~100s; even a stress-test 10 M rows
/// stay well below). The regression test
/// `tests/node_id_collisions.rs::no_chrome_id_has_companion_bits`
/// asserts every chrome const stays out of the companion-bit range
/// after hashing.
const COMPANION_ROW_ID_MAX: u64 = 1u64 << 32;

/// Derive the eye-toggle companion NodeId for a hierarchy row.
/// Inverse: [`hier_eye_companion_to_row`].
#[inline]
pub fn hier_eye_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | EYE_TOGGLE_BIT)
}

/// Recognize whether `id` is an eye-toggle companion and recover the
/// row NodeId. Returns `Some(row_id)` only when the high bit is set
/// AND the un-masked portion falls in the valid hierarchy-row id
/// space (< [`COMPANION_ROW_ID_MAX`]); used by `apply_event` to
/// route eye clicks without touching the BlenderHit pattern.
#[inline]
pub fn hier_eye_companion_to_row(id: NodeId) -> Option<NodeId> {
    if id.0 & EYE_TOGGLE_BIT != 0 {
        let masked = id.0 & !EYE_TOGGLE_BIT;
        if masked < COMPANION_ROW_ID_MAX {
            return Some(NodeId(masked));
        }
    }
    None
}

/// M14.6C: derive the chevron-companion NodeId for the
/// expand/collapse hit-rect at the start of a hierarchy row.
#[inline]
pub fn hier_expand_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | EXPAND_TOGGLE_BIT)
}

/// Inverse of [`hier_expand_companion`]. Returns `Some(row_id)` only
/// when [`EXPAND_TOGGLE_BIT`] is set AND the un-masked portion falls
/// in the valid hierarchy-row id space (< [`COMPANION_ROW_ID_MAX`]).
#[inline]
pub fn hier_expand_companion_to_row(id: NodeId) -> Option<NodeId> {
    if id.0 & EXPAND_TOGGLE_BIT != 0 {
        let masked = id.0 & !EXPAND_TOGGLE_BIT;
        if masked < COMPANION_ROW_ID_MAX {
            return Some(NodeId(masked));
        }
    }
    None
}

/// 2026-05-26: lock-toggle companion (per hierarchy row). Mirrors
/// `hier_eye_companion` — XORs `LOCK_TOGGLE_BIT` so dispatch can
/// recognize a click on the row's lock icon.
#[inline]
pub fn hier_lock_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | LOCK_TOGGLE_BIT)
}

#[inline]
pub fn hier_lock_companion_to_row(id: NodeId) -> Option<NodeId> {
    if id.0 & LOCK_TOGGLE_BIT != 0 {
        let masked = id.0 & !LOCK_TOGGLE_BIT;
        if masked < COMPANION_ROW_ID_MAX {
            return Some(NodeId(masked));
        }
    }
    None
}

/// 2026-05-26: group-lock-toggle companion (per hierarchy row).
#[inline]
pub fn hier_group_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | GROUP_TOGGLE_BIT)
}

#[inline]
pub fn hier_group_companion_to_row(id: NodeId) -> Option<NodeId> {
    if id.0 & GROUP_TOGGLE_BIT != 0 {
        let masked = id.0 & !GROUP_TOGGLE_BIT;
        if masked < COMPANION_ROW_ID_MAX {
            return Some(NodeId(masked));
        }
    }
    None
}

/// 2026-05-26: entity-icon companion (left glyph in a hierarchy row).
/// Double-click here triggers focus (View → Selected); double-click
/// on the row's name body triggers rename.
#[inline]
pub fn hier_icon_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | ICON_COMPANION_BIT)
}

#[inline]
pub fn hier_icon_companion_to_row(id: NodeId) -> Option<NodeId> {
    if id.0 & ICON_COMPANION_BIT != 0 {
        let masked = id.0 & !ICON_COMPANION_BIT;
        if masked < COMPANION_ROW_ID_MAX {
            return Some(NodeId(masked));
        }
    }
    None
}

// ─── Wave 6+7 Phase 2: consolidated section + grid-snap ids that
// dispatch (in editor-core) needs to query. Definitions originally
// lived in `screens/hero/inspector/mod.rs` and `grid_snap/ids.rs`;
// moved here so editor-core can reference them without depending
// back on ph2d-editor. Inspector + grid_snap re-export for legacy
// import-path stability.

/// Stable id list for every collapsible section header in the
/// Inspector (Widget Gallery showcase mode).
pub const SECTION_IDS: [NodeId; 11] = [
    INSP_SECTION_INPUTS,
    INSP_SECTION_SLIDER,
    INSP_SECTION_SWITCHES,
    INSP_SECTION_LISTS,
    INSP_SECTION_VECTOR,
    INSP_SECTION_STATUS,
    INSP_SECTION_COLOR,
    INSP_SECTION_ACTIONS,
    INSP_SECTION_IDENTITY,
    INSP_SECTION_CARD,
    INSP_SECTION_W6,
];

/// Live Inspector section headers (Name / Visibility / Transform /
/// Render / Color & Tint / Sprite Sheet / Ordering / Sampling /
/// Material & Blend / Physics Body / Physics Joint). Right-click opens the
/// SectionOutline menu.
///
/// ⚠️ `finish_section` reads `store.section_outline_color(<section id>)` for
/// EVERY live section, so a section missing from this list has an outline the
/// paint path is ready to draw and no gesture that can ever set it.
pub const LIVE_SECTION_IDS: [NodeId; 12] = [
    INSP_LIVE_NAME_SECTION,
    INSP_LIVE_VISIBILITY_SECTION,
    INSP_LIVE_TRANSFORM_SECTION,
    INSP_LIVE_RENDER_SECTION,
    INSP_LIVE_COLOR_SECTION,
    INSP_LIVE_SHEET_SECTION,
    INSP_LIVE_ORDERING_SECTION,
    INSP_LIVE_SAMPLING_SECTION,
    INSP_LIVE_BLEND_SECTION,
    INSP_LIVE_PHYSICS_SECTION,
    INSP_LIVE_JOINT_SECTION,
    INSP_LIVE_WHEEL_SECTION,
];

/// Grid-snap floating panel root id.
pub const GS_PANEL: NodeId = NodeId(1000);

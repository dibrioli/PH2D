//! [`NodeId`] constants for the grid-snap floating panel widgets.
//!
//! Allocated in the `1000..1099` range — verified free against
//! `crates/ph2d-editor/src/screens/hero/ids.rs` (TopBar 100s,
//! LeftRail 200s, Inspector 300s, Hierarchy 400s, ColorPicker 600s,
//! Widget Gallery 950s). Coordenador reserves this range for
//! `grid_snap` in the hero ids file during integration; until then
//! these consts live here and are wired by absolute value.
//!
//! Sub-allocation:
//! - 1000..1009 — panel chrome (handle / resize / close)
//! - 1010..1019 — header (grid-kind dropdown + snap toggle)
//! - 1020..1059 — per-kind config widgets (cell size, orientation,
//!   neighborhood radios, etc.)
//! - 1060..1069 — snap policy
//! - 1070..1079 — display (overlay toggle, opacity, color)
//! - 1080..1099 — inspect subsection (probe A/B + reset)

use crate::NodeId;

// ─── Panel chrome ───────────────────────────────────────────────
// Wave 6+7 Phase 2: GS_PANEL definition lives in
// `crate::ids` so dispatch (in editor-core) can match
// against it without depending back on ph2d-editor. Re-exported
// here for legacy `grid_snap::ids::GS_PANEL` import-path stability.
pub use crate::ids::GS_PANEL;
pub const GS_DRAG_HANDLE: NodeId = NodeId(1001);
pub const GS_RESIZE_HANDLE: NodeId = NodeId(1002);
pub const GS_CLOSE: NodeId = NodeId(1003);
/// Bottom-LEFT resize gripper. Mirror of [`GS_RESIZE_HANDLE`].
pub const GS_RESIZE_HANDLE_BL: NodeId = NodeId(1004);

// Re-export from the root ids module so `crate::grid_snap::ids::GS_TITLE_COLOR`
// works the same as `crate::ids::GS_TITLE_COLOR`.
pub use crate::ids::GS_TITLE_COLOR;

// ─── Header ────────────────────────────────────────────────────
pub const GS_KIND_DROPDOWN: NodeId = NodeId(1010);
pub const GS_KIND_OPT_SQUARE: NodeId = NodeId(1011);
pub const GS_KIND_OPT_HEX: NodeId = NodeId(1012);
pub const GS_KIND_OPT_ISO: NodeId = NodeId(1013);
pub const GS_KIND_OPT_STAGGERED_SQ: NodeId = NodeId(1014);
pub const GS_KIND_OPT_STAGGERED_HEX: NodeId = NodeId(1015);
pub const GS_KIND_OPT_TRI: NodeId = NodeId(1016);
pub const GS_KIND_OPT_QUADTREE: NodeId = NodeId(1017);
pub const GS_KIND_OPT_VORONOI: NodeId = NodeId(1018);
pub const GS_KIND_OPT_CHUNKS: NodeId = NodeId(1019);

// ─── Per-kind config (collapsed lanes; only the active kind is painted) ─
pub const GS_CFG_CELL_SIZE: NodeId = NodeId(1020);
pub const GS_CFG_NEIGHBORHOOD_4: NodeId = NodeId(1021);
pub const GS_CFG_NEIGHBORHOOD_8: NodeId = NodeId(1022);
pub const GS_CFG_HEX_POINTY: NodeId = NodeId(1023);
pub const GS_CFG_HEX_FLAT: NodeId = NodeId(1024);
pub const GS_CFG_ISO_TILE_W: NodeId = NodeId(1026);
pub const GS_CFG_ISO_TILE_H: NodeId = NodeId(1027);
pub const GS_CFG_STAGGER_PARITY_ODD: NodeId = NodeId(1028);
pub const GS_CFG_STAGGER_PARITY_EVEN: NodeId = NodeId(1029);
pub const GS_CFG_TRI_EDGE3: NodeId = NodeId(1030);
pub const GS_CFG_TRI_VERTEX12: NodeId = NodeId(1031);
pub const GS_CFG_QT_MAX_PER_LEAF: NodeId = NodeId(1032);
pub const GS_CFG_QT_MAX_DEPTH: NodeId = NodeId(1033);
pub const GS_CFG_VORONOI_SEED_COUNT: NodeId = NodeId(1034);
pub const GS_CFG_VORONOI_RNG_SEED: NodeId = NodeId(1035);
pub const GS_CFG_VORONOI_LLOYD_ITERS: NodeId = NodeId(1036);
pub const GS_CFG_VORONOI_RESEED: NodeId = NodeId(1037);
pub const GS_CFG_CHUNKS_SIZE: NodeId = NodeId(1038);
/// Hex offset variant options (segmented button group, 4 options).
pub const GS_CFG_HEX_OFFSET_ODDR: NodeId = NodeId(1056);
pub const GS_CFG_HEX_OFFSET_EVENR: NodeId = NodeId(1057);
pub const GS_CFG_HEX_OFFSET_ODDQ: NodeId = NodeId(1058);
pub const GS_CFG_HEX_OFFSET_EVENQ: NodeId = NodeId(1059);

// ─── Universal per-kind extras (origin + subdivisions) ─────────
/// World-space X offset of the active grid's (0, 0) anchor.
pub const GS_CFG_ORIGIN_X: NodeId = NodeId(1039);
/// World-space Y offset of the active grid's (0, 0) anchor.
pub const GS_CFG_ORIGIN_Y: NodeId = NodeId(1040);
/// Major-line spacing (Square / Chunks only — every Nth line drawn
/// with a thicker stroke).
pub const GS_CFG_SPACING_MAJOR: NodeId = NodeId(1041);

// ─── Grid color (R/G/B NumberInputs + ColorSwatch preview) ─────
/// Red channel 0..=255.
pub const GS_CFG_COLOR_R: NodeId = NodeId(1042);
/// Green channel 0..=255.
pub const GS_CFG_COLOR_G: NodeId = NodeId(1043);
/// Blue channel 0..=255.
pub const GS_CFG_COLOR_B: NodeId = NodeId(1044);

// ─── Snap subdivisions (sub-grid factor) ───────────────────────
pub const GS_CFG_SNAP_SUBDIVISIONS: NodeId = NodeId(1045);

// ─── Snap magnetism radius (world meters; bug 2 — attraction zone)
/// World-space distance (meters) within which the cursor / sprite is
/// pulled to a snap target. `0` disables the gate (always-snap).
pub const GS_CFG_SNAP_MAGNETISM_RADIUS: NodeId = NodeId(1066);

// ─── Quadtree bounds + demo points (state.rs has the fields) ───
pub const GS_CFG_QT_BOUNDS_MIN_X: NodeId = NodeId(1046);
pub const GS_CFG_QT_BOUNDS_MIN_Y: NodeId = NodeId(1047);
pub const GS_CFG_QT_BOUNDS_MAX_X: NodeId = NodeId(1048);
pub const GS_CFG_QT_BOUNDS_MAX_Y: NodeId = NodeId(1049);
pub const GS_CFG_QT_DEMO_POINTS: NodeId = NodeId(1050);
pub const GS_CFG_QT_DEMO_SEED: NodeId = NodeId(1051);

// ─── Voronoi bounds (rng_seed/seed_count/lloyd already exist) ──
pub const GS_CFG_VORONOI_BOUNDS_MIN_X: NodeId = NodeId(1052);
pub const GS_CFG_VORONOI_BOUNDS_MIN_Y: NodeId = NodeId(1053);
pub const GS_CFG_VORONOI_BOUNDS_MAX_X: NodeId = NodeId(1054);
pub const GS_CFG_VORONOI_BOUNDS_MAX_Y: NodeId = NodeId(1055);

// ─── Snap policy ───────────────────────────────────────────────
pub const GS_SNAP_ENABLED: NodeId = NodeId(1060);
/// Target option: snap to cell Center.
pub const GS_SNAP_CENTER: NodeId = NodeId(1061);
/// Target option: snap to cell vertex (Intersection).
pub const GS_SNAP_INTERSECTION: NodeId = NodeId(1062);
/// Target option: snap a sprite Corner to the nearest grid vertex.
pub const GS_SNAP_TARGET_OPT_CORNER: NodeId = NodeId(1063);
/// Target option: composite (Center + Intersection).
pub const GS_SNAP_TARGET_OPT_CENTER_AND_INTERSECTION: NodeId = NodeId(1064);
/// Target option: composite (Center + Intersection + Corners).
pub const GS_SNAP_TARGET_OPT_CENTER_INTERSECTION_AND_CORNERS: NodeId = NodeId(1065);

// ─── Display ───────────────────────────────────────────────────
pub const GS_SHOW_OVERLAY: NodeId = NodeId(1070);
pub const GS_COLOR_PICKER: NodeId = NodeId(1071);
pub const GS_OPACITY_SLIDER: NodeId = NodeId(1072);
/// Layer-order segmented buttons — In front of vs Behind sprites.
pub const GS_LAYER_IN_FRONT: NodeId = NodeId(1073);
pub const GS_LAYER_BEHIND: NodeId = NodeId(1074);

// ─── Inspect subsection ────────────────────────────────────────
pub const GS_INSPECT_HEADER: NodeId = NodeId(1080);
pub const GS_PROBE_A_X: NodeId = NodeId(1081);
pub const GS_PROBE_A_Y: NodeId = NodeId(1082);
pub const GS_PROBE_B_X: NodeId = NodeId(1083);
pub const GS_PROBE_B_Y: NodeId = NodeId(1084);
pub const GS_PROBE_RESET: NodeId = NodeId(1085);

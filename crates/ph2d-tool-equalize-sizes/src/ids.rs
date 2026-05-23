//! NodeId consts for the Equalize Sizes panel widgets.
//!
//! Owned by the tool crate (not editor-core) so the fan-out is a true
//! drop-crate: no edit in `ph2d-editor-core/src/ids.rs` is needed when a
//! new tool ships. `handle_panel_event` in [`crate::tool`] matches on
//! these consts; the panel crate (`ph2d-panel-equalize-sizes`) imports
//! them and registers them in its `WidgetStore` populate pass.
//!
//! Naming convention: `EQS_<chip>` (Equalize Sizes), hashed from
//! `"eqsizes.<chip>"` to keep the FNV-1a string namespace distinct from
//! `PAD_*` / `BGR_*` (the `node_id_collisions` arch test in editor-core
//! covers any accidental hash collision globally). The panel envelope
//! itself is hashed from `"panel.equalize_sizes"` so it matches
//! `EqualizeSizesPanel::NODE_ID` (the dispatcher's `panel_at` lookup
//! keys on that hash); mismatch silently broke hit-testing pre-fix.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Panel envelope (right-docked, Inspector geometry slot). Hash matches
/// `ph2d_panel_equalize_sizes::EqualizeSizesPanel::NODE_ID` and the
/// editor-core `EQS_PANEL` mirror used by `paint_hero_screen`'s
/// `z_order` fallback.
pub const EQS_PANEL: NodeId = hash_node_id("panel.equalize_sizes");

// ── Target mode (3-way radio implemented as 3 toggle-buttons) ────────
/// "Max of selection" mode — target = max(W,H) over all selected sprites.
pub const EQS_MODE_MAX: NodeId = hash_node_id("eqsizes.mode_max");
/// "Fixed" mode — target = exact (`EQS_FIXED_W`, `EQS_FIXED_H`) entered
/// in the W/H chips.
pub const EQS_MODE_FIXED: NodeId = hash_node_id("eqsizes.mode_fixed");
/// "Grid unit" mode — target snaps each sprite to the nearest multiple
/// of `EQS_GRID_UNIT` (in pixels).
pub const EQS_MODE_GRID: NodeId = hash_node_id("eqsizes.mode_grid");

// ── Fixed-mode numeric chips ─────────────────────────────────────────
pub const EQS_FIXED_W: NodeId = hash_node_id("eqsizes.fixed_w");
pub const EQS_FIXED_H: NodeId = hash_node_id("eqsizes.fixed_h");

// ── Grid-mode slider + chip ──────────────────────────────────────────
pub const EQS_GRID_UNIT: NodeId = hash_node_id("eqsizes.grid_unit");
pub const EQS_GRID_UNIT_NUM: NodeId = hash_node_id("eqsizes.grid_unit_num");

// ── Boolean toggles (painted as accent-when-on buttons) ──────────────
pub const EQS_UPSCALE_IF_SMALLER: NodeId = hash_node_id("eqsizes.upscale_if_smaller");
pub const EQS_RASTERIZE_AFTER: NodeId = hash_node_id("eqsizes.rasterize_after");

// ── Upscale algorithm (3-way radio, visible when upscale_if_smaller) ─
pub const EQS_ALG_LANCZOS: NodeId = hash_node_id("eqsizes.alg_lanczos");
pub const EQS_ALG_NEAREST: NodeId = hash_node_id("eqsizes.alg_nearest");
pub const EQS_ALG_XBR: NodeId = hash_node_id("eqsizes.alg_xbr");

// ── Apply / Cancel ───────────────────────────────────────────────────
pub const EQS_APPLY: NodeId = hash_node_id("eqsizes.apply");
pub const EQS_CANCEL: NodeId = hash_node_id("eqsizes.cancel");

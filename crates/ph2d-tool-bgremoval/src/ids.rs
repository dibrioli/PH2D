//! Tool-owned `NodeId`s for panel controls that route semantics
//! through [`super::tool::BgRemovalTool::handle_panel_event`].
//!
//! The bulk of the bgremoval panel ids live in `ph2d_editor_core::ids`
//! (`BGR_*`) for historical reasons. This module hosts ids added after
//! that centralization: declaring them in the tool crate (instead of
//! editor-core) lets a single feature land without editing the shared
//! ids file, which removes a contention point with parallel agents.
//! The panel crate re-exports these alongside its `editor-core` re-exports
//! so `crate::ids::BGR_*` stays the single namespace inside the panel.
//!
//! All NodeIds use [`hash_node_id`] (FNV-1a 64-bit, `const fn`), the same
//! mechanism the editor-core ids use, so collision detection at registry
//! build covers them automatically.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// "Separate Islands" toggle in the panel. When enabled, the Apply pass
/// also runs connected-component extraction on the final alpha matte and
/// stashes the resulting per-island RGBA payloads in
/// [`super::tool::BgRemovalTool`] for the shell to drain into new sprites.
pub const BGR_SEPARATE_ISLANDS: NodeId = hash_node_id("bgr_separate_islands");

/// "Min island pixels" slider (normalized 0..1). Maps to a pixel count
/// in `[1, MIN_ISLAND_PIXELS_FULL_SCALE]` — islands with fewer than this
/// many pixels are skipped (noise filter).
pub const BGR_MIN_ISLAND_PX: NodeId = hash_node_id("bgr_min_island_px");

/// Numeric chip paired with [`BGR_MIN_ISLAND_PX`]. Displays the unmapped
/// integer pixel count via `display_override`.
pub const BGR_MIN_ISLAND_PX_NUM: NodeId = hash_node_id("bgr_min_island_px_num");

/// "Detect subject" toggle (Enio 2026-05-26 edge-aware silhouette
/// upgrade). When pressed, the tool runs the silhouette detector
/// before every pipeline tick and force-keeps the subject interior —
/// fixing the case where the bg colour also appears inside the
/// subject (e.g. beige skin against beige bg).
pub const BGR_AUTO_PROTECT_SUBJECT: NodeId = hash_node_id("bgr_auto_protect_subject");

//! Widget `NodeId`s for the Color Equalization panel.
//!
//! Defined here in the tool crate (NOT in `ph2d-editor-core::ids`) so the
//! fan-out path (DIRETRIZ §3.8) stays pure: the new tool drops its own
//! pasta + the panel crate-irmão re-exports these via `pub use`. Tool's
//! `handle_panel_event` matches against `crate::ids::*`; the panel's
//! `populate` / `event` consume `ph2d_tool_color_equalization::ids::*`.
//!
//! All ids are derived via `hash_node_id("color_eq.<chip>")` (FNV-1a 64);
//! the [`node_id_collisions`] arch test in `ph2d-tool-registry` catches
//! accidental hash collisions across the project.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── Panel root ────────────────────────────────────────────────────
pub const CEQ_PANEL: NodeId = hash_node_id("panel.color_equalization");

// ── Sliders + paired px / numeric chips ───────────────────────────
// Track is normalized 0..1; the chip stores the displayed value in
// the slider's natural unit (clip limit, tile count, brightness, …).
pub const CEQ_CLIP_LIMIT: NodeId = hash_node_id("color_eq.clip_limit");
pub const CEQ_CLIP_LIMIT_NUM: NodeId = hash_node_id("color_eq.clip_limit.num");

pub const CEQ_TILE_GRID: NodeId = hash_node_id("color_eq.tile_grid_size");
pub const CEQ_TILE_GRID_NUM: NodeId = hash_node_id("color_eq.tile_grid_size.num");

pub const CEQ_BRIGHTNESS: NodeId = hash_node_id("color_eq.brightness");
pub const CEQ_BRIGHTNESS_NUM: NodeId = hash_node_id("color_eq.brightness.num");

pub const CEQ_CONTRAST: NodeId = hash_node_id("color_eq.contrast");
pub const CEQ_CONTRAST_NUM: NodeId = hash_node_id("color_eq.contrast.num");

pub const CEQ_SATURATION: NodeId = hash_node_id("color_eq.saturation");
pub const CEQ_SATURATION_NUM: NodeId = hash_node_id("color_eq.saturation.num");

// ── Toggles + buttons ─────────────────────────────────────────────
pub const CEQ_AUTO_WB: NodeId = hash_node_id("color_eq.auto_wb");
pub const CEQ_APPLY: NodeId = hash_node_id("color_eq.apply");
pub const CEQ_CANCEL: NodeId = hash_node_id("color_eq.cancel");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_distinct() {
        let all = [
            CEQ_PANEL,
            CEQ_CLIP_LIMIT,
            CEQ_CLIP_LIMIT_NUM,
            CEQ_TILE_GRID,
            CEQ_TILE_GRID_NUM,
            CEQ_BRIGHTNESS,
            CEQ_BRIGHTNESS_NUM,
            CEQ_CONTRAST,
            CEQ_CONTRAST_NUM,
            CEQ_SATURATION,
            CEQ_SATURATION_NUM,
            CEQ_AUTO_WB,
            CEQ_APPLY,
            CEQ_CANCEL,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate Color EQ NodeId: {a:?} == {b:?}");
            }
        }
    }
}

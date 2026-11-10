//! Os ids da janela LINT do Pixel (P13 U-Lint, `ph2d-panel-pixel-lint`): o rect e a linha do menu
//! *Window*.

use super::super::hash_node_id;
use ph2d_a11y::NodeId;

/// O rect da janela LINT.
pub const PIXEL_LINT_PANEL: NodeId = hash_node_id("pixel_lint.panel");
/// A linha *Lint* do menu *Window*.
pub const TOPBAR_PIXEL_LINT: NodeId = hash_node_id("topbar_pixel_lint");

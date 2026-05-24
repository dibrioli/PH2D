//! Widget `NodeId`s for the Upscale panel.
//!
//! Unlike the older bgremoval / padding panels (whose IDs lived in
//! `ph2d_editor_core::ids` for legacy reasons), the Upscale panel
//! derives its IDs via [`hash_node_id`] right here in the satellite —
//! no slot in editor-core needs to be allocated, keeping the tool
//! drop-in (ADR-0040 §3.8). The tool crate (`ph2d_tool_upscale::tool`)
//! re-derives the same hashes from the same string keys; both sides
//! match by construction because the hash is deterministic.

pub use ph2d_editor_core::ids::{INSP_BLENDER_PICKER, UPS_TITLE_COLOR};

pub use ph2d_tool_upscale::tool::ids::{
    UPS_ALGO_LANCZOS3, UPS_ALGO_NEAREST, UPS_ALGO_XBR, UPS_APPLY, UPS_CANCEL, UPS_RESET, UPS_SCALE,
    UPS_SCALE_NUM,
};

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Panel root `NodeId` — used by paint to publish the panel rect and
/// by `clear_panel_rect` on hide.
pub const UPS_PANEL: NodeId = hash_node_id("panel.upscale");

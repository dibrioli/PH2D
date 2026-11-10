//! Os ids de controlo da aba (plano §7: na crate que os lê).

use ph2d_a11y::NodeId;
use ph2d_editor_core::registry::hash_node_id;

pub const VIEW_LIST: NodeId = hash_node_id("input_map_tab.view.list");
pub const VIEW_KEYBOARD: NodeId = hash_node_id("input_map_tab.view.keyboard");
pub const VIEW_TRY: NodeId = hash_node_id("input_map_tab.view.try");

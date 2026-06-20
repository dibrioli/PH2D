//! Left rail + transform tool NodeIds (HIERARCHY_ADD, TOOL_*, RAIL_*).
use super::{NodeId, hash_node_id};

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

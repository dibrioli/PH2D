//! Os ids das peças do cromo na galeria (`cards_cromo`): literais, para os censos.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

pub const CARD_TOOL_BAR: NodeId = hash_node_id("gallery.cromo.card_tool_bar");
pub const CARD_FLOATING_LIST: NodeId = hash_node_id("gallery.cromo.card_floating_list");
pub const BAR_RAIL: NodeId = hash_node_id("gallery.cromo.bar_rail");
pub const BAR_MOVE: NodeId = hash_node_id("gallery.cromo.bar_move");
pub const BAR_ROTATE: NodeId = hash_node_id("gallery.cromo.bar_rotate");
pub const BAR_TRANSFORM: NodeId = hash_node_id("gallery.cromo.bar_transform");
pub const LIST_MENU: NodeId = hash_node_id("gallery.cromo.list_menu");
pub const LIST_COPY: NodeId = hash_node_id("gallery.cromo.list_copy");
pub const LIST_DUPLICATE: NodeId = hash_node_id("gallery.cromo.list_duplicate");
pub const LIST_DELETE: NodeId = hash_node_id("gallery.cromo.list_delete");
pub const LIST_SEPARATOR: NodeId = hash_node_id("gallery.cromo.list_separator");
pub const LIST_TIP: NodeId = hash_node_id("gallery.cromo.list_tip");

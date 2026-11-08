//! O id do recibo do Add Component (plano GEF 25 §2.4).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O botão **Undo** do recibo.
pub const INSP_RECEIPT_UNDO: NodeId = hash_node_id("insp_receipt_undo");

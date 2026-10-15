//! **Os ids do painel MACROS** (ADR-0186, W5d) — o rect do painel e o abridor do menu *Window*, que o
//! chrome lê (o passeio de z, as tabelas da barra) sem ver a crate do painel.

use super::super::hash_node_id;
use ph2d_a11y::NodeId;

pub const MACROS_PANEL: NodeId = hash_node_id("macros.panel");

/// A linha *Macros* do menu *Window*.
pub const TOPBAR_MACROS: NodeId = hash_node_id("topbar_macros");

//! **Os ids do Pixel Art que o chrome lê** (ADR-0192): o rect do painel PIXEL e o abridor dele no
//! menu *Window*.
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use super::super::hash_node_id;
use ph2d_a11y::NodeId;

pub const PIXEL_PANEL: NodeId = hash_node_id("pixel.panel");

/// A linha *Pixel* do menu *Window*.
pub const TOPBAR_PIXEL: NodeId = hash_node_id("topbar_pixel");

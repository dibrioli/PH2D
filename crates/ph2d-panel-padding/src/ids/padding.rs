//! Padding panel widget NodeIds (PAD_*).
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/padding.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

pub const PAD_CANCEL: NodeId = hash_node_id("pad_cancel");

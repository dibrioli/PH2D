//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/inspector.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Title-bar color dot for the Grid Snap panel. Kept (Grid Snap is a
/// settings panel, not an image tool). The original broadcast added
/// PAD/BGR/CEQ/UPS/EQS dots too, but those were removed 2026-05-24
/// per user feedback: image-tool panels are transient operation
/// surfaces, not annotation surfaces.
pub const GS_TITLE_COLOR: NodeId = hash_node_id("gs_title_color");

/// ⭐ **Os cabeçalhos DOBRÁVEIS das secções do painel** (ordem do dono, 2026-09-24: *«no Grid as
/// seções não fecham»*). Registados por `mark_collapsible_section` no `populate`; o despacho dobra-os
/// ao clique. O da *Inspect* é o `GS_INSPECT_HEADER` que a fundação já declarava.
pub const GS_SEC_KIND: NodeId = hash_node_id("grid_snap.sec.kind");
pub const GS_SEC_TARGET: NodeId = hash_node_id("grid_snap.sec.target");
pub const GS_SEC_DISPLAY: NodeId = hash_node_id("grid_snap.sec.display");

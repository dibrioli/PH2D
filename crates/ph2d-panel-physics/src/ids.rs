//! Widget `NodeId`s for the Physics world panel.
//!
//! Like every other panel crate, the ids stay defined in editor-core
//! (`ph2d_editor_core::ids`) — the layout, the z-order walk and the
//! `node_id_collisions` arch test all reference them, and re-defining them here
//! would fork the source of truth. This is a convenience re-export.

pub use ph2d_editor_core::ids::PHYSICS_PANEL;

// ⚠️ **`PHYSICS_SLEEP_SPIN_NUM` saiu desta lista em 2026-08-30** e a ausência é a nota: ele era o
// chip numérico do slider *Spin*, e o slider virou um interruptor (`paint::body`), que não tem
// chip. ✅ O remate que esta nota pedia aconteceu em 2026-09-12: o const órfão saiu da
// `ph2d-editor-core`, e a tabela à mão do `node_id_collisions` que o mantinha «lido» morreu com o
// censo derivado.

mod physics;
pub use physics::*;

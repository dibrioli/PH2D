//! `NodeId`s dos widgets do painel da cena 3D.
//!
//! Como em todo crate de painel, os ids continuam definidos na editor-core
//! (`ph2d_editor_core::ids`) — o layout, o walk de z-order e o arch-test
//! `node_id_collisions` os referenciam, e redefini-los aqui bifurcaria a fonte
//! da verdade. Isto é um re-export de conveniência.

pub use ph2d_editor_core::ids::SCULPT3D_PANEL;

mod inspector;
pub use inspector::*;
mod sculpt3d;
pub use sculpt3d::*;
mod sculpt3d_cloth;
pub use sculpt3d_cloth::*;

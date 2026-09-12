//! Os `NodeId`s da tira — re-export dos ids canônicos do editor-core (a fonte
//! única; o z-order walk e o teste de colisão os enumeram lá).

pub use ph2d_editor_core::ids::FLIP_STRIP_PANEL;

mod flip;
pub use flip::*;

//! Os ids que este painel usa — **re-exportados** da `ph2d-editor-core`, nunca redefinidos.
//!
//! ⚠️ Uma segunda definição do mesmo id (mesmo com o mesmo hash) seria a segunda resposta a *"qual
//! é o id desta row?"*, e o `node_id_collisions` só vê a que mora na foundational.

pub use ph2d_editor_core::ids::{
    AUTHORED_CLOSE, AUTHORED_DRAG_HANDLE, AUTHORED_PANEL, AUTHORED_RESIZE_HANDLE,
    AUTHORED_RESIZE_HANDLE_BL, authored_row_id,
};

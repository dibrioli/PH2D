//! Os ids que este painel usa — **re-exportados** da `ph2d-editor-core`, nunca redefinidos.
//!
//! ⚠️ Uma segunda definição do mesmo id (mesmo com o mesmo hash) seria a segunda resposta a *"qual
//! é o id desta swatch?"*, e o `node_id_collisions` só vê a que mora na foundational.

pub use ph2d_editor_core::ids::{
    TOKENS_CLOSE, TOKENS_PANEL, TOKENS_RESET_ALL, tokens_reset_id, tokens_swatch_id,
};

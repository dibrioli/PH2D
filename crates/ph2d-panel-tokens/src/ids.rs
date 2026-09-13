//! Os ids que este painel usa.
//!
//! ⚠️ **Uma definição e ZERO re-exportações** (auditoria A5b, 2026-09-12). Os ids que só este painel
//! lê moram aqui; os que a `ph2d-editor-core` também lê (o corpo do painel, que o *z-order walk*
//! percorre) ficaram lá, e quem os usa nomeia `ph2d_editor_core::ids::X` — nunca por este módulo.
//! A regra que ele seguia continua certa — *uma segunda definição do mesmo id seria a segunda
//! resposta a «qual é o id desta swatch?»* — e é ela que proíbe a re-exportação: o censo de colisões
//! (`node_id_collisions`) lê os literais da workspace inteira, não só os da fundação.

mod tokens;
pub use tokens::*;

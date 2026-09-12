//! Os `NodeId` de widget que esta crate LÊ — declarados AQUI, e não na `ph2d-editor-core`.
//!
//! ⭐ **Desceram da fundação em 2026-09-12** (auditoria de arquitectura A5b): em 30 dias, 221 dos 437
//! commits à `ph2d-editor-core/src` tocaram `ids/`, e cada um recompilava as 43 crates que dependem
//! dela. O dono de um id é a crate MAIS BAIXA que todo leitor dele vê (`scripts/censo-ids.py`), e
//! um ficheiro de ids — um ASSUNTO — desce inteiro.
//!
//! ⚠️ **UMA definição e ZERO re-exportações**: quem lê de fora nomeia `ph2d_tool_painter::ids::X`. As colisões
//! de slug vigia-as o censo DERIVADO (`ph2d-editor-core/tests/it/node_id_collisions.rs`), que lê os
//! literais da workspace inteira — o id não precisa de morar ao lado dele.

mod painter;
pub use painter::*;
mod painter_brush_sections;
pub use painter_brush_sections::*;
mod painter_deform;
pub use painter_deform::*;
mod painter_gradient;
pub use painter_gradient::*;
mod painter_impasto;
pub use painter_impasto::*;
mod painter_line;
pub use painter_line::*;
mod painter_sculpt;
pub use painter_sculpt::*;
mod painter_selection;
pub use painter_selection::*;
mod painter_shape;
pub use painter_shape::*;
mod painter_stroke_op;
pub use painter_stroke_op::*;
mod painter_substrate;
pub use painter_substrate::*;
mod painter_symmetry;
pub use painter_symmetry::*;
mod painter_texture;
pub use painter_texture::*;
mod painter_tiling;
pub use painter_tiling::*;
mod painter_watercolor;
pub use painter_watercolor::*;
mod painter_wetpaint;
pub use painter_wetpaint::*;
mod wet_tuning;
pub use wet_tuning::*;

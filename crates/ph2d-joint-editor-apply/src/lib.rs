//! **As edições da aba Joint Editor no mundo** ([`apply`]): o documento da aba (`ph2d-joint-editor-doc`)
//! junta os pedidos; a família (`ph2d-app-joint-editor`) entrega-os aqui, um passo de desfazer cada.

mod apply;

pub use apply::{Outcome, apply};

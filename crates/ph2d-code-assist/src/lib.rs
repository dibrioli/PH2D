//! ⭐ **A lógica pura do editor de scripts** (CodeEditor): o que a família (`ph2d-app-code`) calcula
//! sem tocar nas abas nem no quadro. Cada módulo é o que era no `ph2d-app-code/src/<módulo>.rs`; a
//! família importa-os pelo mesmo nome (`crate::problems`, `crate::complete`, …).

pub mod actions;
pub mod complete;
pub mod help_doc;
pub mod lint;
pub mod nav;
pub mod problems;
pub mod recovery;
pub mod seen;
pub mod semantic;
pub mod skeleton;
pub mod snippet;

//! **O menu de vistas** — a lei vive na [`ph2d_viewport3d::view_menu`], partilhada com a escultura.
//!
//! ⚠️ **Este alias existe para o TESTE, não para o código.** `view_menu_tests.rs` exercita a lei
//! **através** do smoke do 3D (`crate::smoke`, `crate::input`) e usa `use super::*` — o que lê os
//! ALIASES do módulo pai, não só os itens públicos dele. Sem este módulo o ficheiro teria de ser
//! reescrito import a import, e ele mede exactamente a coisa certa como está: *um gate sobre uma
//! moldura provado por um gesto real vale mais do que um provado por um `Rect` escrito à mão.*

pub use ph2d_viewport3d::view_menu::*;

#[cfg(test)]
#[path = "view_menu_tests.rs"]
mod tests;

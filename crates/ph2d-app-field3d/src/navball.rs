//! **O gizmo de navegação** — a lei vive na [`ph2d_viewport3d::navball`], partilhada com a escultura.
//!
//! ⚠️ **Este alias existe para o TESTE, não para o código.** `navball_tests.rs` exercita a lei
//! **através** do smoke do 3D (`crate::smoke`, `crate::input`) e usa `use super::*` — o que lê os
//! ALIASES do módulo pai, não só os itens públicos dele. Sem este módulo o ficheiro teria de ser
//! reescrito import a import, e ele mede exactamente a coisa certa como está: *um gate sobre uma
//! moldura provado por um gesto real vale mais do que um provado por um `Rect` escrito à mão.*

pub use ph2d_viewport3d::navball::*;

#[cfg(test)]
#[allow(unused_imports)]
pub use {
    ph2d_editor_core::zones::Rect as EditorRect, ph2d_field_render::Orbit,
    ph2d_viewport3d::views::Standard,
};

#[cfg(test)]
#[path = "navball_tests.rs"]
mod tests;

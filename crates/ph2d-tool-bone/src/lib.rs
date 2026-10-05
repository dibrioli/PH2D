//! ⭐⭐ **A FERRAMENTA DE OSSO** (A14, plano `docs/Skeleton/05_plano_o_esqueleto_e_um_objecto.md`).
//!
//! Dona do que o arrasto faz sobre um esqueleto — o verbo ([`BoneAction`]) e o pincel de peso
//! ([`WeightDirection`], [`WeightMode`], raio e força). Saiu da ferramenta Vector (era o
//! `DrawMode::Bone`): desde a onda dos modos ela só está na mão no Edit de uma forma, e o osso ficou
//! preso a ela. Os gestos (o corpo) vivem em `ph2d-app-skeleton`; a shell espelha o
//! [`BoneConfig`] por quadro.

#![forbid(unsafe_code)]

pub mod ids;
mod params;
mod tool;

pub use params::*;
pub use tool::{BONE, BoneConfig, BoneTool};

/// Constrói a ferramenta como trait object. Alvo do codegen do `ph2d-tool-sync`.
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(BoneTool::default())
}

//! **A exploração do SFX Lab, sem tela** (`docs/SfxLab/05_W4_plano.md`): como nasce uma variação ([`lei`]), o que o artista
//! pediu e ouviu ([`sessao`]) e quem rende oito de uma vez ([`ninhada`]). Quem pinta é `ph2d-sfx-explore-tela`.

#![forbid(unsafe_code)]

pub mod lei;
pub mod ninhada;
pub mod sessao;

pub use lei::{TAMANHO_MUTATE, TAMANHO_ROLL, varia};
pub use ninhada::{AVULSO, FILHOS, Filho, Ninhada};
pub use sessao::{Exploracao, Grade, MAX_OUVIDOS, Sessao};

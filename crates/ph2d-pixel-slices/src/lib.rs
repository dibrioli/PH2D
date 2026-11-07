//! **O gesto da ferramenta Slice** (P11b). Medido: `docs/PixelArt/P11_oraculos_ferramenta.md` (T1–T5,
//! G3, G4); o que não se mediu e nós decidimos: `docs/PixelArt/P11_plano.md` §10.
//!
//! Em células: quem chama converte o ponteiro de ecrã e dá a tolerância das alças, também em células.
//! Nada toca no documento antes do largar: a meio do arrasto, [`Gesto::previa`] diz o que desenhar.

#![forbid(unsafe_code)]

pub mod alcas;
pub mod gesto;

pub use alcas::{Grab, Lados, aplica, hit};
pub use gesto::{Gesto, Marca, marcas, no_quadro};

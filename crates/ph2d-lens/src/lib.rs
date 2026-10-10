//! **A LENTE da câmera** — as leis do pós-processamento da cena inteira, em Rust e sem placa
//! ([doc 129](../../../docs/Motion%20Nodes/129_ciclo_20_o_acabamento.md) §3.2).
//!
//! Quatro efeitos em ORDEM FIXA (a física decide, não o artista): [`aberracao`] → [`vinheta`] →
//! [`grade`] (com a LUT de [`cubo`]) → [`grao`]. O passe da placa (`ph2d-render`) é a mesma lei em
//! WGSL e é gateado contra esta crate; os oráculos corridos (ImageMagick, ffmpeg) gateiam esta
//! crate (`tests/oraculos.rs`, fixtures de `docs/Motion Nodes/ferramentas/lens_oracle/corre.sh`).
//!
//! Convenção de coordenadas: pixel `i` cobre `[i, i+1)`, o centro dele é `i + 0,5`.

pub mod aberracao;
pub mod cubo;
pub mod grade;
pub mod grao;
pub mod lente;
pub mod vinheta;

pub use aberracao::Aberracao;
pub use cubo::{Cubo, ErroCubo};
pub use grade::Grade;
pub use grao::Grao;
pub use lente::{Imagem, Lente, aplica_cpu};
pub use vinheta::Vinheta;

//! **O traço à mão e o desenho livre do Quadro** (MiroClone, W4).
//!
//! Duas PORTAS de bibliotecas MIT, conferidas contra elas próprias corridas como oráculo
//! (`docs/MiroClone/ferramentas/excalidraw_oracle/corre_portas.mjs` → `saidas/portas_*.json`):
//!
//! - [`rough`]: o rough.js 4.6.4 — linhas, rectângulos, elipses, polígonos, curvas e caminhos com o
//!   tremor da mão, e o preenchimento [`fill`] (sólido, tracejado, tracejado cruzado: hachure-fill
//!   0.5.2). A [`Random`] é a do rough.js: a MESMA semente dá o MESMO traço, por isso cada elemento
//!   guarda a sua.
//! - [`freehand`]: o perfect-freehand 1.2.0 — de pontos com pressão ao contorno que se PREENCHE.
//!
//! Os avisos de copyright estão em `LICENSE-THIRD-PARTY.md` (copiados dos artefactos instalados).
//! Tudo em `f64` e `[f64; 2]`, sem dependências; a ordem das operações em ponto flutuante segue a do
//! JavaScript, porque a paridade com o oráculo é ao último dígito (tolerância nos testes `1e-9`).

mod curve_points;
pub mod fill;
pub mod freehand;
mod random;
pub mod rough;

pub use random::Random;

/// Um ponto.
pub type P = [f64; 2];

#[cfg(test)]
mod oracle_tests;

//! ⭐ **Os tipos que a família do editor de scripts entrega à pintura** — a cor de cada trecho
//! (`Ink`), as marcas de problema e o modelo da página de ajuda. Folha: o `ph2d-panel-code` pinta-os
//! e o `ph2d-code-assist` produ-los, sem que a lógica dependa do painel (a camada `Painel` só se lê de
//! uma família, `architecture_no_dependency_climbs_a_layer`).

pub mod help;
mod ink;
mod marks;

pub use ink::Ink;
pub use marks::{Explanation, PreviewLine, Problem, Severity};

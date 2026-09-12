//! **O TEXTO VECTORIAL** — o layout ([`glyph`]: linhas, alinhamento, refluxo, caret) e a
//! construção dos glifos em caminhos do documento ([`glyph_build`]).
//!
//! ⭐ Nasceu na auditoria de arquitectura de 2026-09-12 (A1). O layout vivia na `ph2d-app-vec`, e o
//! gerador de texto do Motion (`motion_text_gen`) usa a MESMA porta — *um segundo laço responderia
//! «onde cai cada letra?» uma segunda vez, e as duas respostas divergiriam* —, então a família
//! Motion dependia da família Vector inteira. O [`TextAlign`] desceu da `ph2d-tool-vector` com ele:
//! uma folha não depende de uma ferramenta, e a ferramenta nunca o usou — só o hospedava.
#![forbid(unsafe_code)]

pub mod glyph;
pub mod glyph_build;

/// Horizontal text alignment for a text block (mirror of the panel's L / C / R row).
/// `Left` = lines start at the click origin; `Center` = centred on it; `Right` = lines
/// end at it. Mora com o layout que o lê (auditoria A1 — morava na `ph2d-tool-vector`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

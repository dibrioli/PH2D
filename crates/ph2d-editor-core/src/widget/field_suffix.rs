//! ⭐ **O sufixo de um campo de texto** — uma palavra curta à direita DENTRO da caixa, como uma
//! unidade (GEF02 onda 1: `1 listens` / `nobody listens` ao lado de um nome de sinal).
//!
//! Quem sabe o que dizer é o painel (o Inspector instala um fornecedor por quadro); quem pinta é o
//! pintor da caixa, porque só ele sabe a largura do texto: a largura do sufixo é RESERVADA antes de
//! pintar o texto, e o texto digitado nunca fica por baixo dele.

use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};
use ph2d_vector::{Color, VectorScene};
use std::cell::Cell;

/// O que o fornecedor diz de uma caixa: o texto e a tinta (`Text3`, ou `Warn` para «ninguém»).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sufixo {
    pub texto: String,
    pub tinta: ColorToken,
}

/// O fornecedor: `(id da caixa, texto da caixa) → sufixo`.
pub type Fornecedor = fn(NodeId, &str) -> Option<Sufixo>;

thread_local! {
    static FORNECEDOR: Cell<Option<Fornecedor>> = const { Cell::new(None) };
}

/// Instala (ou tira, com `None`) o fornecedor deste fio. Devolve o anterior.
pub fn instala(f: Option<Fornecedor>) -> Option<Fornecedor> {
    FORNECEDOR.with(|c| c.replace(f))
}

/// O sufixo da caixa `id` com o texto `texto`, se o fornecedor tiver um.
#[must_use]
pub fn de(id: NodeId, texto: &str) -> Option<Sufixo> {
    FORNECEDOR.with(Cell::get).and_then(|f| f(id, texto))
}

/// O tamanho do sufixo: o degrau pequeno da escala.
#[must_use]
pub fn tamanho() -> f32 {
    TypeToken::Sm.px()
}

/// A largura que o sufixo tira ao texto: a dele mais um vão.
#[must_use]
pub fn largura(text_system: &mut TextSystem, s: &Sufixo) -> f32 {
    text_system.prefix_width(&s.texto, tamanho()) + Spacing::Md.px()
}

/// Pinta `s` encostado à direita de `rect`, com `pad` de margem, centrado na altura.
pub fn pinta(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    s: &Sufixo,
    rect: Rect,
    pad: f32,
    cor: Color,
) {
    let w = text_system.prefix_width(&s.texto, tamanho());
    crate::paint::paint_text(
        text_system,
        scene,
        &s.texto,
        rect.x + rect.w - pad - w,
        rect.y + crate::paint::folga_do_texto(rect.h, tamanho()),
        tamanho(),
        f32::INFINITY,
        cor,
    );
}

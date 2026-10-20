//! O que a família entrega à página: blocos já com o realce do código e os links numerados (o
//! painel não conhece o catálogo; a família guarda o destino de cada número).

use crate::style::Ink;

/// A espécie de um trecho.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanKind {
    Prose,
    Strong,
    Muted,
    /// Mono, na cor do realce.
    Code(Ink),
}

/// Um trecho de texto; `link` = o número do destino (a família resolve-o).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub kind: SpanKind,
    pub link: Option<u32>,
}

impl Span {
    #[must_use]
    pub fn new(text: impl Into<String>, kind: SpanKind) -> Self {
        Self {
            text: text.into(),
            kind,
            link: None,
        }
    }

    #[must_use]
    pub fn link(mut self, n: u32) -> Self {
        self.link = Some(n);
        self
    }
}

/// Um bloco da página, de cima para baixo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// O nome da página.
    Title(String),
    /// Um título de secção (entra no índice).
    Heading(String),
    Para(Vec<Span>),
    /// Um parágrafo a negrito (a frase do topo).
    Strong(Vec<Span>),
    /// Um parágrafo recuado (o corpo de um membro).
    Body(Vec<Span>),
    /// A assinatura de um membro (a âncora do Ctrl+clique); `first` = sem linha antes.
    Sig {
        spans: Vec<Span>,
        member: u32,
        first: bool,
    },
    /// Uma linha da tabela de irmãs: o retorno alinhado à direita e a chamada.
    Summary {
        ret: Vec<Span>,
        call: Vec<Span>,
    },
    /// Um exemplo; `copy` = o número do botão de copiar.
    Code {
        lines: Vec<Vec<Span>>,
        copy: u32,
    },
    Gap,
}

/// Uma página inteira.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HelpDoc {
    /// Muda quando o conteúdo muda (a disposição guarda-se por ela e pela largura).
    pub key: u64,
    pub title: String,
    pub blocks: Vec<Block>,
    /// O rótulo de cada `Heading`, pela ordem (o índice à esquerda).
    pub sections: Vec<String>,
}

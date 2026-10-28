//! **O relatório da importação**: o que entrou e o que ficou de fora, e porquê. A UI traduz cada
//! [`LossKind`] (chave i18n por variante); o `detail` é o nome do objecto do Trello, para o artista o
//! achar.

use std::fmt;

/// Porque é que uma coisa do Trello não entrou (ou entrou diferente).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LossKind {
    /// Dados de Power-Ups (`pluginData`, `boardPlugins`): são de código de terceiros.
    PowerUp,
    /// Um voto de alguém que não é membro do quadro exportado.
    VoteWithoutMember,
    /// Um membro (de cartão ou item) que não está em `members`.
    UnknownMember,
    /// Uma cor de etiqueta, de capa ou de lista que não é das do Trello: a etiqueta entra sem cor.
    UnknownColor,
    /// Um registo que aponta para um pai que não existe (cartão sem lista, checklist sem cartão…).
    Orphan,
    /// O mesmo id duas vezes: fica o primeiro.
    DuplicateId,
    /// Capa de imagem cujo anexo não veio com os bytes (ou um fundo carregado à parte).
    CoverImage,
    /// Fundo fotográfico do quadro (a fotografia não vem no JSON): fica o azul.
    BackgroundImage,
    /// O quadro tem uma descrição (o Kanban não tem onde a guardar).
    BoardDescription,
    /// Uma ação do Trello sem par na atividade do PH2D (a lista dos tipos vai no `detail`).
    UnsupportedAction,
    /// Uma ação sem autor.
    ActionWithoutAuthor,
    /// Um valor do tipo errado (data que não é data, número que não é número…).
    BadValue,
    /// Pediu-se a pasta dos ficheiros e este anexo não estava lá: entrou como link.
    AttachmentFileMissing,
    /// Separador, espelho ou ligação (`cardRole`): entra como cartão comum.
    CardRole,
    /// «Seguir» (`subscribed`) é de quem exportou, não do quadro.
    Subscription,
}

/// Uma perda: o tipo e o objecto (nome ou id do Trello).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loss {
    pub kind: LossKind,
    pub detail: String,
}

/// Quantos registos entraram, por tabela.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub lists: usize,
    pub cards: usize,
    pub labels: usize,
    pub checklists: usize,
    pub items: usize,
    pub comments: usize,
    pub attachments: usize,
    pub fields: usize,
    pub activities: usize,
    pub members: usize,
    pub files: usize,
}

/// O relatório inteiro.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub counts: Counts,
    pub losses: Vec<Loss>,
}

impl ImportReport {
    pub(crate) fn lose(&mut self, kind: LossKind, detail: impl Into<String>) {
        self.losses.push(Loss {
            kind,
            detail: detail.into(),
        });
    }

    /// Quantas perdas de um tipo.
    #[must_use]
    pub fn count(&self, kind: LossKind) -> usize {
        self.losses.iter().filter(|l| l.kind == kind).count()
    }
}

/// O ficheiro não é um quadro do Trello (nem JSON, ou JSON que não é um objecto).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportError {
    NotJson(String),
    NotABoard,
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportError::NotJson(e) => write!(f, "not JSON: {e}"),
            ImportError::NotABoard => f.write_str("not a Trello board export"),
        }
    }
}

impl std::error::Error for ImportError {}

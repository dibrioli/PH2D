//! **O fundo do Kanban**: a cor da paleta, um gradiente do Trello, uma imagem e (v8) uma cor livre
//! ou um padrão desenhado — o tipo e as duas cores. Só a CHAVE; quem pinta é
//! `ph2d_board_render::paint_kanban_bg` (a mesma lei na aba e no Quadro).
//!
//! ⛔ Postcard grava a variante pelo ÍNDICE: as novas entram sempre no FIM.

use serde::{Deserialize, Serialize};

use crate::color::Hue;
use crate::files::FileId;

/// Uma cor livre do documento (sRGB, opaca) — dado do artista, não da UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgb(pub [u8; 3]);

/// Uma cor do fundo: um matiz da paleta (segue o tema) ou uma cor livre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tint {
    Hue(Hue),
    Rgb(Rgb),
}

/// Os desenhos de um fundo com padrão, pela ordem do seletor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternKind {
    Dots,
    Grid,
    Lines,
    Diagonal,
    Checks,
    CrossHatch,
    Waves,
    Triangles,
}

impl PatternKind {
    pub const ALL: [PatternKind; 8] = [
        PatternKind::Dots,
        PatternKind::Grid,
        PatternKind::Lines,
        PatternKind::Diagonal,
        PatternKind::Checks,
        PatternKind::CrossHatch,
        PatternKind::Waves,
        PatternKind::Triangles,
    ];
}

/// Um fundo desenhado: o tipo, a cor de fundo (`base`) e a do traço (`ink`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pattern {
    pub kind: PatternKind,
    pub base: Tint,
    pub ink: Tint,
}

impl Pattern {
    /// O padrão `kind` nas cores de nascença: o azul do Trello com o traço claro.
    #[must_use]
    pub fn new(kind: PatternKind) -> Self {
        Self {
            kind,
            base: Tint::Hue(Hue::Blue),
            ink: Tint::Hue(Hue::Sky),
        }
    }
}

/// O fundo do Kanban.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Background {
    Color(Hue),
    /// Um dos gradientes do Trello, pelo índice (o do seletor e do JSON do Trello).
    Gradient(u16),
    /// Uma imagem do projecto pela chave do conteúdo (ADR-0184), recortada como `cover`.
    Image(FileId),
    /// v8 — uma cor livre.
    Custom(Rgb),
    /// v8 — um padrão desenhado.
    Pattern(Pattern),
}

#[cfg(test)]
#[path = "background_tests.rs"]
pub(crate) mod tests;

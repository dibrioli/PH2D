//! As cores do DOCUMENTO — chaves, nunca RGB: a paleta vive nos tokens do Kanban (claro e escuro),
//! e o documento guarda só qual delas o artista escolheu.
//!
//! As 30 cores de etiqueta do Trello: 10 matizes × 3 intensidades (sutil · normal · forte).

use serde::{Deserialize, Serialize};

/// Um matiz — o das etiquetas, das listas, das capas e do fundo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Hue {
    Green,
    Yellow,
    Orange,
    Red,
    Purple,
    Blue,
    Sky,
    Lime,
    Pink,
    Black,
}

impl Hue {
    /// Pela ordem da paleta do Trello.
    pub const ALL: [Hue; 10] = [
        Hue::Green,
        Hue::Yellow,
        Hue::Orange,
        Hue::Red,
        Hue::Purple,
        Hue::Blue,
        Hue::Sky,
        Hue::Lime,
        Hue::Pink,
        Hue::Black,
    ];
}

/// A intensidade de uma cor de etiqueta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Shade {
    Subtle,
    Normal,
    Bold,
}

impl Shade {
    pub const ALL: [Shade; 3] = [Shade::Subtle, Shade::Normal, Shade::Bold];
}

/// Uma das 30 cores de etiqueta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LabelColor {
    pub hue: Hue,
    pub shade: Shade,
}

impl LabelColor {
    /// As 30, na grelha do seletor: uma linha por intensidade, uma coluna por matiz.
    #[must_use]
    pub fn all() -> [LabelColor; 30] {
        std::array::from_fn(|i| LabelColor {
            hue: Hue::ALL[i % 10],
            shade: Shade::ALL[i / 10],
        })
    }
}

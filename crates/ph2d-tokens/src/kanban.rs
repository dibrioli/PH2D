//! As cores do Kanban (TrelloClone) como ÍNDICE: a etiqueta guarda `(tom, matiz)` e pinta-se por
//! aqui. A paleta do Trello; os valores vivem no `tokens.json`, os pares medidos em `contrast.rs`.

use crate::color::ColorToken as T;

/// Os tons: `subtle = 0`, `normal = 1`, `bold = 2`.
pub const SHADES: usize = 3;
/// Os matizes, na ordem do Trello: verde, amarelo, laranja, vermelho, roxo, azul, céu, lima,
/// rosa, preto.
pub const HUES: usize = 10;

/// `LABELS[tom][matiz]`.
pub const LABELS: [[T; HUES]; SHADES] = [
    [
        T::KanbanGreenSubtle,
        T::KanbanYellowSubtle,
        T::KanbanOrangeSubtle,
        T::KanbanRedSubtle,
        T::KanbanPurpleSubtle,
        T::KanbanBlueSubtle,
        T::KanbanSkySubtle,
        T::KanbanLimeSubtle,
        T::KanbanPinkSubtle,
        T::KanbanBlackSubtle,
    ],
    [
        T::KanbanGreen,
        T::KanbanYellow,
        T::KanbanOrange,
        T::KanbanRed,
        T::KanbanPurple,
        T::KanbanBlue,
        T::KanbanSky,
        T::KanbanLime,
        T::KanbanPink,
        T::KanbanBlack,
    ],
    [
        T::KanbanGreenBold,
        T::KanbanYellowBold,
        T::KanbanOrangeBold,
        T::KanbanRedBold,
        T::KanbanPurpleBold,
        T::KanbanBlueBold,
        T::KanbanSkyBold,
        T::KanbanLimeBold,
        T::KanbanPinkBold,
        T::KanbanBlackBold,
    ],
];

/// O texto sobre uma etiqueta do tom `shade`: o `bold` tem tinta própria.
#[must_use]
pub const fn label_ink(shade: usize) -> T {
    if shade == 2 {
        T::KanbanLabelInkBold
    } else {
        T::KanbanLabelInk
    }
}

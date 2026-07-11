//! [`FlipFrame`] — uma chave na tira: guarda QUAL desenho aparece a partir deste
//! quadro. A **duração é implícita** (segura até a próxima chave).
//!
//! Dois mecanismos de duração espelham o GP (`02_referencia §1`):
//! - **Implicit hold** ([`FlipFrame::implicit_hold`] `= true`): o desenho segura
//!   até a próxima chave, seja ela qual for. É o default (inserir com
//!   [`Hold::Implicit`]).
//! - **End-frame sentinela** ([`FlipFrame::is_end`]): um frame com `drawing =
//!   None` fecha uma duração fixa — de `key` em diante não aparece nada até a
//!   próxima chave. Inserir com [`Hold::Fixed`] cria essa sentinela em
//!   `key + dur`.
//!
//! Note: a duração NÃO é um número guardado no frame — é derivada (próxima chave
//! − esta). O `implicit_hold` é só o flag que decide, ao remover um frame, se o
//! anterior pode se estender (implicit) ou precisa de uma sentinela (fixed). É a
//! mesma modelagem do GP (flag + sentinela), não `Fixed(n)` armazenado.

use crate::ids::DrawingId;
use serde::{Deserialize, Serialize};

/// Como um frame é inserido — o **parâmetro** de `insert_frame`, não um campo
/// guardado. `Implicit` segura até a próxima chave; `Fixed(dur)` fixa a duração
/// e cria uma sentinela de fim em `key + dur`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Hold {
    #[default]
    Implicit,
    /// Duração fixa em quadros (`> 0`; `0` é tratado como [`Hold::Implicit`]).
    Fixed(u32),
}

impl Hold {
    /// A duração em quadros que este hold representa (`0` = implicit).
    #[must_use]
    pub fn duration(self) -> u32 {
        match self {
            Hold::Implicit => 0,
            Hold::Fixed(d) => d,
        }
    }

    /// É um hold implícito (inclui `Fixed(0)`, que o GP degrada a implicit).
    #[must_use]
    pub fn is_implicit(self) -> bool {
        self.duration() == 0
    }
}

/// Tipo de keyframe — usado pelo filtro dos Ghost Frames (W3). Minimal por ora.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum KeyKind {
    /// Um quadro-chave normal.
    #[default]
    Keyframe,
    /// Um breakdown (inbetween marcado).
    Breakdown,
}

/// Uma chave da tira de frames.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlipFrame {
    /// O desenho que aparece a partir deste quadro. `None` = **end-frame**
    /// (sentinela de fim: nada aparece daqui até a próxima chave).
    pub drawing: Option<DrawingId>,
    /// Segura até a próxima chave (não precisa de sentinela ao remover o
    /// seguinte). `false` = duração fixa (delimitada por sentinela).
    pub implicit_hold: bool,
    /// Tipo do keyframe (filtro de Ghost Frames).
    pub kind: KeyKind,
}

impl FlipFrame {
    /// Uma sentinela de fim (`drawing = None`).
    #[must_use]
    pub fn end() -> Self {
        Self {
            drawing: None,
            implicit_hold: false,
            kind: KeyKind::Keyframe,
        }
    }

    /// É uma sentinela de fim.
    #[must_use]
    pub fn is_end(&self) -> bool {
        self.drawing.is_none()
    }
}

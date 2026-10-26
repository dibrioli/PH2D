//! As regras de desenho técnico (W4, `docs/BlenderConstraints/04_W4_cad.md` §3.2): o MODELO, que viaja no
//! documento; quem as resolve é `ph2d-vec-constraints`. Endereçam pontos por `(path, VertId)`, nunca por índice.

use serde::{Deserialize, Serialize};

use crate::{VecPathId, VertId};

/// Um ponto de uma regra: um vértice de um path, ou o centro de um path que é um círculo.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PontoCad {
    Vert(VecPathId, VertId),
    Centro(VecPathId),
}

/// Um segmento recto de `0` a `1` (a ordem importa na tangente: o centro fica à ESQUERDA).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SegCad(pub PontoCad, pub PontoCad);

/// A lei de uma regra. Medidas em px da folha; ângulo em radianos, de `0` para `1` no sentido anti-horário.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Lei {
    Coincidente(PontoCad, PontoCad),
    Horizontal(SegCad),
    Vertical(SegCad),
    Paralelo(SegCad, SegCad),
    Perpendicular(SegCad, SegCad),
    /// Recta tangente ao círculo `VecPathId`, do lado em que nasceu (o centro à esquerda de `SegCad`).
    Tangente(SegCad, VecPathId),
    Igual(SegCad, SegCad),
    /// Os dois pontos são o espelho um do outro pela recta.
    Simetrico(PontoCad, PontoCad, SegCad),
    SobreRecta(PontoCad, SegCad),
    Distancia(PontoCad, PontoCad, f64),
    Angulo(SegCad, SegCad, f64),
    Raio(VecPathId, f64),
    Fixo(PontoCad, [f64; 2]),
}

/// Uma regra do documento. `referencia` = REF: só mostra a medida, não manda na geometria.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegraCad {
    pub lei: Lei,
    pub referencia: bool,
}

impl RegraCad {
    #[must_use]
    pub const fn nova(lei: Lei) -> Self {
        Self { lei, referencia: false }
    }
}

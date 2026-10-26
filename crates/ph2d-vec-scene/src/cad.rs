//! As regras de desenho técnico (W4, `docs/BlenderConstraints/04_W4_cad.md` §3.2): o MODELO, que viaja no
//! documento; quem as resolve é `ph2d-vec-constraints`. Endereçam pontos por `(path, VertId)`, nunca por índice.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{VecPath, VecPathId, VecScene, VertId};

mod circulo;
pub use circulo::circulo_de;

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
        Self {
            lei,
            referencia: false,
        }
    }
}

impl Lei {
    /// Todo ponto que a lei cita; um círculo citado (tangente, raio) conta como o seu `Centro`.
    #[must_use]
    pub fn pontos(&self) -> Vec<PontoCad> {
        let seg = |s: &SegCad| [s.0, s.1];
        match self {
            Self::Coincidente(a, b) | Self::Distancia(a, b, _) => vec![*a, *b],
            Self::Horizontal(s) | Self::Vertical(s) => seg(s).to_vec(),
            Self::Paralelo(s, t)
            | Self::Perpendicular(s, t)
            | Self::Igual(s, t)
            | Self::Angulo(s, t, _) => [seg(s), seg(t)].concat(),
            Self::Tangente(s, c) => vec![s.0, s.1, PontoCad::Centro(*c)],
            Self::Simetrico(a, b, s) => vec![*a, *b, s.0, s.1],
            Self::SobreRecta(a, s) => vec![*a, s.0, s.1],
            Self::Raio(c, _) => vec![PontoCad::Centro(*c)],
            Self::Fixo(a, _) => vec![*a],
        }
    }
}

impl PontoCad {
    #[must_use]
    pub const fn path(self) -> VecPathId {
        match self {
            Self::Vert(p, _) | Self::Centro(p) => p,
        }
    }
}

impl VecScene {
    #[must_use]
    pub fn regras(&self) -> &[RegraCad] {
        &self.regras
    }

    pub fn regras_mut(&mut self) -> &mut Vec<RegraCad> {
        &mut self.regras
    }

    /// Tira as regras que citam um ponto que já não existe (vértice apagado, path apagado). Corre no
    /// assentamento dos ids ([`VecScene::normaliza_ids_desde`]), que precede toda captura de undo ⇒ a regra sai no
    /// MESMO passo que o ponto. Um `Centro` só pede o path: um path que deixou de ser círculo fica `SemAlvo`.
    pub(crate) fn poda_regras(&mut self) -> bool {
        let existe = |p: PontoCad, path: Option<&VecPath>| match (p, path) {
            (PontoCad::Vert(_, v), Some(path)) => path.vert_by_id(v).is_some(),
            (PontoCad::Centro(_), Some(_)) => true,
            (_, None) => false,
        };
        let antes = self.regras.len();
        let paths = &self.paths;
        self.regras.retain(|r| {
            r.lei
                .pontos()
                .into_iter()
                .all(|p| existe(p, paths.iter().find(|q| q.id == p.path())))
        });
        self.regras.len() != antes
    }
}

/// A `VecScene` v23 — ⛔ congelada: mudá-la deixa de ler ficheiros reais.
#[derive(Deserialize)]
struct VecSceneV23 {
    paths: Vec<VecPath>,
    next_id: VecPathId,
}

#[derive(Deserialize)]
struct VecSceneV24 {
    paths: Vec<VecPath>,
    next_id: VecPathId,
    regras: Vec<RegraCad>,
}

impl<'de> Deserialize<'de> for VecScene {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if crate::stroke_wire::reading_geometry_v23() {
            let o = VecSceneV23::deserialize(d)?;
            Ok(Self {
                paths: o.paths,
                next_id: o.next_id,
                regras: Vec::new(),
            })
        } else {
            let o = VecSceneV24::deserialize(d)?;
            Ok(Self {
                paths: o.paths,
                next_id: o.next_id,
                regras: o.regras,
            })
        }
    }
}

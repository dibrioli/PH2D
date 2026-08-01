//! **A FRONTEIRA COM O MOTOR** — o que entra no `linesweeper` e o que ele devolve.
//!
//! Módulo irmão de [`super`] pelo teto de 700 LOC, e o corte é por RESPONSABILIDADE: ali moram as
//! OPERAÇÕES que o artista pede (`BoolOp`, `apply`, `apply_many`, `area`); aqui mora a passagem
//! para o motor exato — a conversão, a **guarda de entrada** e o motivo da recusa.
//!
//! ⚠️ É a passagem ÚNICA: todo caminho que chega ao `linesweeper` passa por
//! [`binary_grouped_checked`]. É isso que faz a guarda de finitude cobrir também o Expand e o
//! Shape Builder, que não sabem que ela existe.

use kurbo::{BezPath, PathEl, Point};
use linesweeper::{BinaryOp, FillRule as LsFillRule};

/// **A porta única do motor**: uma operação binária, com o resultado já AGRUPADO por
/// containment (o contorno de fora primeiro, os de dentro depois). `None` = o sweep
/// falhou.
///
/// Os contornos que saem daqui vêm **orientados** pelo linesweeper — e é isso que torna
/// `NonZero` e `EvenOdd` equivalentes a jusante. Quem constrói um conjunto a partir de
/// geometria de fora (o [`crate::expand`], que recebe contornos da kurbo) tem de passar por
/// aqui antes de compor, senão dois contornos de sentidos opostos se CANCELAM sob NonZero e
/// o resultado ganha um buraco que ninguém pediu.
pub(crate) fn binary_grouped(
    a: &BezPath,
    b: &BezPath,
    rule: LsFillRule,
    op: BinaryOp,
) -> Option<Vec<Vec<BezPath>>> {
    binary_grouped_checked(a, b, rule, op).ok()
}

/// Como [`binary_grouped`], mas **diz por que falhou**.
///
/// ⚠️ A versão que descarta o motivo existe para os consumidores que já tratam vazio como
/// resposta legítima (o Expand, o Shape Builder). Para o ARTISTA ela não serve: uma varredura que
/// falhou e um resultado legitimamente vazio (interseção de disjuntos) produzem o **mesmo nada** na
/// tela, e ele fica sem saber se a operação não tinha resposta ou se o motor desistiu — num crate
/// que se autodeclara *early beta*.
pub(crate) fn binary_grouped_checked(
    a: &BezPath,
    b: &BezPath,
    rule: LsFillRule,
    op: BinaryOp,
) -> Result<Vec<Vec<BezPath>>, SweepFailed> {
    // ⚠️ **A guarda de finitude é NOSSA, e não é cerimônia: sem ela o app CAI.** Medido — uma
    // coordenada `NaN` faz o `linesweeper` **PANICAR** lá dentro (`geom.rs:63`,
    // `assert!(x.is_finite())`) em vez de devolver o `Error::NaN` que ele declara: o `binary_op`
    // dele só examina o BOUNDING BOX, e um `min`/`max` com NaN devolve o outro operando, então o
    // NaN atravessa a checagem e explode no sweep.
    //
    // E a entrada é alcançável de verdade: um `Transform` degenerado assado na geometria
    // (ADR-0111) produz exactamente isto. Recusar aqui é a diferença entre um toast e um crash.
    for p in [a, b] {
        if !all_finite(p) {
            return Err(SweepFailed(
                "coordenada nao-finita (NaN ou infinito) na geometria".to_owned(),
            ));
        }
    }
    let contours = linesweeper::binary_op(a, b, rule, op).map_err(SweepFailed::from)?;
    Ok(contours
        .grouped()
        .iter()
        .map(|g| g.iter().map(|&i| contours[i].path.clone()).collect())
        .collect())
}

/// Toda coordenada do caminho é finita? A pergunta que o motor **assume** e não verifica.
fn all_finite(p: &BezPath) -> bool {
    p.elements().iter().all(|el| {
        let pts: &[Point] = match el {
            PathEl::MoveTo(a) | PathEl::LineTo(a) => std::slice::from_ref(a),
            PathEl::QuadTo(a, b) => {
                return a.x.is_finite() && a.y.is_finite() && b.x.is_finite() && b.y.is_finite();
            }
            PathEl::CurveTo(a, b, c) => {
                return [a, b, c].iter().all(|q| q.x.is_finite() && q.y.is_finite());
            }
            PathEl::ClosePath => &[],
        };
        pts.iter().all(|q| q.x.is_finite() && q.y.is_finite())
    })
}

/// **A varredura falhou** — e o motivo, na língua do motor.
///
/// Existe para o erro do `linesweeper` deixar de ser ENGOLIDO. Ele é um tipo do crate interno
/// (`kurbo`/`linesweeper` não cruzam esta fronteira, ADR-0108), então o que sai daqui é a
/// mensagem: quem a consome é um toast, não um `match`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SweepFailed(String);

impl SweepFailed {
    /// O motivo, para o artista ler.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.0
    }
}

impl From<linesweeper::Error> for SweepFailed {
    fn from(e: linesweeper::Error) -> Self {
        Self(format!("{e}"))
    }
}

impl std::fmt::Display for SweepFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

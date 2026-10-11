//! ⭐ **A CARA de cada alça** (F11, 11/10) — o idioma dos *Live Shapes* do Illustrator, lido pelas
//! fotos (doc 17 §1): cada widget diz o que faz pela forma, não pela cor. Derivada da LEI da alça
//! ([`HandleDesc`]), nunca de uma lista à parte: uma alça nova da tabela nasce com a cara certa.

use ph2d_vec_scene::{ShapeKind, ShapeValues};

use crate::shape_handles::{HandleDesc, Law, Spot};
use crate::shapes::{self, FieldUnit};

/// O desenho de uma alça no canvas.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Glyph {
    /// ⊙ — o *corner widget*: um aro com um ponto, um pouco para DENTRO do canto. Arredonda.
    Corner,
    /// ⇕ — o *side/points widget*: um aro com duas setas, na borda da caixa. Conta (lados, pontas).
    Count,
    /// ⊙ numa HASTE que sai do aro — o *pie widget*: o ângulo de um arco.
    Angle,
    /// ○ — o *R1/R2* da estrela: um aro claro maior, SOBRE o ponto que mede (um raio, um ponto).
    Radius,
}

/// A cara de `h`: o que arredonda é ⊙, o que conta é ⇕, o que gira é ⊙ na haste, o resto ○.
#[must_use]
pub fn glyph_of(h: &HandleDesc) -> Glyph {
    match (h.spot, h.law) {
        (Spot::Rounding { .. } | Spot::Apex(_), _) => Glyph::Corner,
        (_, Law::Steps | Law::Count) => Glyph::Count,
        (_, Law::Sweep { .. } | Law::ArcStart { .. }) => Glyph::Angle,
        _ => Glyph::Radius,
    }
}

/// ⭐ **A ETIQUETA do arrasto** — `«Points: 7»`, `«Inner: 0.38»`, `«R: 24 px»` (a ficha cinzenta do
/// Illustrator ao lado do cursor). `name` traduz o rótulo do campo (a porta dos nomes do painel);
/// `length` formata um comprimento de MUNDO na unidade do projecto (a da ficha da caixa).
#[must_use]
pub fn readout(
    kind: ShapeKind,
    h: &HandleDesc,
    values: &ShapeValues,
    name: &dyn Fn(&'static str) -> String,
    length: &dyn Fn(f64) -> String,
) -> String {
    let fields = shapes::desc(kind).fields;
    h.fields
        .iter()
        .filter_map(|&i| {
            let f = fields.get(i)?;
            let v = *values.get(i)?;
            let shown = match f.unit {
                FieldUnit::Count => format!("{}", v.round()),
                FieldUnit::Degrees => format!("{v:.0}\u{b0}"),
                FieldUnit::Px => length(v),
                FieldUnit::Ratio | FieldUnit::Choice(_) => format!("{v:.2}"),
            };
            Some(format!("{}: {shown}", name(f.label)))
        })
        .collect::<Vec<_>>()
        .join("  ")
}

/// O CÍRCULO TRACEJADO do arrasto de um raio (o *R2* do Illustrator): a elipse da caixa à escala
/// NORMALIZADA do ponto `at` (local) — `n` pontos, fechada. Vazio fora da lei `Radial`.
#[must_use]
pub fn radius_ring(h: &HandleDesc, at: [f64; 2], half: [f64; 2], n: usize) -> Vec<[f64; 2]> {
    if h.law != Law::Radial || half[0] <= 0.0 || half[1] <= 0.0 || n < 3 {
        return Vec::new();
    }
    let k = (at[0] / half[0]).hypot(at[1] / half[1]);
    #[expect(clippy::cast_precision_loss, reason = "n é pequeno")]
    (0..=n)
        .map(|j| {
            let a = std::f64::consts::TAU * j as f64 / n as f64;
            [half[0] * k * a.cos(), half[1] * k * a.sin()]
        })
        .collect()
}

#[cfg(test)]
#[path = "shape_handles_face_tests.rs"]
mod tests;

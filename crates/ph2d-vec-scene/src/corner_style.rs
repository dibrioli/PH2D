//! **O TIPO da quina** (F10) — módulo irmão de [`crate::corner_live`], que acha os pontos de
//! corte; aqui só se escolhe a LIGAÇÃO entre eles. Os quatro tipos do Affinity (*Rounded ·
//! Straight · Concave · Cutout*; o Illustrator: *Round · Chamfer · Inverted Round*).
//! Doc: `docs/UI_New_and_Simple/vetor_nova_ui/15_mais_parametros_das_formas.md`.
//!
//! ⭐ **O corte é o MESMO nos quatro**: o recuo sai do raio pela conta de sempre, então trocar o
//! tipo nunca mexe onde a quina começa (o widget de canto do Illustrator troca o tipo e mantém
//! os pontos). Gate: `the_four_corner_styles_cut_at_the_same_points`.

use crate::corners::circular_fillet;
use crate::{VecVertex, VertexKind};

/// O tipo da quina. O valor gravado no campo da forma é o índice (`0` = arredondada: o neutro,
/// o que todo save anterior a esta feature já desenhava).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum CornerStyle {
    /// O arco tangente às duas arestas (o filete).
    #[default]
    Round,
    /// A reta entre os dois cortes (o chanfro).
    Straight,
    /// O arco CENTRADO na quina — morde a forma para dentro (o *Inverted Round*).
    Concave,
    /// O entalhe: duas retas paralelas às arestas (o *Cutout* do Affinity).
    Cutout,
}

impl CornerStyle {
    /// O tipo de um valor de campo (`round` e fora da faixa = arredondada).
    #[must_use]
    pub fn of(v: f64) -> Self {
        match v.round() {
            x if x == 1.0 => Self::Straight,
            x if x == 2.0 => Self::Concave,
            x if x == 3.0 => Self::Cutout,
            _ => Self::Round,
        }
    }
}

/// A ligação entre `p_in` (o corte na aresta que CHEGA, marcha `t_in` — para a quina) e `p_out`
/// (o corte na que SAI, marcha `t_out` — para longe dela): o alçapão de saída de `p_in`, os
/// vértices do meio e o alçapão de entrada de `p_out`. O arredondado é do chamador (o filete do
/// `corner_live`, com a correcção do alçapão pelo raio) — aqui os outros três.
pub(crate) fn link(
    style: CornerStyle,
    p_in: [f64; 2],
    t_in: [f64; 2],
    p_out: [f64; 2],
    t_out: [f64; 2],
) -> ([f64; 2], Vec<VecVertex>, [f64; 2]) {
    let straight = || (third(p_in, p_out), Vec::new(), third(p_out, p_in));
    let Some(apex) = apex(p_in, t_in, p_out, t_out) else {
        return straight();
    };
    match style {
        CornerStyle::Round | CornerStyle::Straight => straight(),
        CornerStyle::Concave => {
            let (a, b) = (sub(p_in, apex), sub(p_out, apex));
            let r = 0.5 * (len(a) + len(b));
            let theta = (dot(a, b) / (len(a) * len(b)).max(1e-300))
                .clamp(-1.0, 1.0)
                .acos();
            let chord = sub(p_out, p_in);
            let toward = |v: [f64; 2]| {
                let n = [-v[1], v[0]];
                let n = if dot(n, chord) < 0.0 {
                    [-n[0], -n[1]]
                } else {
                    n
                };
                let l = len(n).max(1e-300);
                [n[0] / l, n[1] / l]
            };
            let f = circular_fillet(p_in, toward(a), p_out, toward(b), theta, r);
            (f.out1, f.mid.into_iter().collect(), f.in2)
        }
        CornerStyle::Cutout => {
            let c = [p_in[0] + p_out[0] - apex[0], p_in[1] + p_out[1] - apex[1]];
            let notch = VecVertex {
                anchor: c,
                in_handle: third(c, p_in),
                out_handle: third(c, p_out),
                kind: VertexKind::Corner,
                corner_radius: 0.0,
            };
            (third(p_in, c), vec![notch], third(p_out, c))
        }
    }
}

/// A QUINA que os dois cortes recortaram: o encontro das duas tangentes. `None` = paralelas.
fn apex(p_in: [f64; 2], t_in: [f64; 2], p_out: [f64; 2], t_out: [f64; 2]) -> Option<[f64; 2]> {
    // p_in + s·t_in = p_out − u·t_out
    let det = t_in[0] * t_out[1] - t_in[1] * t_out[0];
    if det.abs() < 1e-12 {
        return None;
    }
    let d = sub(p_out, p_in);
    let s = (d[0] * t_out[1] - d[1] * t_out[0]) / det;
    (s > 0.0).then(|| [p_in[0] + t_in[0] * s, p_in[1] + t_in[1] * s])
}

/// O alçapão de uma reta na forma canônica (⅓ do caminho de `p` a `q`) — a mesma do chanfro.
fn third(p: [f64; 2], q: [f64; 2]) -> [f64; 2] {
    [p[0] + (q[0] - p[0]) / 3.0, p[1] + (q[1] - p[1]) / 3.0]
}

fn sub(p: [f64; 2], q: [f64; 2]) -> [f64; 2] {
    [p[0] - q[0], p[1] - q[1]]
}

fn dot(p: [f64; 2], q: [f64; 2]) -> f64 {
    p[0] * q[0] + p[1] * q[1]
}

fn len(v: [f64; 2]) -> f64 {
    v[0].hypot(v[1])
}

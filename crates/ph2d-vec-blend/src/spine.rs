//! **O flow dos passos ao longo do SPINE** (ADR-0128) — módulo irmão, pelo teto de LOC.
//!
//! O motor de correspondência dá a cada passo uma FORMA (a interpolada) e uma posição default: o
//! **lerp dos centros das fontes**, numa reta. O spine editável re-distribui os passos por
//! **comprimento de arco** — cada passo, cujo lugar natural é a fração `s` da reta entre os
//! centros, é movido para a fração `s` do spine. É a "camada nova sobre o motor" do ADR-0128.
//!
//! Devolve um **deslocamento** por passo (não a forma pronta): o chamador tem a forma (do `Plan`) e
//! só a translada. `offset[k] = spine(s) − reta(s)`. Quando o spine **É** a reta default, todo
//! deslocamento é zero — e o chamador nem chama isto (o caminho não-editado fica byte-idêntico).

use kurbo::{CubicBez, ParamCurve, ParamCurveArclen, Point};
use ph2d_vec_scene::VecPath;

/// Precisão do comprimento de arco — apertada como o [`crate::ARCLEN_EPS`] do motor: um viés aqui
/// vira "o passo tremeu" quando o spine é animado.
const EPS: f64 = 1e-11;

/// O deslocamento de CADA passo (ordem link-major: elo 0 passos 1..=n, depois elo 1, …) para
/// segui-lo ao longo do `spine`, dados os `centers` das fontes (a reta default) e `n` passos/elo.
///
/// Vazio se o spine degenerar (< 2 vértices ou comprimento nulo) ou não houver elo/passo — o
/// chamador então não desloca nada (os passos ficam na reta do lerp).
#[must_use]
pub fn spine_offsets(spine: &VecPath, centers: &[[f64; 2]], n: usize) -> Vec<[f64; 2]> {
    let m = centers.len().saturating_sub(1); // nº de elos
    if m == 0 || n == 0 {
        return Vec::new();
    }
    let Some(arc) = SpineArc::new(spine) else {
        return Vec::new();
    };
    // A RETA default (a polilinha pelos centros): arco acumulado por elo.
    let mut cum = Vec::with_capacity(m + 1);
    let mut total = 0.0;
    for i in 0..m {
        cum.push(total);
        total += dist(centers[i], centers[i + 1]);
    }
    cum.push(total);
    if total <= EPS {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(m * n);
    for i in 0..m {
        let seg_len = cum[i + 1] - cum[i];
        for j in 1..=n {
            let lt = j as f64 / (n + 1) as f64;
            let default_pos = lerp(centers[i], centers[i + 1], lt);
            // A fração de arco DESTE passo na reta default → o mesmo ponto de arco no spine.
            let s = (cum[i] + lt * seg_len) / total;
            let sp = arc.eval(s);
            out.push([sp.x - default_pos[0], sp.y - default_pos[1]]);
        }
    }
    out
}

/// O `spine` como cadeia de cúbicas + arco acumulado — o mesmo padrão do [`crate::Outline`], mas
/// para um caminho ABERTO (uma reta entre âncoras vira a cúbica degenerada, cujo `inv_arclen`
/// devolve o ponto na fração de arco certa — o que faz a reta default casar com o lerp).
struct SpineArc {
    segs: Vec<CubicBez>,
    cum: Vec<f64>,
    total: f64,
}

impl SpineArc {
    fn new(spine: &VecPath) -> Option<SpineArc> {
        let v = &spine.verts;
        if v.len() < 2 {
            return None;
        }
        let segs: Vec<CubicBez> = v
            .windows(2)
            .map(|w| {
                CubicBez::new(
                    pt(w[0].anchor),
                    pt(w[0].out_handle),
                    pt(w[1].in_handle),
                    pt(w[1].anchor),
                )
            })
            .collect();
        let mut cum = Vec::with_capacity(segs.len() + 1);
        let mut total = 0.0;
        for s in &segs {
            cum.push(total);
            total += s.arclen(EPS);
        }
        cum.push(total);
        (total > EPS).then_some(SpineArc { segs, cum, total })
    }

    /// O ponto na fração de arco `s ∈ [0,1]` do spine inteiro.
    fn eval(&self, s: f64) -> Point {
        let target = s.clamp(0.0, 1.0) * self.total;
        let i = self.cum[1..]
            .partition_point(|&c| c <= target)
            .min(self.segs.len() - 1);
        let local = self.segs[i].inv_arclen((target - self.cum[i]).max(0.0), EPS);
        self.segs[i].eval(local.clamp(0.0, 1.0))
    }
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
fn lerp(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::{VecPath, VecVertex};

    fn spine(pts: &[[f64; 2]]) -> VecPath {
        VecPath {
            verts: pts.iter().map(|&p| VecVertex::corner(p)).collect(),
            closed: false,
            ..VecPath::default()
        }
    }

    /// **O spine RETO não desloca nada.** Quando o spine é a reta entre os centros, cada passo já
    /// está onde o lerp o pôs — o flow é o mapa identidade, e o caminho não-editado fica
    /// byte-idêntico (offset ~0).
    #[test]
    fn a_straight_spine_moves_nothing() {
        let centers = [[0.0, 0.0], [10.0, 0.0]];
        let offs = spine_offsets(&spine(&centers), &centers, 3);
        assert_eq!(offs.len(), 3);
        for o in offs {
            assert!(
                o[0].abs() < 1e-6 && o[1].abs() < 1e-6,
                "reta não desloca: {o:?}"
            );
        }
    }

    /// **O spine CURVO puxa os passos para a curva.** Um pico no meio (`[5,5]`) leva o passo do
    /// meio (fração de arco ½) até o topo — deslocado para CIMA da reta.
    #[test]
    fn a_bent_spine_pulls_the_steps_onto_the_curve() {
        let centers = [[0.0, 0.0], [10.0, 0.0]];
        // Spine com um pico no meio: ½ do arco cai exatamente no topo.
        let bent = spine(&[[0.0, 0.0], [5.0, 5.0], [10.0, 0.0]]);
        let offs = spine_offsets(&bent, &centers, 1);
        assert_eq!(offs.len(), 1);
        // O passo do meio (lerp em [5,0]) sobe para ~[5,5]: deslocamento ~[0,5].
        assert!(offs[0][0].abs() < 1e-6, "não desliza no x: {:?}", offs[0]);
        assert!(
            (offs[0][1] - 5.0).abs() < 1e-3,
            "sobe ~5 para o pico: {:?}",
            offs[0]
        );
    }

    /// A cadeia (3 fontes) devolve `n` deslocamentos POR ELO, em ordem link-major.
    #[test]
    fn a_chain_yields_n_offsets_per_link() {
        let centers = [[0.0, 0.0], [10.0, 0.0], [20.0, 0.0]];
        let offs = spine_offsets(&spine(&centers), &centers, 4);
        assert_eq!(offs.len(), 8, "2 elos × 4 passos");
    }
}

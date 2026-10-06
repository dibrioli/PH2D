//! **As duas perguntas antes da rota** — por onde cada ponta SAI da forma, e que formas a rota
//! precisa ENXERGAR. Partilhadas pelo modo Vector (`ph2d-app-vec::connector_live`) e pelo Quadro
//! (`ph2d-board-route`): a forma do contorno entra por um fecho, e a lei fica num sítio só.

use crate::{Aabb, Dir, RouteKind, side_towards};

/// O quanto o `spread` pode deslizar ao longo da face, como fração da meia-extensão dela. Passando
/// disto o ponto escorrega pela quina, e a linha parece brotar do canto da caixa.
pub const SPREAD_FACE_K: f64 = 0.45;

/// O quanto a região de interesse de uma rota se estende além das duas caixas, em múltiplos do
/// jetty — a folga em que uma forma ainda consegue empurrar a rota.
pub const ROI_PAD_K: f64 = 3.0;

/// **Onde a linha encosta, e por que lado ela sai** — uma ponta com âncora flutuante.
///
/// O lado vem do quadrante da diagonal da CAIXA (com a histerese de `prev`); o ponto vem do
/// contorno REAL (`hit(origem, raio)`, o cruzamento de MAIOR `t`), e não da caixa — numa estrela a
/// linha sai da ponta em vez de flutuar no vale. `hit = None` (ponta solta, ou forma sem contorno
/// fechado) cai na caixa.
///
/// O raio: para a rota **reta**, do centro rumo a `other` (é por ali que a linha passa); para a
/// **ortogonal**, ao longo do eixo do lado (a linha sai perpendicular à face).
///
/// O `spread` desliza a ORIGEM do raio, não o resultado: o ponto continua no contorno por
/// construção, seja qual for a forma — é o que separa dois conectores no mesmo par de formas.
#[must_use]
pub fn exit_point(
    bbox: Aabb,
    bound: bool,
    other: [f64; 2],
    kind: RouteKind,
    prev: Option<Dir>,
    spread: f64,
    hit: impl Fn([f64; 2], [f64; 2]) -> Option<[f64; 2]>,
) -> ([f64; 2], Dir) {
    let c = bbox.center();
    let (hw, hh) = half(bbox);
    let d = [other[0] - c[0], other[1] - c[1]];
    let side = side_towards(d, hw, hh, prev);
    let ray = if kind == RouteKind::Straight {
        d
    } else {
        side.vec()
    };
    let n = perp(ray);
    if !bound {
        // Ponta solta: não há face por onde deslizar, então o spread desloca o ponto direto.
        return ([c[0] + n[0] * spread, c[1] + n[1] * spread], side);
    }
    let limit = SPREAD_FACE_K * (n[0].abs() * hw + n[1].abs() * hh);
    let s = spread.clamp(-limit, limit);
    let from = [c[0] + n[0] * s, c[1] + n[1] * s];
    let at = hit(from, ray).unwrap_or_else(|| bbox_exit(bbox, from, ray));
    (at, side)
}

/// O lado por onde sai uma ponta presa a um PONTO FIXO `p` da forma de caixa `bbox`: a face que
/// olha para ele, vista do centro (com a mesma histerese).
#[must_use]
pub fn port_side(bbox: Aabb, p: [f64; 2], prev: Option<Dir>) -> Dir {
    let c = bbox.center();
    let (hw, hh) = half(bbox);
    side_towards([p[0] - c[0], p[1] - c[1]], hw, hh, prev)
}

/// Onde o raio `from + t·dir` SAI da caixa (o slab de maior `t`): o fallback de uma forma sem
/// contorno fechado — nunca o centro, que deixaria a linha por baixo da forma.
#[must_use]
pub fn bbox_exit(b: Aabb, from: [f64; 2], dir: [f64; 2]) -> [f64; 2] {
    let mut t = f64::INFINITY;
    for i in 0..2 {
        if dir[i].abs() > 1e-12 {
            let edge = if dir[i] > 0.0 { b.max[i] } else { b.min[i] };
            let tt = (edge - from[i]) / dir[i];
            if tt >= 0.0 {
                t = t.min(tt);
            }
        }
    }
    if !t.is_finite() {
        return from;
    }
    [from[0] + dir[0] * t, from[1] + dir[1] * t]
}

/// **Os obstáculos que ESTA rota precisa enxergar.**
///
/// O grafo de visibilidade tem `(2n+3)²` nós, então passar o documento inteiro encareceria toda
/// rota. A seleção é um PONTO FIXO, não um filtro: quem cruza a região entra, a região engole a
/// caixa de quem entrou (inflada por `pad`), repete — senão uma forma que obriga a linha a
/// contorná-la a empurraria para dentro de OUTRA que o filtro descartou. Termina sempre (o
/// conjunto só cresce) e, num diagrama denso, pega tudo — que é a resposta certa.
#[must_use]
pub fn obstacles_in_play(shapes: &[Aabb], a: Aabb, b: Aabb, pad: f64) -> Vec<Aabb> {
    let mut roi = a.union(b).inflate(pad);
    let mut taken = vec![false; shapes.len()];
    loop {
        let mut grew = false;
        for (i, s) in shapes.iter().enumerate() {
            if !taken[i] && roi.overlaps(*s) {
                taken[i] = true;
                roi = roi.union(s.inflate(pad));
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    shapes
        .iter()
        .zip(&taken)
        .filter_map(|(s, &t)| t.then_some(*s))
        .collect()
}

fn half(b: Aabb) -> (f64, f64) {
    ((b.max[0] - b.min[0]) * 0.5, (b.max[1] - b.min[1]) * 0.5)
}

/// A normal unitária de `v` (girada 90° à esquerda); `[0, 0]` para um vetor degenerado (o spread
/// não desloca nada: não há face definida).
fn perp(v: [f64; 2]) -> [f64; 2] {
    let l = v[0].hypot(v[1]);
    if l < 1e-12 {
        return [0.0, 0.0];
    }
    [-v[1] / l, v[0] / l]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bx(x0: f64, y0: f64, x1: f64, y1: f64) -> Aabb {
        Aabb::new([x0, y0], [x1, y1])
    }

    /// A região CRESCE: uma forma fora do corredor, colada a uma que o cruza, também barra a linha.
    #[test]
    fn the_region_swallows_the_neighbour_of_a_shape_that_crosses_it() {
        let (a, b) = (bx(0.0, 0.0, 1.0, 1.0), bx(9.0, 0.0, 10.0, 1.0));
        let crossing = bx(4.0, -3.0, 5.0, 2.0);
        let glued = bx(4.0, -6.0, 5.0, -3.5);
        let far = bx(40.0, 40.0, 41.0, 41.0);
        let got = obstacles_in_play(&[crossing, glued, far], a, b, 1.0);
        assert_eq!(got, vec![crossing, glued], "a colada entra, a distante não");
    }

    /// Ponta solta: o ponto é o próprio centro (mais o spread), nunca um cruzamento.
    #[test]
    fn a_loose_end_leaves_from_its_own_point() {
        let p = bx(3.0, 4.0, 3.0, 4.0);
        let (at, side) = exit_point(
            p,
            false,
            [10.0, 4.0],
            RouteKind::Orthogonal,
            None,
            0.0,
            |_, _| panic!("ponta solta não procura contorno"),
        );
        assert_eq!((at, side), ([3.0, 4.0], Dir::East));
    }

    /// Sem contorno, a saída é a borda da caixa na direcção do lado — nunca o centro.
    #[test]
    fn without_an_outline_the_exit_is_the_box_edge() {
        let b = bx(0.0, 0.0, 4.0, 2.0);
        let (at, side) = exit_point(
            b,
            true,
            [20.0, 1.0],
            RouteKind::Orthogonal,
            None,
            0.0,
            |_, _| None,
        );
        assert_eq!((at, side), ([4.0, 1.0], Dir::East));
    }

    /// O spread nunca leva o ponto além de 45 % da meia-face.
    #[test]
    fn the_spread_stays_on_the_face() {
        let b = bx(0.0, 0.0, 4.0, 2.0);
        let (at, _) = exit_point(
            b,
            true,
            [20.0, 1.0],
            RouteKind::Orthogonal,
            None,
            50.0,
            |_, _| None,
        );
        assert!((at[1] - (1.0 + SPREAD_FACE_K)).abs() < 1e-12, "{at:?}");
    }
}

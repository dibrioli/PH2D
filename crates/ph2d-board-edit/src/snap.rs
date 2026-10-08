//! **As guias de alinhamento** — uma borda ou o centro da caixa que se move «cola» numa borda ou no
//! centro de outro elemento quando passa a menos de `tol` dele, e desenha-se a linha que o diz; e o
//! **espaçamento igual** (o Miro: *«smart guides … equal spacing»*) — os vãos com os vizinhos da
//! mesma fila ficam iguais. A lei vale em quaisquer eixos: quem chama projecta (`guides.rs`).

/// Uma linha de guia (mundo), de `a` a `b`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    pub a: [f64; 2],
    pub b: [f64; 2],
}

/// Os três valores que alinham numa caixa `[x0, y0, x1, y1]` ao longo de um eixo.
fn stops(b: &[f64; 4], axis: usize) -> [f64; 3] {
    let (lo, hi) = (b[axis], b[axis + 2]);
    [lo, (lo + hi) / 2.0, hi]
}

/// Um encaixe: o deslocamento, o MENOR dos encaixes por eixo (`INFINITY` sem nenhum — é por ele que
/// se escolhe entre dois referenciais) e as guias.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Snap {
    pub delta: [f64; 2],
    pub best: f64,
    pub guides: Vec<Guide>,
}

/// O deslocamento que leva `moving` a colar nos `targets` (por eixo, o mais perto dentro de `tol`),
/// e as guias desse encaixe. `(0, 0)` e nenhuma guia quando nada está perto.
#[must_use]
pub(crate) fn snap_box(moving: [f64; 4], targets: &[[f64; 4]], tol: f64) -> Snap {
    let mut delta = [0.0; 2];
    let mut best = f64::INFINITY;
    let mut guides = Vec::new();
    for (axis, slot) in delta.iter_mut().enumerate() {
        let mine = stops(&moving, axis);
        let found = targets
            .iter()
            .flat_map(|t| stops(t, axis))
            .flat_map(|v| mine.iter().map(move |m| v - m))
            .filter(|d| d.abs() <= tol)
            .min_by(|a, b| a.abs().total_cmp(&b.abs()));
        let gap = gap_snap(&moving, targets, axis, tol);
        if let Some((d, g)) = gap.filter(|(g, _)| found.is_none_or(|e| g.abs() < e.abs())) {
            *slot = d;
            best = best.min(d.abs());
            guides.extend(g);
            continue;
        }
        let Some(d) = found else {
            continue;
        };
        *slot = d;
        best = best.min(d.abs());
        // A guia vai de uma ponta à outra de TUDO o que se alinha naquela linha.
        let snapped = moved(moving, axis, d);
        let other = 1 - axis;
        for line in stops(&snapped, axis) {
            let mut lo = snapped[other];
            let mut hi = snapped[other + 2];
            let mut hit = false;
            for t in targets {
                if stops(t, axis).iter().any(|v| (v - line).abs() < 1e-9) {
                    lo = lo.min(t[other]);
                    hi = hi.max(t[other + 2]);
                    hit = true;
                }
            }
            if hit {
                guides.push(if axis == 0 {
                    Guide {
                        a: [line, lo],
                        b: [line, hi],
                    }
                } else {
                    Guide {
                        a: [lo, line],
                        b: [hi, line],
                    }
                });
            }
        }
    }
    Snap {
        delta,
        best,
        guides,
    }
}

/// Um segmento ao longo do eixo `axis`, de `from` a `to`, à altura `at` do outro eixo.
fn seg(axis: usize, from: f64, to: f64, at: f64) -> Guide {
    if axis == 0 {
        Guide {
            a: [from, at],
            b: [to, at],
        }
    } else {
        Guide {
            a: [at, from],
            b: [at, to],
        }
    }
}

/// O meio da sobreposição de `a` e `b` no eixo `other`.
fn mid(a: &[f64; 4], b: &[f64; 4], other: usize) -> f64 {
    (a[other].max(b[other]) + a[other + 2].min(b[other + 2])) / 2.0
}

/// ⭐ O espaçamento igual de `m` no eixo `axis`, entre os alvos da sua FILA (os que se sobrepõem a
/// ele no outro eixo): ao meio entre os dois vizinhos, ou ao lado de um par com o vão desse par. O
/// deslocamento mais pequeno dentro de `tol` e os vãos a mostrar (dois segmentos do mesmo tamanho).
fn gap_snap(
    m: &[f64; 4],
    targets: &[[f64; 4]],
    axis: usize,
    tol: f64,
) -> Option<(f64, Vec<Guide>)> {
    let (lo, hi, other) = (axis, axis + 2, 1 - axis);
    let row: Vec<&[f64; 4]> = targets
        .iter()
        .filter(|t| t[other] < m[other + 2] && m[other] < t[other + 2])
        .collect();
    let before = |edge: f64| {
        row.iter()
            .copied()
            .filter(|t| t[hi] <= edge)
            .max_by(|a, b| a[hi].total_cmp(&b[hi]))
    };
    let after = |edge: f64| {
        row.iter()
            .copied()
            .filter(|t| t[lo] >= edge)
            .min_by(|a, b| a[lo].total_cmp(&b[lo]))
    };
    let c = (m[lo] + m[hi]) / 2.0;
    let (l, r) = (before(c), after(c));
    let at = |t: &[f64; 4]| mid(t, m, other);
    let mut found: Vec<(f64, Vec<Guide>)> = Vec::new();
    if let (Some(l), Some(r)) = (l, r) {
        let g = (r[lo] - l[hi] - (m[hi] - m[lo])) / 2.0;
        if g > 0.0 {
            let gl = seg(axis, l[hi], l[hi] + g, at(l));
            let gr = seg(axis, r[lo] - g, r[lo], at(r));
            found.push((l[hi] + g - m[lo], vec![gl, gr]));
        }
    }
    if let Some(l) = l
        && let Some(l2) = before(l[lo])
        && l[lo] > l2[hi]
    {
        let g = l[lo] - l2[hi];
        let pair = seg(axis, l2[hi], l[lo], mid(l2, l, other));
        found.push((
            l[hi] + g - m[lo],
            vec![pair, seg(axis, l[hi], l[hi] + g, at(l))],
        ));
    }
    if let Some(r) = r
        && let Some(r2) = after(r[hi])
        && r2[lo] > r[hi]
    {
        let g = r2[lo] - r[hi];
        let pair = seg(axis, r[hi], r2[lo], mid(r, r2, other));
        found.push((
            r[lo] - g - m[hi],
            vec![pair, seg(axis, r[lo] - g, r[lo], at(r))],
        ));
    }
    found
        .into_iter()
        .filter(|(d, _)| d.abs() <= tol)
        .min_by(|a, b| a.0.abs().total_cmp(&b.0.abs()))
}

fn moved(mut b: [f64; 4], axis: usize, d: f64) -> [f64; 4] {
    b[axis] += d;
    b[axis + 2] += d;
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_near_an_edge_snaps_to_it_and_draws_the_guide() {
        let target = [100.0, 0.0, 200.0, 50.0];
        // A borda esquerda (103) a 3 da borda esquerda do alvo.
        let Snap {
            delta: d,
            best,
            guides: g,
        } = snap_box([103.0, 100.0, 153.0, 150.0], &[target], 5.0);
        assert_eq!((d, best), ([-3.0, 0.0], 3.0));
        // As duas bordas da caixa (50 de largo) caem nas bordas 100 e 150 do alvo: duas guias.
        assert_eq!(
            g,
            vec![
                Guide {
                    a: [100.0, 0.0],
                    b: [100.0, 150.0]
                },
                Guide {
                    a: [150.0, 0.0],
                    b: [150.0, 150.0]
                },
            ]
        );
    }

    #[test]
    fn centres_snap_too_and_far_boxes_do_not() {
        let target = [0.0, 0.0, 100.0, 100.0];
        let s = snap_box([30.0, 200.0, 72.0, 240.0], &[target], 2.0);
        assert_eq!(s.delta, [-1.0, 0.0], "o centro 51 cola no centro 50");
        let far = snap_box([300.0, 300.0, 340.0, 340.0], &[target], 2.0);
        assert_eq!(
            (far.delta, far.best, far.guides.len()),
            ([0.0, 0.0], f64::INFINITY, 0)
        );
    }
}

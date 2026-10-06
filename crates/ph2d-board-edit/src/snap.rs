//! **As guias de alinhamento** — uma borda ou o centro da caixa que se move «cola» numa borda ou no
//! centro de outro elemento quando passa a menos de `tol` dele, e desenha-se a linha que o diz.

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

/// O deslocamento que leva `moving` a colar nos `targets` (por eixo, o mais perto dentro de `tol`),
/// e as guias desse encaixe. `(0, 0)` e nenhuma guia quando nada está perto.
#[must_use]
pub fn snap_box(moving: [f64; 4], targets: &[[f64; 4]], tol: f64) -> ([f64; 2], Vec<Guide>) {
    let mut delta = [0.0; 2];
    let mut guides = Vec::new();
    for (axis, slot) in delta.iter_mut().enumerate() {
        let mine = stops(&moving, axis);
        let best = targets
            .iter()
            .flat_map(|t| stops(t, axis))
            .flat_map(|v| mine.iter().map(move |m| v - m))
            .filter(|d| d.abs() <= tol)
            .min_by(|a, b| a.abs().total_cmp(&b.abs()));
        let Some(d) = best else {
            continue;
        };
        *slot = d;
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
    (delta, guides)
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
        let (d, g) = snap_box([103.0, 100.0, 153.0, 150.0], &[target], 5.0);
        assert_eq!(d, [-3.0, 0.0]);
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
        let (d, _) = snap_box([30.0, 200.0, 72.0, 240.0], &[target], 2.0);
        assert_eq!(d, [-1.0, 0.0], "o centro 51 cola no centro 50");
        let (far, g) = snap_box([300.0, 300.0, 340.0, 340.0], &[target], 2.0);
        assert_eq!((far, g.len()), ([0.0, 0.0], 0));
    }
}

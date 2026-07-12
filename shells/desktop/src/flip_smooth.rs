//! ADR-0114 W2 T2.7 — **active smoothing** do traço do Flip (o "assentar" que
//! separa uma mão de desenho premium de uma medíocre). Clean-room do
//! `paint.cc:544-621` (Grease Pencil 5.2): a intenção, não o código.
//!
//! **O mecanismo, e por que a cauda assenta enquanto a ponta ainda vibra:** o
//! smoothing é um **blur binomial 1D** (kernel `[1,2,1]/4` repetido — a
//! aproximação gaussiana transcendental-**free**, HR-5 limpo). O blur é **local**:
//! o valor suavizado de um ponto só depende dos ~`raio` vizinhos dele. Então, ao
//! adicionar uma amostra NOVA na ponta, a vizinhança de um ponto ANTIGO não muda
//! → ele "congela" sozinho, sem bookkeeping de convergência. Só os pontos perto
//! do cursor (vizinhança ainda incompleta) continuam se ajustando. Reblurar o
//! buffer inteiro a partir do CRU a cada frame dá exatamente esse comportamento.
//!
//! **Endpoints ancorados:** o 1º ponto (início do traço) e o ÚLTIMO (o cursor)
//! não se movem — a ponta segue o cursor sem lag; o interior é que assenta.

use ph2d_core::Vec2;

/// Máximo de iterações de blur (em `smoothing = 1.0`). Escala linear com o
/// slider Smoothing do painel.
const MAX_ITERS: u32 = 24;

/// Nº de iterações de blur para uma dada intensidade `smoothing ∈ [0,1]`.
#[must_use]
fn iters_for(smoothing: f32) -> u32 {
    (smoothing.clamp(0.0, 1.0) * MAX_ITERS as f32).round() as u32
}

/// Suaviza `raw` por blur binomial `[1,2,1]/4` com as PONTAS ancoradas, `iters`
/// vezes (de `smoothing`). Mantém a contagem de pontos (as pressões seguem 1:1);
/// a reamostragem/fit é do pen-up (T2.8). `iters = 0` (ou `< 3` pontos) devolve o
/// cru intacto.
#[must_use]
pub(crate) fn active_smooth(raw: &[Vec2], smoothing: f32) -> Vec<Vec2> {
    let iters = iters_for(smoothing);
    if iters == 0 || raw.len() < 3 {
        return raw.to_vec();
    }
    let mut cur = raw.to_vec();
    let mut next = cur.clone();
    for _ in 0..iters {
        // Interior só; `cur[0]` e `cur[last]` ficam ( pontas ancoradas ).
        for i in 1..cur.len() - 1 {
            next[i] = (cur[i - 1] + cur[i] * 2.0 + cur[i + 1]) * 0.25;
        }
        std::mem::swap(&mut cur, &mut next);
    }
    cur
}

/// Distância perpendicular de `p` à reta `a→b` (transcendental-free além do
/// `sqrt` do comprimento). `|cross(ab, ap)| / |ab|`.
fn perp_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let len = (ab.x * ab.x + ab.y * ab.y).sqrt();
    if len < 1e-9 {
        return (ap.x * ap.x + ap.y * ap.y).sqrt();
    }
    (ab.x * ap.y - ab.y * ap.x).abs() / len
}

/// **Simplify RDP** (Ramer–Douglas–Peucker) do pen-up (`paint.cc:1673-1799`):
/// devolve os ÍNDICES a manter (assim o chamador filtra as pressões junto),
/// preservando os pontos que desviam > `tol` (em MUNDO) da corda. Reduz a
/// contagem de pontos sem mudar a forma visível. `< 3` pontos = mantém tudo.
#[must_use]
pub(crate) fn simplify_rdp(points: &[Vec2], tol: f32) -> Vec<usize> {
    let n = points.len();
    if n < 3 {
        return (0..n).collect();
    }
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;
    // Pilha explícita (evita recursão profunda num traço longo).
    let mut stack = vec![(0usize, n - 1)];
    while let Some((first, last)) = stack.pop() {
        if last <= first + 1 {
            continue;
        }
        let mut max_d = 0.0;
        let mut idx = first;
        for i in (first + 1)..last {
            let d = perp_dist(points[i], points[first], points[last]);
            if d > max_d {
                max_d = d;
                idx = i;
            }
        }
        if max_d > tol {
            keep[idx] = true;
            stack.push((first, idx));
            stack.push((idx, last));
        }
    }
    (0..n).filter(|&i| keep[i]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(n: usize) -> Vec<Vec2> {
        (0..n).map(|i| Vec2::new(i as f32, 0.0)).collect()
    }

    #[test]
    fn anchors_endpoints() {
        let raw = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 5.0),
            Vec2::new(2.0, -5.0),
            Vec2::new(3.0, 0.0),
        ];
        let s = active_smooth(&raw, 1.0);
        assert_eq!(s.first(), raw.first(), "1º ponto ancorado");
        assert_eq!(s.last(), raw.last(), "último (cursor) ancorado");
    }

    #[test]
    fn smoothing_zero_is_identity() {
        let raw = line(10);
        assert_eq!(active_smooth(&raw, 0.0), raw);
    }

    #[test]
    fn blur_reduces_a_spike() {
        // Um pico no meio de uma linha reta deve baixar (interior suavizado).
        let mut raw = line(9);
        raw[4].y = 10.0;
        let s = active_smooth(&raw, 1.0);
        assert!(s[4].y < 10.0, "o pico foi suavizado ({} < 10)", s[4].y);
        assert!(s[4].y > 0.0, "mas não zerado de vez");
    }

    /// A propriedade PREMIUM: adicionar uma amostra na PONTA não mexe num ponto
    /// já longe do cursor (ele "congelou") — o blur é local.
    #[test]
    fn far_points_freeze_when_the_tip_grows() {
        let mut raw = line(20);
        // dá um caráter ao traço pra o blur ter o que fazer
        for (i, p) in raw.iter_mut().enumerate() {
            p.y = if i % 2 == 0 { 1.0 } else { -1.0 };
        }
        raw[0].y = 0.0;
        let before = active_smooth(&raw, 1.0);
        // cresce a ponta com mais uma amostra
        let mut grown = raw.clone();
        grown.push(Vec2::new(20.0, 0.5));
        let after = active_smooth(&grown, 1.0);
        // Um ponto bem atrás (índice 5) não pode ter mudado — está fora do alcance
        // do kernel a partir da nova ponta (24 iters ⇒ raio ~5; índice 5 vs ponta
        // ~20 está muito além).
        let d = (after[5] - before[5]).x.abs() + (after[5] - before[5]).y.abs();
        assert!(d < 1e-4, "ponto distante congelou (Δ={d})");
    }

    #[test]
    fn rdp_collapses_a_straight_line_to_endpoints() {
        let raw = line(20);
        let keep = simplify_rdp(&raw, 0.5);
        assert_eq!(keep, vec![0, 19], "reta vira só as pontas");
    }

    #[test]
    fn rdp_keeps_a_corner() {
        // "V": desce e sobe → o vértice do fundo é preservado.
        let raw = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(2.0, -2.0), // vértice
            Vec2::new(3.0, -1.0),
            Vec2::new(4.0, 0.0),
        ];
        let keep = simplify_rdp(&raw, 0.2);
        assert!(keep.contains(&2), "o canto do V é mantido: {keep:?}");
        assert!(keep.len() < 5, "mas os colineares somem");
    }

    #[test]
    fn rdp_preserves_short_strokes() {
        assert_eq!(simplify_rdp(&line(2), 0.5), vec![0, 1]);
    }
}

//! ADR-0113 W2 T2.7 — **active smoothing** do traço do Flip (o "assentar" que
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
}

//! ⭐ **O REFINO LOCAL do bake** (A13, 2026-10-06) — pontos a mais só onde a pose ESTICA o contorno.
//!
//! A `=6` com uma junta a `170°`: as amostras uniformes caem SOBRE a curva verdadeira (`2e-4`), mas
//! a tampa de fora da junta fica ENTRE duas delas e o ajuste corta-a por uma corda — até `0,28` de
//! cor que falta. Amostrar mais por igual não serve (piora o máximo, cabeçalho de
//! [`super::refit_pelo_bake`]): a cura é partir só o intervalo cujo comprimento POSTO passa de
//! `k × mediana` do segmento. Sem nenhum partido o bake é o uniforme, ao bit.
//!
//! ⛔ Medido e recusado (a tabela vive no commit da A13): o factor UNIFORME por segmento
//! (`⌈máx/mediana⌉`; `2,5×` o custo, nunca ao bit, e faz renascer o gancho do
//! `skin_desenho_zona_tests`) e os nós por CORDA (`α = 1` e `α = ½`, Barry–Goldman), piores que o
//! `t` da fonte contra o padrão-ouro e mais caros.

use super::{Assado, Point, SegmentoDaPele, Vec2};

/// ⭐ **O refino local** — parte cada intervalo uniforme cujo comprimento POSTO passa de
/// `k × mediana` do segmento, ao meio, até caber.
///
/// O único chão é de PRECISÃO: nunca abaixo da tolerância do ajuste (um intervalo posto mais curto
/// que ela não muda o que ele aceita) nem da resolução do `f64` (um `t` que já não se parte).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Refino {
    /// O múltiplo da mediana acima do qual um intervalo posto se parte.
    pub k: f64,
}

/// ⭐ **O refino do PRODUTO** (A13): `k = 2` com o `t` da fonte venceu `k = 3` e os nós por corda
/// (cordal, centrípeta) no padrão-ouro e na tampa da `=6` (de `0,27` a `0,036`). A tabela está no
/// commit da A13; as sondas `diag_a13_*` refazem-na.
pub const REFINO_DO_PRODUTO: Option<Refino> = Some(Refino { k: 2.0 });

/// O `t` da amostra `i` de `m` — a MESMA expressão de sempre (o bit do caso uniforme depende dela).
#[expect(clippy::cast_precision_loss, reason = "i <= m, um punhado")]
fn t_de(i: usize, m: usize) -> f64 {
    i as f64 / m as f64
}

/// As amostras de um segmento e, quando o refino partiu algum intervalo, os NÓS delas — o `t` da
/// fonte de cada uma (`None` = uniformes, e então o bake é o de sempre).
pub(super) fn amostra(
    s: &SegmentoDaPele<'_>,
    amostras: usize,
    tolerancia: f64,
    refino: Option<Refino>,
) -> (Vec<Point>, Option<Vec<f64>>) {
    let base: Vec<Point> = (0..=amostras).map(|i| s.ponto(t_de(i, amostras))).collect();
    let Some(Refino { k }) = refino else {
        return (base, None);
    };
    let mut l: Vec<f64> = base.windows(2).map(|w| (w[1] - w[0]).hypot()).collect();
    l.sort_by(f64::total_cmp);
    let limiar = (k * l.get(l.len() / 2).copied().unwrap_or(0.0)).max(tolerancia);
    let (mut pts, mut ts) = (vec![base[0]], vec![0.0]);
    for i in 0..amostras {
        parte(
            s,
            (t_de(i, amostras), base[i]),
            (t_de(i + 1, amostras), base[i + 1]),
            limiar,
            &mut pts,
            &mut ts,
        );
    }
    if pts.len() == base.len() {
        (base, None)
    } else {
        (pts, Some(ts))
    }
}

/// Parte `[a, b]` ao meio enquanto o comprimento POSTO passar do `limiar`; empurra `b` (e os do
/// meio) em ordem.
fn parte(
    s: &SegmentoDaPele<'_>,
    a: (f64, Point),
    b: (f64, Point),
    limiar: f64,
    pts: &mut Vec<Point>,
    ts: &mut Vec<f64>,
) {
    let tm = 0.5 * (a.0 + b.0);
    // Um `NaN` não parte. O `tm` que coincide com uma ponta é o fim da resolução.
    let longo = (b.1 - a.1).hypot() > limiar;
    if longo && tm > a.0 && tm < b.0 {
        let m = (tm, s.ponto(tm));
        parte(s, a, m, limiar, pts, ts);
        parte(s, m, b, limiar, pts, ts);
    } else {
        pts.push(b.1);
        ts.push(b.0);
    }
}

/// O intervalo de nós `[nos[i], nos[i+1]]` que contém `t` (o último fecha à direita).
fn intervalo(nos: &[f64], t: f64) -> usize {
    nos.partition_point(|&x| x <= t)
        .saturating_sub(1)
        .min(nos.len().saturating_sub(2))
}

impl Assado<'_> {
    /// ⭐ **O `em` com NÓS** — a Hermite de sempre em cada intervalo, com a tangente no nó `k` a
    /// diferença `(p[k+1] − p[k−1]) / (t[k+1] − t[k−1])`. Nas pontas, um nó VIRTUAL reflectido e o
    /// ponto repetido — sobre nós uniformes, a metade de corda da Catmull-Rom uniforme.
    pub(super) fn em_nos(&self, nos: &[f64], t: f64) -> (Point, Vec2) {
        let p = self.0;
        let n = p.len();
        if n < 2 {
            return (
                p.first().copied().unwrap_or(Point::ZERO),
                Vec2::new(1.0, 0.0),
            );
        }
        let t = t.clamp(0.0, 1.0);
        let i = intervalo(nos, t);
        let h = nos[i + 1] - nos[i];
        let f = (t - nos[i]) / h;
        let tg = |k: usize| -> Vec2 {
            let ta = if k == 0 {
                2.0 * nos[0] - nos[1]
            } else {
                nos[k - 1]
            };
            let tb = if k == n - 1 {
                2.0 * nos[n - 1] - nos[n - 2]
            } else {
                nos[k + 1]
            };
            (p[(k + 1).min(n - 1)] - p[k.saturating_sub(1)]) / (tb - ta)
        };
        let (m0, m1) = (tg(i) * h, tg(i + 1) * h);
        let (f2, f3) = (f * f, f * f * f);
        let (h00, h10, h01, h11) = (
            2.0 * f3 - 3.0 * f2 + 1.0,
            f3 - 2.0 * f2 + f,
            -2.0 * f3 + 3.0 * f2,
            f3 - f2,
        );
        let (d00, d10, d01, d11) = (
            6.0 * f2 - 6.0 * f,
            3.0 * f2 - 4.0 * f + 1.0,
            -6.0 * f2 + 6.0 * f,
            3.0 * f2 - 2.0 * f,
        );
        let (p1, p2) = (p[i].to_vec2(), p[i + 1].to_vec2());
        let q = p1 * h00 + m0 * h10 + p2 * h01 + m1 * h11;
        let d = (p1 * d00 + m0 * d10 + p2 * d01 + m1 * d11) / h;
        (q.to_point(), d)
    }

    /// O pedaço `range` cabe num intervalo de amostragem? (o fundo da recursão do ajuste, onde a
    /// Hermite do próprio bake sai). Com nós: não é mais longo que o MENOR intervalo que toca —
    /// sobre nós uniformes, a regra de sempre.
    pub(super) fn cabe_num_intervalo(&self, range: &core::ops::Range<f64>) -> bool {
        let len = range.end - range.start;
        match self.1 {
            None => {
                #[expect(clippy::cast_precision_loss, reason = "um punhado de amostras")]
                let passo = 1.0 / (self.0.len().max(2) - 1) as f64;
                len <= passo
            }
            Some(nos) => {
                let (a, b) = (intervalo(nos, range.start), intervalo(nos, range.end));
                let menor = (a..=b)
                    .map(|i| nos[i + 1] - nos[i])
                    .fold(f64::INFINITY, f64::min);
                len <= menor
            }
        }
    }

    /// A polilinha densa de `range` em que o [`super::fecha`] mede — `4` por intervalo de
    /// amostragem. Com nós: `4` por pedaço entre nós.
    pub(super) fn densos(&self, range: &core::ops::Range<f64>) -> Vec<Point> {
        let Some(nos) = self.1 else {
            #[expect(clippy::cast_precision_loss, reason = "um punhado de amostras")]
            let intervalos = (self.0.len().max(2) - 1) as f64;
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "o alcance é finito e positivo"
            )]
            let densos = ((range.end - range.start) * intervalos * 4.0)
                .ceil()
                .max(4.0) as usize;
            #[expect(clippy::cast_precision_loss, reason = "um punhado")]
            return (0..=densos)
                .map(|i| {
                    self.em(range.start + (range.end - range.start) * i as f64 / densos as f64)
                        .0
                })
                .collect();
        };
        let mut cortes = vec![range.start];
        cortes.extend(
            nos.iter()
                .copied()
                .filter(|&x| x > range.start && x < range.end),
        );
        cortes.push(range.end);
        let mut out = vec![self.em(range.start).0];
        for w in cortes.windows(2) {
            for j in 1..=4 {
                out.push(self.em(w[0] + (w[1] - w[0]) * f64::from(j) / 4.0).0);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ GATE — com nós UNIFORMES a Catmull-Rom com nós dá a uniforme (à máquina), e com nós
    /// IRREGULARES ela interpola e a tangente é a derivada (o contrato do fitter).
    #[test]
    fn os_nos_generalizam_a_uniforme_e_a_tangente_e_a_derivada() {
        let f = |t: f64| Point::new(10.0 * t + 2.0 * (t * 6.0).sin(), 3.0 * (t * 4.0).cos());
        let uni: Vec<f64> = (0..=12).map(|i| t_de(i, 12)).collect();
        let pts: Vec<Point> = uni.iter().map(|&t| f(t)).collect();
        let a = Assado(&pts, None);
        let b = Assado(&pts, Some(&uni));
        for k in 0..=100 {
            let t = f64::from(k) / 100.0;
            let (pa, da) = a.em(t);
            let (pb, db) = b.em(t);
            assert!(
                (pa - pb).hypot() < 1e-12 && (da - db).hypot() < 1e-9,
                "t={t}"
            );
        }
        let irr = [0.0, 0.05, 0.07, 0.1, 0.3, 0.31, 0.32, 0.6, 0.9, 1.0];
        let pts: Vec<Point> = irr.iter().map(|&t| f(t)).collect();
        let c = Assado(&pts, Some(&irr));
        for (t, p) in irr.iter().zip(&pts) {
            assert!((c.em(*t).0 - *p).hypot() < 1e-12, "não interpola em {t}");
        }
        let (mut pior, mut escala) = (0.0_f64, 0.0_f64);
        for k in 1..400 {
            let t = f64::from(k) / 400.0;
            if irr.iter().any(|x| (x - t).abs() < 1e-5) {
                continue;
            }
            const H: f64 = 1e-7;
            let fd = (c.em(t + H).0 - c.em(t - H).0) / (2.0 * H);
            pior = pior.max((fd - c.em(t).1).hypot());
            escala = escala.max(c.em(t).1.hypot());
        }
        assert!(
            pior < 1e-4 * escala,
            "tangente {pior} fora da derivada (escala {escala})"
        );
    }
}

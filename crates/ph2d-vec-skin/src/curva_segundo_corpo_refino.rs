//! ⭐ **O REFINO do bake pelos PESOS** (A13, 2026-10-06) — pontos a mais onde os pesos VARIAM ao
//! longo do segmento (as zonas de mistura das juntas), decididos no REPOUSO e nunca pela pose.
//!
//! A `=6` com uma junta a `170°`: as amostras uniformes caem sobre a curva verdadeira, mas a tampa
//! de fora da junta fica ENTRE duas e o ajuste corta-a por uma corda (`0,27`). Amostrar mais por
//! igual não serve ([`super::refit_pelo_bake`]). A 1.ª cura partia o intervalo cujo comprimento
//! POSTO passasse de `2 ×` a mediana — e o conjunto de amostras mudava com a pose: o desenho saltava
//! `0,041` num passo de `0,1°` com a verdade a mexer `0,0003` (report do dono, «aos saltos»).
//! Aqui o conjunto só depende da fonte e do campo: o desenho é contínuo na pose.
//!
//! ⛔ Medido e recusado (as tabelas vivem nos commits da A13): o esticão posto (`k = 2`, saltos),
//! o factor uniforme por segmento, os nós por corda (cordal, centrípeta) e a zona fixa (`Δw > ε`
//! em `m` pedaços: `m = 4` corta a tampa a `0,028`, `m = 8` custa mais `30 %` que esta lei).

use super::{Assado, Point, SegmentoDaPele, Vec2};
use kurbo::ParamCurve;

/// ⭐ **O refino pelos pesos** — cada intervalo uniforme parte-se em `⌈Δw / passo⌉` pedaços iguais
/// em `t`, com `Δw` a maior variação de um peso de mistura entre as suas pontas EM REPOUSO.
///
/// O número de pedaços não tem tecto: `Δw ≤ 1` ⇒ no máximo `⌈1 / passo⌉`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Refino {
    /// A variação de peso por pedaço.
    pub passo: f64,
}

/// ⭐ **O refino do PRODUTO**: `passo = 0,025` (a tampa da `=6` a `170°` fica a `0,0045`–`0,0077`
/// da verdade, contra `0,018`–`0,028` a `0,05`; tabela no commit da A13, sondas `diag_a13_*`).
pub const REFINO_DO_PRODUTO: Option<Refino> = Some(Refino { passo: 0.025 });

/// O `t` da amostra `i` de `m` — a MESMA expressão de sempre (o bit do caso uniforme depende dela).
#[expect(clippy::cast_precision_loss, reason = "i <= m, um punhado")]
fn t_de(i: usize, m: usize) -> f64 {
    i as f64 / m as f64
}

/// As amostras de um segmento e, quando o refino partiu algum intervalo, os NÓS delas — o `t` da
/// fonte de cada uma (`None` = uniformes, e então o bake é o de sempre, ao bit).
pub(super) fn amostra(
    s: &SegmentoDaPele<'_>,
    amostras: usize,
    refino: Option<Refino>,
) -> (Vec<Point>, Option<Vec<f64>>) {
    let Some(Refino { passo }) = refino else {
        return (
            (0..=amostras).map(|i| s.ponto(t_de(i, amostras))).collect(),
            None,
        );
    };
    // O `ponto` de cada amostra, aberto: os pesos ficam para a decisão (ao bit o `ponto`).
    let base: Vec<(Vec<f64>, Point)> = (0..=amostras)
        .map(|i| {
            let t = t_de(i, amostras);
            let c = s.src.eval(t);
            let p = [c.x, c.y];
            let w = s.pesos(p, t);
            let q = s.mistura(p, &w);
            (w, q)
        })
        .collect();
    let pedacos: Vec<u32> = base
        .windows(2)
        .map(|par| {
            let dw = par[0]
                .0
                .iter()
                .zip(&par[1].0)
                .map(|(a, b)| (b - a).abs())
                .fold(0.0, f64::max);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "Δw ≤ 1 ⇒ ⌈Δw/passo⌉ pequeno"
            )]
            let m = (dw / passo).ceil() as u32;
            m.max(1)
        })
        .collect();
    if pedacos.iter().all(|m| *m == 1) {
        return (base.into_iter().map(|x| x.1).collect(), None);
    }
    let (mut pts, mut ts) = (vec![base[0].1], vec![0.0]);
    for (i, &m) in pedacos.iter().enumerate() {
        let (t0, t1) = (t_de(i, amostras), t_de(i + 1, amostras));
        for j in 1..m {
            let f = f64::from(j) / f64::from(m);
            let t = t0 + (t1 - t0) * f;
            let c = s.src.eval(t);
            // Os pesos do pedaço são a MISTURA dos das pontas: a amostra a mais resolve a GEOMETRIA
            // da pose sem resolver os bicos do campo abaixo do passo uniforme.
            let w: Vec<f64> = base[i]
                .0
                .iter()
                .zip(&base[i + 1].0)
                .map(|(a, b)| (b - a).mul_add(f, *a))
                .collect();
            pts.push(s.mistura([c.x, c.y], &w));
            ts.push(t);
        }
        pts.push(base[i + 1].1);
        ts.push(t1);
    }
    (pts, Some(ts))
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
        // A tangente num nó INTERIOR é a diferença centrada pelos nós vizinhos (os passos à volta de
        // `0,05` são `0,05` e `0,02`: o `2h` de um deles daria outra).
        for k in [1, 4, 7] {
            let fd = (pts[k + 1] - pts[k - 1]) / (irr[k + 1] - irr[k - 1]);
            let tg = c.em(irr[k]).1;
            assert!(
                (tg - fd).hypot() < 1e-9 * fd.hypot(),
                "nó {k}: tangente {tg:?}, diferença {fd:?}"
            );
        }
        // Um pedaço EXACTAMENTE entre dois nós cabe (com nós e sem eles).
        assert!(
            c.cabe_num_intervalo(&(irr[1]..irr[2])),
            "o intervalo [nó 1, nó 2]"
        );
        assert!(
            !c.cabe_num_intervalo(&(irr[1]..irr[3])),
            "controlo: dois intervalos"
        );
        assert!(a.cabe_num_intervalo(&(0.0..1.0 / 12.0)), "o passo uniforme");
        // `4` densos por pedaço entre nós, e só os nós DENTRO do alcance cortam.
        assert_eq!(c.densos(&(0.0..1.0)).len(), 1 + 4 * (irr.len() - 1));
        let r = 0.06..0.5;
        let d = c.densos(&r);
        assert_eq!(d.len(), 1 + 4 * 6, "os cortes de {r:?}");
        assert_eq!(d[0], c.em(r.start).0);
        assert!(
            (d[d.len() - 1] - c.em(r.end).0).hypot() < 1e-12,
            "a ponta de {r:?}"
        );
    }

    /// Um osso deitado no `+X` de `(x0, 0)` a `(x0 + len, 0)`, rodado `rot` em torno da raiz.
    fn osso(x0: f64, len: f64, rot: f64) -> ph2d_skeleton::SkinBone {
        use ph2d_skeleton::Xform;
        let (c, s) = (rot.cos(), rot.sin());
        ph2d_skeleton::SkinBone::new(
            Xform([1.0, 0.0, 0.0, 1.0, x0, 0.0]),
            len,
            1.0,
            Xform([c, s, -s, c, x0, 0.0]),
            Xform::IDENTITY,
        )
        .expect("repouso não singular")
    }

    fn bits(p: &ph2d_vec_scene::VecPath) -> Vec<u64> {
        p.verts_all()
            .flat_map(|v| [v.anchor, v.in_handle, v.out_handle])
            .flat_map(|q| q.map(f64::to_bits))
            .collect()
    }

    /// ⭐⭐ GATE — **onde nenhum peso varia, o refino é o bake uniforme AO BIT.** Uma forma presa a UM
    /// osso tem peso `1` em todo ponto (medido aqui: `Δw = 0`), logo nenhum intervalo se parte. O
    /// CONTROLO: a mesma asserção com DOIS ossos (a junta a meio) tem de falhar.
    #[test]
    fn sem_peso_que_varie_o_refino_e_ao_bit_o_bake_uniforme() {
        let fonte = ph2d_vec_scene::cook(
            ph2d_vec_scene::ShapeKind::Rectangle,
            [0.0, 0.0],
            [40.0, 10.0],
            &[],
        );
        let um = ph2d_skeleton::Skin::new(vec![osso(0.0, 40.0, 0.5)]).expect("1 osso");
        let dois = ph2d_skeleton::Skin::new(vec![osso(0.0, 20.0, 0.0), osso(20.0, 20.0, 0.8)])
            .expect("2 ossos");
        let assa = |pele: &ph2d_skeleton::Skin, refino: Option<Refino>| {
            super::super::assa_a_pele(
                pele,
                &fonte,
                &[],
                &[],
                true,
                super::super::CampoIndexado::default(),
                super::super::Bake {
                    amostras: 16,
                    tolerancia: 3e-4 * 41.2,
                    refino,
                },
            )
        };
        let variacao = |pele: &ph2d_skeleton::Skin| {
            let mut w0 = pele.scratch();
            let mut w1 = pele.scratch();
            (0..400)
                .map(|i| {
                    let x = 40.0 * f64::from(i) / 400.0;
                    pele.weights_corrected([x, 0.0], None, &mut w0, &[]);
                    pele.weights_corrected([x + 0.1, 0.0], None, &mut w1, &[]);
                    w0.iter()
                        .zip(&w1)
                        .map(|(a, b)| (b - a).abs())
                        .fold(0.0, f64::max)
                })
                .fold(0.0, f64::max)
        };
        assert!(
            variacao(&um) == 0.0,
            "a fixtura de UM osso tem um peso que varia"
        );
        assert_eq!(
            bits(&assa(&um, REFINO_DO_PRODUTO)),
            bits(&assa(&um, None)),
            "sem peso que varie o refino mexeu no bake"
        );
        assert!(
            variacao(&dois) > 0.0,
            "o CONTROLO: dois ossos e nenhum peso varia"
        );
        assert_ne!(
            bits(&assa(&dois, REFINO_DO_PRODUTO)),
            bits(&assa(&dois, None)),
            "o CONTROLO: com a junta a meio o refino não partiu nada"
        );
    }
}

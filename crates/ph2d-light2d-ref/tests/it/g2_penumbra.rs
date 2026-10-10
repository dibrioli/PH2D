//! **G2** — disco + oclusor de aresta viva, NÃO alinhado aos eixos: a penumbra é a projeção
//! geométrica do disco pela aresta, e cresce com o raio `a`.
//!
//! Fixture, num referencial girado de `PHI` (25°) com origem na aresta `E`: o oclusor é o
//! quadrante `x' ≥ 0, y' ≤ 0` (um quadrado de 1 km com um canto em `E`); o disco está em
//! `(-2,5; -1,5)` — dentro de `x' < 0, y' < 0`, de onde `E` é a silhueta para TODO ponto do disco;
//! os receptores estão numa reta perpendicular a `L→E`, `D` metros além de `E` (em `x', y' > 0`).
//!
//! Oráculo analítico, sem varredura: de um receptor `P`, o raio de direção `θ` desce até ao disco
//! e cruza `y' = 0` num `x'_c`; ele é BLOQUEADO sse `x'_c ≥ 0`. O corte é exatamente a direção
//! `P→E`, logo `F(P) = L·(|parte iluminada| + (1−α)·|parte bloqueada|)` do intervalo
//! `[γ−δ, γ+δ]` subtendido pelo disco. A penumbra vai de `s = −D·tan(asin(a/d))` a `+D·tan(…)`:
//! largura `W = 2·D·a/√(d² − a²)`.

use ph2d_light2d_ecs::{CenaDeLuz, Oclusor};
use ph2d_light2d_ref::fluencia;

use crate::comum::{cena, disco, dist, f64x2, para_mundo};

const PHI: f64 = 25.0 * std::f64::consts::PI / 180.0;
const E: [f32; 2] = [1.3, -0.7];
const CENTRO_LOCAL: [f64; 2] = [-2.5, -1.5];
const D: f64 = 2.0;
const L: f32 = 1.75;

/// Contra o oráculo, relativo a `2·δ·L`. Medido: 6,5e-14 (handoff da W1); folga ~15×.
const TOL_G2: f64 = 1e-12;
/// Largura achada por bissecção contra `W` previsto, em metros. Medido: 5,7e-13; folga ~17×.
const TOL_LARGURA_M: f64 = 1e-11;

struct Fixture {
    cena: CenaDeLuz,
    c: [f64; 2],
    e: [f64; 2],
    a: f64,
    alfa: f64,
}

fn fixture(a: f32, alfa: f32) -> Fixture {
    const S: f64 = 1000.0;
    let quadrante = [[0.0, 0.0], [S, 0.0], [S, -S], [0.0, -S]];
    let c = para_mundo(PHI, E, CENTRO_LOCAL);
    Fixture {
        cena: cena(
            vec![disco(c, a, [L; 3], f32::INFINITY)],
            vec![Oclusor {
                vertices: quadrante.iter().map(|&v| para_mundo(PHI, E, v)).collect(),
                opacidade: alfa,
                albedo: [0.0; 3],
            }],
        ),
        c: f64x2(c),
        e: f64x2(E),
        a: f64::from(a),
        alfa: f64::from(alfa),
    }
}

impl Fixture {
    fn d(&self) -> f64 {
        dist(self.c, self.e)
    }
    /// `u` = direção `L→E`; `v` = perpendicular.
    fn receptor(&self, s: f64) -> [f64; 2] {
        let d = self.d();
        let u = [(self.e[0] - self.c[0]) / d, (self.e[1] - self.c[1]) / d];
        let v = [-u[1], u[0]];
        [
            self.e[0] + D * u[0] + s * v[0],
            self.e[1] + D * u[1] + s * v[1],
        ]
    }
    fn largura_prevista(&self) -> f64 {
        let d = self.d();
        2.0 * D * self.a / (d * d - self.a * self.a).sqrt()
    }
    fn bloqueado(&self, p: [f64; 2], theta: f64) -> bool {
        let (s, c) = (-PHI).sin_cos();
        let local = |x: f64, y: f64| [c * x - s * y, s * x + c * y];
        let pl = local(p[0] - self.e[0], p[1] - self.e[1]);
        let dl = local(theta.cos(), theta.sin());
        if dl[1] >= 0.0 {
            return false;
        }
        pl[0] + (-pl[1]) / dl[1] * dl[0] >= 0.0
    }
    /// O oráculo: `(F, 2·δ·L)`.
    fn analitico(&self, p: [f64; 2]) -> (f64, f64) {
        let l = f64::from(L);
        let rho = dist(p, self.c);
        let delta = (self.a / rho).asin();
        let gama = (self.c[1] - p[1]).atan2(self.c[0] - p[0]);
        let eps = (self.e[1] - p[1]).atan2(self.e[0] - p[0]);
        let (lo, hi) = (gama - delta, gama + delta);
        let mut e = eps - gama;
        e -= std::f64::consts::TAU * (e / std::f64::consts::TAU).round();
        let e = gama + e;
        let (iluminada, bloqueada) = if e <= lo || e >= hi {
            if self.bloqueado(p, gama) {
                (0.0, 2.0 * delta)
            } else {
                (2.0 * delta, 0.0)
            }
        } else if self.bloqueado(p, 0.5 * (e + hi)) {
            (e - lo, hi - e)
        } else {
            (hi - e, e - lo)
        };
        (
            l * (iluminada + (1.0 - self.alfa) * bloqueada),
            2.0 * delta * l,
        )
    }
    fn f(&self, s: f64) -> f64 {
        fluencia(&self.cena, self.receptor(s))[0]
    }
}

#[test]
fn g2_a_fluencia_na_penumbra_e_a_do_oraculo_geometrico() {
    let mut pior = 0.0_f64;
    for alfa in [1.0_f32, 0.5] {
        for a in [0.05_f32, 0.1, 0.2, 0.4] {
            let fx = fixture(a, alfa);
            let w = fx.largura_prevista();
            for k in 0..=200 {
                let s = -w + 2.0 * w * f64::from(k) / 200.0;
                let p = fx.receptor(s);
                let (esperado, escala) = fx.analitico(p);
                let obtido = fluencia(&fx.cena, p);
                for ch in obtido {
                    let erro = (ch - esperado).abs() / escala;
                    pior = pior.max(erro);
                    assert!(
                        erro <= TOL_G2,
                        "α={alfa} a={a} s={s}: gabarito {ch} × oráculo {esperado} (erro {erro:e})"
                    );
                }
            }
        }
    }
    eprintln!("G2: erro máximo medido contra o oráculo = {pior:e} (relativo a 2δL)");
}

/// Bissecção sobre o GABARITO até a borda onde o predicado muda.
fn borda(mut dentro: f64, mut fora: f64, pred: impl Fn(f64) -> bool) -> f64 {
    assert!(
        pred(dentro) && !pred(fora),
        "controlo: a bissecção começa com o predicado nos dois lados"
    );
    for _ in 0..200 {
        let m = 0.5 * (dentro + fora);
        if pred(m) {
            dentro = m;
        } else {
            fora = m;
        }
    }
    0.5 * (dentro + fora)
}

#[test]
fn g2_a_largura_da_penumbra_e_a_projecao_do_disco_pela_aresta_e_cresce_com_a() {
    let mut larguras = Vec::new();
    let mut pior = 0.0_f64;
    for a in [0.05_f32, 0.1, 0.2, 0.4] {
        let fx = fixture(a, 1.0);
        let w = fx.largura_prevista();
        let vazio = |s: f64| fx.analitico(fx.receptor(s)).1;
        // De que lado fica a sombra: o gabarito diz (umbra = 0 exato).
        let (umbra, luz) = if fx.f(-w) == 0.0 { (-w, w) } else { (w, -w) };
        assert_eq!(fx.f(umbra), 0.0, "a={a}: a umbra tem de ser escura");
        assert!(
            (fx.f(luz) - vazio(luz)).abs() <= TOL_G2 * vazio(luz),
            "a={a}: fora da penumbra a luz é a do vazio"
        );
        let s0 = borda(umbra, luz, |s| fx.f(s) == 0.0);
        let s1 = borda(luz, umbra, |s| fx.f(s) >= vazio(s) * (1.0 - 1e-12));
        let medida = (s1 - s0).abs();
        let erro = (medida - w).abs();
        pior = pior.max(erro);
        assert!(
            erro <= TOL_LARGURA_M,
            "a={a}: penumbra medida {medida} × prevista {w} (erro {erro:e} m)"
        );
        larguras.push(medida);
    }
    assert!(
        larguras.windows(2).all(|p| p[1] > p[0]),
        "a penumbra tem de crescer com a: {larguras:?}"
    );
    eprintln!("G2: larguras {larguras:?}, erro máximo da largura = {pior:e} m");
}

//! ⭐⭐⭐ **A ÚNICA PROPRIEDADE QUE TORNA UM MAJORANTE UM MAJORANTE: contenção.**
//!
//! Um intervalo que não contém algum ponto que diz conter não é conservador — é **errado**, e o
//! erro dele é o que não falha: o bound sai pequeno demais e o campo fura.
//!
//! ⚠️ **A metade da DERIVADA é a que quase não se escreve**, e é onde um sinal trocado se esconde:
//! aqui ela é medida por diferença central sobre o mesmo ponto, contra o intervalo que a
//! diferenciação automática devolveu.

use super::{D, Iv};

/// Um gerador determinístico — ⚠️ um teste de propriedade com semente do relógio reprova noutro dia
/// sobre o mesmo código.
struct Lcg(u64);

impl Lcg {
    fn f(&mut self, lo: f64, hi: f64) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        #[allow(clippy::cast_precision_loss)]
        let u = (self.0 >> 11) as f64 / (1u64 << 53) as f64;
        lo + u * (hi - lo)
    }
}

fn dentro(x: f64, i: Iv, folga: f64) -> bool {
    x >= i.lo - folga && x <= i.hi + folga
}

/// ⭐ **Cada operação, sozinha**: o resultado pontual cai dentro do intervalo que ela promete.
#[test]
fn an_interval_contains_every_point_it_claims_to() {
    let mut r = Lcg(0x5DEE_CE66);
    let mut medidos = 0usize;
    for _ in 0..4000 {
        let (a0, a1) = (r.f(-3.0, 3.0), r.f(-3.0, 3.0));
        let (b0, b1) = (r.f(-3.0, 3.0), r.f(-3.0, 3.0));
        let (a, b) = (Iv::new(a0, a1), Iv::new(b0, b1));
        for _ in 0..6 {
            let (x, y) = (r.f(a.lo, a.hi), r.f(b.lo, b.hi));
            medidos += 1;
            assert!(dentro(x + y, a.add(b), 1e-12), "add");
            assert!(dentro(x - y, a.sub(b), 1e-12), "sub");
            assert!(dentro(-x, a.neg(), 1e-12), "neg");
            assert!(dentro(x * y, a.mul(b), 1e-12), "mul");
            assert!(dentro(x * x, a.square(), 1e-12), "square");
            assert!(dentro(x.abs(), a.abs(), 1e-12), "abs");
            assert!(dentro(x.max(0.5), a.max_pt(0.5), 1e-12), "max_pt");
            assert!(dentro(x.min(0.5), a.min_pt(0.5), 1e-12), "min_pt");
            assert!(dentro(x.cos(), a.cos(), 1e-12), "cos em {a:?}");
            assert!(dentro(x.sin(), a.sin(), 1e-12), "sin em {a:?}");
            assert!(dentro(x.atan(), a.atan(), 1e-12), "atan");
            if a.lo >= 0.0 {
                assert!(dentro(x.sqrt(), a.sqrt(), 1e-12), "sqrt");
            }
            let q = a.div(b);
            if q.is_finite() {
                assert!(dentro(x / y, q, 1e-9), "div {a:?}/{b:?}");
            }
        }
    }
    assert!(medidos > 20_000, "só {medidos} amostras — o laço partiu-se");
}

/// A expressão de prova: ela usa **todas** as operações que as três leis usam, encadeadas.
fn expr_pt(p: [f64; 3]) -> f64 {
    let k = (0.6f64.mul_add(p[1], 1.0)).max(0.35);
    let a = (1.3 - p[0] / k).max(0.13);
    let b = p[2] / k;
    let rr = (a * a + b * b).sqrt();
    let th = (b / a).atan();
    let c = th.clamp(-0.4, 0.4);
    ((1.3 - rr * (th - c).cos()) * k + rr * (th - c).sin() * p[1].abs()).sin()
}

fn expr_iv(p: [D; 3]) -> D {
    let k = p[1].escala(0.6).add(D::cte(1.0)).max_pt(0.35);
    let a = D::cte(1.3).sub(p[0].div(k)).max_pt(0.13);
    let b = p[2].div(k);
    let rr = a.square().add(b.square()).sqrt();
    let th = b.div(a).atan();
    let c = th.max_pt(-0.4).min_pt(0.4);
    let d = th.sub(c);
    D::cte(1.3)
        .sub(rr.mul(d.cos()))
        .mul(k)
        .add(rr.mul(d.sin()).mul(p[1].abs()))
        .sin()
}

/// ⭐⭐⭐ **A metade que vale por todas: o VALOR e as TRÊS DERIVADAS caem dentro.**
///
/// ⛔ Prova de mutação: trocar o sinal da derivada do `cos` (`−sin` para `+sin`) reprova aqui, e
/// **não** reprova no gate das operações isoladas — que só olha valores.
#[test]
fn the_dual_contains_the_value_and_every_partial() {
    let mut r = Lcg(0x1234_5678);
    let mut medidos = 0usize;
    for _ in 0..1500 {
        let c = [r.f(-0.8, 0.8), r.f(-0.8, 0.8), r.f(-0.8, 0.8)];
        // ⚠️ Uma caixa **larga** de propósito: se ela fosse um ponto, a contenção seria trivial.
        let raio = r.f(0.0, 0.15);
        let caixa: [Iv; 3] = std::array::from_fn(|e| Iv::new(c[e] - raio, c[e] + raio));
        let out = expr_iv(std::array::from_fn(|e| D::var(caixa[e], e)));
        if !out.v.is_finite() || out.d.iter().any(|x| !x.is_finite()) {
            continue;
        }
        for _ in 0..4 {
            let p: [f64; 3] = std::array::from_fn(|e| r.f(caixa[e].lo, caixa[e].hi));
            medidos += 1;
            assert!(
                dentro(expr_pt(p), out.v, 1e-9),
                "o VALOR {} caiu fora de {:?}",
                expr_pt(p),
                out.v
            );
            for e in 0..3 {
                let h = 1.0e-6;
                let (mut ap, mut bp) = (p, p);
                ap[e] += h;
                bp[e] -= h;
                let dd = (expr_pt(ap) - expr_pt(bp)) / (2.0 * h);
                assert!(
                    dentro(dd, out.d[e], 2e-4),
                    "a DERIVADA {e} vale {dd:.6} e o intervalo diz {:?} (caixa {caixa:?})",
                    out.d[e]
                );
            }
        }
    }
    assert!(
        medidos > 2_000,
        "só {medidos} amostras — a população evaporou"
    );
}

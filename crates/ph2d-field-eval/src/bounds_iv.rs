//! ⭐⭐⭐ **INTERVALOS COM DERIVADA** — o substrato do bound da composição
//! (`docs/3DModeling/09_o_bound_da_composicao.md`).
//!
//! # Por que um tipo, e não uma conta à mão
//!
//! O bound novo precisa do **jacobiano da composição** majorado sobre uma caixa. Escrever cada
//! entrada à mão para as três leis dá ~40 expressões, e uma delas com um sinal trocado devolve um
//! majorante **pequeno demais** — que é o defeito que não falha: o campo fura.
//!
//! ⇒ as leis escrevem-se **uma vez**, genéricas, e a derivada sai por **diferenciação automática
//! para a frente** sobre intervalos. Se as operações deste ficheiro estiverem certas, o jacobiano
//! está certo por construção.
//!
//! ⚠️ **A régua deste módulo é a CONTENÇÃO**: para toda operação e toda amostra dentro do
//! intervalo de entrada, o resultado pontual tem de cair dentro do intervalo de saída. É isso que o
//! gate `an_interval_contains_every_point_it_claims_to` mede — e é a única propriedade que torna
//! um majorante um majorante.

/// Um intervalo fechado `[lo, hi]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Iv {
    pub lo: f64,
    pub hi: f64,
}

impl Iv {
    pub(crate) const ZERO: Iv = Iv { lo: 0.0, hi: 0.0 };

    pub(crate) fn pt(x: f64) -> Self {
        Iv { lo: x, hi: x }
    }

    pub(crate) fn new(a: f64, b: f64) -> Self {
        Iv {
            lo: a.min(b),
            hi: a.max(b),
        }
    }

    /// O maior módulo do intervalo — o que uma norma precisa.
    pub(crate) fn mag(self) -> f64 {
        self.lo.abs().max(self.hi.abs())
    }

    pub(crate) fn add(self, o: Self) -> Self {
        Iv {
            lo: self.lo + o.lo,
            hi: self.hi + o.hi,
        }
    }

    pub(crate) fn sub(self, o: Self) -> Self {
        Iv {
            lo: self.lo - o.hi,
            hi: self.hi - o.lo,
        }
    }

    pub(crate) fn neg(self) -> Self {
        Iv {
            lo: -self.hi,
            hi: -self.lo,
        }
    }

    /// ⚠️ Os QUATRO cantos: um produto de intervalos com sinais mistos não é o produto das pontas.
    pub(crate) fn mul(self, o: Self) -> Self {
        let p = [
            self.lo * o.lo,
            self.lo * o.hi,
            self.hi * o.lo,
            self.hi * o.hi,
        ];
        Iv {
            lo: p.iter().copied().fold(f64::INFINITY, f64::min),
            hi: p.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        }
    }

    /// ⛔ **Um divisor que contém zero devolve a recta inteira** — e quem a receber tem de desistir.
    /// *Devolver um número finito ali seria inventar um majorante.*
    pub(crate) fn div(self, o: Self) -> Self {
        if o.lo <= 0.0 && o.hi >= 0.0 {
            return Iv {
                lo: f64::NEG_INFINITY,
                hi: f64::INFINITY,
            };
        }
        self.mul(Iv {
            lo: 1.0 / o.hi,
            hi: 1.0 / o.lo,
        })
    }

    pub(crate) fn square(self) -> Self {
        if self.lo >= 0.0 {
            Iv::new(self.lo * self.lo, self.hi * self.hi)
        } else if self.hi <= 0.0 {
            Iv::new(self.hi * self.hi, self.lo * self.lo)
        } else {
            // ⚠️ **Contém zero ⇒ o mínimo é ZERO**, e não a menor das pontas ao quadrado.
            Iv::new(0.0, self.lo.abs().max(self.hi.abs()).powi(2))
        }
    }

    pub(crate) fn sqrt(self) -> Self {
        Iv::new(self.lo.max(0.0).sqrt(), self.hi.max(0.0).sqrt())
    }

    pub(crate) fn abs(self) -> Self {
        if self.lo >= 0.0 {
            self
        } else if self.hi <= 0.0 {
            self.neg()
        } else {
            Iv::new(0.0, self.lo.abs().max(self.hi.abs()))
        }
    }

    pub(crate) fn max_pt(self, c: f64) -> Self {
        Iv::new(self.lo.max(c), self.hi.max(c))
    }

    pub(crate) fn min_pt(self, c: f64) -> Self {
        Iv::new(self.lo.min(c), self.hi.min(c))
    }

    /// A menor caixa que contém as duas — usada onde uma derivada salta.
    pub(crate) fn hull(self, o: Self) -> Self {
        Iv {
            lo: self.lo.min(o.lo),
            hi: self.hi.max(o.hi),
        }
    }

    pub(crate) fn is_finite(self) -> bool {
        self.lo.is_finite() && self.hi.is_finite()
    }

    /// ⚠️ **O extremo INTERIOR declara-se** — ver a irmã em [`crate::bounds_bend`]: um intervalo que
    /// passa por cima de `θ = 0` tem `cos = 1` lá dentro, e ler só as pontas devolveria menos.
    pub(crate) fn cos(self) -> Self {
        use std::f64::consts::{PI, TAU};
        let (mut lo, mut hi) = (
            self.lo.cos().min(self.hi.cos()),
            self.lo.cos().max(self.hi.cos()),
        );
        if tem_multiplo(self.lo, self.hi, 0.0, TAU) {
            hi = 1.0;
        }
        if tem_multiplo(self.lo, self.hi, PI, TAU) {
            lo = -1.0;
        }
        Iv::new(lo, hi)
    }

    pub(crate) fn sin(self) -> Self {
        use std::f64::consts::{PI, TAU};
        let (mut lo, mut hi) = (
            self.lo.sin().min(self.hi.sin()),
            self.lo.sin().max(self.hi.sin()),
        );
        if tem_multiplo(self.lo, self.hi, PI * 0.5, TAU) {
            hi = 1.0;
        }
        if tem_multiplo(self.lo, self.hi, -PI * 0.5, TAU) {
            lo = -1.0;
        }
        Iv::new(lo, hi)
    }

    /// ⭐ **Monótona** — e é por isso que a dobra usa `atan(b/a)` e não `atan2`: o piso dela garante
    /// `a > 0`, logo os dois são a mesma função e este é trivial sobre intervalos.
    pub(crate) fn atan(self) -> Self {
        Iv::new(self.lo.atan(), self.hi.atan())
    }
}

fn tem_multiplo(a: f64, b: f64, fase: f64, periodo: f64) -> bool {
    if !(a.is_finite() && b.is_finite()) {
        return true;
    }
    let n = ((a - fase) / periodo).ceil();
    fase + n * periodo <= b
}

/// ⭐⭐⭐ **Um número com as três derivadas parciais ao lado**, tudo em intervalos.
///
/// ⚠️ **É diferenciação para a FRENTE**: cada operação carrega a regra da cadeia dela, e por isso a
/// lei escreve-se uma vez só. *A alternativa — escrever as ~40 entradas do jacobiano à mão — tem um
/// modo de falha que não falha: um sinal trocado devolve um majorante pequeno demais.*
#[derive(Clone, Copy, Debug)]
pub(crate) struct D {
    pub v: Iv,
    pub d: [Iv; 3],
}

impl D {
    pub(crate) fn cte(x: f64) -> Self {
        D {
            v: Iv::pt(x),
            d: [Iv::ZERO; 3],
        }
    }

    /// A variável `eixo`, com valor no intervalo dado.
    pub(crate) fn var(v: Iv, eixo: usize) -> Self {
        let mut d = [Iv::ZERO; 3];
        d[eixo] = Iv::pt(1.0);
        D { v, d }
    }

    pub(crate) fn add(self, o: Self) -> Self {
        D {
            v: self.v.add(o.v),
            d: std::array::from_fn(|i| self.d[i].add(o.d[i])),
        }
    }

    pub(crate) fn sub(self, o: Self) -> Self {
        D {
            v: self.v.sub(o.v),
            d: std::array::from_fn(|i| self.d[i].sub(o.d[i])),
        }
    }

    pub(crate) fn neg(self) -> Self {
        D {
            v: self.v.neg(),
            d: std::array::from_fn(|i| self.d[i].neg()),
        }
    }

    pub(crate) fn mul(self, o: Self) -> Self {
        D {
            v: self.v.mul(o.v),
            d: std::array::from_fn(|i| self.d[i].mul(o.v).add(self.v.mul(o.d[i]))),
        }
    }

    pub(crate) fn div(self, o: Self) -> Self {
        let q = self.v.div(o.v);
        D {
            v: q,
            // `(u/w)' = (u' − q·w')/w`
            d: std::array::from_fn(|i| self.d[i].sub(q.mul(o.d[i])).div(o.v)),
        }
    }

    pub(crate) fn escala(self, c: f64) -> Self {
        self.mul(D::cte(c))
    }

    pub(crate) fn square(self) -> Self {
        D {
            v: self.v.square(),
            d: std::array::from_fn(|i| self.d[i].mul(self.v).mul(Iv::pt(2.0))),
        }
    }

    pub(crate) fn sqrt(self) -> Self {
        let r = self.v.sqrt();
        D {
            v: r,
            d: std::array::from_fn(|i| self.d[i].div(r.mul(Iv::pt(2.0)))),
        }
    }

    pub(crate) fn sin(self) -> Self {
        let c = self.v.cos();
        D {
            v: self.v.sin(),
            d: std::array::from_fn(|i| self.d[i].mul(c)),
        }
    }

    pub(crate) fn cos(self) -> Self {
        let s = self.v.sin().neg();
        D {
            v: self.v.cos(),
            d: std::array::from_fn(|i| self.d[i].mul(s)),
        }
    }

    pub(crate) fn atan(self) -> Self {
        let den = Iv::pt(1.0).add(self.v.square());
        D {
            v: self.v.atan(),
            d: std::array::from_fn(|i| self.d[i].div(den)),
        }
    }

    /// ⛔⛔ **Onde o `max` MUDA DE RAMO, a derivada é a UNIÃO dos dois** — e não a de um deles.
    ///
    /// Uma caixa que contenha o cruzamento tem pontos dos dois lados; escolher um ramo daria um
    /// majorante que não cobre o outro, e um majorante que não cobre **fura**.
    pub(crate) fn max_pt(self, c: f64) -> Self {
        if self.v.lo >= c {
            return self;
        }
        if self.v.hi <= c {
            return D::cte(c);
        }
        D {
            v: self.v.max_pt(c),
            d: std::array::from_fn(|i| self.d[i].hull(Iv::ZERO)),
        }
    }

    /// O espelho do [`D::max_pt`] — e escrito **simétrico** de propósito: pela dupla negação ele
    /// ficava correcto e ilegível, e o `Iv::min_pt` que a cerca de contenção mede ficava sem leitor.
    pub(crate) fn min_pt(self, c: f64) -> Self {
        if self.v.hi <= c {
            return self;
        }
        if self.v.lo >= c {
            return D::cte(c);
        }
        D {
            v: self.v.min_pt(c),
            d: std::array::from_fn(|i| self.d[i].hull(Iv::ZERO)),
        }
    }

    pub(crate) fn abs(self) -> Self {
        if self.v.lo >= 0.0 {
            return self;
        }
        if self.v.hi <= 0.0 {
            return self.neg();
        }
        D {
            v: self.v.abs(),
            d: std::array::from_fn(|i| self.d[i].hull(self.d[i].neg())),
        }
    }
}

#[cfg(test)]
#[path = "bounds_iv_tests.rs"]
mod tests;

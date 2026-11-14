//! ⭐ **A curva Custom com PONTOS** ([`EaseSpline`], dono 13/11: «permita criar e deletar pontos
//! na curva custom» — o ponto mora SÓ dentro da curva, as chaves da timeline não mudam).
//!
//! Uma função de tempo `u ↦ v` no quadrado unitário, de `(0, 0)` a `(1, 1)`, feita de cúbicas de
//! Bézier encadeadas: cada nó tem a posição e as duas alças (entrada, saída). Sem pontos é
//! exatamente a [`crate::Interp::Bezier`] (a alça de saída do início e a de chegada do fim).
//!
//! Invariantes que toda operação repõe ([`EaseSpline::arruma`]): o `x` dos nós cresce
//! estritamente, e o `x` de cada alça fica dentro do trecho dela — é o que mantém cada cúbica
//! monótona em `x` (a regra CSS do `cubic-bezier`), logo `u ↦ v` é uma FUNÇÃO.
//!
//! ⚠️ **O tecto de [`SPLINE_MAX_POINTS`] é de memória:** a `Interp` é `Copy` e mora em TODA chave
//! (com ou sem pontos); cada ponto custa 24 B em cada uma. Medido: `size_of::<Interp>()` 40 B antes,
//! 216 B com 8 pontos, 408 B com 16 — e 16 reprova o `clippy::large_enum_variant` (a variante grande
//! desperdiça o resto em toda chave). Mais detalhe que 8 pontos num só trecho é trabalho para uma
//! chave a mais.

use serde::{Deserialize, Serialize};

/// Quantos pontos (além do início e do fim) uma curva Custom guarda.
pub const SPLINE_MAX_POINTS: usize = 8;

/// A menor distância em `x` entre dois nós (evita trechos de largura zero).
const X_EPS: f32 = 1e-3;

/// Um nó da curva: a posição e as duas alças, no quadrado unitário.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SplinePoint {
    /// Onde a curva passa.
    pub p: [f32; 2],
    /// A alça que CHEGA ao nó (controla o trecho da esquerda).
    pub h_in: [f32; 2],
    /// A alça que SAI do nó (controla o trecho da direita).
    pub h_out: [f32; 2],
}

/// Qual das alças de um nó.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplineSide {
    /// A que chega ([`SplinePoint::h_in`]).
    In,
    /// A que sai ([`SplinePoint::h_out`]).
    Out,
}

/// A curva Custom com pontos. Os nós contam-se de `0` (o início, `(0,0)`) a
/// [`EaseSpline::node_count`]` − 1` (o fim, `(1,1)`); os do meio são os pontos.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EaseSpline {
    n: u8,
    out0: [f32; 2],
    in1: [f32; 2],
    pts: [SplinePoint; SPLINE_MAX_POINTS],
}

impl EaseSpline {
    /// A Bézier de duas alças `P1 = (x1, y1)`, `P2 = (x2, y2)`, sem pontos.
    #[must_use]
    pub fn from_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let mut s = Self {
            n: 0,
            out0: [x1, y1],
            in1: [x2, y2],
            pts: [SplinePoint::default(); SPLINE_MAX_POINTS],
        };
        s.arruma();
        s
    }

    /// Os pontos do meio (sem o início e o fim).
    #[must_use]
    pub fn points(&self) -> &[SplinePoint] {
        &self.pts[..usize::from(self.n)]
    }

    /// Quantos nós ao todo (pontos + início + fim).
    #[must_use]
    pub fn node_count(&self) -> usize {
        usize::from(self.n) + 2
    }

    /// `true` se cabe mais um ponto.
    #[must_use]
    pub fn has_room(&self) -> bool {
        usize::from(self.n) < SPLINE_MAX_POINTS
    }

    /// O nó `i` (`0` = início, `node_count() − 1` = fim).
    #[must_use]
    pub fn node(&self, i: usize) -> SplinePoint {
        let fim = self.node_count() - 1;
        match i {
            0 => SplinePoint {
                p: [0.0, 0.0],
                h_in: [0.0, 0.0],
                h_out: self.out0,
            },
            _ if i >= fim => SplinePoint {
                p: [1.0, 1.0],
                h_in: self.in1,
                h_out: [1.0, 1.0],
            },
            _ => self.pts[i - 1],
        }
    }

    fn set_node(&mut self, i: usize, no: SplinePoint) {
        let fim = self.node_count() - 1;
        match i {
            0 => self.out0 = no.h_out,
            _ if i >= fim => self.in1 = no.h_in,
            _ => self.pts[i - 1] = no,
        }
    }

    /// As alças das pontas: `((saída do início), (chegada ao fim))`.
    #[must_use]
    pub fn end_handles(&self) -> ([f32; 2], [f32; 2]) {
        (self.out0, self.in1)
    }

    /// A mesma curva com as alças das pontas trocadas (os pontos ficam).
    #[must_use]
    pub fn with_end_handles(mut self, out0: [f32; 2], in1: [f32; 2]) -> Self {
        self.out0 = out0;
        self.in1 = in1;
        self.arruma();
        self
    }

    /// Repõe os invariantes: `x` dos nós a crescer, cada alça dentro do seu trecho.
    fn arruma(&mut self) {
        let n = self.node_count();
        for i in 1..n - 1 {
            let lo = self.node(i - 1).p[0] + X_EPS;
            let hi = 1.0 - X_EPS * (n - 1 - i) as f32;
            let mut no = self.node(i);
            no.p[0] = no.p[0].clamp(lo, hi.max(lo));
            self.set_node(i, no);
        }
        for i in 0..n - 1 {
            let (mut a, mut b) = (self.node(i), self.node(i + 1));
            let (x0, x1) = (a.p[0], b.p[0]);
            a.h_out[0] = a.h_out[0].clamp(x0, x1);
            b.h_in[0] = b.h_in[0].clamp(x0, x1);
            self.set_node(i, a);
            self.set_node(i + 1, b);
        }
    }

    /// O trecho que contém `u` (o índice do nó da esquerda).
    fn trecho(&self, u: f64) -> usize {
        let n = self.node_count();
        (0..n - 1)
            .find(|&i| u <= f64::from(self.node(i + 1).p[0]))
            .unwrap_or(n - 2)
    }

    /// Os quatro pontos de controle do trecho `i`, em `f64`: `([x0..x3], [y0..y3])`.
    fn controle(&self, i: usize) -> ([f64; 4], [f64; 4]) {
        let (a, b) = (self.node(i), self.node(i + 1));
        let c = [a.p, a.h_out, b.h_in, b.p];
        (c.map(|q| f64::from(q[0])), c.map(|q| f64::from(q[1])))
    }

    /// O valor da curva em `u` (preso a `[0, 1]`).
    #[must_use]
    pub fn eval(&self, u: f64) -> f64 {
        let u = u.clamp(0.0, 1.0);
        let (xs, ys) = self.controle(self.trecho(u));
        cubica(ys, resolve_t(xs, u))
    }

    /// A inclinação `dv/du` em `u` (pode ser `±∞` numa tangente vertical).
    #[must_use]
    pub fn slope(&self, u: f64) -> f64 {
        let u = u.clamp(0.0, 1.0);
        let (xs, ys) = self.controle(self.trecho(u));
        let t = resolve_t(xs, u);
        derivada(ys, t) / derivada(xs, t)
    }

    /// A mesma curva tocada ao contrário: espelhada por `(½, ½)`, nós na ordem inversa, alças trocadas.
    #[must_use]
    pub fn reversed(&self) -> Self {
        let m = |q: [f32; 2]| [1.0 - q[0], 1.0 - q[1]];
        let n = self.node_count();
        let mut s = *self;
        for j in 0..n {
            let o = self.node(n - 1 - j);
            s.set_node(
                j,
                SplinePoint {
                    p: m(o.p),
                    h_in: m(o.h_out),
                    h_out: m(o.h_in),
                },
            );
        }
        s.arruma();
        s
    }

    /// Cria um ponto SOBRE a curva em `u`, sem mudar a forma (corte de de Casteljau). Devolve o
    /// índice do nó novo; `None` se já não cabe ou se `u` cai em cima de um nó.
    pub fn insert(&mut self, u: f64) -> Option<usize> {
        if !self.has_room() {
            return None;
        }
        let i = self.trecho(u);
        let (xs, ys) = self.controle(i);
        if u - xs[0] < f64::from(X_EPS) || xs[3] - u < f64::from(X_EPS) {
            return None;
        }
        let t = resolve_t(xs, u);
        let lerp = |a: [f64; 2], b: [f64; 2]| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        let p: [[f64; 2]; 4] = core::array::from_fn(|k| [xs[k], ys[k]]);
        let (p01, p12, p23) = (lerp(p[0], p[1]), lerp(p[1], p[2]), lerp(p[2], p[3]));
        let (p012, p123) = (lerp(p01, p12), lerp(p12, p23));
        let meio = lerp(p012, p123);
        let f = |q: [f64; 2]| [q[0] as f32, q[1] as f32];
        let (mut a, mut b) = (self.node(i), self.node(i + 1));
        a.h_out = f(p01);
        b.h_in = f(p23);
        self.set_node(i, a);
        self.set_node(i + 1, b);
        // O nó novo entra na posição `i` dos pontos (nó `i + 1`); os de depois andam um.
        let k = i;
        self.pts.copy_within(k..usize::from(self.n), k + 1);
        self.pts[k] = SplinePoint {
            p: f(meio),
            h_in: f(p012),
            h_out: f(p123),
        };
        self.n += 1;
        self.arruma();
        Some(i + 1)
    }

    /// Apaga o ponto do nó `i` (o início e o fim não se apagam). `false` se `i` não é um ponto.
    pub fn remove(&mut self, i: usize) -> bool {
        if i == 0 || i >= self.node_count() - 1 {
            return false;
        }
        let k = i - 1;
        self.pts.copy_within(k + 1..usize::from(self.n), k);
        self.n -= 1;
        self.pts[usize::from(self.n)] = SplinePoint::default();
        self.arruma();
        true
    }

    /// Leva o ponto do nó `i` a `(x, y)`, com as alças dele; o `x` fica entre os vizinhos.
    pub fn move_point(&mut self, i: usize, x: f32, y: f32) {
        if i == 0 || i >= self.node_count() - 1 {
            return;
        }
        let mut no = self.node(i);
        let lo = self.node(i - 1).p[0] + X_EPS;
        let hi = self.node(i + 1).p[0] - X_EPS;
        let x = x.clamp(lo, hi.max(lo));
        let (dx, dy) = (x - no.p[0], y - no.p[1]);
        for q in [&mut no.p, &mut no.h_in, &mut no.h_out] {
            q[0] += dx;
            q[1] += dy;
        }
        self.set_node(i, no);
        self.arruma();
    }

    /// Leva a alça `lado` do nó `i` a `(x, y)`. Num ponto do meio a outra alça ALINHA (o nó fica
    /// liso) e guarda o comprimento.
    pub fn move_handle(&mut self, i: usize, lado: SplineSide, x: f32, y: f32) {
        let fim = self.node_count() - 1;
        let mut no = self.node(i);
        let p = no.p;
        // O trecho de cada alça: a de chegada vive entre o nó anterior e este; a de saída entre
        // este e o seguinte.
        let (antes, depois) = (
            if i == 0 { p[0] } else { self.node(i - 1).p[0] },
            if i == fim {
                p[0]
            } else {
                self.node(i + 1).p[0]
            },
        );
        let (esta, outra, (lo, hi), (olo, ohi)) = match lado {
            SplineSide::In => (&mut no.h_in, &mut no.h_out, (antes, p[0]), (p[0], depois)),
            SplineSide::Out => (&mut no.h_out, &mut no.h_in, (p[0], depois), (antes, p[0])),
        };
        *esta = [x.clamp(lo, hi), y];
        if i != 0 && i != fim {
            // A outra ALINHA com esta (o nó fica liso), com o comprimento dela — encurtada ao
            // longo da mesma reta se sairia do trecho dela.
            let (dx, dy) = (esta[0] - p[0], esta[1] - p[1]);
            let d = (dx * dx + dy * dy).sqrt();
            let mut comp = ((outra[0] - p[0]).powi(2) + (outra[1] - p[1]).powi(2)).sqrt();
            if d > f32::EPSILON {
                let (ux, uy) = (-dx / d, -dy / d);
                let limite = if ux > 0.0 {
                    (ohi - p[0]) / ux
                } else if ux < 0.0 {
                    (olo - p[0]) / ux
                } else {
                    comp
                };
                comp = comp.min(limite.max(0.0));
                *outra = [p[0] + ux * comp, p[1] + uy * comp];
            }
        }
        self.set_node(i, no);
        self.arruma();
    }
}

/// A cúbica de Bézier de controles `c` em `t`.
fn cubica(c: [f64; 4], t: f64) -> f64 {
    let s = 1.0 - t;
    s * s * s * c[0] + 3.0 * s * s * t * c[1] + 3.0 * s * t * t * c[2] + t * t * t * c[3]
}

/// A derivada da cúbica de controles `c` em `t`.
fn derivada(c: [f64; 4], t: f64) -> f64 {
    let s = 1.0 - t;
    3.0 * (s * s * (c[1] - c[0]) + 2.0 * s * t * (c[2] - c[1]) + t * t * (c[3] - c[2]))
}

/// O `t` com `x(t) = u` (monótona em `t`, pelas invariantes): Newton com rede de bissecção.
fn resolve_t(xs: [f64; 4], u: f64) -> f64 {
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    let mut t = if xs[3] > xs[0] {
        ((u - xs[0]) / (xs[3] - xs[0])).clamp(0.0, 1.0)
    } else {
        0.5
    };
    for _ in 0..64 {
        let e = cubica(xs, t) - u;
        if e.abs() < 1e-12 {
            break;
        }
        if e > 0.0 {
            hi = t;
        } else {
            lo = t;
        }
        let d = derivada(xs, t);
        let novo = if d.abs() > 1e-12 { t - e / d } else { -1.0 };
        t = if novo > lo && novo < hi {
            novo
        } else {
            (lo + hi) * 0.5
        };
    }
    t
}

#[cfg(test)]
#[path = "curve_spline_tests.rs"]
mod tests;

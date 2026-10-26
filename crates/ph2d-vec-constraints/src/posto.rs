//! REF/redundante = a regra que não sobe o POSTO do jacobiano, em ordem de nascimento (§1.4, §3.2).
//! O jacobiano da ezpz é privado: os resíduos escrevem-se aqui uma vez, sobre um dual (derivada exacta).
//! Posto por Gram–Schmidt modificado duas vezes, linha a linha: a QR de Householder SEM pivô inventa uma
//! direcção quando uma coluna é dependente e estraga o teste das seguintes.

use std::ops::{Add, Div, Mul, Neg, Sub};

use ph2d_vec_scene::VecPathId;
use ph2d_vec_scene::cad::{Lei, PontoCad, SegCad};

/// Quatro pontos (8 variáveis) é o máximo de uma regra.
const N: usize = 8;
/// Uma linha que, normalizada, sobra abaixo disto depois de projectada é combinação das anteriores.
const TOL_POSTO: f64 = 1e-8;

#[derive(Copy, Clone)]
pub(crate) struct D {
    pub(crate) v: f64,
    d: [f64; N],
}

impl D {
    const fn k(v: f64) -> Self {
        Self { v, d: [0.0; N] }
    }
    fn var(v: f64, i: usize) -> Self {
        let mut d = [0.0; N];
        d[i] = 1.0;
        Self { v, d }
    }
    fn map(self, v: f64, dv: f64) -> Self {
        Self {
            v,
            d: self.d.map(|x| x * dv),
        }
    }
    fn sqrt(self) -> Self {
        let s = self.v.sqrt();
        self.map(s, 0.5 / s)
    }
    fn atan2(self, x: Self) -> Self {
        let q = x.v * x.v + self.v * self.v;
        let mut d = [0.0; N];
        for (i, di) in d.iter_mut().enumerate() {
            *di = (x.v * self.d[i] - self.v * x.d[i]) / q;
        }
        Self {
            v: self.v.atan2(x.v),
            d,
        }
    }
}

impl Add for D {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        let mut d = self.d;
        d.iter_mut().zip(o.d).for_each(|(a, b)| *a += b);
        Self { v: self.v + o.v, d }
    }
}
impl Sub for D {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        self + -o
    }
}
impl Neg for D {
    type Output = Self;
    fn neg(self) -> Self {
        self.map(-self.v, -1.0)
    }
}
impl Mul for D {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        let mut d = [0.0; N];
        for (i, di) in d.iter_mut().enumerate() {
            *di = self.d[i] * o.v + self.v * o.d[i];
        }
        Self { v: self.v * o.v, d }
    }
}
impl Div for D {
    type Output = Self;
    fn div(self, o: Self) -> Self {
        let mut d = [0.0; N];
        for (i, di) in d.iter_mut().enumerate() {
            *di = (self.d[i] * o.v - self.v * o.d[i]) / (o.v * o.v);
        }
        Self { v: self.v / o.v, d }
    }
}

type P = [D; 2];

fn menos(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}
fn cruz(a: P, b: P) -> D {
    a[0] * b[1] - a[1] * b[0]
}
fn escalar(a: P, b: P) -> D {
    a[0] * b[0] + a[1] * b[1]
}
fn norma(a: P) -> D {
    escalar(a, a).sqrt()
}

/// As variáveis de que uma regra depende, pela ordem em que os resíduos as lêem.
pub(crate) enum Var {
    X(PontoCad),
    Y(PontoCad),
    Raio(VecPathId),
}

/// Resíduos assinados da regra (a mesma superfície-zero que a ezpz resolve) e as variáveis locais deles.
pub(crate) fn residuos(lei: &Lei, valor: &dyn Fn(&Var) -> f64) -> (Vec<Var>, Vec<D>) {
    let mut vars: Vec<Var> = Vec::new();
    let ponto = |p: PontoCad, vars: &mut Vec<Var>| -> P {
        let i = vars.len();
        vars.push(Var::X(p));
        vars.push(Var::Y(p));
        [
            D::var(valor(&vars[i]), i),
            D::var(valor(&vars[i + 1]), i + 1),
        ]
    };
    let seg = |s: SegCad, vars: &mut Vec<Var>| -> P {
        let a = ponto(s.0, vars);
        let b = ponto(s.1, vars);
        menos(b, a)
    };
    let r = match *lei {
        Lei::Coincidente(a, b) => {
            let d = menos(ponto(a, &mut vars), ponto(b, &mut vars));
            vec![d[0], d[1]]
        }
        Lei::Horizontal(s) => vec![seg(s, &mut vars)[1]],
        Lei::Vertical(s) => vec![seg(s, &mut vars)[0]],
        Lei::Paralelo(s, t) => vec![cruz(seg(s, &mut vars), seg(t, &mut vars))],
        Lei::Perpendicular(s, t) => vec![escalar(seg(s, &mut vars), seg(t, &mut vars))],
        Lei::Igual(s, t) => vec![norma(seg(s, &mut vars)) - norma(seg(t, &mut vars))],
        Lei::Angulo(s, t, _) => {
            let (u, v) = (seg(s, &mut vars), seg(t, &mut vars));
            vec![cruz(u, v).atan2(escalar(u, v))]
        }
        Lei::Distancia(a, b, _) => vec![norma(menos(ponto(a, &mut vars), ponto(b, &mut vars)))],
        Lei::SobreRecta(p, s) => {
            let p = ponto(p, &mut vars);
            let (a, b) = (ponto(s.0, &mut vars), ponto(s.1, &mut vars));
            vec![cruz(menos(b, a), menos(p, a))]
        }
        Lei::Simetrico(p, q, s) => {
            let (p, q) = (ponto(p, &mut vars), ponto(q, &mut vars));
            let (a, b) = (ponto(s.0, &mut vars), ponto(s.1, &mut vars));
            let meio = [(p[0] + q[0]) * D::k(0.5), (p[1] + q[1]) * D::k(0.5)];
            vec![
                cruz(menos(b, a), menos(meio, a)),
                escalar(menos(q, p), menos(b, a)),
            ]
        }
        Lei::Tangente(s, c) => {
            let (a, b) = (ponto(s.0, &mut vars), ponto(s.1, &mut vars));
            let centro = ponto(PontoCad::Centro(c), &mut vars);
            let i = vars.len();
            vars.push(Var::Raio(c));
            let raio = D::var(valor(&vars[i]), i);
            let v = menos(b, a);
            vec![cruz(v, menos(centro, a)) / norma(v) - raio]
        }
        Lei::Raio(c, _) => {
            vars.push(Var::Raio(c));
            vec![D::var(valor(&vars[0]), 0)]
        }
        Lei::Fixo(p, _) => {
            let p = ponto(p, &mut vars);
            vec![p[0], p[1]]
        }
    };
    (vars, r)
}

/// A base ortonormal das linhas já aceites; `aceita` diz se as linhas novas sobem o posto em TODAS as direcções.
#[derive(Default)]
pub(crate) struct Posto {
    base: Vec<Vec<f64>>,
}

impl Posto {
    fn projecta(&self, l: &mut [f64]) {
        for _ in 0..2 {
            for q in &self.base {
                let p: f64 = l.iter().zip(q).map(|(a, b)| a * b).sum();
                l.iter_mut().zip(q).for_each(|(a, b)| *a -= p * b);
            }
        }
    }

    /// Junta as linhas se cada uma for independente das aceites e das irmãs; senão não junta nenhuma.
    pub(crate) fn aceita(&mut self, linhas: Vec<Vec<f64>>) -> bool {
        let n0 = self.base.len();
        for mut l in linhas {
            let n = l.iter().map(|x| x * x).sum::<f64>().sqrt();
            if n == 0.0 {
                self.base.truncate(n0);
                return false;
            }
            l.iter_mut().for_each(|x| *x /= n);
            self.projecta(&mut l);
            let m = l.iter().map(|x| x * x).sum::<f64>().sqrt();
            if m < TOL_POSTO {
                self.base.truncate(n0);
                return false;
            }
            l.iter_mut().for_each(|x| *x /= m);
            self.base.push(l);
        }
        true
    }
}

/// As linhas densas (sobre `n` variáveis globais) dos resíduos de uma regra.
pub(crate) fn linhas(
    vars: &[Var],
    res: &[D],
    n: usize,
    global: &dyn Fn(&Var) -> usize,
) -> Vec<Vec<f64>> {
    res.iter()
        .map(|r| {
            let mut l = vec![0.0; n];
            for (i, v) in vars.iter().enumerate() {
                l[global(v)] += r.d[i];
            }
            l
        })
        .collect()
}

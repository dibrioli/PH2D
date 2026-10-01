//! ⭐ **A geometria do TOQUE da bola** — onde dois offsets se cruzam e onde fica o pé de um centro
//! numa cúbica. Saiu de [`super`] por responsabilidade (F46): a bola decide ONDE rolar, este módulo
//! responde onde ela pousa.

use kurbo::{CubicBez, ParamCurve, ParamCurveDeriv, Point, Vec2};

use super::unit;

pub(super) fn cruza_segmentos(a: Point, b: Point, c: Point, d: Point) -> Option<(f64, f64)> {
    let r = b - a;
    let s = d - c;
    let den = r.cross(s);
    if den.abs() < 1e-300 {
        return None;
    }
    let u = (c - a).cross(s) / den;
    let w = (c - a).cross(r) / den;
    ((0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&w)).then_some((u, w))
}

/// ⭐ F46 — **o offset de um nó CONVEXO**: o arco de raio `r` à volta do nó `c`, da normal de entrada
/// `n0` à de saída `n1` (as duas unitárias, já do lado da bola).
#[derive(Clone, Copy)]
pub(super) struct ArcoDoNo {
    pub(super) c: Point,
    pub(super) n0: Vec2,
    pub(super) n1: Vec2,
}

impl ArcoDoNo {
    /// A direcção `v` (unitária, a partir do nó) cai dentro do arco?
    fn contem(&self, v: Vec2) -> bool {
        let lado = self.n0.cross(self.n1).signum();
        self.n0.cross(v) * lado >= -1e-12 && v.cross(self.n1) * lado >= -1e-12
    }

    fn no_arco(&self, p: Point, r: f64) -> bool {
        unit(p - self.c).is_some_and(|v| self.contem(v))
            && ((p - self.c).hypot() - r).abs() < 1e-9 * r.max(1.0)
    }

    /// Onde o arco cruza o segmento `p`→`q`, com o parâmetro no segmento.
    pub(super) fn com_segmento(&self, p: Point, q: Point, r: f64) -> Vec<(Point, f64)> {
        let d = q - p;
        let f = p - self.c;
        let (a, b, cc) = (d.dot(d), 2.0 * f.dot(d), f.dot(f) - r * r);
        let disc = b * b - 4.0 * a * cc;
        if a < 1e-300 || disc < 0.0 {
            return Vec::new();
        }
        let raiz = disc.sqrt();
        [(-b - raiz) / (2.0 * a), (-b + raiz) / (2.0 * a)]
            .into_iter()
            .filter(|w| (0.0..=1.0).contains(w))
            .map(|w| (p + d * w, w))
            .filter(|(x, _)| self.no_arco(*x, r))
            .collect()
    }

    /// Onde dois arcos do mesmo raio se cruzam.
    pub(super) fn com_arco(&self, outro: &ArcoDoNo, r: f64) -> Vec<Point> {
        let d = outro.c - self.c;
        let l = d.hypot();
        if l < 1e-12 || l > 2.0 * r {
            return Vec::new();
        }
        let meio = self.c + d * 0.5;
        let h = (r * r - 0.25 * l * l).max(0.0).sqrt();
        let perp = Vec2::new(-d.y, d.x) / l;
        [meio + perp * h, meio - perp * h]
            .into_iter()
            .filter(|x| self.no_arco(*x, r) && outro.no_arco(*x, r))
            .collect()
    }
}

/// O parâmetro do segmento `c` em `[lo, hi]` mais perto de `alvo` (secção áurea).
pub(super) fn pe(c: &CubicBez, alvo: Point, mut lo: f64, mut hi: f64) -> f64 {
    let g = 0.5 * (5.0_f64.sqrt() - 1.0);
    let f = |t: f64| (c.eval(t) - alvo).hypot2();
    let (mut x1, mut x2) = (hi - g * (hi - lo), lo + g * (hi - lo));
    let (mut f1, mut f2) = (f(x1), f(x2));
    for _ in 0..60 {
        if f1 < f2 {
            hi = x2;
            x2 = x1;
            f2 = f1;
            x1 = hi - g * (hi - lo);
            f1 = f(x1);
        } else {
            lo = x1;
            x1 = x2;
            f1 = f2;
            x2 = lo + g * (hi - lo);
            f2 = f(x2);
        }
    }
    // ⚠️ A secção áurea sobre o QUADRADO da distância pára a `~√ε` (o mínimo é chato) — dois
    // toques da mesma bola desviavam `1e-9` conforme o segmento amostrado. Newton sobre a derivada
    // (`(c(t) − alvo) · c′(t) = 0`) leva-o à precisão da máquina.
    let (a, b) = (lo, hi);
    let mut t = 0.5 * (lo + hi);
    let (d1, d2) = (c.deriv(), c.deriv().deriv());
    for _ in 0..8 {
        let (v, dv, ddv) = (c.eval(t) - alvo, d1.eval(t).to_vec2(), d2.eval(t).to_vec2());
        let den = dv.dot(dv) + v.dot(ddv);
        if den.abs() < 1e-300 {
            break;
        }
        let novo = (t - v.dot(dv) / den).clamp(a.min(t), b.max(t));
        if (novo - t).abs() < 1e-16 {
            break;
        }
        t = novo;
    }
    t
}

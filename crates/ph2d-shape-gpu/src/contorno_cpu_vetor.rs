//! Os vetores da porta CPU da lei do contorno (`contorno_cpu.rs`): o `vec2<f32>` do WGSL e as funções dele
//! que a lei usa, com os MESMOS arredondamentos.

use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct V {
    pub(super) x: f32,
    pub(super) y: f32,
}

pub(super) const fn v(x: f32, y: f32) -> V {
    V { x, y }
}

impl Add for V {
    type Output = V;
    fn add(self, o: V) -> V {
        v(self.x + o.x, self.y + o.y)
    }
}

impl Sub for V {
    type Output = V;
    fn sub(self, o: V) -> V {
        v(self.x - o.x, self.y - o.y)
    }
}

impl Neg for V {
    type Output = V;
    fn neg(self) -> V {
        v(-self.x, -self.y)
    }
}

impl Mul<f32> for V {
    type Output = V;
    fn mul(self, k: f32) -> V {
        v(self.x * k, self.y * k)
    }
}

pub(super) fn dot(a: V, b: V) -> f32 {
    a.x * b.x + a.y * b.y
}

pub(super) fn length(a: V) -> f32 {
    dot(a, a).sqrt()
}

pub(super) fn perp(u: V) -> V {
    v(-u.y, u.x)
}

/// O `sign` do WGSL (`0` em zero; o `f32::signum` dá `1`).
pub(super) fn sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

pub(super) fn pt(p: [f32; 2]) -> V {
    v(p[0], p[1])
}

/// O sinal que o `orienta(v, a, b, c)` daria.
pub(super) fn positivo(a: V, b: V, c: V) -> bool {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) >= 0.0
}

/// O `mix` do WGSL.
pub(super) fn mix(a: V, b: V, t: f32) -> V {
    a * (1.0 - t) + b * t
}

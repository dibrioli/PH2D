//! A aritmética do plano das VELOCIDADES (e das posições): `f64`, só `+ − × ÷ sqrt`.

/// Um vector do plano.
pub type V2 = [f64; 2];

/// ⚠️ **A tolerância de PARALELISMO** — adimensional: o `det` de duas direcções UNITÁRIAS é o seno
/// do ângulo entre elas, e abaixo disto duas rectas são a mesma direcção. O recurso é o `f64` das
/// direcções (passo `~1e-16`), com folga de sete ordens para a aritmética que as normaliza; e fica
/// seis ordens abaixo de qualquer ângulo que um artista desenhe. Também é a folga do «já coberto»
/// das paredes, aí em unidades de velocidade (m/s): seis ordens abaixo de um passo por tique.
pub const EPS: f64 = 1e-9;

#[inline]
pub fn add(a: V2, b: V2) -> V2 {
    [a[0] + b[0], a[1] + b[1]]
}

#[inline]
pub fn sub(a: V2, b: V2) -> V2 {
    [a[0] - b[0], a[1] - b[1]]
}

#[inline]
pub fn scale(a: V2, s: f64) -> V2 {
    [a[0] * s, a[1] * s]
}

#[inline]
pub fn neg(a: V2) -> V2 {
    [-a[0], -a[1]]
}

#[inline]
pub fn dot(a: V2, b: V2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// O determinante `a × b` — positivo quando `b` está à ESQUERDA de `a`.
#[inline]
pub fn det(a: V2, b: V2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

#[inline]
pub fn abs_sq(a: V2) -> f64 {
    dot(a, a)
}

#[inline]
pub fn len(a: V2) -> f64 {
    abs_sq(a).sqrt()
}

/// A direcção de `a`; o vector nulo devolve-se nulo (nenhuma divisão por zero chega a um `NaN`).
#[inline]
pub fn normalize(a: V2) -> V2 {
    let l = len(a);
    if l > 0.0 { scale(a, 1.0 / l) } else { [0.0, 0.0] }
}

/// A rotação de `+90°`: `(x, y) → (−y, x)`.
#[inline]
pub fn perp(a: V2) -> V2 {
    [-a[1], a[0]]
}

/// `> 0` quando `c` está à ESQUERDA da recta orientada `a → b`.
#[inline]
pub fn left_of(a: V2, b: V2, c: V2) -> f64 {
    det(sub(a, c), sub(b, a))
}

/// A distância AO QUADRADO de `c` ao segmento `a → b`.
pub fn dist_sq_to_segment(a: V2, b: V2, c: V2) -> f64 {
    let ab = sub(b, a);
    let l2 = abs_sq(ab);
    if l2 <= 0.0 {
        return abs_sq(sub(c, a));
    }
    let r = dot(sub(c, a), ab) / l2;
    if r < 0.0 {
        abs_sq(sub(c, a))
    } else if r > 1.0 {
        abs_sq(sub(c, b))
    } else {
        abs_sq(sub(c, add(a, scale(ab, r))))
    }
}

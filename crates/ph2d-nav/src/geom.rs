//! A aritmética de ponto da navegação: `f64`, e só `+ − × ÷ sqrt` (a cerca do `Cargo.toml`).
//!
//! # A tolerância é UMA, e diz de que recurso é
//!
//! [`EPS`] é um **comprimento** (metros, a unidade do `Transform`): dois pontos mais perto que isto
//! são o mesmo ponto, e um ponto mais perto que isto de uma recta está nela. O recurso é a precisão
//! de quem ALIMENTA a malha — a `ph2d-navmesh` escreve vértices numa grelha inteira de
//! `1/65 536 m ≈ 1,5e-5 m` (o factor dela), e o `f32` do mundo a `1 000 m` tem um passo de
//! `6,1e-5 m`. `1e-9` fica seis ordens abaixo de ambos e seis acima do passo do `f64` a `1e4 m`
//! (`1,8e-12`) ⇒ nenhum vértice legítimo cai dentro dela e nenhuma conta honesta sai dela.

/// Um ponto (ou vector) do plano, em metros.
pub type V2 = [f64; 2];

/// A tolerância de comprimento — ver o cabeçalho do módulo.
pub const EPS: f64 = 1e-9;

#[inline]
pub fn sub(a: V2, b: V2) -> V2 {
    [a[0] - b[0], a[1] - b[1]]
}

#[inline]
pub fn add(a: V2, b: V2) -> V2 {
    [a[0] + b[0], a[1] + b[1]]
}

#[inline]
pub fn scale(a: V2, s: f64) -> V2 {
    [a[0] * s, a[1] * s]
}

#[inline]
pub fn dot(a: V2, b: V2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

#[inline]
pub fn cross(a: V2, b: V2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

/// `> 0` quando `c` está à ESQUERDA da recta orientada `a → b` (área com sinal ×2).
#[inline]
pub fn orient(a: V2, b: V2, c: V2) -> f64 {
    cross(sub(b, a), sub(c, a))
}

#[inline]
pub fn len(a: V2) -> f64 {
    dot(a, a).sqrt()
}

#[inline]
pub fn dist(a: V2, b: V2) -> f64 {
    len(sub(a, b))
}

#[inline]
pub fn lerp(a: V2, b: V2, t: f64) -> V2 {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// O mesmo ponto, à tolerância [`EPS`].
#[inline]
pub fn same(a: V2, b: V2) -> bool {
    dist(a, b) <= EPS
}

/// A distância COM SINAL de `c` à recta `a → b` (positiva à esquerda). Para `a == b` devolve a
/// distância a `a` com sinal `+`, que é a leitura que ninguém usa mas que não divide por zero.
#[inline]
pub fn side_dist(a: V2, b: V2, c: V2) -> f64 {
    let l = dist(a, b);
    if l <= EPS {
        return dist(a, c);
    }
    orient(a, b, c) / l
}

/// O parâmetro `t ∈ [0, 1]` do ponto do segmento `a → b` mais perto de `p`, e esse ponto.
pub fn closest_on_segment(a: V2, b: V2, p: V2) -> (f64, V2) {
    let ab = sub(b, a);
    let l2 = dot(ab, ab);
    if l2 <= EPS * EPS {
        return (0.0, a);
    }
    let t = (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0);
    (t, lerp(a, b, t))
}

/// A distância de `p` ao segmento `a → b`.
#[inline]
pub fn dist_to_segment(a: V2, b: V2, p: V2) -> f64 {
    dist(closest_on_segment(a, b, p).1, p)
}

/// Os dois segmentos cruzam-se PROPRIAMENTE (no interior dos dois, sem colinearidade) — a pergunta
/// do oráculo de visibilidade. Um toque num extremo NÃO é cruzamento.
pub fn proper_cross(p: V2, q: V2, a: V2, b: V2) -> bool {
    let d1 = side_dist(a, b, p);
    let d2 = side_dist(a, b, q);
    let d3 = side_dist(p, q, a);
    let d4 = side_dist(p, q, b);
    ((d1 > EPS && d2 < -EPS) || (d1 < -EPS && d2 > EPS))
        && ((d3 > EPS && d4 < -EPS) || (d3 < -EPS && d4 > EPS))
}

/// Uma chave de ordem TOTAL para um `f64` — o heap da procura ordena por ela (determinismo: o
/// desempate a seguir é um contador, nunca um endereço).
#[inline]
pub fn ord_key(x: f64) -> u64 {
    let b = x.to_bits();
    if b >> 63 == 1 { !b } else { b | (1 << 63) }
}

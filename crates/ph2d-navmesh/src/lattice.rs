//! A GRELHA INTEIRA onde a construção decide tudo o que é topologia.
//!
//! # O factor e de que recurso ele é
//!
//! Um metro são [`SCALE`] `= 2¹⁶` unidades ⇒ resolução de `1,5e-5 m`. Potência de dois de propósito:
//! `k / 2¹⁶` é EXACTO em `f64` para todo `|k| < 2⁵³`, logo a malha que sai para a `ph2d-nav` tem
//! exactamente os pontos que a grelha decidiu. O tecto do mundo é o do `f64` exacto (`2⁵³ / 2¹⁶ =
//! 2³⁷ m ≈ 1,4e11 m`) — muito antes disso manda o `f32` do `Transform`, cujo passo a `1 000 m` é
//! `6,1e-5 m` (quatro unidades nossas): a grelha não é mais grossa que o mundo que ela representa.
//!
//! # A orientação é exacta
//!
//! Diferenças de coordenadas cabem em `i64` com folga e os produtos em `i128` ⇒ [`orient`] devolve o
//! SINAL CERTO sempre, incluindo para três pontos quase colineares — que é exactamente onde um `f64`
//! erra o sinal e a malha sai com um polígono virado ao contrário.

use ph2d_nav::V2;

/// Unidades da grelha por metro (`2¹⁶`).
pub const SCALE: f64 = 65_536.0;

/// Um ponto da grelha.
pub type P = (i64, i64);

/// Para a grelha (arredonda ao mais perto — o IEEE fixa-o ao bit).
#[inline]
pub fn to_lattice(p: V2) -> P {
    ((p[0] * SCALE).round() as i64, (p[1] * SCALE).round() as i64)
}

/// Da grelha para metros — exacto.
#[inline]
pub fn to_world(p: P) -> V2 {
    [p.0 as f64 / SCALE, p.1 as f64 / SCALE]
}

/// `> 0` com `c` à ESQUERDA de `a → b`, EXACTO.
#[inline]
pub fn orient(a: P, b: P, c: P) -> i128 {
    let (bx, by) = ((b.0 - a.0) as i128, (b.1 - a.1) as i128);
    let (cx, cy) = ((c.0 - a.0) as i128, (c.1 - a.1) as i128);
    bx * cy - by * cx
}

/// O quadrado do comprimento de `a → b`, exacto.
#[inline]
pub fn len2(a: P, b: P) -> i128 {
    let (dx, dy) = ((b.0 - a.0) as i128, (b.1 - a.1) as i128);
    dx * dx + dy * dy
}

/// O fecho convexo (cadeia monótona de Andrew), anti-horário, SEM pontos colineares — exacto.
pub fn hull(mut pts: Vec<P>) -> Vec<P> {
    pts.sort_unstable();
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let mut lower: Vec<P> = Vec::with_capacity(pts.len());
    for &p in &pts {
        while lower.len() >= 2 && orient(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper: Vec<P> = Vec::with_capacity(pts.len());
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && orient(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Os `n` vectores unitários do polígono regular que CIRCUNSCREVE o disco unidade a menos do factor
/// `1 / cos(π/n)` (devolvido à parte): vértices em `(k + ½)·2π/n`, logo as ARESTAS têm normais em
/// `k·2π/n` — com `n` múltiplo de 4 há arestas a `0°`, `90°`, `180°` e `270°`, e uma caixa alinhada
/// aos eixos recua EXACTAMENTE o raio nos lados (só os cantos arredondam).
///
/// ⚠️ **Só `sqrt`** (a cerca do determinismo): `cos(π/4) = √½`, e cada metade do ângulo sai de
/// `cos(θ/2) = √((1 + cos θ)/2)`, `sin(θ/2) = sin θ / (2 cos(θ/2))`. A volta é por multiplicação de
/// rotações — o erro acumulado a `n = 256` é `~1e-14`, e é o MESMO bit nos três sistemas.
///
/// `n` tem de ser uma potência de dois `≥ 4` (o chamador garante-o).
pub fn disk_dirs(n: u32) -> (Vec<V2>, f64) {
    debug_assert!(n >= 4 && n.is_power_of_two());
    // (c, s) = (cos, sin) de π/n.
    let mut c = 0.5f64.sqrt();
    let mut s = c;
    let mut k = 4u32;
    while k < n {
        let c2 = ((1.0 + c) * 0.5).sqrt();
        s /= 2.0 * c2;
        c = c2;
        k *= 2;
    }
    // A rotação de UM passo inteiro (2π/n): o ângulo duplo de (c, s).
    let (cd, sd) = (c * c - s * s, 2.0 * c * s);
    let mut v = [c, s];
    let mut out = Vec::with_capacity(n as usize);
    for _ in 0..n {
        out.push(v);
        v = [v[0] * cd - v[1] * sd, v[0] * sd + v[1] * cd];
    }
    (out, 1.0 / c)
}

//! A lei: mistura (Duncan), camada (Kubelka 1948), cobertura e diluição. F2 §3.2 eq. (1)–(3).

use crate::bandas::Bandas;
use crate::pigmento::Pigmento;

/// Índice de refração do pigmento (orgânico típico) — o mesmo `N_PIG` do oráculo.
const N_PIG: f32 = 1.75;
const N_OLEO: f32 = 1.48;
const N_AGUA_MOLHADA: f32 = 1.33;
/// Aquarela SECA: goma + ar, índice efectivo ~1,25.
const N_AGUA_SECA: f32 = 1.25;

/// Índice do que rodeia a partícula, meio `m ∈ [0,1]` (0 = água, 1 = linhaça).
#[must_use]
pub fn n_veiculo(m: f32, molhado: bool) -> f32 {
    let agua = if molhado { N_AGUA_MOLHADA } else { N_AGUA_SECA };
    (1.0 - m) * agua + m * N_OLEO
}

/// `s(n_v)`: espalhamento ∝ contraste de índice ao quadrado (Fresnel), normalizado na linhaça.
#[must_use]
pub fn s_fator(m: f32, molhado: bool) -> f32 {
    let n = n_veiculo(m, molhado);
    let c = (N_PIG - n) / (N_PIG + n);
    let c0 = (N_PIG - N_OLEO) / (N_PIG + N_OLEO);
    (c * c) / (c0 * c0)
}

/// Espessuras ópticas da mistura, por banda (adimensionais).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Optica<const N: usize> {
    pub k: Bandas<N>,
    pub s: Bandas<N>,
}

/// Eq. (1), Duncan 1940: `𝒦 = Σ v_i K_i`, `𝒮 = s · Σ v_i S_i`, somados na ORDEM do slice.
/// `v_i` em µm de pigmento puro, `≥ 0` (debug-assert: um volume negativo é a «cor negativa» que o
/// teorema exclui).
#[must_use]
pub fn mistura<const N: usize>(itens: &[(&Pigmento<N>, f32)], s_fator: f32) -> Optica<N> {
    let mut k = [0.0f32; N];
    let mut s = [0.0f32; N];
    for &(p, v) in itens {
        debug_assert!(v >= 0.0 && v.is_finite(), "volume inválido: {v}");
        for b in 0..N {
            k[b] += v * p.k.0[b];
            s[b] += v * p.s.0[b];
        }
    }
    for sb in &mut s {
        *sb *= s_fator;
    }
    Optica {
        k: Bandas(k),
        s: Bandas(s),
    }
}

/// Eq. (2), Kubelka 1948, UMA banda: camada de espessuras ópticas `k`, `s` sobre o fundo `rg`.
///
/// Forma usada (multiplicada por `𝒮`, para não dividir por `𝒮`): com `x = b𝒮 = √(𝒦(𝒦+2𝒮))` e
/// `h = x(coth x − 1) = 2x / expm1(2x)`,
/// `R = (𝒮(1−R_g) + R_g·𝒮(𝒦+x)/(𝒦+𝒮+x) + R_g·h) / ((𝒦+x) + 𝒮(1−R_g) + h)`.
/// Numerador e denominador são SOMAS de termos `≥ 0` (sem cancelação: a forma directa
/// `𝒮 − R_g(𝒮²/(𝒦+𝒮+x) − h)` perdia ~2e-6 com `R_g ≈ 1` e `𝒮 ~ 10³`, medido pelo Lema 2).
/// Regimes: `𝒦+𝒮 = 0` → `R_g` exacto (nada de pigmento) · `x = 0` (`𝒦 = 0`) → `h = 1`, o limite
/// exacto · `x` pequeno → `expm1f` mantém a precisão (sem `coth 0`) · `𝒮 → 0` → o mesmo ramo dá
/// `R_g e^{−2𝒦}` sem caso especial · `x ≳ 44` → `expm1f` satura em `∞`, `h = 0` e `R = R∞`.
/// O denominador é `≥ 𝒦 + x + h + 𝒮(1−R_g) > 0` para `R_g ≤ 1`.
#[must_use]
pub fn camada_banda(k: f32, s: f32, rg: f32) -> f32 {
    debug_assert!(
        k >= 0.0 && s >= 0.0 && (0.0..=1.0).contains(&rg),
        "k={k} s={s} rg={rg}"
    );
    let ks = k + s;
    if ks <= 0.0 {
        return rg;
    }
    let x = (k * (k + 2.0 * s)).sqrt();
    let h = if x > 0.0 {
        (2.0 * x) / libm::expm1f(2.0 * x)
    } else {
        1.0
    };
    let um_menos_rg = 1.0 - rg;
    let num = s * um_menos_rg + rg * ((s * (k + x)) / (ks + x)) + rg * h;
    let den = (k + x) + s * um_menos_rg + h;
    num / den
}

/// Eq. (2) por banda.
#[must_use]
pub fn camada<const N: usize>(o: &Optica<N>, rg: &Bandas<N>) -> Bandas<N> {
    Bandas(core::array::from_fn(|b| {
        camada_banda(o.k.0[b], o.s.0[b], rg.0[b])
    }))
}

/// Masstone `R∞(q)`, `q = K/S`, na forma racionalizada `1/(1 + q + √(q²+2q))` — igual a
/// `1 + q − √(q²+2q)` sem a cancelação.
#[must_use]
pub fn r_infinito(q: f32) -> f32 {
    1.0 / (1.0 + q + (q * q + 2.0 * q).sqrt())
}

/// Eq. (3): cobertura DERIVADA, `C = 1 − (R|_{R_g=1} − R|_{R_g=0})`, por banda.
#[must_use]
pub fn cobertura<const N: usize>(o: &Optica<N>) -> Bandas<N> {
    Bandas(core::array::from_fn(|b| {
        1.0 - (camada_banda(o.k.0[b], o.s.0[b], 1.0) - camada_banda(o.k.0[b], o.s.0[b], 0.0))
    }))
}

/// Diluir a tinta do tubo: a fracção volúmica de pigmento cai com o diluente acrescentado,
/// `φ' = φ(1 − w)` (o veículo tem `K = S = 0`, Lema 3). `w ∈ [0,1]`.
#[must_use]
pub fn dilui(phi: f32, agua: f32) -> f32 {
    phi * (1.0 - agua.clamp(0.0, 1.0))
}

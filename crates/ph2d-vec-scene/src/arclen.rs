//! **Comprimento de arco de uma cúbica** — e o inverso dele.
//!
//! É o motor que faltava a esta crate, e ele é pré-requisito de metade da fila de efeitos
//! (Trim, Repeater, Pattern Along Path, texto em caminho). A pesquisa
//! `20_pesquisa_ferramentas_de_artista.md` §1.2 nomeia a armadilha com todas as letras:
//!
//! > o parâmetro `t` de uma Bézier **não é proporcional ao comprimento de arco**.
//!
//! Espaçar por `t` aglomera nas curvas e espalha nas retas — e *parece certo numa reta*,
//! que é o que faz a versão errada passar no primeiro olhar.
//!
//! # Por que aqui, e não pelo `kurbo`
//!
//! O `kurbo` tem `arclen`/`inv_arclen` prontos, e esta crate **não o alcança de propósito**:
//! o `Cargo.toml` declara *"modelo puro de documento — sem vello/kurbo; a conversão para
//! `kurbo::BezPath` vive no render"*. Como o `cooked()` é chamado de dentro desta própria
//! crate (`inside`, `boundary`, `path_ops`, `space`), o efeito tem de ser avaliável aqui —
//! e arrastar a stack Linebender para dentro do modelo de documento para obter 40 linhas de
//! quadratura seria pagar caro por uma cerca que já foi decidida.
//!
//! # O método
//!
//! Comprimento por **Gauss-Legendre de 16 nós** sobre `|B'(t)|` — exato para polinómios até
//! grau 31, e `|B'|` não é polinómio (tem a raiz), mas o erro cai a ~1e-12 numa cúbica de
//! curvatura sã. O inverso é **bisseção** sobre o comprimento acumulado: converge sempre,
//! não precisa da derivada, e 40 iterações levam o intervalo a 1e-12 do domínio.
//!
//! Só `sqrt` — que é exatamente arredondado em IEEE-754 e portanto não é fonte de skew entre
//! plataformas, ao contrário de `sin`/`exp`/`powf`.

/// Uma cúbica: `[P0, C1, C2, P3]`.
pub type Cubic = [[f64; 2]; 4];

/// Nós e pesos de Gauss-Legendre de 16 pontos no intervalo `[-1, 1]` (simétricos: guardamos
/// metade). Constantes de tabela — nada é calculado em runtime.
const GL16: [(f64, f64); 8] = [
    (0.095_012_509_837_637_44, 0.189_450_610_455_068_50),
    (0.281_603_550_779_258_9, 0.182_603_415_044_923_60),
    (0.458_016_777_657_227_4, 0.169_156_519_395_002_54),
    (0.617_876_244_402_643_7, 0.149_595_988_816_576_73),
    (0.755_404_408_355_003_0, 0.124_628_971_255_533_87),
    (0.865_631_202_387_831_7, 0.095_158_511_682_492_78),
    (0.944_575_023_073_232_6, 0.062_253_523_938_647_88),
    (0.989_400_934_991_649_9, 0.027_152_459_411_754_09),
];

/// `B'(t)` de uma cúbica — a derivada é uma quadrática nos três deltas de controlo.
#[must_use]
fn deriv(c: &Cubic, t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let mut d = [0.0; 2];
    for k in 0..2 {
        let a = c[1][k] - c[0][k];
        let b = c[2][k] - c[1][k];
        let e = c[3][k] - c[2][k];
        d[k] = 3.0 * (a * u * u + 2.0 * b * u * t + e * t * t);
    }
    d
}

/// `|B'(t)|` — a velocidade escalar com que o ponto anda quando `t` anda.
#[must_use]
fn speed(c: &Cubic, t: f64) -> f64 {
    let d = deriv(c, t);
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// **O comprimento de arco de `c` no sub-intervalo `[0, t]`** (com `t = 1` dando o total).
///
/// Gauss-Legendre de 16 nós, transposto de `[-1, 1]` para `[0, t]`.
#[must_use]
pub fn arclen_to(c: &Cubic, t: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    let half = t * 0.5;
    let mut sum = 0.0;
    for (x, w) in GL16 {
        sum += w * (speed(c, half * (1.0 - x)) + speed(c, half * (1.0 + x)));
    }
    sum * half
}

/// O comprimento total de `c`.
#[must_use]
pub fn arclen(c: &Cubic) -> f64 {
    arclen_to(c, 1.0)
}

/// **O `t` em que a cúbica alcançou o comprimento `s`** — o inverso de [`arclen_to`].
///
/// Bisseção: monótona por construção (`|B'| >= 0`), então nunca diverge, e não pede a
/// derivada do comprimento. Fora do domínio, satura nas pontas — um chamador que peça mais
/// arco do que existe quer o fim da curva, não um erro.
#[must_use]
pub fn inv_arclen(c: &Cubic, s: f64) -> f64 {
    let total = arclen(c);
    if s <= 0.0 || total <= 0.0 {
        return 0.0;
    }
    if s >= total {
        return 1.0;
    }
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if arclen_to(c, mid) < s {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// **O pedaço de `c` entre `t0` e `t1`**, ele mesmo uma cúbica exata.
///
/// Dois de Casteljau: corta em `t1` e fica com a cabeça, depois corta essa cabeça em `t0`
/// reparametrizado e fica com a cauda. A curva nunca é achatada em polilinha — é a mesma
/// disciplina que o `ph2d-vec-blend` impôs ao pareamento (`subsegment` exato).
#[must_use]
pub fn subsegment(c: &Cubic, t0: f64, t1: f64) -> Cubic {
    let head = split_head(c, t1);
    if t1 <= 0.0 {
        return [c[0], c[0], c[0], c[0]];
    }
    split_tail(&head, (t0 / t1).clamp(0.0, 1.0))
}

/// A parte `[0, t]` de `c`.
#[must_use]
fn split_head(c: &Cubic, t: f64) -> Cubic {
    let p01 = lerp(c[0], c[1], t);
    let p12 = lerp(c[1], c[2], t);
    let p23 = lerp(c[2], c[3], t);
    let a = lerp(p01, p12, t);
    let b = lerp(p12, p23, t);
    [c[0], p01, a, lerp(a, b, t)]
}

/// A parte `[t, 1]` de `c`.
#[must_use]
fn split_tail(c: &Cubic, t: f64) -> Cubic {
    let p01 = lerp(c[0], c[1], t);
    let p12 = lerp(c[1], c[2], t);
    let p23 = lerp(c[2], c[3], t);
    let a = lerp(p01, p12, t);
    let b = lerp(p12, p23, t);
    [lerp(a, b, t), b, p23, c[3]]
}

#[must_use]
fn lerp(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

#[cfg(test)]
#[path = "arclen_tests.rs"]
mod tests;

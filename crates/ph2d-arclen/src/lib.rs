//! **Comprimento de arco de uma cúbica** — e o inverso dele.
//!
//! **A única resposta do repo para *"quanto andei nesta curva?"***. A pesquisa
//! `20_pesquisa_ferramentas_de_artista.md` §1.2 nomeia a armadilha com todas as letras:
//!
//! > o parâmetro `t` de uma Bézier **não é proporcional ao comprimento de arco**.
//!
//! Espaçar por `t` aglomera nas curvas e espalha nas retas — e *parece certo numa reta*,
//! que é o que faz a versão errada passar no primeiro olhar.
//!
//! # Por que uma crate própria
//!
//! Nasceu dentro da `ph2d-vec-scene` (pilha de efeitos do Vector, [ADR-0132]: Trim, Repeater,
//! Pattern Along Path, texto em caminho) e mudou-se para cá quando apareceu um **segundo
//! consumidor**: o motion path da timeline ([ADR-0141]), cuja track escalar mede exatamente
//! comprimento de arco. Duas cópias divergiriam — e a crate de origem é um modelo de
//! **documento**, que o runtime de animação não pode passar a depender só para obter 180
//! linhas de quadratura.
//!
//! **Zero dependências, de propósito.** É o que permite ao modelo de documento do Vector (que
//! se declara *sem-kurbo* no próprio `Cargo.toml`) e ao runtime da timeline consumirem a mesma
//! resposta sem arrastar nada. A `ph2d-vec-scene` re-exporta este módulo como `arclen`, então
//! os chamadores de lá seguem escrevendo `crate::arclen::…`.
//!
//! # O método
//!
//! Comprimento por **Gauss-Legendre de 16 nós** sobre `|B'(t)|` — exato para polinómios até
//! grau 31, e `|B'|` não é polinómio (tem a raiz), mas o erro cai a ~1e-12 numa cúbica de
//! curvatura sã.
//!
//! O inverso é **Newton com cerca de bisseção** (o `rtsafe` do Numerical Recipes): a derivada
//! do comprimento é `ds/dt = |B'(t)|` e está disponível **de graça**, que é precisamente a
//! condição em que Newton bate bisseção. O intervalo `[lo, hi]` é mantido a cada passo, e um
//! passo de Newton que saia dele é substituído pelo ponto médio — então **converge sempre**,
//! como a bisseção, e em ~4 iterações em vez de 40.
//!
//! ⚠️ **Foi bisseção pura de 40 iterações até 2026-07-23**, e o preço estava medido: cada
//! iteração chama [`arclen_to`] (16 nós × 2 avaliações), ou seja **~1300 `sqrt` por inversa**,
//! **1700 ns**. O motion path amostra isto por entidade e por frame; a medição está na Fatia 0
//! do [ADR-0141] (`ph2d-timeline/tests/measure_motion_path.rs`) e deu **12×**.
//!
//! Só `sqrt` — que é exatamente arredondado em IEEE-754 e portanto não é fonte de skew entre
//! plataformas, ao contrário de `sin`/`exp`/`powf`.
//!
//! [ADR-0132]: ../../../docs/architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md
//! [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md

/// Uma cúbica: `[P0, C1, C2, P3]`.
pub type Cubic = [[f64; 2]; 4];

/// Nós e pesos de Gauss-Legendre de 16 pontos no intervalo `[-1, 1]` (simétricos: guardamos
/// metade). Constantes de tabela — nada é calculado em runtime.
const GL16: [(f64, f64); 8] = [
    (0.095_012_509_837_637_44, 0.189_450_610_455_068_5),
    (0.281_603_550_779_258_9, 0.182_603_415_044_923_6),
    (0.458_016_777_657_227_4, 0.169_156_519_395_002_54),
    (0.617_876_244_402_643_7, 0.149_595_988_816_576_73),
    (0.755_404_408_355_003, 0.124_628_971_255_533_87),
    (0.865_631_202_387_831_7, 0.095_158_511_682_492_78),
    (0.944_575_023_073_232_6, 0.062_253_523_938_647_88),
    (0.989_400_934_991_649_9, 0.027_152_459_411_754_09),
];

/// **O ponto da cúbica em `t`** — Bernstein direto, sem de Casteljau (é mais barato e a
/// precisão basta: os coeficientes são pequenos e o grau é 3).
#[must_use]
pub fn point_at(c: &Cubic, t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (a, b, d, e) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        a * c[0][0] + b * c[1][0] + d * c[2][0] + e * c[3][0],
        a * c[0][1] + b * c[1][1] + d * c[2][1] + e * c[3][1],
    ]
}

/// **A tangente UNITÁRIA em `t`**, ou `None` numa cúspide **VERDADEIRA**.
///
/// # ⭐⭐⭐ Velocidade zero **não** é o mesmo que direção ausente (2026-08-30)
///
/// `B'(t) = 0` era lido como *"aqui não há direção"*, e nas **pontas** de um segmento isso é
/// falso com uma frequência que não é rara: `B'(0) = 3·(P₁ − P₀)`, e um segmento **RETO**
/// autorado por dois vértices de quina tem `P₁ = P₀` e `P₃ = P₂` — logo `B'` anula-se nas
/// **duas** pontas de **todo** segmento reto do repositório. A curva ali é uma reta e a direção
/// dela é óbvia; o que não existe é a *velocidade*.
///
/// ⚠️ **O preço estava medido antes desta cura** (plano 36 §11.1): o pincel de contorno **não
/// desenhava** `1`–`2` cópias por quadrado — em **todo** tamanho testado, de lado `2` a `20` —,
/// porque uma cópia cujo centro caísse numa quina era **pulada em silêncio**. O mesmo `None`
/// atravessa o Trim, o Repeater, o texto em caminho, o Zig Zag e o motion path da timeline.
///
/// # A regra, e ela é a canónica de Bézier
///
/// Quando `B'` se anula, a direção sai do **polígono de controlo**, saltando os pontos
/// coincidentes:
///
/// | onde | cascata |
/// |---|---|
/// | `t = 0` | `P₁ − P₀`, senão `P₂ − P₀`, senão `P₃ − P₀` |
/// | `t = 1` | `P₃ − P₂`, senão `P₃ − P₁`, senão `P₃ − P₀` |
///
/// É equivalente a cair para a primeira derivada não-nula, e diz-se em pontos de controlo
/// para não haver caso de SINAL — pela esquerda o limite de `B'` é `−B''`, pela direita é
/// `+B''`, e uma das duas versões desse cuidado está sempre errada.
///
/// # ⛔ E a cúspide INTERIOR continua a devolver `None`, de propósito
///
/// Com `0 < t < 1` os dois lados existem, e numa cúspide a **marcha inverte**: a reta tangente
/// é a mesma, o sentido não. Inventar um sentido ali é o que produz *"o pico solto que ninguém
/// sabe de onde veio"* — a razão original desta função devolver `Option`, e ela fica.
#[must_use]
pub fn tangent_at(c: &Cubic, t: f64) -> Option<[f64; 2]> {
    if let Some(u) = unit(deriv(c, t)) {
        return Some(u);
    }
    // `B'(t) = 0`. Numa PONTA a direção existe na mesma — e só numa ponta, porque só ali existe
    // um único lado de onde vir.
    //
    // ⚠️⚠️ **A condição é o POLÍGONO ser degenerado naquela ponta, e NÃO `t == 0`/`t == 1`.**
    // A 1.ª redacção desta cura testava `t <= 0.0` / `t >= 1.0` e **falhava em metade das
    // fixturas**, com um mecanismo que só a medição mostra: o prefixo de arco é somado por
    // quadratura, e o comprimento de um segmento reto de `2` sai `2,000000000000000_4`. Quem
    // pergunta pelo arco `2,0` cai então no segmento **anterior**, em `t = 0,999999999999999_8`
    // — perto da ponta, nunca **na** ponta —, e ali `|B'|` vale `~2,6e-15`: abaixo do piso do
    // versor e **acima** de zero. ⇒ *uma cura escrita sobre a igualdade de um `t` derivado de
    // uma soma de quadratura é uma cura que passa nos testes de mesa e falha na árvore.*
    //
    // ⛔ E a cerca contra «inventar direção» continua a ser exacta: só se entra na cascata quando
    // aquela ponta **é** degenerada. Numa cúspide interior verdadeira (`P₁ ≠ P₀` e `P₃ ≠ P₂`)
    // nenhum dos dois ramos abre.
    let sub = |a: [f64; 2], b: [f64; 2]| [a[0] - b[0], a[1] - b[1]];
    if t < 0.5 {
        if unit(sub(c[1], c[0])).is_none() {
            return unit(sub(c[2], c[0])).or_else(|| unit(sub(c[3], c[0])));
        }
    } else if unit(sub(c[3], c[2])).is_none() {
        return unit(sub(c[3], c[1])).or_else(|| unit(sub(c[3], c[0])));
    }
    None
}

/// O versor de `v`, ou `None` se ele não tem comprimento que se meça.
///
/// O piso é `1e-12` **em norma**, o mesmo desde que esta função existe: abaixo dele a divisão
/// devolve lixo amplificado, e um versor de lixo é pior que a ausência.
fn unit(v: [f64; 2]) -> Option<[f64; 2]> {
    let n = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (n > 1e-12).then(|| [v[0] / n, v[1] / n])
}

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

/// Quão perto do comprimento pedido a inversa tem de aterrissar, como fração do comprimento
/// **total da cúbica**. `1e-12` é o que a bisseção de 40 iterações entregava (`2^-40 ≈ 9e-13`
/// do domínio de `t`), e é deliberado: quem já dependia da precisão antiga não a perde.
const INV_TOL_REL: f64 = 1e-12;

/// Teto de iterações da inversa. Nunca alcançado no caminho normal (~4 passos), e a cerca de
/// bisseção garante que mesmo o pior caso seja pelo menos tão bom quanto a bisseção pura.
const INV_MAX_ITERS: usize = 40;

/// **O `t` em que a cúbica alcançou o comprimento `s`** — o inverso de [`arclen_to`].
///
/// **Newton com cerca de bisseção** (`rtsafe`): `ds/dt = |B'(t)|` é a derivada exata e sai de
/// graça de [`speed`], então o passo de Newton é quase sempre o certo; o intervalo `[lo, hi]`
/// é mantido a cada passo e um Newton que saia dele (ou uma velocidade nula, numa cúspide) cai
/// no ponto médio. **Converge sempre**, como a bisseção, em ~4 iterações em vez de 40.
///
/// Fora do domínio, satura nas pontas — um chamador que peça mais arco do que existe quer o fim
/// da curva, não um erro.
#[must_use]
pub fn inv_arclen(c: &Cubic, s: f64) -> f64 {
    let total = arclen(c);
    if s <= 0.0 || total <= 0.0 {
        return 0.0;
    }
    if s >= total {
        return 1.0;
    }
    let tol = total * INV_TOL_REL;
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    // Palpite inicial LINEAR em `s/total`: é exato numa reta e já perto em qualquer curva sã.
    let mut t = s / total;
    for _ in 0..INV_MAX_ITERS {
        let err = arclen_to(c, t) - s;
        if err.abs() <= tol {
            return t;
        }
        // O intervalo encolhe SEMPRE, mesmo quando o passo de Newton é descartado — é isso que
        // torna o pior caso desta função a bisseção, e não uma divergência.
        if err < 0.0 {
            lo = t;
        } else {
            hi = t;
        }
        let sp = speed(c, t);
        let next = t - err / sp;
        t = if sp > 1e-12 && next > lo && next < hi {
            next
        } else {
            0.5 * (lo + hi)
        };
    }
    t
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
#[path = "tests.rs"]
mod tests;

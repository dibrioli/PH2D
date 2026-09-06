//! ⭐⭐⭐ **A SUPERQUADRÁTICA** (W127) — *um* knob que atravessa a família inteira: losango →
//! esfera → caixa, e no perfil bipirâmide → elipsóide → cilindro.
//!
//! # A fórmula, e por que ela precisa de um DIVISOR
//!
//! A peça é a bola da norma-`n` encaixada: com `q = p / meia_medida`,
//!
//! ```text
//! s = (|qx|^n1 + |qy|^n1)^(1/n1)      — a secção horizontal
//! g = (s^n2  + |qz|^n2)^(1/n2)        — o perfil vertical
//! f = g − 1
//! ```
//!
//! ⚠️ **`f` não é uma distância** — ele é adimensional e o gradiente dele passa de `1`. Mas este
//! módulo **nunca precisou da distância exacta: ele precisa de um MINORANTE** (doc 06 §124), e
//! `f / K` é um com `K = max‖∇f‖`.
//!
//! # ⭐⭐⭐ E o `K` sai em FORMA FECHADA, porque `g` é homogénea de grau 1
//!
//! `g(λq) = λ g(q)` ⇒ `∇g` é homogénea de grau **zero** ⇒ ela é **constante ao longo de cada raio**
//! da origem. Toda direcção está representada na superfície `g = 1` ⇒ *o máximo sobre a superfície
//! **é** o máximo global*, e não é preciso varrer o espaço para o achar.
//!
//! Na superfície, com `t = s^n2` e `u = |qx|^n1 / s^n1` (os dois em `[0, 1]`):
//!
//! ```text
//! ‖∇f‖² = t^α2 · [ u^α1/hx² + (1−u)^α1/hy² ] + (1−t)^α2 / hz²        com  αᵢ = 2 − 2/nᵢ
//! ```
//!
//! São **duas** maximizações independentes da MESMA forma — `Σ uᵢ^α wᵢ` sobre o simplexo — e é isso
//! que o [`simplex_max`] resolve, aplicado duas vezes.
//!
//! # ⛔ A cerca `n ≥ 1`, e ela é do CAMPO, não de conforto
//!
//! `α = 2 − 2/n` é **negativa** abaixo de `n = 1`, e aí `u^α → ∞` quando `u → 0`: o gradiente na
//! superfície **não tem limite**. É a astróide, cuja superfície tem **cúspides** — e uma cúspide não
//! admite minorante com divisor constante. ⇒ `n = 1` (o octaedro) é o fim honesto do controlo, e é
//! uma forma útil, não uma degenerescência.

use fidget::context::Tree;

/// A regularização que tira o `ln(0)` do caminho: `|q|^n` é calculado como `(q² + EPS)^(n/2)`.
///
/// ⚠️ **Não é conforto numérico, é o GRADIENTE**: `exp(n · ln|q|)` avalia bem em `q = 0` (dá `0`),
/// e a **derivada** dele ali é `0 · ∞ = NaN` — e este módulo lê o gradiente para a normal, logo um
/// `NaN` seria um pixel preto no plano `x = 0` de toda peça. Com `q² + EPS` a derivada é
/// `n·q·(q²+EPS)^(n/2−1)`, que vale `0` na origem.
///
/// ⚠️ E ela só **infla** a soma, logo a peça sai um fio mais pequena — a `1e-12`, `6` ordens de
/// grandeza abaixo do que uma malha a `64³` resolve.
const EPS: f64 = 1.0e-12;

/// O piso do divisor da norma estável. ⚠️ Ele só é atingido **na origem**, onde `f ≈ −1` e o valor
/// exacto não interessa a ninguém; em toda a peça o maior `|qᵢ|` é ordens de grandeza acima.
const FLOOR: f64 = 1.0e-9;

/// `|v|^n`, sem o `ln(0)`. Ver [`EPS`]. ⚠️ **O chamador entrega uma RAZÃO em `[0, 1]`** — ver
/// [`norma`].
fn abs_pow(v: &Tree, n: f64) -> Tree {
    let q = v.clone() * v.clone() + Tree::constant(EPS);
    // ⭐⭐ **Os dois expoentes do meio da faixa são EXACTOS sem transcendental** (auditoria de
    // 06/09): `2` é o quadrado e `1` é a raiz. ⚠️ E `2` é a **esfera**, que é o ponto por onde toda
    // travessia desta forma passa.
    if (n - 2.0).abs() < 1.0e-12 {
        return q;
    }
    if (n - 1.0).abs() < 1.0e-12 {
        return q.sqrt();
    }
    (q.ln() * Tree::constant(n * 0.5)).exp()
}

/// ⭐⭐⭐ **A norma-`n` de dois números, pelo caminho ESTÁVEL** — `m · Σ(|vᵢ|/m)^n` elevado a `1/n`,
/// com `m` o maior deles.
///
/// # ⛔⛔ A rota ingénua ESTOURA, e está medido
///
/// `exp(n · ln|q|)` com `|q| = 5,7` (uma peça de `0,35` vista a `2` da origem) passa o `f64` em
/// `n ≈ 407`, e a marcha **viaja lá**. Medido na caixa `±2`: a `n = 512` só **`4 913` de `15 625`**
/// amostras eram finitas, e a `1 024` só `729`. *Um campo que devolve `inf` longe da peça faz a
/// marcha dar um passo infinito.*
///
/// ⭐ Com a razão, a base de cada potência fica em `[0, 1]` e `Σ` em `[1, 2]` ⇒ **não há expoente
/// que estoure**, e o `ln` da soma passa a ser o de um número entre `1` e `2`, que é o regime
/// melhor condicionado que existe. ⇒ o tecto do controlo deixa de ser da representação.
fn norma(a: &Tree, b: &Tree, n: f64) -> Tree {
    // ⭐⭐⭐ **A norma-2 é a hipotenusa** — sem razão, sem `ln`, sem `exp`, e sem o `max` do
    // denominador estável (ali não há nada que estoure). *A esfera é o ponto neutro desta forma, e
    // ela passa a custar o que uma esfera custa.*
    if (n - 2.0).abs() < 1.0e-12 {
        return (a.clone() * a.clone() + b.clone() * b.clone()).sqrt();
    }
    let m = a.abs().max(b.abs()).max(Tree::constant(FLOOR));
    let soma = abs_pow(&(a.clone() / m.clone()), n) + abs_pow(&(b.clone() / m.clone()), n);
    m * (soma.ln() * Tree::constant(1.0 / n)).exp()
}

/// ⭐⭐ **`max Σ uᵢ^α wᵢ` sobre o simplexo `Σuᵢ = 1`**, com `α = 2 − 2/n` — o núcleo do divisor.
///
/// | `n` | `α` | `uᵢ^α` é | o máximo está |
/// |---|---|---|---|
/// | `≥ 2` | `[1, 2)` | **convexa** | num **vértice** ⇒ `max wᵢ` |
/// | `[1, 2)` | `[0, 1)` | **côncava** | no **interior** ⇒ `(Σ wᵢ^β)^(1/β)`, `β = n/(2−n)` |
///
/// ⚠️ **As duas concordam em `n = 2`**: ali `β → ∞` e a média-potência colapsa no máximo. *Um
/// divisor com dois ramos que discordassem na fronteira seria um degrau no meio do knob.*
///
/// # Panics
/// Nunca — o chamador garante `n ≥ 1` (a cerca do documento).
fn simplex_max(w: &[f64], n: f64) -> f64 {
    debug_assert!(n >= 1.0, "a cerca do documento é n ≥ 1");
    let maior = w.iter().copied().fold(0.0_f64, f64::max);
    if n >= 2.0 {
        return maior;
    }
    let beta = n / (2.0 - n);
    w.iter().map(|x| x.powf(beta)).sum::<f64>().powf(1.0 / beta)
}

/// ⭐ **O maior `‖∇f‖` da peça** — o divisor que faz do campo um minorante.
///
/// ⚠️ É público porque o **gate** o lê: uma sonda que recalculasse a conta ao lado seria um oráculo
/// que usa a função sob teste, e a fórmula fechada tem de ser conferida contra a **medição**, não
/// contra si própria.
#[must_use]
pub fn superquadric_gradient_bound(half: [f64; 3], n_top: f64, n_side: f64) -> f64 {
    let w = |h: f64| 1.0 / (h * h);
    // ⚠️ **`X` e `Z` são o de CIMA; `Y` é o de LADO** — o eixo de cima desta casa é o `Y`, e a
    // ordem é a mesma em [`sd_superquadric`]. *Duas permutações na ordem errada são a identidade no
    // caso de omissão (uma peça cúbica), e só uma peça TORTA as separa.*
    let de_cima = simplex_max(&[w(half[0]), w(half[2])], n_top);
    simplex_max(&[de_cima, w(half[1])], n_side).sqrt()
}

/// ⭐⭐⭐ **SUPERQUADRÁTICA** — a bola da norma-`n` encaixada, com meia-medida por eixo.
///
/// `n_top` governa o que se vê **de cima** (`1` losango · `2` círculo · alto quadrado) e `n_side` o
/// que se vê **de lado** (`1` bipirâmide · `2` elipse · alto prisma). A esfera é `2` nos dois, e
/// nesse ponto o campo é a distância **exacta** — ver o gate.
///
/// ⛔ **Ela não tem filete nem chanfro:** o expoente **é** o arredondamento desta forma, e um
/// segundo número sobre a mesma aresta seriam duas verdades sobre ela. (A mesma lei do bojo do
/// `RoundedCylinder`, W125.)
#[must_use]
pub fn sd_superquadric(half: [f64; 3], n_top: f64, n_side: f64) -> Tree {
    let q = |t: Tree, h: f64| t / Tree::constant(h);
    let (qx, qy, qz) = (
        q(Tree::x(), half[0]),
        q(Tree::y(), half[1]),
        q(Tree::z(), half[2]),
    );
    // O que se vê de cima (`X–Z`) na norma `n_top`, e o perfil (`Y`) na `n_side` — **encaixados**,
    // que é o que dá a família inteira em vez de só a bola da norma-`n`.
    let g = norma(&norma(&qx, &qz, n_top), &qy, n_side);
    (g - 1.0) / Tree::constant(superquadric_gradient_bound(half, n_top, n_side))
}

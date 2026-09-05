//! **A MEMBRANA** — quanto o pano resiste a esticar e a cisalhar, por triângulo.
//!
//! St. Venant–Kirchhoff, que é a escolha do próprio paper do VBD para tecido:
//! `ψ = μ·tr(G²) + (λ/2)·tr(G)²`, com `G = ½(FᵀF − I)` a deformação de Green e
//! `F` o gradiente de deformação `3×2` do triângulo.
//!
//! ⚠️ **Ela é ZERO no repouso por CONSTRUÇÃO, não por calibração:** `F` é medido
//! contra a forma de repouso do próprio triângulo, então `G = 0` ali e o gradiente
//! é nulo ao bit. É isso que faz o gate *«o repouso é ponto fixo»* poder existir.
//!
//! ⚠️ **A Hessiana pode ser INDEFINIDA sob compressão**, e isso é da lei, não um
//! descuido — o VBD explicitamente não a projeta (o `3×3` analítico anda para o
//! extremo da aproximação quadrática). Quem trata a degenerescência é o
//! [`crate::vbd`], por salto.

use crate::{V3, dot, norm, scale, sub};

/// A forma de repouso de um triângulo: o inverso `2×2` e a área.
///
/// O referencial local põe `u₀` na origem e `u₁` no eixo `x` — qualquer
/// referencial serve, porque a energia é invariante a rotação do repouso; este é
/// o mais barato de construir.
///
/// ⚠️ **Degenerado devolve área ZERO e inverso zerado**, e com área zero o
/// triângulo não contribui gradiente nenhum: *ele fica MUDO, nunca `NaN`*. Uma
/// escultura tem triângulos assim, e recusar a pegada inteira por causa de um
/// seria o pincel morrer exatamente onde a malha está feia.
///
/// # ⭐⭐⭐ Por que a MÉTRICA de repouso é guardada em vez de se supor `I`
///
/// A definição de livro é `G = ½(FᵀF − I)`, e ela supõe que a parametrização do
/// repouso é **exatamente** isométrica ao repouso — o que é verdade em álgebra e
/// **falso em `f64`**: montar o referencial local, inverter `Dm` e recompor `F`
/// deixa `FᵀF` a `~1e-16` da identidade. Medido: a energia no repouso lia
/// **`4,2e-30`**, e o resíduo movia a malha.
///
/// ⇒ guarda-se `FᵀF` **medido no repouso, pelo mesmo caminho de código**, e a
/// deformação passa a ser `G = ½(FᵀF − G_repouso)`. Isso torna o repouso um zero
/// **ao bit** por construção, e não por um epsilon escolhido — *a representação
/// apaga o caso especial*. Custo: três `f64` por triângulo.
#[derive(Clone, Debug)]
pub(crate) struct TriRest {
    /// O inverso `2×2` da forma de repouso.
    pub(crate) dm_inv: [[f64; 2]; 2],
    /// `FᵀF` medido no repouso — ver acima.
    pub(crate) metric: [[f64; 2]; 2],
    /// A área de repouso. `0` quer dizer MUDO.
    pub(crate) area: f64,
}

pub(crate) fn rest_of(x: &[V3], t: [u32; 3]) -> TriRest {
    let (x0, x1, x2) = (x[t[0] as usize], x[t[1] as usize], x[t[2] as usize]);
    let (e1, e2) = (sub(x1, x0), sub(x2, x0));
    let l1 = norm(e1);
    let mudo = TriRest {
        dm_inv: [[0.0; 2]; 2],
        metric: [[0.0; 2]; 2],
        area: 0.0,
    };
    if l1 < 1e-12 {
        return mudo;
    }
    let ex = scale(e1, 1.0 / l1);
    let px = dot(e2, ex);
    let py = norm(sub(e2, scale(ex, px)));
    // `Dm` tem as colunas `u₁ = (l1, 0)` e `u₂ = (px, py)`.
    let det = l1 * py;
    if det.abs() < 1e-18 {
        return mudo;
    }
    let inv = 1.0 / det;
    // `m[r][c]`: linha `r`, coluna `c`.
    let m = [[py * inv, -px * inv], [0.0, l1 * inv]];
    TriRest {
        dm_inv: m,
        metric: metric(&deform(x, t, &m)),
        area: 0.5 * det,
    }
}

/// `FᵀF` — a métrica da superfície nesta pose.
fn metric(f: &[V3; 2]) -> [[f64; 2]; 2] {
    [
        [dot(f[0], f[0]), dot(f[0], f[1])],
        [dot(f[1], f[0]), dot(f[1], f[1])],
    ]
}

/// Os pesos `wⱼ` que escrevem `F = Σⱼ xⱼ wⱼᵀ` — a ponte entre o vértice e o
/// gradiente de deformação, e a razão de o gradiente por vértice ser tão curto.
fn weights(m: &[[f64; 2]; 2]) -> [[f64; 2]; 3] {
    let (w1, w2) = ([m[0][0], m[0][1]], [m[1][0], m[1][1]]);
    [[-w1[0] - w2[0], -w1[1] - w2[1]], w1, w2]
}

/// `F` (3×2), coluna a coluna.
pub(crate) fn deform(x: &[V3], t: [u32; 3], m: &[[f64; 2]; 2]) -> [V3; 2] {
    let (x0, x1, x2) = (x[t[0] as usize], x[t[1] as usize], x[t[2] as usize]);
    let (e1, e2) = (sub(x1, x0), sub(x2, x0));
    let col = |c: usize| {
        [
            e1[0] * m[0][c] + e2[0] * m[1][c],
            e1[1] * m[0][c] + e2[1] * m[1][c],
            e1[2] * m[0][c] + e2[2] * m[1][c],
        ]
    };
    [col(0), col(1)]
}

/// `G = ½(FᵀF − G_repouso)` e `S = 2μG + λ·tr(G)·I`, as duas matrizes `2×2`.
pub(crate) fn strain(
    f: &[V3; 2],
    rest: &[[f64; 2]; 2],
    mu: f64,
    lambda: f64,
) -> ([[f64; 2]; 2], [[f64; 2]; 2]) {
    let m = metric(f);
    let g = [
        [0.5 * (m[0][0] - rest[0][0]), 0.5 * (m[0][1] - rest[0][1])],
        [0.5 * (m[1][0] - rest[1][0]), 0.5 * (m[1][1] - rest[1][1])],
    ];
    let tr = g[0][0] + g[1][1];
    let s = [
        [2.0 * mu * g[0][0] + lambda * tr, 2.0 * mu * g[0][1]],
        [2.0 * mu * g[1][0], 2.0 * mu * g[1][1] + lambda * tr],
    ];
    (g, s)
}

/// A energia do triângulo.
pub(crate) fn energy(x: &[V3], t: [u32; 3], r: &TriRest, mu: f64, lambda: f64) -> f64 {
    if r.area <= 0.0 {
        return 0.0;
    }
    let f = deform(x, t, &r.dm_inv);
    let (g, _) = strain(&f, &r.metric, mu, lambda);
    let tr = g[0][0] + g[1][1];
    let frob = g[0][0] * g[0][0] + g[0][1] * g[0][1] + g[1][0] * g[1][0] + g[1][1] * g[1][1];
    r.area * (mu * frob + 0.5 * lambda * tr * tr)
}

/// **O GRADIENTE e a HESSIANA em UM vértice do triângulo** — as duas coisas que o
/// bloco `3×3` do VBD precisa, na mesma passada.
///
/// Com `δF = δxᵢ·wᵢᵀ`, a derivação fecha em três termos:
///
/// ```text
/// ∂E/∂xᵢ  = A · F·(S·wᵢ)
/// ∂²E/∂xᵢ² = A · [ (wᵢᵀ S wᵢ)·I₃  +  μ·(wᵢ·wᵢ)·F Fᵀ  +  (μ+λ)·g gᵀ ],  g = F wᵢ
/// ```
///
/// ⚠️ **Os dois saem juntos de propósito.** Separá-los faria o triângulo ser
/// percorrido duas vezes por vértice por iteração — e a lei desta casa é que uma
/// pergunta feita duas vezes é o sítio onde as duas respostas divergem.
pub(crate) fn accumulate(
    x: &[V3],
    t: [u32; 3],
    r: &TriRest,
    mu: f64,
    lambda: f64,
    slot: usize,
) -> (V3, [[f64; 3]; 3]) {
    if r.area <= 0.0 {
        return ([0.0; 3], [[0.0; 3]; 3]);
    }
    let area = r.area;
    let f = deform(x, t, &r.dm_inv);
    let (_, s) = strain(&f, &r.metric, mu, lambda);
    let w = weights(&r.dm_inv)[slot];

    let sw = [
        s[0][0] * w[0] + s[0][1] * w[1],
        s[1][0] * w[0] + s[1][1] * w[1],
    ];
    let grad = [
        area * (f[0][0] * sw[0] + f[1][0] * sw[1]),
        area * (f[0][1] * sw[0] + f[1][1] * sw[1]),
        area * (f[0][2] * sw[0] + f[1][2] * sw[1]),
    ];

    // ⛔⛔⛔ **A PROJEÇÃO QUE FALTAVA, E ELA É SÓ DA MÉTRICA.**
    //
    // `wᵀSw` carrega o 2.º Piola-Kirchhoff, que é **negativo sob compressão** — e
    // este é o único termo diagonal cheio do bloco. Sob compressão ele engole os
    // dois termos PSD e o que resta a segurar o bloco positivo é a inércia
    // `mᵢ/h²`, que **desaparece ao refinar a malha** enquanto o elástico é `O(μ)`.
    //
    // Medido em 2026-09-05, sem esta linha: a `35 %` de compressão a energia de
    // um retalho vai de `6,4e1` para **`5,26e8`** num sub-passo e um vértice anda
    // `20×` a peça. E é um **POLO**, não uma escala grande — refinar a amostragem
    // da compressão `10×` faz o pico ir `0,9 → 188 → 2,0e3 → 4,9e5` **sem
    // convergir**, que é a assinatura de `det H` a cruzar o zero.
    //
    // ⚠️ **O GRADIENTE FICA EXATO** — quem é projetado é só a Hessiana, que aqui
    // é a métrica do passo de Newton. É a **mesma troca que a dobra já declara**
    // e já tem gate (`a_hessiana_da_dobra_e_semi_definida_positiva`); com ela o
    // bloco vira soma de três PSD (`μ·ww·FᵀF`, `(μ+λ)ggᵀ` e `wsw·I` com
    // `wsw ≥ 0`), e o passo passa a descer a energia sempre.
    //
    // ⚠️⚠️ **E é isto que corrige o Rayleigh de graça:** o `(1+kd)` do
    // [`vbd`](crate::vbd) multiplica **só** a Hessiana elástica, logo ele
    // amplificava `13×` exatamente o termo que estava negativo.
    //
    // ⭐ **O buraco de simetria nomeava a cura:** a dobra tinha gate de PSD e a
    // membrana **não tinha o gate irmão** — e o gate que existia prova que ela
    // está *certa* contra diferença finita. *Uma Hessiana indefinida CORRETA é
    // precisamente o defeito: ninguém perguntava se ela era **utilizável**.*
    let wsw = (w[0] * sw[0] + w[1] * sw[1]).max(0.0);
    let ww = w[0] * w[0] + w[1] * w[1];
    let g = [
        f[0][0] * w[0] + f[1][0] * w[1],
        f[0][1] * w[0] + f[1][1] * w[1],
        f[0][2] * w[0] + f[1][2] * w[1],
    ];
    let mut h = [[0.0f64; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            let fft = f[0][r] * f[0][c] + f[1][r] * f[1][c];
            h[r][c] = area
                * (mu * ww * fft + (mu + lambda) * g[r] * g[c] + if r == c { wsw } else { 0.0 });
        }
    }
    (grad, h)
}

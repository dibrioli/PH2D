//! **O CAMPO DE ORIENTAÇÃO 4-RoSy** — o primeiro dos dois campos do ADR-0160.
//!
//! Um campo 4-RoSy guarda **uma direção tangente por vértice** e a interpreta
//! como **quatro**: `o`, `n×o`, `−o`, `−(n×o)`. É essa simetria que faz dele a
//! representação certa para uma grade de quads — uma grade não tem *"a"*
//! direção, ela tem duas famílias perpendiculares, e trocar uma pela outra (ou
//! virar qualquer uma) descreve **a mesma grade**.
//!
//! # A lei, e por que ela é local
//!
//! A energia que se minimiza é a soma, sobre as arestas, do desalinhamento entre
//! os campos das duas pontas — medido **entre os representantes mais próximos**,
//! nunca entre os vetores guardados. Minimizá-la é o que faz a grade correr *ao
//! longo* da forma.
//!
//! ⚠️ **O Instant Meshes não resolve um sistema global; ele SUAVIZA.** Cada
//! vértice absorve os vizinhos num acumulador, e o passe roda até convergir. É
//! isso que o faz escalar para milhões de vértices, e é a diferença de família
//! contra a parametrização por inteiros (ADR-0160 §2a), que compra qualidade com
//! um solver de programação inteira mista.
//!
//! ⚠️ **`extrinsic`, e é a escolha da referência.** A comparação entre dois
//! campos ignora o transporte paralelo entre os planos tangentes — ela compara os
//! vetores 3D direto. A versão intrínseca (rodar um dos dois para o plano do
//! outro) é mais correta em malhas muito grosseiras e mais cara em todas; o
//! `instant-meshes` shipa a extrínseca por default, e o `optimizer.cpp` dele
//! roteia as duas pelo mesmo nome com um sufixo.
//!
//! # O que esta wave (Q1) NÃO tem, de propósito
//!
//! ⚠️ **A HIERARQUIA MULTIRRESOLUÇÃO.** O Instant Meshes suaviza numa pilha de
//! malhas simplificadas e propaga o resultado para baixo — sem ela a informação
//! viaja **uma aresta por iteração**, então uma malha de `n` vértices de diâmetro
//! `d` precisa de O(`d`) iterações para uma decisão de um lado alcançar o outro.
//! ⚠️ **Ela é um ACELERADOR DE CONVERGÊNCIA, não uma lei diferente** — o ponto
//! fixo é o mesmo. É por isso que ela é Q1.5/Q2 e não um pré-requisito: o gate
//! `the_energy_never_climbs` mede a convergência, e é ele que dirá quando o
//! número de iterações deixar de ser pagável.

use ph2d_mesh::Mesh;

/// **O campo, uma direção tangente por vértice.**
///
/// ⚠️ **Um `Vec` e não um componente da malha.** O campo é um resultado
/// INTERMEDIÁRIO do remesh — ele morre quando a malha nova nasce. Pendurá-lo na
/// [`Mesh`] o faria viajar na persistência, no undo e na doação, três sítios que
/// não têm nada a perguntar-lhe.
#[derive(Clone, Debug, PartialEq)]
pub struct OrientationField {
    dirs: Vec<[f32; 3]>,
}

impl OrientationField {
    /// A direção representante do vértice `v`. As outras três são `n×o`, `−o` e
    /// `−(n×o)` — elas **não** são guardadas, porque guardá-las seria guardar
    /// quatro vezes o mesmo fato.
    #[must_use]
    pub fn dir(&self, v: usize) -> [f32; 3] {
        self.dirs[v]
    }

    /// Quantos vértices o campo cobre.
    #[must_use]
    pub fn len(&self) -> usize {
        self.dirs.len()
    }

    /// Um campo sem vértices — o caso de uma malha vazia.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dirs.is_empty()
    }

    /// **A ENERGIA do campo** — a soma, sobre as arestas dirigidas, de
    /// `1 − |cos θ|` entre os representantes mais próximos das duas pontas.
    ///
    /// ⚠️ **É a régua da convergência, e ela é o oráculo do gate.** Zero quer
    /// dizer *toda aresta vê os dois lados alinhados*; um passe de suavização que
    /// a fizesse SUBIR estaria a divergir, e é isso que
    /// `the_energy_never_climbs` mede — não *"o campo parece bom"*.
    ///
    /// ⚠️ Sobre arestas DIRIGIDAS (cada par contado duas vezes) porque é assim
    /// que a adjacência as enumera; a constante 2 é irrelevante para uma régua de
    /// monotonia, e derivá-la seria inventar uma normalização que ninguém lê.
    #[must_use]
    pub fn energy(&self, mesh: &Mesh) -> f64 {
        let n = mesh.normals();
        let adj = mesh.adjacency();
        let mut sum = 0.0f64;
        for v in 0..self.dirs.len() {
            for &w in adj.vert_verts.neighbours(v) {
                let (a, b) = compat_orientation_extrinsic_4(
                    self.dirs[v],
                    n[v],
                    self.dirs[w as usize],
                    n[w as usize],
                );
                sum += f64::from(1.0 - dot(a, b).abs());
            }
        }
        sum
    }
}

/// **OS DOIS REPRESENTANTES MAIS PRÓXIMOS de dois campos 4-RoSy.**
///
/// Devolve `(a, b)` com `a` tirado das quatro direções de `q0` e `b` das quatro
/// de `q1`, escolhidos para maximizar `a·b`. É a peça de que toda a lei depende:
/// somar `q0 + q1` direto seria somar duas descrições da **mesma** grade que por
/// acaso escolheram representantes girados de 90°, e o resultado se cancelaria.
///
/// Porte de `compat_orientation_extrinsic_4` (`instant-meshes`,
/// `src/field.cpp`), BSD-3-Clause.
///
/// ⚠️ **Só DUAS candidatas por lado, não quatro.** `−o` e `−(n×o)` não precisam
/// ser enumeradas: o critério é `|a·b|` e o sinal é resolvido no fim, de uma vez.
/// Enumerar as quatro daria dezasseis comparações para o mesmo resultado.
///
/// ⚠️ **`n×q` e não um segundo campo:** a segunda família é DERIVADA da primeira
/// e da normal. Guardá-la seria um fato em dois sítios, e o dia em que a normal
/// mudasse (um dab, um smooth) eles discordariam.
#[must_use]
pub fn compat_orientation_extrinsic_4(
    q0: [f32; 3],
    n0: [f32; 3],
    q1: [f32; 3],
    n1: [f32; 3],
) -> ([f32; 3], [f32; 3]) {
    let a = [q0, cross(n0, q0)];
    let b = [q1, cross(n1, q1)];

    let (mut best, mut bi, mut bj) = (f32::NEG_INFINITY, 0usize, 0usize);
    for (i, ai) in a.iter().enumerate() {
        for (j, bj_v) in b.iter().enumerate() {
            let score = dot(*ai, *bj_v).abs();
            // ⚠️ `>` e não `>=`: com o empate a primeira vence, e é isso que
            // torna a função DETERMINÍSTICA sobre um campo simétrico (uma esfera
            // perfeita empata o tempo todo). Um `>=` daria a última, e a última
            // depende da ordem em que o laço corre.
            if score > best {
                best = score;
                bi = i;
                bj = j;
            }
        }
    }

    let chosen = (a[bi], b[bj]);
    let dp = dot(chosen.0, chosen.1);
    // O sinal, resolvido uma vez: `b` é virado se os dois apontam para lados
    // opostos. É o que substitui as quatro candidatas por lado.
    let s = if dp < 0.0 { -1.0f32 } else { 1.0f32 };
    (chosen.0, scale(chosen.1, s))
}

/// **RESOLVE o campo de orientação** — `iterations` varreduras de suavização.
///
/// ⚠️ **Gauss-Seidel e não Jacobi**: cada vértice lê os vizinhos **já
/// atualizados** nesta varredura. É o que a referência faz, converge em cerca de
/// metade das varreduras, e — porque a ordem de visita é fixa — continua
/// **determinístico** (HR-5). ⚠️ É também o que impede o paralelismo ingênuo:
/// dois vértices vizinhos em threads diferentes leriam estados diferentes um do
/// outro. Paralelizar isto é ADR próprio (ADR-0160 §6), e a forma honesta é a
/// coloração de grafo que a referência usa.
///
/// ⚠️ **A semente é DERIVADA da normal, nunca aleatória.** Uma semente aleatória
/// daria um campo diferente a cada corrida, e o remesh deixaria de ser
/// reproduzível — a malha do artista mudaria ao reabrir o projeto.
///
/// # Quantas varreduras — MEDIDO, não escolhido
///
/// Toro 48×24 (1 152 vértices, 4 608 arestas dirigidas), pelo gate
/// `the_energy_never_climbs`:
///
/// | varreduras | 0 | 1 | 2 | 4 | 8 | 16 | 32 | **64** | 128 | 256 | 512 |
/// |---|---|---|---|---|---|---|---|---|---|---|---|
/// | energia | 46,99 | 39,69 | 37,82 | 35,71 | 32,73 | 29,60 | 27,80 | **27,50** | 27,4959 | 27,4956 | 27,4956 |
///
/// ⚠️ **A convergência é em ~64 varreduras NESTA malha, e o número NÃO viaja:**
/// sem hierarquia a informação anda **uma aresta por varredura**, então o custo
/// escala com o **diâmetro** do grafo, não com a contagem de vértices. É
/// exactamente esta tabela que a hierarquia multirresolução da Q2 tem de
/// melhorar — e é contra ela que a melhoria será medida.
///
/// ⚠️ **O piso 27,4956 não é zero, e isso está certo:** um toro não admite um
/// campo cruzado sem defeito em toda parte (a característica de Euler manda), e a
/// energia residual é a soma do que sobra nas singularidades. Um campo que
/// chegasse a zero aqui estaria a mentir sobre a topologia.
#[must_use]
pub fn solve_orientation(mesh: &Mesh, iterations: usize) -> OrientationField {
    let normals = mesh.normals();
    let adj = crate::im_weights::cotangent_adjacency(mesh);
    let mut dirs = seed_dirs(normals);
    smooth_on(&mut dirs, normals, &adj, iterations);
    OrientationField { dirs }
}

/// **A SEMENTE de um conjunto de normais** — a porta única, para que a
/// hierarquia semeie exatamente como o caminho direto.
#[must_use]
pub fn seed_dirs(normals: &[[f32; 3]]) -> Vec<[f32; 3]> {
    normals.iter().map(|n| seed_tangent(*n)).collect()
}

/// **O CAMPO, a partir de direções já resolvidas** — a porta por onde a
/// hierarquia devolve o resultado do nível 0.
#[must_use]
pub fn field_from(dirs: Vec<[f32; 3]>) -> OrientationField {
    OrientationField { dirs }
}

/// **A projeção de uma direção no plano tangente de `n`**, re-normalizada — o que
/// a PROLONGAÇÃO faz ao dar ao filho a direção do pai.
#[must_use]
pub fn project_tangent(d: [f32; 3], n: [f32; 3]) -> [f32; 3] {
    normalize_or(reject(d, n), seed_tangent(n))
}

/// **O NÚCLEO da suavização, sobre slices** — o mesmo laço, sem a [`Mesh`].
///
/// ⚠️ **Existe para a HIERARQUIA poder correr a MESMA lei nos níveis grossos**,
/// que não são malhas: eles são posições, normais e uma vizinhança induzida. Uma
/// segunda cópia do laço lá seria a segunda resposta a *"como um campo 4-RoSy
/// suaviza"*, e as duas divergiriam no dia em que uma delas ganhasse um peso.
pub fn smooth_on(
    dirs: &mut [[f32; 3]],
    normals: &[[f32; 3]],
    adjacency: &[Vec<crate::im_weights::Link>],
    iterations: usize,
) {
    let count = dirs.len();
    for _ in 0..iterations {
        for v in 0..count {
            let nv = normals[v];
            // ⚠️ **`weight_sum` começa em ZERO, e o próprio valor entra só como
            // MOLDURA.** A primeira parcela é `a·0 + b·w`, ou seja o vizinho
            // rotacionado — o valor antigo serve para escolher o representante
            // 4-RoSy e depois sai. A minha versão começava em 1 e mantinha o
            // valor antigo com peso próprio, o que trava o campo: cada vértice
            // resistia à vizinhança com um voto que não é de ninguém.
            let mut sum = dirs[v];
            let mut weight_sum = 0.0f32;
            for link in &adjacency[v] {
                let w = link.weight;
                if w == 0.0 {
                    continue;
                }
                let j = link.id as usize;
                let (a, b) = compat_orientation_extrinsic_4(sum, nv, dirs[j], normals[j]);
                sum = add(scale(a, weight_sum), scale(b, w));
                sum = reject(sum, nv);
                weight_sum += w;
                sum = normalize_or(sum, dirs[v]);
            }
            if weight_sum > 0.0 {
                dirs[v] = sum;
            }
        }
    }
}

/// **A SEMENTE: uma tangente qualquer, mas SEMPRE a mesma.**
///
/// Base ortonormal sem ramificação de *Duff et al., "Building an Orthonormal
/// Basis, Revisited" (JCGT 2017)* — a mesma construção que o resto da indústria
/// usa, escolhida aqui por três propriedades: ela é **exata** (não normaliza
/// nada), **contínua** exceto num polo, e **determinística**.
///
/// ⚠️ **O `copysign` é o que a torna estável perto de `n.z = −1`.** A versão
/// ingênua (`cross(n, eixo_menos_alinhado)`) perde precisão exactamente onde a
/// normal se aproxima do eixo escolhido, e o erro aparece como um campo que gira
/// sozinho numa faixa fina do modelo.
fn seed_tangent(n: [f32; 3]) -> [f32; 3] {
    let sign = 1.0f32.copysign(n[2]);
    let a = -1.0 / (sign + n[2]);
    let b = n[0] * n[1] * a;
    let t = [sign.mul_add(n[0] * n[0] * a, 1.0), sign * b, -sign * n[0]];
    normalize_or(t, [1.0, 0.0, 0.0])
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: [f32; 3], k: f32) -> [f32; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// `a` menos a componente ao longo de `n` — a projeção no plano tangente.
fn reject(a: [f32; 3], n: [f32; 3]) -> [f32; 3] {
    let d = dot(a, n);
    [
        d.mul_add(-n[0], a[0]),
        d.mul_add(-n[1], a[1]),
        d.mul_add(-n[2], a[2]),
    ]
}

/// Normaliza, ou devolve `fallback` se o vetor colapsou.
///
/// ⚠️ **O colapso é REAL e não defensivo:** duas famílias perpendiculares em
/// oposição exata somam zero, e sobre uma esfera regular isso acontece. Devolver
/// um `NaN` ali envenenaria o campo inteiro na varredura seguinte, em silêncio.
fn normalize_or(a: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    let len = dot(a, a).sqrt();
    if len > 1.0e-6 {
        scale(a, 1.0 / len)
    } else {
        fallback
    }
}

#[cfg(test)]
#[path = "orientation_tests.rs"]
mod tests;

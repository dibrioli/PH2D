//! **CAMPO CRUZADO 4-RoSy COM DECISÃO GLOBAL** (ADR-0162, F2).
//!
//! Clean-room a partir de **Bommes, Zimmer, Kobbelt, *Mixed-Integer
//! Quadrangulation*, SIGGRAPH 2009** (`ph2d-quadbench/docs/papers/miq-2009.pdf`)
//! e QuadWild 2021 §5. ⛔ Nenhuma linha traduzida de fonte GPL — ADR-0162.
//!
//! # Por que ele existe: a medição, não a opinião
//!
//! O campo que a `ph2d-quadflow` resolve é **local**: cada vértice absorve os
//! vizinhos e o passe roda até convergir. Ele nunca negocia globalmente, e o
//! preço está medido no corpus (`ph2d-quadbench`, 2026-08-20): **21 a 49 % dos
//! vértices da saída são irregulares**, contra **0,2 %** do oráculo. Uma grade
//! numa esfera admite **oito**.
//!
//! ⚠️ **E o F1 provou que não era a malha de entrada.** Uniformizar a entrada
//! (`ph2d-remesh-iso`) moveu a agulha alguns pontos, **para os dois lados**. Se a
//! doença fosse a entrada, curá-la teria derrubado o número. *É a classe do
//! algoritmo.*
//!
//! # A formulação, numa frase
//!
//! Cada **face** guarda um ângulo `θ_f` (a direção da cruz na moldura dela). Cada
//! **aresta dual** guarda um inteiro `p_e` — quantos quartos de volta a cruz dá
//! ao atravessar aquela aresta. A energia é
//!
//! ```text
//! E = Σ_e w_e · ( θ_f − θ_g + κ_e + (π/2)·p_e )²
//! ```
//!
//! onde `κ_e` é o desencontro entre as duas molduras, medido **através da aresta
//! partilhada**. ⭐ **Os `p_e` são a decisão global**: eles são inteiros, e é a
//! escolha deles — não a suavização — que decide **onde as singularidades ficam**.
//! É exatamente o que a família local não tem.
//!
//! # O que este crate NÃO faz, de propósito
//!
//! ⛔ Ele não extrai malha. O F2 entrega **o campo**; a decomposição em patches
//! (F3), a quantização Bi-MDF (F4) e a quadrangulação por patch (F5) são as fases
//! seguintes do [`PLAN.md`](../../../docs/3D/quad-remesh/PLAN.md). O que se pode
//! fazer hoje é **converter para por-vértice** ([`to_vertex_dirs`]) e alimentar o
//! extrator que já existe — que é como o F2 se mede antes de o F5 existir.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use ph2d_mesh::Mesh;

/// ⭐⭐ **PENTEAR uma região e MEDIR o que sobra** — ver [`comb`].
pub mod comb;
mod constrain;
mod continuation;
mod index;
mod solve;

pub use comb::{Holonomy, holonomy};
pub use constrain::{CONSTRAINT_AGREEMENT, ConstrainReport};
pub use continuation::{ALIGN_WEIGHT, Continuation, solve_miq_aligned, solve_miq_continued};
pub use index::{IndexReport, ring_totals, singularities, vertex_index, vertex_index_with_report};
pub use solve::{
    Rounding, SolveReport, cycle_count, energy, solve_alternating, solve_miq, solve_miq_with,
};

/// **UM QUARTO DE VOLTA** — o passo do campo 4-RoSy.
pub const QUARTER: f32 = core::f32::consts::FRAC_PI_2;

/// **O campo resolvido.**
///
/// ⚠️ **Os dois vetores juntos SÃO o campo, e nenhum deles sozinho é.** O `theta`
/// diz para onde a cruz aponta *na moldura de cada face*; os `period` dizem como
/// as molduras se costuram. Guardar só o primeiro perde as singularidades, que é
/// a informação inteira.
#[derive(Clone, Debug, PartialEq)]
pub struct CrossField {
    /// O ângulo da cruz na moldura de cada face.
    theta: Vec<f32>,
    /// O salto de período de cada aresta dual, na ordem de [`Dual::edges`].
    period: Vec<i32>,
}

impl CrossField {
    /// O ângulo da face `f`, na moldura dela.
    #[must_use]
    pub fn theta(&self, f: usize) -> f32 {
        self.theta[f]
    }

    /// O salto de período da aresta dual `e`.
    #[must_use]
    pub fn period(&self, e: usize) -> i32 {
        self.period[e]
    }

    /// Quantas faces o campo cobre.
    #[must_use]
    pub fn len(&self) -> usize {
        self.theta.len()
    }

    /// Um campo sem faces.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.theta.is_empty()
    }

    /// ⭐⭐⭐ **RECONSTRÓI UM CAMPO A PARTIR DE DIREÇÕES CRUAS** — uma por face, na
    /// ordem das faces. É a inversa da [`Self::direction`].
    ///
    /// # ⭐ Para que ela existe
    ///
    /// O oráculo GPL da bancada **grava o campo dele** (`*_rem.rosy`: uma direção por
    /// face). Ler a saída de um binário não é obra derivada — e sem esta porta o campo
    /// dele só se podia comparar por olho, porque toda régua deste crate pede um
    /// [`CrossField`] e não um vetor de direções. ⇒ *com ela, o campo dele passa pelas
    /// MESMAS funções que o nosso*, e a comparação deixa de depender de duas
    /// implementações concordarem.
    ///
    /// # ⚠️ Por que uma direção basta, apesar de `theta` não ser recuperável
    ///
    /// Uma cruz tem **quatro braços**: a direção grava só um representante, e o `theta`
    /// que sai daqui difere do original por um múltiplo de 90°. ⭐ **Isso não perde
    /// nada**, porque o múltiplo é absorvido pelo [`Self::period`] da aresta ao lado —
    /// e toda grandeza que interessa (índice, singularidades, energia) lê
    /// `κ + 90°·p` em volta de um ciclo, onde os dois se recompõem. *A informação que
    /// se perderia é a que nenhuma régua olha.*
    ///
    /// ⚠️ **`None` quando a contagem não bate com a do dual** — um campo de outra
    /// malha lido nesta produziria índices plausíveis e errados, que é a assinatura do
    /// defeito de 2026-08-21.
    #[must_use]
    pub fn from_directions(dual: &Dual, dirs: &[[f32; 3]]) -> Option<Self> {
        if dirs.len() != dual.frames().len() {
            return None;
        }
        let theta: Vec<f32> = dual
            .frames()
            .iter()
            .zip(dirs)
            .map(|(fr, d)| {
                let t = cross(fr.n, fr.e);
                dot(*d, t).atan2(dot(*d, fr.e))
            })
            .collect();
        // A MESMA lei do arredondamento do `solve_alternating`: o resíduo que o campo
        // minimiza é `θ_f − θ_g + κ + 90°·p`.
        #[allow(clippy::cast_possible_truncation)]
        let period: Vec<i32> = dual
            .edges()
            .iter()
            .map(|de| {
                let r = theta[de.f as usize] - theta[de.g as usize] + de.kappa;
                -((r / QUARTER).round() as i32)
            })
            .collect();
        Some(Self { theta, period })
    }

    /// **A DIREÇÃO 3D da cruz na face `f`** — o representante.
    #[must_use]
    pub fn direction(&self, dual: &Dual, f: usize) -> [f32; 3] {
        let fr = &dual.frames[f];
        let (s, c) = self.theta[f].sin_cos();
        let t = cross(fr.n, fr.e);
        [
            c.mul_add(fr.e[0], s * t[0]),
            c.mul_add(fr.e[1], s * t[1]),
            c.mul_add(fr.e[2], s * t[2]),
        ]
    }
}

/// A moldura de uma face: uma tangente de referência e a normal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    /// A tangente de referência — a primeira aresta da face, normalizada.
    pub e: [f32; 3],
    /// A normal da face.
    pub n: [f32; 3],
}

/// Uma aresta do grafo DUAL: as duas faces e o desencontro entre as molduras.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DualEdge {
    /// A face de um lado.
    pub f: u32,
    /// A face do outro.
    pub g: u32,
    /// `κ` — o desencontro entre as molduras, medido **através da aresta
    /// partilhada**. Ver [`Dual::build`].
    pub kappa: f32,
    /// O peso do termo, na energia.
    pub weight: f32,
}

/// **O GRAFO DUAL da malha** — as faces são os nós, as arestas partilhadas são
/// as ligações.
///
/// ⚠️ **Ele é construído UMA vez e vive fora do campo**, porque o solver o
/// percorre dezenas de vezes e reconstruí-lo por iteração seria pagar o grafo
/// para resolver o campo.
#[derive(Clone, Debug)]
pub struct Dual {
    frames: Vec<Frame>,
    edges: Vec<DualEdge>,
    /// ⭐⭐ **O ALINHAMENTO por face** — `(α, confiança)`, onde `α` é a direção
    /// principal de curvatura **medida na moldura da face** e a confiança é a
    /// anisotropia em `[0, 1]`.
    ///
    /// ⛔ **Sem este termo a energia é SÓ suavidade**, e o campo mais suave sobre
    /// uma esfera com duas orelhas é o campo de uma esfera lisa — *ele não tem como
    /// ver as orelhas*. Medido em 2026-08-22 com a régua
    /// [`ph2d_quadfill::follows_relief`] na fixtura com cristas: a nossa cadeia dava
    /// **25,7°** de desvio (pior que os **22,5°** de uma grade aleatória) contra
    /// **13,7°** do porte do Instant Meshes, que semeia o campo na superfície.
    /// *A obediência ao relevo é um TERMO, não uma afinação.*
    align: Vec<(f32, f32)>,
    /// ⭐⭐⭐ **O `θ` FIXO de cada face restringida** — ver [`Dual::constrain`].
    ///
    /// ⚠️ **Ele NÃO é um alinhamento mais forte: é a ausência de uma incógnita.** Uma
    /// face com valor aqui sai do sistema linear, e o valor dela viaja no lado
    /// constante das arestas duais que a tocam — a mesma lei da costura da obra A.
    constrained: Vec<Option<f32>>,
    /// Por face, os índices das arestas duais que a tocam.
    incident: Vec<Vec<u32>>,
}

impl Dual {
    /// As molduras, uma por face.
    #[must_use]
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    /// As arestas duais.
    #[must_use]
    pub fn edges(&self) -> &[DualEdge] {
        &self.edges
    }

    /// As arestas duais que tocam a face `f`.
    #[must_use]
    pub fn incident(&self, f: usize) -> &[u32] {
        &self.incident[f]
    }

    /// **CONSTRÓI o grafo dual e mede os `κ`.**
    ///
    /// ⚠️ **A malha é TRIANGULADA na porta.** Uma face de quatro lados não tem
    /// uma moldura única (os dois triângulos dela podem não ser coplanares), e o
    /// `κ` deixaria de ser o desencontro de duas molduras para ser a média de
    /// duas coisas diferentes.
    ///
    /// ⚠️ **`κ` é medido ATRAVÉS DA ARESTA PARTILHADA, e é a única forma que
    /// funciona.** A alternativa óbvia — comparar as duas tangentes de referência
    /// direto — mede o ângulo entre duas arestas quaisquer de dois triângulos
    /// quaisquer, e não tem nada a ver com o transporte paralelo. Aqui a direção
    /// da aresta comum é escrita nas DUAS molduras, e `κ` é a diferença dos dois
    /// ângulos: é exatamente quanto se tem de girar a moldura de `g` para ela
    /// concordar com a de `f` ao cruzar aquela aresta.
    #[must_use]
    pub fn build(mesh: &Mesh) -> Self {
        let p = mesh.positions();
        let faces = mesh.faces();
        let normals = mesh.face_normals();

        let frames: Vec<Frame> = faces
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let v = f.verts();
                let e = normalize_or(sub(p[v[1] as usize], p[v[0] as usize]), [1.0, 0.0, 0.0]);
                let n = normals[i];
                // A referência tem de ser TANGENTE: uma aresta de um triângulo
                // quase degenerado pode não o ser, depois do normalize.
                let d = dot(e, n);
                Frame {
                    e: normalize_or(
                        [
                            d.mul_add(-n[0], e[0]),
                            d.mul_add(-n[1], e[1]),
                            d.mul_add(-n[2], e[2]),
                        ],
                        tangent_of(n),
                    ),
                    n,
                }
            })
            .collect();

        // Aresta da malha -> as faces que a usam. ⚠️ `BTreeMap`, nunca `HashMap`:
        // a ordem das arestas duais entra na numeração dos `p_e`, e a numeração
        // tem de ser byte-reprodutível (HR-5).
        let mut owner: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
        for (fi, f) in faces.iter().enumerate() {
            let v = f.verts();
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                let key = if a < b { (a, b) } else { (b, a) };
                owner.entry(key).or_default().push(fi as u32);
            }
        }

        let mut edges: Vec<DualEdge> = Vec::new();
        let mut incident: Vec<Vec<u32>> = vec![Vec::new(); faces.len()];
        for ((a, b), who) in &owner {
            if who.len() != 2 {
                // Uma aresta de borda não liga duas faces — não há transporte a
                // medir, e ela simplesmente não entra no grafo dual.
                continue;
            }
            let (fi, gi) = (who[0], who[1]);
            let u = normalize_or(sub(p[*b as usize], p[*a as usize]), [1.0, 0.0, 0.0]);
            let alpha_f = angle_in(&frames[fi as usize], u);
            let alpha_g = angle_in(&frames[gi as usize], u);
            // ⚠️ `κ = α_g − α_f`: com ele, `θ_f − θ_g + κ` é o desencontro entre
            // as duas cruzes medido no referencial da aresta comum, que é o que
            // a energia quer minimizar (a menos de quartos de volta).
            let kappa = wrap(alpha_g - alpha_f);
            let e = edges.len() as u32;
            incident[fi as usize].push(e);
            incident[gi as usize].push(e);
            edges.push(DualEdge {
                f: fi,
                g: gi,
                kappa,
                // ⚠️ **Peso 1, e é uma DECISÃO por medir.** O MIQ pesa pelo
                // comprimento da aresta dual; o efeito no nosso corpus ainda não
                // foi medido, e um peso escolhido sem medição é um palpite com
                // aparência de teoria. ⛔ Não o mude sem a tabela ao lado.
                weight: 1.0,
            });
        }

        // ⭐⭐ **O ALINHAMENTO** — a direção principal de curvatura de cada face,
        // projectada na moldura dela e reduzida ao 4-RoSy.
        //
        // ⚠️ **`atan2` na moldura, e o resultado vive em `[0, π/2)`:** a cruz tem
        // quatro braços, então `α` e `α + π/2` dizem a mesma coisa. Guardar o
        // ângulo cru faria o solver puxar `θ` para um braço arbitrário dos quatro.
        let align: Vec<(f32, f32)> = ph2d_mesh::principal_dirs(mesh)
            .iter()
            .zip(&frames)
            .map(|(pd, fr)| {
                if pd.anisotropy <= 0.0 {
                    return (0.0, 0.0);
                }
                let b = cross(fr.n, fr.e);
                let (x, y) = (dot(pd.dir, fr.e), dot(pd.dir, b));
                let a = y.atan2(x).rem_euclid(QUARTER);
                (a, pd.anisotropy)
            })
            .collect();

        Self {
            constrained: vec![None; frames.len()],
            frames,
            edges,
            incident,
            align,
        }
    }

    /// **O alinhamento da face `f`** — `(α na moldura, confiança)`. Ver
    /// [`Dual::align`].
    #[must_use]
    pub fn align(&self, f: usize) -> (f32, f32) {
        self.align.get(f).copied().unwrap_or((0.0, 0.0))
    }
}

/// **CONVERTE o campo por-FACE para por-VÉRTICE** — a ponte para o extrator que
/// já existe.
///
/// ⚠️ **É uma PERDA, e ela está declarada.** O campo do MIQ vive nas faces e
/// carrega os saltos de período; um vetor por vértice não tem onde os guardar.
/// Esta porta existe só para medir o F2 **antes** de o F5 existir — a
/// quadrangulação por patch consome o campo de face direto. *Uma conversão que
/// perde informação e não diz que perde é como um número fabricado nasce.*
///
/// A média é feita reduzindo cada face ao representante mais próximo do
/// acumulador (a simetria de 4 dobras), que é a única forma de somar duas
/// descrições da mesma cruz sem elas se cancelarem.
#[must_use]
pub fn to_vertex_dirs(mesh: &Mesh, dual: &Dual, field: &CrossField) -> Vec<[f32; 3]> {
    let normals = mesh.normals();
    let adj = mesh.adjacency();
    (0..mesh.vert_count())
        .map(|v| {
            let nv = normals[v];
            let mut acc: Option<[f32; 3]> = None;
            for &f in adj.vert_faces.neighbours(v) {
                let d = field.direction(dual, f as usize);
                // Projetar no plano tangente do VÉRTICE antes de somar: a
                // direção é tangente à face, e a face tem outro plano.
                let t = dot(d, nv);
                let d = normalize_or(
                    [
                        t.mul_add(-nv[0], d[0]),
                        t.mul_add(-nv[1], d[1]),
                        t.mul_add(-nv[2], d[2]),
                    ],
                    tangent_of(nv),
                );
                acc = Some(match acc {
                    None => d,
                    Some(a) => {
                        let b = nearest_representative(a, d, nv);
                        normalize_or([a[0] + b[0], a[1] + b[1], a[2] + b[2]], a)
                    }
                });
            }
            acc.unwrap_or_else(|| tangent_of(nv))
        })
        .collect()
}

/// O representante de `d` (entre as quatro) mais alinhado com `a`.
fn nearest_representative(a: [f32; 3], d: [f32; 3], n: [f32; 3]) -> [f32; 3] {
    let t = cross(n, d);
    let (mut best, mut out) = (f32::NEG_INFINITY, d);
    for cand in [d, t, [-d[0], -d[1], -d[2]], [-t[0], -t[1], -t[2]]] {
        let s = dot(a, cand);
        if s > best {
            best = s;
            out = cand;
        }
    }
    out
}

/// O ângulo de um vetor tangente `u` na moldura `fr`.
pub(crate) fn angle_in(fr: &Frame, u: [f32; 3]) -> f32 {
    let t = cross(fr.n, fr.e);
    dot(u, t).atan2(dot(u, fr.e))
}

/// Traz um ângulo para `(−π, π]`.
pub(crate) fn wrap(mut a: f32) -> f32 {
    const TAU: f32 = core::f32::consts::TAU;
    while a > core::f32::consts::PI {
        a -= TAU;
    }
    while a <= -core::f32::consts::PI {
        a += TAU;
    }
    a
}

pub(crate) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

pub(crate) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

pub(crate) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn normalize_or(a: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    let len = dot(a, a).sqrt();
    if len > 1.0e-20 {
        [a[0] / len, a[1] / len, a[2] / len]
    } else {
        fallback
    }
}

/// Uma tangente qualquer de `n`, mas sempre a mesma (Duff et al., JCGT 2017).
pub(crate) fn tangent_of(n: [f32; 3]) -> [f32; 3] {
    let sign = 1.0f32.copysign(n[2]);
    let a = -1.0 / (sign + n[2]);
    let b = n[0] * n[1] * a;
    normalize_or(
        [sign.mul_add(n[0] * n[0] * a, 1.0), sign * b, -sign * n[0]],
        [1.0, 0.0, 0.0],
    )
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

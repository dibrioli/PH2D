//! **A HIERARQUIA MULTIRRESOLUÇÃO** — o que faz os campos terem coerência de
//! LONGO ALCANCE (ADR-0160, Q3.5).
//!
//! Sem ela a suavização só propaga **uma aresta por varredura**: uma decisão num
//! lado do modelo precisa de tantas varreduras quanto o diâmetro do grafo para
//! alcançar o outro, e — para o campo de POSIÇÃO — nem isso resolve, porque a
//! malha fina é um ponto fixo da média (medido na Q2: o resíduo entre vizinhos
//! fica em **0,205 célula**, imóvel entre 32 e 2 048 varreduras).
//!
//! ⚠️ **E é por isso que ela deixou de ser opcional.** A Q3 mediu a consequência:
//! sem platôs no campo de posição não há retícula partilhada a que a extração
//! agarre, e ela teve de crescer células por semente — **60,9 %** de quads. A
//! nota do ADR que chamava a hierarquia de *"acelerador de convergência"* estava
//! certa para o campo de orientação e **vacua** para o de posição.
//!
//! # Como
//!
//! 1. **COARSENING** — emparelhar vértices vizinhos, cada par vira um vértice do
//!    nível de cima. Repetir até sobrar um punhado.
//! 2. Resolver o campo **no nível mais GROSSO**, onde ele tem poucos vértices e a
//!    célula da retícula é grande em relação ao espaçamento — é ali que o
//!    arredondamento **morde** e os platôs nascem.
//! 3. **PROLONGAR** para baixo: cada vértice fino herda o campo do pai, e o nível
//!    recebe algumas varreduras para se acomodar.
//!
//! ⚠️ **O emparelhamento é GULOSO e por ordem de índice** — não é a melhor
//! escolha possível, é a **determinística**. Um casamento ótimo (por peso de
//! aresta) seria outra heurística a justificar e medir, e a hierarquia é um
//! andaime: o que ela precisa é de reduzir o grafo pela metade sem preferências.

use ph2d_mesh::Mesh;

use crate::im_weights::{Link, cotangent_adjacency, dual_vertex_areas};

/// Abaixo disto não vale a pena outro nível: o grafo já cabe todo numa
/// vizinhança, e a suavização alcança tudo em poucas varreduras.
pub const COARSEST: usize = 24;

/// Teto de níveis — guarda de RECURSO contra um emparelhamento que pare de
/// reduzir (uma malha com componentes de um vértice só).
const MAX_LEVELS: usize = 24;

/// Um nível: os vértices, a vizinhança PONDERADA, a área dual, e para onde cada
/// um sobe.
pub struct Level {
    /// Posição de cada vértice deste nível.
    pub positions: Vec<[f32; 3]>,
    /// Normal de cada vértice deste nível.
    pub normals: Vec<[f32; 3]>,
    /// **A ÁREA DUAL** de cada vértice — a massa que ele representa.
    ///
    /// ⚠️ **É ela que torna o emparelhamento e as médias corretos.** Sem área,
    /// um vértice de um polo (que representa uma fatia minúscula) puxa a média
    /// tanto quanto um vértice de uma barriga lisa, e o nível grosso deixa de
    /// ser uma amostra da superfície.
    pub areas: Vec<f32>,
    /// Vizinhos de cada vértice **com o peso do Laplaciano**, ordenados.
    pub adjacency: Vec<Vec<Link>>,
    /// Vértice deste nível → vértice do nível ACIMA (mais grosso). Vazio no topo.
    pub parent: Vec<u32>,
}

impl Level {
    /// Quantos vértices este nível tem.
    #[must_use]
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Um nível sem vértices.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// **A pilha de níveis**, do fino (índice 0) ao grosso.
pub struct Hierarchy {
    levels: Vec<Level>,
}

impl Hierarchy {
    /// Constrói a pilha a partir da malha.
    #[must_use]
    pub fn build(mesh: &Mesh) -> Self {
        Self::build_to(mesh, COARSEST)
    }

    /// Constrói parando em `coarsest` vértices — a porta que a sonda varre.
    #[must_use]
    pub fn build_to(mesh: &Mesh, coarsest: usize) -> Self {
        let mut levels = vec![Level {
            positions: mesh.positions().to_vec(),
            normals: mesh.normals().to_vec(),
            areas: dual_vertex_areas(mesh),
            adjacency: cotangent_adjacency(mesh),
            parent: Vec::new(),
        }];

        while levels.len() < MAX_LEVELS {
            let fine = levels.last().expect("a pilha nunca esta' vazia");
            if fine.len() <= coarsest {
                break;
            }
            let Some((coarse, parent)) = coarsen(fine) else {
                break;
            };
            // Um nível que não reduz nada é um laço infinito à espera.
            if coarse.len() >= fine.len() {
                break;
            }
            levels.last_mut().expect("existe").parent = parent;
            levels.push(coarse);
        }

        Self { levels }
    }

    /// Quantos níveis a pilha tem.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.levels.len()
    }

    /// O nível `i` — `0` é o mais fino.
    #[must_use]
    pub fn level(&self, i: usize) -> &Level {
        &self.levels[i]
    }

    /// Do mais GROSSO ao mais fino — a ordem em que os campos são resolvidos.
    pub fn coarse_to_fine(&self) -> impl Iterator<Item = usize> {
        (0..self.levels.len()).rev()
    }
}

/// **UM PASSO DE COARSENING** — porte **FIEL** de `downsample_graph`
/// (`instant-meshes`, `src/hierarchy.cpp`), BSD-3-Clause.
///
/// ⚠️ **A minha versão emparelhava pelo PRIMEIRO vizinho livre, por ordem de
/// índice.** Isso constrói uma hierarquia, mas não *a* hierarquia: os pares
/// saem arbitrários, o nível grosso deixa de descrever a forma, e o campo
/// resolvido lá em cima chega ao nível fino com singularidades a mais. Medido
/// (2026-08-19, com a extração já fiel): **228 nós irregulares** numa esfera que
/// admite **8**.
///
/// A referência ordena TODAS as ligações por `(n_i · n_j) · razão_de_área` e
/// emparelha gulosamente por essa ordem, do maior para o menor:
///
/// - **`n_i · n_j`** — junta primeiro o que é plano. Dobrar uma quina para
///   dentro de um vértice grosso é destruir a informação que o campo mais
///   precisa;
/// - **a razão de área** (sempre ≥ 1) — junta primeiro o desigual, o que
///   **equaliza** as áreas do nível de cima em vez de as espalhar.
///
/// E o vértice grosso é a média **ponderada pela área** dos dois, não o
/// representante de um deles.
///
/// Devolve `(nível grosso, fino → grosso)`, ou `None` se não houver o que
/// reduzir.
fn coarsen(fine: &Level) -> Option<(Level, Vec<u32>)> {
    let n = fine.len();
    if n == 0 {
        return None;
    }

    // Todas as ligações, com a ordem da referência.
    let mut entries: Vec<(f32, u32, u32)> = Vec::new();
    for i in 0..n {
        for link in &fine.adjacency[i] {
            let k = link.id as usize;
            let dp = dot(fine.normals[i], fine.normals[k]);
            let (ai, ak) = (fine.areas[i], fine.areas[k]);
            let ratio = if ai > ak {
                ai / ak.max(1.0e-20)
            } else {
                ak / ai.max(1.0e-20)
            };
            entries.push((dp * ratio, i as u32, link.id));
        }
    }
    // ⚠️ **DECRESCENTE** (a referência inverte o `operator<`), e o desempate é
    // pelos índices — sem ele a rotulagem deixaria de ser byte-reprodutível.
    entries.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

    let mut merged = vec![false; n];
    let mut pairs: Vec<(u32, u32)> = Vec::new();
    for &(_, i, j) in &entries {
        if merged[i as usize] || merged[j as usize] {
            continue;
        }
        merged[i as usize] = true;
        merged[j as usize] = true;
        pairs.push((i, j));
    }

    let k = n - pairs.len();
    let mut positions = Vec::with_capacity(k);
    let mut normals = Vec::with_capacity(k);
    let mut areas = Vec::with_capacity(k);
    let mut to_upper: Vec<(u32, u32)> = Vec::with_capacity(k);
    let mut parent = vec![u32::MAX; n];

    for &(a, b) in &pairs {
        let (ia, ib) = (a as usize, b as usize);
        let (area1, area2) = (fine.areas[ia], fine.areas[ib]);
        let total = area1 + area2;
        let pos = if total > 1.0e-20 {
            [
                (fine.positions[ia][0] * area1 + fine.positions[ib][0] * area2) / total,
                (fine.positions[ia][1] * area1 + fine.positions[ib][1] * area2) / total,
                (fine.positions[ia][2] * area1 + fine.positions[ib][2] * area2) / total,
            ]
        } else {
            [
                (fine.positions[ia][0] + fine.positions[ib][0]) * 0.5,
                (fine.positions[ia][1] + fine.positions[ib][1]) * 0.5,
                (fine.positions[ia][2] + fine.positions[ib][2]) * 0.5,
            ]
        };
        let nrm = [
            fine.normals[ia][0] * area1 + fine.normals[ib][0] * area2,
            fine.normals[ia][1] * area1 + fine.normals[ib][1] * area2,
            fine.normals[ia][2] * area1 + fine.normals[ib][2] * area2,
        ];
        let idx = positions.len() as u32;
        parent[ia] = idx;
        parent[ib] = idx;
        positions.push(pos);
        normals.push(normalize_or(nrm, fine.normals[ia]));
        areas.push(total);
        to_upper.push((a, b));
    }
    for v in 0..n {
        if merged[v] {
            continue;
        }
        let idx = positions.len() as u32;
        parent[v] = idx;
        positions.push(fine.positions[v]);
        normals.push(fine.normals[v]);
        areas.push(fine.areas[v]);
        to_upper.push((v as u32, u32::MAX));
    }

    // A vizinhança induzida: as ligações dos um ou dois pais, mapeadas para
    // baixo. ⚠️ **Os pesos SOMAM** quando duas ligações caem no mesmo par —
    // é o que mantém o Laplaciano do nível grosso a ser o do fino, agregado.
    let mut adjacency: Vec<Vec<Link>> = Vec::with_capacity(positions.len());
    for (i, &(u0, u1)) in to_upper.iter().enumerate() {
        let mut acc: std::collections::BTreeMap<u32, f32> = std::collections::BTreeMap::new();
        for upper in [u0, u1] {
            if upper == u32::MAX {
                continue;
            }
            for link in &fine.adjacency[upper as usize] {
                let target = parent[link.id as usize];
                if target == i as u32 {
                    continue;
                }
                *acc.entry(target).or_insert(0.0) += link.weight;
            }
        }
        adjacency.push(
            acc.into_iter()
                .map(|(id, weight)| Link { id, weight })
                .collect(),
        );
    }

    Some((
        Level {
            positions,
            normals,
            areas,
            adjacency,
            parent: Vec::new(),
        },
        parent,
    ))
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn normalize_or(a: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    let len = dot(a, a).sqrt();
    if len > 1.0e-20 {
        [a[0] / len, a[1] / len, a[2] / len]
    } else {
        fallback
    }
}

#[cfg(test)]
#[path = "hierarchy_tests.rs"]
mod tests;

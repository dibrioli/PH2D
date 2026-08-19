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

/// Abaixo disto não vale a pena outro nível: o grafo já cabe todo numa
/// vizinhança, e a suavização alcança tudo em poucas varreduras.
pub const COARSEST: usize = 24;

/// Teto de níveis — guarda de RECURSO contra um emparelhamento que pare de
/// reduzir (uma malha com componentes de um vértice só).
const MAX_LEVELS: usize = 24;

/// Um nível: os vértices, a vizinhança, e para onde cada um sobe.
pub struct Level {
    /// Posição de cada vértice deste nível.
    pub positions: Vec<[f32; 3]>,
    /// Normal de cada vértice deste nível.
    pub normals: Vec<[f32; 3]>,
    /// Vizinhos de cada vértice, ordenados (determinismo).
    pub adjacency: Vec<Vec<u32>>,
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
        let adj: Vec<Vec<u32>> = (0..mesh.vert_count())
            .map(|v| mesh.adjacency().vert_verts.neighbours(v).to_vec())
            .collect();
        let mut levels = vec![Level {
            positions: mesh.positions().to_vec(),
            normals: mesh.normals().to_vec(),
            adjacency: adj,
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

/// **UM PASSO DE COARSENING** — emparelha vizinhos e induz o grafo de cima.
///
/// Devolve `(nível grosso, fino → grosso)`, ou `None` se não houver o que
/// reduzir.
fn coarsen(fine: &Level) -> Option<(Level, Vec<u32>)> {
    let n = fine.len();
    if n == 0 {
        return None;
    }
    let mut parent = vec![u32::MAX; n];
    let mut groups: Vec<Vec<u32>> = Vec::new();

    for v in 0..n {
        if parent[v] != u32::MAX {
            continue;
        }
        let g = groups.len() as u32;
        parent[v] = g;
        let mut members = vec![v as u32];
        // ⚠️ **O PRIMEIRO vizinho livre, e é a ordem do índice que decide.** Um
        // casamento por peso (a aresta mais curta, o vizinho de menor valência)
        // daria uma hierarquia mais bonita e uma heurística a mais para medir; o
        // que este andaime precisa é de metade dos vértices, sem preferências e
        // com a mesma resposta em toda corrida.
        for &w in &fine.adjacency[v] {
            if parent[w as usize] == u32::MAX {
                parent[w as usize] = g;
                members.push(w);
                break;
            }
        }
        groups.push(members);
    }

    let k = groups.len();
    let mut positions = Vec::with_capacity(k);
    let mut normals = Vec::with_capacity(k);
    for m in &groups {
        // ⚠️ **O REPRESENTANTE, e nunca a média.** A primeira versão mediava a
        // posição e a normal do par — e cada nível encolhia o modelo para dentro
        // um pouco. Sobre oito níveis o topo deixa de ser uma amostra da
        // superfície e passa a ser um caroço perto do centroide, com normais que
        // já não descrevem forma nenhuma; os campos resolvidos ali são ruído, e
        // prolongar ruído é PIOR que partir da semente. Medido: a hierarquia
        // mediada perdia do caminho plano em **todas** as 24 combinações de
        // (topo × varreduras) — 42..55 % contra 60,9 %.
        //
        // Com o representante, **todo nível é uma SUBAMOSTRA da superfície
        // original**: os pontos nunca saem dela, e as normais são as de verdade.
        let v = m[0] as usize;
        positions.push(fine.positions[v]);
        normals.push(fine.normals[v]);
    }

    // A vizinhança induzida: dois grupos são vizinhos se algum par de membros o
    // era. ⚠️ `Vec` ordenado e deduplicado, nunca um `HashSet` — a ordem dos
    // vizinhos entra na suavização, e ela tem de ser a mesma em toda corrida.
    let mut adjacency: Vec<Vec<u32>> = vec![Vec::new(); k];
    for v in 0..n {
        let a = parent[v];
        for &w in &fine.adjacency[v] {
            let b = parent[w as usize];
            if a != b {
                adjacency[a as usize].push(b);
            }
        }
    }
    for list in &mut adjacency {
        list.sort_unstable();
        list.dedup();
    }

    Some((
        Level {
            positions,
            normals,
            adjacency,
            parent: Vec::new(),
        },
        parent,
    ))
}

#[cfg(test)]
#[path = "hierarchy_tests.rs"]
mod tests;

//! O octree **frouxo** de faces — o índice espacial que torna um dab
//! *limitado pela pegada* em vez de limitado pela malha.
//!
//! Adaptado de `reference/sculptgl/src/math3d/OctreeCell.js`, MIT — ver
//! `LICENSES/sculptgl-MIT.txt`.
//!
//! **A ideia que faz um octree de FACES funcionar** (e que vem do original):
//! duas caixas por nó, não uma. A face é arquivada pelo seu **centro** (uma
//! caixa de partição, que particiona o espaço exatamente), mas ela *se estende*
//! além dele — então cada nó guarda também a caixa **frouxa**, a união do que
//! de fato está lá dentro, e é contra essa que a consulta testa. Sem isso uma
//! face grande na fronteira some da resposta.
//!
//! ⚠️ **Divergência do original: arena plana, não células com ponteiro.** Os nós
//! vivem num `Vec` e os índices de face num segundo `Vec` **permutado**, com
//! cada folha dona de uma faixa contígua — o build particiona no lugar em vez de
//! empurrar para 8 `Vec` filhos. Isso troca milhões de alocações por duas, deixa
//! a travessia sequencial em cache, e é a forma que uma varredura paralela por
//! sub-árvores vai querer sem ser reescrita.
//!
//! ⚠️ **Onde mora a correção, e onde mora só a velocidade** — achado ao desenhar
//! as mutações desta wave, e vale para quem for mexer aqui. A resposta da
//! consulta depende **apenas** da caixa frouxa, que é construída a partir da
//! geometria que de fato está no nó. A partição — em que octante cada face cai —
//! decide só quão *funda e desequilibrada* a árvore fica. Consequência: uma
//! partição errada (eixo trocado, caixa de filho fora de ordem) produz uma
//! árvore **lenta e correta**, não uma árvore que esquece geometria. É por isso
//! que os gates deste módulo afirmam a *contenção* das caixas e a *partição dos
//! índices*, e não a atribuição de octante: um gate sobre a atribuição estaria
//! vigiando a metade que não pode dar resposta errada.

use crate::aabb::Aabb;
use crate::face::Face;

/// Faces por folha antes de dividir. **Herdado do SculptGL** (`MAX_FACES = 100`),
/// que é um app que shipou com ele — não um número que eu escolhi. O ponto de
/// equilíbrio verdadeiro (folha grande = varredura linear cara; folha pequena =
/// travessia profunda) é medido pela sonda `measure_query` do M3, e se ele
/// disser outro número, é esse que fica aqui, com a tabela ao lado.
const MAX_FACES_PER_LEAF: usize = 100;

/// Profundidade máxima. Também do SculptGL (`MAX_DEPTH = 8`). Ele existe para
/// que geometria degenerada — mil faces no mesmo ponto, que nenhuma divisão
/// separa — termine em vez de dividir para sempre.
const MAX_DEPTH: u32 = 8;

const NO_CHILD: u32 = u32::MAX;

#[derive(Clone, Copy, Debug)]
struct Node {
    /// A caixa do que ESTÁ aqui (union das faces, ou dos filhos). É contra ela
    /// que a consulta testa — ver o doc do módulo.
    loose: Aabb,
    /// A caixa de PARTIÇÃO — a região do espaço que este nó reparte entre os
    /// oito filhos, e que não depende da geometria que caiu aqui.
    ///
    /// ⚠️ **Ela é o que torna uma folha capaz de se dividir DEPOIS do build.**
    /// A caixa frouxa não serve: ela encolhe até a geometria e sobrepõe as
    /// irmãs, então cortá-la ao meio não particiona o espaço. Sem este campo uma
    /// folha que recebe faces novas só pode engordar, e uma folha gorda faz toda
    /// consulta naquela região devolver centenas de candidatas.
    split: Aabb,
    /// Índice do primeiro dos 8 filhos, ou `NO_CHILD` numa folha.
    first_child: u32,
    /// Faixa em `face_indices` (folha).
    start: u32,
    len: u32,
    /// De quem este nó é filho (`NO_CHILD` na raiz) — o caminho que o refit
    /// sobe para reunir as caixas dos ancestrais.
    parent: u32,
    /// Profundidade, para o refit processar filhos ANTES de pais sem ordenar.
    depth: u8,
}

/// Índice espacial sobre as faces de uma malha.
#[derive(Clone, Debug, Default)]
pub struct Octree {
    nodes: Vec<Node>,
    face_indices: Vec<u32>,
    /// Face → a folha que a contém. É este mapa que torna o refit possível: a
    /// face MOVEU, então descer a árvore pelo centro dela hoje pode chegar a
    /// outro nó, e recomputar a caixa do nó errado é perder geometria em
    /// silêncio. Custa 4 B por face, e a sonda de memória o reporta.
    face_leaf: Vec<u32>,
    /// Entradas de `face_indices` que nenhuma folha usa mais — o rastro das
    /// faixas realocadas por [`Octree::insert_alongside`], irmão do
    /// [`crate::adjacency::Csr::dead`] e recolhido pela mesma lei de fração.
    dead: u32,
}

impl Octree {
    #[must_use]
    pub fn build(positions: &[[f32; 3]], faces: &[Face]) -> Self {
        let mut tree = Self::default();
        if faces.is_empty() {
            return tree;
        }
        let centers: Vec<[f32; 3]> = faces.iter().map(|f| face_center(positions, *f)).collect();
        let mut split = Aabb::EMPTY;
        for c in &centers {
            split.expand_point(*c);
        }
        tree.face_indices = (0..faces.len() as u32).collect();
        tree.face_leaf = vec![0; faces.len()];
        tree.nodes.push(Node {
            loose: Aabb::EMPTY,
            split,
            first_child: NO_CHILD,
            start: 0,
            len: faces.len() as u32,
            parent: NO_CHILD,
            depth: 0,
        });
        // Os centros são pré-computados aqui porque o build os lê `O(n log n)`
        // vezes; a inserção incremental passa a closure que os deriva na hora,
        // porque ela toca uma folha e pré-computar seria `O(malha)`.
        tree.subdivide(0, split, 0, positions, faces, &|fi: u32| {
            centers[fi as usize]
        });
        tree
    }

    /// **ABSORVE FACES NOVAS** — cada uma na folha da face que a gerou.
    ///
    /// `births` é `(face nova, face de onde ela saiu)`. As faces novas são as do
    /// FIM do vetor de faces (o corte só APENDA), então nenhum índice antigo se
    /// move e o `face_leaf` cresce por append.
    ///
    /// ⚠️ **A folha da mãe é a resposta certa, e não uma aproximação:** as filhas
    /// de uma face partida vivem dentro da extensão dela, que já estava nesta
    /// folha. O deslocamento do ponto médio ao longo da normal pode empurrar uma
    /// filha um pouco para fora — e é por isso que quem chama **tem de** rodar um
    /// [`Self::refit`] com as faces novas logo depois: é ele que devolve a caixa
    /// frouxa exata, subindo até a raiz.
    ///
    /// ⚠️ **A faixa da folha se muda para o FIM de `face_indices`**, o mesmo
    /// idioma do CSR editável e pela mesma razão: as faixas são contíguas e sem
    /// folga, então crescer uma no lugar deslocaria todas as seguintes. Copiar
    /// uma centena de índices é mais barato que reservar folga por folha para
    /// sempre.
    ///
    /// ⚠️ **E ela agrupa por folha antes de mover:** um dab parte várias faces da
    /// MESMA folha, e realocar uma vez por face seria `O(len²)` exatamente onde o
    /// trabalho se concentra.
    pub fn insert_alongside(
        &mut self,
        positions: &[[f32; 3]],
        faces: &[Face],
        births: &[(u32, u32)],
    ) {
        if self.nodes.is_empty() || births.is_empty() {
            return;
        }
        self.face_leaf.resize(faces.len(), NO_CHILD);

        let mut by_leaf: Vec<(u32, u32)> = births
            .iter()
            .map(|&(new, parent)| (self.face_leaf[parent as usize], new))
            .collect();
        by_leaf.sort_unstable();

        let mut fattened: Vec<u32> = Vec::new();
        let mut i = 0;
        while i < by_leaf.len() {
            let leaf = by_leaf[i].0;
            let mut j = i;
            while j < by_leaf.len() && by_leaf[j].0 == leaf {
                j += 1;
            }
            let node = self.nodes[leaf as usize];
            let (s, n) = (node.start as usize, node.len as usize);
            let dest = u32::try_from(self.face_indices.len()).unwrap_or(u32::MAX);
            self.face_indices.reserve(n + (j - i));
            for k in 0..n {
                self.face_indices.push(self.face_indices[s + k]);
            }
            for &(_, new) in &by_leaf[i..j] {
                self.face_indices.push(new);
                self.face_leaf[new as usize] = leaf;
            }
            let len = n + (j - i);
            self.nodes[leaf as usize].start = dest;
            self.nodes[leaf as usize].len = u32::try_from(len).unwrap_or(u32::MAX);
            self.dead = self.dead.saturating_add(u32::try_from(n).unwrap_or(0));
            if len > MAX_FACES_PER_LEAF && u32::from(node.depth) < MAX_DEPTH {
                fattened.push(leaf);
            }
            i = j;
        }

        // A folha que passou do teto se divide — senão ela só engorda, e uma
        // folha gorda faz TODA consulta naquela região devolver centenas de
        // candidatas. É aqui que a `split` guardada se paga.
        for leaf in fattened {
            let split = self.nodes[leaf as usize].split;
            let depth = u32::from(self.nodes[leaf as usize].depth);
            self.subdivide(leaf as usize, split, depth, positions, faces, &|fi: u32| {
                face_center(positions, faces[fi as usize])
            });
        }
        self.compact_if_needed();
    }

    /// **ESQUECE FACES E ACOMPANHA A RENUMERAÇÃO** — o inverso exato do
    /// [`Self::insert_alongside`].
    ///
    /// `dead` são as faces que somem (numeração ANTIGA, crescente) e o `remap`
    /// diz para onde as sobreviventes se mudaram. Ver [`Remap`] para por que a
    /// lista de mudanças é uma SEQUÊNCIA.
    ///
    /// ⚠️ **A caixa frouxa fica GRANDE demais, e isso é o lado seguro.** Ela
    /// ainda contém a face que saiu, então a consulta devolve candidatas a mais —
    /// nunca a menos. É a mesma assimetria que o [`Self::refit`] documenta ao
    /// contrário (*uma caixa pequena demais perde geometria em silêncio*), e o
    /// refit que o chamador roda logo depois a devolve exata nas folhas tocadas.
    ///
    /// ⚠️ **A faixa da folha encolhe pelo FIM**, e é por isso que a ordem dentro
    /// dela não importa: a entrega da consulta é o conjunto, não a sequência. O
    /// buraco que sobra é contado no rastro e recolhido pela mesma lei de fração
    /// do irmão.
    pub fn shrink_faces(&mut self, dead: &[u32], remap: &crate::Remap) {
        if self.nodes.is_empty() {
            return;
        }
        for &d in dead {
            let leaf = self.face_leaf[d as usize];
            if leaf == NO_CHILD {
                continue;
            }
            let node = self.nodes[leaf as usize];
            let (s, n) = (node.start as usize, node.len as usize);
            let Some(at) = self.face_indices[s..s + n].iter().position(|&f| f == d) else {
                continue;
            };
            self.face_indices[s + at] = self.face_indices[s + n - 1];
            self.nodes[leaf as usize].len = u32::try_from(n - 1).unwrap_or(0);
            self.dead = self.dead.saturating_add(1);
        }
        for &(from, to) in &remap.face_moves {
            let leaf = self.face_leaf[from as usize];
            self.face_leaf[to as usize] = leaf;
            if leaf == NO_CHILD {
                continue;
            }
            let node = self.nodes[leaf as usize];
            let (s, n) = (node.start as usize, node.len as usize);
            if let Some(at) = self.face_indices[s..s + n].iter().position(|&f| f == from) {
                self.face_indices[s + at] = to;
            }
        }
        self.face_leaf.truncate(remap.faces);
        self.compact_if_needed();
    }

    /// Recolhe o rastro das faixas realocadas quando ele passa da metade.
    ///
    /// ⚠️ **Só os índices se movem; a árvore não.** `face_leaf` é face → NÓ e
    /// sobrevive intacto; o que é reescrito é o `start` de cada folha. Um nó
    /// interno não tem faixa própria, então ele é pulado — e é isso que mantém
    /// as faixas das folhas contíguas e disjuntas depois da compactação.
    fn compact_if_needed(&mut self) {
        if usize::try_from(self.dead).unwrap_or(0) * 2 <= self.face_indices.len() {
            return;
        }
        let mut packed = Vec::with_capacity(self.face_indices.len() - self.dead as usize);
        for ni in 0..self.nodes.len() {
            if self.nodes[ni].first_child != NO_CHILD {
                continue;
            }
            let (s, n) = (self.nodes[ni].start as usize, self.nodes[ni].len as usize);
            self.nodes[ni].start = u32::try_from(packed.len()).unwrap_or(u32::MAX);
            packed.extend_from_slice(&self.face_indices[s..s + n]);
        }
        self.face_indices = packed;
        self.dead = 0;
    }

    /// Quantos nós — o que a sonda de memória multiplica.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Bytes que a árvore ocupa. Existe porque `Node` é privado e a sonda de
    /// memória precisa da decomposição — o TOTAL ela mede com o dhat, esta
    /// parcela é derivada, e a sonda diz qual é qual.
    #[must_use]
    pub fn memory_bytes(&self) -> usize {
        self.nodes.capacity() * size_of::<Node>()
            + (self.face_indices.capacity() + self.face_leaf.capacity()) * size_of::<u32>()
    }

    /// A caixa de tudo que o octree indexa.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        self.nodes.first().map_or(Aabb::EMPTY, |n| n.loose)
    }

    /// As faixas `(início, fim)` de cada folha em `face_indices`, e o tamanho
    /// dele — o que um gate precisa para afirmar que elas continuam **disjuntas**
    /// depois de realocadas.
    ///
    /// ⚠️ Existe porque `Node` é privado e o invariante é estrutural: um gate que
    /// só olhasse a RESPOSTA da consulta ficaria verde com duas folhas
    /// sobrepostas (as duas devolvem faces reais), e o sintoma só apareceria
    /// numa face contada duas vezes muito depois.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn leaf_spans_for_gate(&self) -> (Vec<(usize, usize)>, usize) {
        let spans = self
            .nodes
            .iter()
            .filter(|n| n.first_child == NO_CHILD && n.len > 0)
            .map(|n| (n.start as usize, (n.start + n.len) as usize))
            .collect();
        (spans, self.face_indices.len())
    }

    /// Quantas faces a folha mais cheia carrega, e a média — a régua de quão
    /// degradada a árvore ficou depois de absorver faces novas.
    #[must_use]
    pub fn leaf_occupancy(&self) -> (usize, f64) {
        let leaves: Vec<usize> = self
            .nodes
            .iter()
            .filter(|n| n.first_child == NO_CHILD)
            .map(|n| n.len as usize)
            .collect();
        let max = leaves.iter().copied().max().unwrap_or(0);
        let mean = if leaves.is_empty() {
            0.0
        } else {
            leaves.iter().sum::<usize>() as f64 / leaves.len() as f64
        };
        (max, mean)
    }

    /// As faces cujo bloco alcança a esfera. **Conservador**: devolve todas as
    /// faces das folhas tocadas, e quem quer exatidão filtra. Essa é a divisão
    /// certa — o octree responde *onde procurar*, não *o que acertou*, e o
    /// filtro exato depende do que o chamador está perguntando (o centro do
    /// dab? o vértice? a aresta?).
    pub fn faces_in_sphere(&self, center: [f32; 3], radius: f32, out: &mut Vec<u32>) {
        out.clear();
        if self.nodes.is_empty() {
            return;
        }
        let mut stack = vec![0u32];
        while let Some(ni) = stack.pop() {
            let node = self.nodes[ni as usize];
            if !node.loose.intersects_sphere(center, radius) {
                continue;
            }
            if node.first_child == NO_CHILD {
                let s = node.start as usize;
                let e = s + node.len as usize;
                out.extend_from_slice(&self.face_indices[s..e]);
            } else {
                for k in 0..8 {
                    stack.push(node.first_child + k);
                }
            }
        }
    }

    /// Visita as folhas que um raio atravessa, **da mais próxima para a mais
    /// distante**, podando com o melhor `t` que o visitante já achou.
    ///
    /// ⚠️ **Por que um visitante em vez de um `Vec` de candidatas** — e a
    /// diferença é grande, ao contrário do irmão [`Octree::faces_in_sphere`].
    /// Uma esfera é pequena e não tem ordem interna, então juntar as candidatas
    /// e filtrar custa o mesmo. Um raio atravessa a malha inteira: sem poda, um
    /// pick lê as faces da superfície de trás para descobrir que a da frente já
    /// tinha ganhado. O visitante devolve o `t` do melhor acerto até agora, e é
    /// isso que corta os nós atrás dele. O octree segue sem saber o que é um
    /// triângulo — ele responde *onde procurar, e em que ordem*.
    ///
    /// A ordenação é por distância de ENTRADA da caixa: empilha-se do mais
    /// distante para o mais próximo, para que o topo da pilha seja o próximo.
    pub fn ray_visit_leaves(
        &self,
        origin: [f32; 3],
        inv_dir: [f32; 3],
        mut visit: impl FnMut(&[u32]) -> f32,
    ) {
        if self.nodes.is_empty() {
            return;
        }
        let mut best = f32::INFINITY;
        let mut stack: Vec<(u32, f32)> = vec![(0, 0.0)];
        while let Some((ni, entry)) = stack.pop() {
            // O `t` de entrada foi medido quando o nó foi EMPILHADO; o `best`
            // pode ter encolhido desde então, e este é o teste que aproveita.
            if entry > best {
                continue;
            }
            let node = self.nodes[ni as usize];
            if node.first_child == NO_CHILD {
                if node.len > 0 {
                    let s = node.start as usize;
                    let e = s + node.len as usize;
                    best = best.min(visit(&self.face_indices[s..e]));
                }
                continue;
            }
            let mut kids = [(0u32, 0.0f32); 8];
            for (k, slot) in kids.iter_mut().enumerate() {
                let ci = node.first_child + k as u32;
                let (t0, t1) = self.nodes[ci as usize].loose.ray_slab(origin, inv_dir);
                *slot = (ci, if t0 <= t1 { t0 } else { f32::INFINITY });
            }
            kids.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
            for (ci, t0) in kids {
                if t0 <= best {
                    stack.push((ci, t0));
                }
            }
        }
    }

    /// Re-ajusta as caixas frouxas depois de a GEOMETRIA se mover.
    ///
    /// ⚠️ **A partição não muda, só as caixas.** Um dab desloca vértices; ele não
    /// cria nem apaga face nenhuma, então cada face continua na folha em que
    /// nasceu. O que envelhece é a caixa frouxa — e uma caixa que ficou PEQUENA
    /// demais perde geometria em silêncio: o vértice que saiu dela vira
    /// invisível para o pincel, e o sintoma é um buraco no traço que ninguém
    /// liga ao índice.
    ///
    /// ⚠️ **É por isso que existe o `face_leaf`.** Descer a árvore pelo centro
    /// ATUAL da face pode chegar a outro nó — o centro se moveu —, e recomputar
    /// a caixa do nó errado deixaria a do nó certo velha. O mapa custa 4 B por
    /// face e é a única coisa que torna esta operação correta.
    ///
    /// Custo: `O(faces movidas + folhas tocadas × faces por folha)`, ou seja
    /// limitado pela PEGADA. A ordem sai de graça — os nós afetados são
    /// agrupados por profundidade e processados do mais fundo para a raiz, então
    /// todo filho já está pronto quando o pai o lê, sem ordenar nada.
    pub fn refit(
        &mut self,
        positions: &[[f32; 3]],
        faces: &[Face],
        moved_faces: &[u32],
        scratch: &mut RefitScratch,
    ) {
        if self.nodes.is_empty() || moved_faces.is_empty() {
            return;
        }
        scratch.begin(self.nodes.len(), MAX_DEPTH as usize + 1);
        for &fi in moved_faces {
            let mut ni = self.face_leaf[fi as usize];
            // Subir para quando encontra um ancestral já marcado: ele e tudo
            // acima dele já estão na lista. Sem esta saída, uma pegada grande
            // reescreveria o caminho até a raiz uma vez por face.
            while ni != NO_CHILD && scratch.mark(ni) {
                let node = &self.nodes[ni as usize];
                scratch.by_depth[node.depth as usize].push(ni);
                ni = node.parent;
            }
        }
        for d in (0..scratch.by_depth.len()).rev() {
            for k in 0..scratch.by_depth[d].len() {
                let ni = scratch.by_depth[d][k] as usize;
                let node = self.nodes[ni];
                let mut loose = Aabb::EMPTY;
                if node.first_child == NO_CHILD {
                    let (s, e) = (node.start as usize, (node.start + node.len) as usize);
                    for &fi in &self.face_indices[s..e] {
                        for &v in faces[fi as usize].verts() {
                            loose.expand_point(positions[v as usize]);
                        }
                    }
                } else {
                    for c in 0..8 {
                        loose.expand(&self.nodes[(node.first_child + c) as usize].loose);
                    }
                }
                self.nodes[ni].loose = loose;
            }
        }
    }

    fn subdivide(
        &mut self,
        node: usize,
        split: Aabb,
        depth: u32,
        positions: &[[f32; 3]],
        faces: &[Face],
        center: &impl Fn(u32) -> [f32; 3],
    ) {
        let start = self.nodes[node].start as usize;
        let len = self.nodes[node].len as usize;
        self.nodes[node].split = split;

        if len <= MAX_FACES_PER_LEAF || depth >= MAX_DEPTH || split.is_empty() {
            let mut loose = Aabb::EMPTY;
            for i in start..start + len {
                let fi = self.face_indices[i];
                self.face_leaf[fi as usize] = node as u32;
                for &v in faces[fi as usize].verts() {
                    loose.expand_point(positions[v as usize]);
                }
            }
            self.nodes[node].loose = loose;
            return;
        }

        let mid = split.center();
        let range = &mut self.face_indices[start..start + len];

        // Três partições binárias encadeadas dão os 8 octantes como faixas
        // contíguas — sem oito vetores temporários.
        let mx = partition(range, center, 0, mid[0]);
        let (lo, hi) = range.split_at_mut(mx);
        let my_lo = partition(lo, center, 1, mid[1]);
        let my_hi = partition(hi, center, 1, mid[1]);

        let mut bounds = [0usize; 9];
        bounds[0] = 0;
        bounds[2] = my_lo;
        bounds[4] = mx;
        bounds[6] = mx + my_hi;
        bounds[8] = len;
        for q in 0..4 {
            let (a, b) = (bounds[q * 2], bounds[q * 2 + 2]);
            let z = partition(&mut range[a..b], center, 2, mid[2]);
            bounds[q * 2 + 1] = a + z;
        }

        let first_child = self.nodes.len() as u32;
        self.nodes[node].first_child = first_child;
        for k in 0..8 {
            let (a, b) = (bounds[k], bounds[k + 1]);
            self.nodes.push(Node {
                loose: Aabb::EMPTY,
                split: Aabb::EMPTY,
                first_child: NO_CHILD,
                start: (start + a) as u32,
                len: (b - a) as u32,
                parent: node as u32,
                depth: (depth + 1).min(u32::from(u8::MAX)) as u8,
            });
        }

        let mut loose = Aabb::EMPTY;
        for k in 0..8 {
            // O índice do octante em `bounds` é `x*4 + y*2 + z` pela ordem em
            // que as partições foram encadeadas; a caixa do filho tem de casar
            // com essa ordem ou as faces caem em nós cuja caixa não as contém.
            let child_split = octant_box(split, mid, k >= 4, (k / 2) % 2 == 1, k % 2 == 1);
            let ci = first_child as usize + k;
            self.subdivide(ci, child_split, depth + 1, positions, faces, center);
            loose.expand(&self.nodes[ci].loose);
        }
        self.nodes[node].loose = loose;
    }
}

/// Reordena `range` para que os centros com `coord[axis] <= mid` fiquem à
/// esquerda; devolve o tamanho da parte esquerda.
fn partition(range: &mut [u32], center: &impl Fn(u32) -> [f32; 3], axis: usize, mid: f32) -> usize {
    let mut i = 0;
    for j in 0..range.len() {
        if center(range[j])[axis] <= mid {
            range.swap(i, j);
            i += 1;
        }
    }
    i
}

fn octant_box(split: Aabb, mid: [f32; 3], hx: bool, hy: bool, hz: bool) -> Aabb {
    let pick = |k: usize, hi: bool| {
        if hi {
            (mid[k], split.max[k])
        } else {
            (split.min[k], mid[k])
        }
    };
    let (x0, x1) = pick(0, hx);
    let (y0, y1) = pick(1, hy);
    let (z0, z1) = pick(2, hz);
    Aabb {
        min: [x0, y0, z0],
        max: [x1, y1, z1],
    }
}

fn face_center(positions: &[[f32; 3]], face: Face) -> [f32; 3] {
    let vs = face.verts();
    let inv = 1.0 / vs.len() as f32;
    let mut c = [0.0f32; 3];
    for &v in vs {
        let p = positions[v as usize];
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    [c[0] * inv, c[1] * inv, c[2] * inv]
}

/// Buffers do [`Octree::refit`], reusados entre dabs.
///
/// O `stamp` é do tamanho da ÁRVORE, mas é carimbado por época e nunca varrido
/// — o mesmo idioma do `QueryScratch`. É isso que mantém o refit função da
/// pegada em vez da malha.
#[derive(Clone, Debug, Default)]
pub struct RefitScratch {
    stamp: Vec<u32>,
    epoch: u32,
    by_depth: Vec<Vec<u32>>,
}

impl RefitScratch {
    fn begin(&mut self, nodes: usize, depths: usize) {
        if self.stamp.len() != nodes {
            self.stamp = vec![0; nodes];
            self.epoch = 0;
        }
        self.epoch = self.epoch.wrapping_add(1);
        // O carimbo 0 é o "nunca visto"; a época nunca pode valer 0.
        if self.epoch == 0 {
            self.epoch = 1;
            self.stamp.fill(0);
        }
        if self.by_depth.len() != depths {
            self.by_depth = vec![Vec::new(); depths];
        }
        for b in &mut self.by_depth {
            b.clear();
        }
    }

    /// Marca `ni`; devolve `false` se ele já estava marcado nesta rodada.
    fn mark(&mut self, ni: u32) -> bool {
        let slot = &mut self.stamp[ni as usize];
        if *slot == self.epoch {
            return false;
        }
        *slot = self.epoch;
        true
    }

    /// Bytes segurados.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        self.stamp.capacity() * size_of::<u32>()
            + self
                .by_depth
                .iter()
                .map(|b| b.capacity() * size_of::<u32>())
                .sum::<usize>()
    }
}

#[cfg(test)]
#[path = "octree_tests.rs"]
mod tests;

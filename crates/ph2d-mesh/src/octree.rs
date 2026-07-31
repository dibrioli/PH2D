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
    /// Índice do primeiro dos 8 filhos, ou `NO_CHILD` numa folha.
    first_child: u32,
    /// Faixa em `face_indices` (folha).
    start: u32,
    len: u32,
}

/// Índice espacial sobre as faces de uma malha.
#[derive(Clone, Debug, Default)]
pub struct Octree {
    nodes: Vec<Node>,
    face_indices: Vec<u32>,
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
        tree.nodes.push(Node {
            loose: Aabb::EMPTY,
            first_child: NO_CHILD,
            start: 0,
            len: faces.len() as u32,
        });
        tree.subdivide(0, split, 0, positions, faces, &centers);
        tree
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
        self.nodes.capacity() * size_of::<Node>() + self.face_indices.capacity() * size_of::<u32>()
    }

    /// A caixa de tudo que o octree indexa.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        self.nodes.first().map_or(Aabb::EMPTY, |n| n.loose)
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

    fn subdivide(
        &mut self,
        node: usize,
        split: Aabb,
        depth: u32,
        positions: &[[f32; 3]],
        faces: &[Face],
        centers: &[[f32; 3]],
    ) {
        let start = self.nodes[node].start as usize;
        let len = self.nodes[node].len as usize;

        if len <= MAX_FACES_PER_LEAF || depth >= MAX_DEPTH || split.is_empty() {
            let mut loose = Aabb::EMPTY;
            for &fi in &self.face_indices[start..start + len] {
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
        let mx = partition(range, centers, 0, mid[0]);
        let (lo, hi) = range.split_at_mut(mx);
        let my_lo = partition(lo, centers, 1, mid[1]);
        let my_hi = partition(hi, centers, 1, mid[1]);

        let mut bounds = [0usize; 9];
        bounds[0] = 0;
        bounds[2] = my_lo;
        bounds[4] = mx;
        bounds[6] = mx + my_hi;
        bounds[8] = len;
        for q in 0..4 {
            let (a, b) = (bounds[q * 2], bounds[q * 2 + 2]);
            let z = partition(&mut range[a..b], centers, 2, mid[2]);
            bounds[q * 2 + 1] = a + z;
        }

        let first_child = self.nodes.len() as u32;
        self.nodes[node].first_child = first_child;
        for k in 0..8 {
            let (a, b) = (bounds[k], bounds[k + 1]);
            self.nodes.push(Node {
                loose: Aabb::EMPTY,
                first_child: NO_CHILD,
                start: (start + a) as u32,
                len: (b - a) as u32,
            });
        }

        let mut loose = Aabb::EMPTY;
        for k in 0..8 {
            // O índice do octante em `bounds` é `x*4 + y*2 + z` pela ordem em
            // que as partições foram encadeadas; a caixa do filho tem de casar
            // com essa ordem ou as faces caem em nós cuja caixa não as contém.
            let child_split = octant_box(split, mid, k >= 4, (k / 2) % 2 == 1, k % 2 == 1);
            let ci = first_child as usize + k;
            self.subdivide(ci, child_split, depth + 1, positions, faces, centers);
            loose.expand(&self.nodes[ci].loose);
        }
        self.nodes[node].loose = loose;
    }
}

/// Reordena `range` para que os centros com `coord[axis] <= mid` fiquem à
/// esquerda; devolve o tamanho da parte esquerda.
fn partition(range: &mut [u32], centers: &[[f32; 3]], axis: usize, mid: f32) -> usize {
    let mut i = 0;
    for j in 0..range.len() {
        if centers[range[j] as usize][axis] <= mid {
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

#[cfg(test)]
#[path = "octree_tests.rs"]
mod tests;

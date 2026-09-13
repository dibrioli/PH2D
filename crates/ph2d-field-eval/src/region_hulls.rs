//! ⭐⭐⭐ **OS CASCOS DE UMA REGIÃO, POR FOLHA** (W148) — o que uma fita especializada de facto
//! pergunta à região, escrito como um VALOR que se compara.
//!
//! A especialização ([`RegionCompiler::compile_at`]) corta as arestas de cada perfil contra o
//! **casco** da região no plano DELE (W59). Até aqui esse casco nascia e morria dentro da compilação,
//! e quem guarda fitas entre quadros (a `TapeCache` do `ph2d-field-render`) não tinha como o
//! perguntar — por isso guardava cada fita para a **caixa**. Medido a `TILE = 24`
//! (`docs/3DModeling/06` §145): a fita servida guardava **`1,9×`–`2,4×`** as arestas da do caminho sem
//! cache, e o custo de uma amostra da marcha segue as arestas.
//!
//! ⭐ Com os cascos como valor, a pergunta da cache passa a ser a mesma que a compilação faz:
//! *«a região nova cabe, EM CADA FOLHA, na região para que esta fita foi construída?»*.
//!
//! ⚠️ **Só os cascos que a compilação CONSOME entram**, e a regra de quem consome é UMA função
//! ([`RegionCompiler::specialised_leaf`]) com dois leitores — a compilação e esta porta. Um casco a
//! mais só tornaria a cache mais exigente; um a menos serviria uma fita onde ela não vale.
//!
//! ⚠️ **A caixa do MUNDO continua a ser pergunta de quem chama.** Ela é o que garante o sinal e a
//! âncora (que saem da caixa, W59) e as folhas que cortam só por caixa — o torno, e as que ficam por
//! baixo de um modificador que remapeia coordenadas.

use crate::RegionCompiler;
use crate::affine::Affine;
use crate::hull::{hull_uv, in_convex};
use crate::profile_index::ProfileIndex;
use fidget::context::Tree;
use ph2d_field::{FieldDoc, Node, NodeKind, Primitive};

/// O que UMA folha de perfil vê da região: a caixa local (o sinal e a âncora saem dela) e o casco em
/// `(u, v)` (a distância corta por ele). Casco vazio = degenerado, e a distância cortou pela caixa.
#[derive(Clone, Debug, PartialEq)]
struct LeafRegion {
    node: usize,
    lo: [f32; 2],
    hi: [f32; 2],
    hull: Vec<[f32; 2]>,
}

/// ⭐ **Os cascos de uma região do mundo** — um por folha que a compilação especializa contra o casco,
/// na ordem da arena.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RegionHulls {
    leaves: Vec<LeafRegion>,
}

const fn rect(lo: [f32; 2], hi: [f32; 2]) -> [[f32; 2]; 4] {
    [[lo[0], lo[1]], [hi[0], lo[1]], [hi[0], hi[1]], [lo[0], hi[1]]]
}

impl LeafRegion {
    /// ⚠️ **Sem alocar**: isto corre por candidata de cada consulta da cache, ~600 vezes por quadro.
    fn holds(&self, inner: &Self) -> bool {
        if self.node != inner.node
            || !(0..2).all(|k| inner.lo[k] >= self.lo[k] && inner.hi[k] <= self.hi[k])
        {
            return false;
        }
        let (outer_rect, inner_rect) = (rect(self.lo, self.hi), rect(inner.lo, inner.hi));
        let outer: &[[f32; 2]] = if self.hull.len() >= 3 {
            &self.hull
        } else {
            &outer_rect
        };
        let polygon: &[[f32; 2]] = if inner.hull.len() >= 3 {
            &inner.hull
        } else {
            &inner_rect
        };
        // ⭐ Convexo dentro de convexo ⇔ todo vértice dentro: é a mesma lei que deixa o corte olhar só
        // os vértices da região (`distance_edges_hull`).
        polygon.iter().all(|p| in_convex(*p, outer))
    }
}

impl RegionHulls {
    /// ⭐⭐ **Esta região contém `inner` em TODAS as folhas?** — a pergunta que decide se a fita
    /// construída para `self` pode responder por `inner`.
    ///
    /// ⚠️ Os dois lados têm de vir do MESMO documento (a mesma lista de folhas, na mesma ordem) — é o
    /// que a cache já garante ao etiquetar cada fita com o documento que a construiu. Listas que não
    /// casam respondem `false`, que é o lado seguro.
    #[must_use]
    pub fn contains(&self, inner: &Self) -> bool {
        self.leaves.len() == inner.leaves.len()
            && self
                .leaves
                .iter()
                .zip(&inner.leaves)
                .all(|(o, i)| o.holds(i))
    }

    /// O casco que a compilação desta região usa na folha `node` — vazio quando não há, e aí a
    /// distância corta pela caixa (o degenerado seguro).
    pub(crate) fn hull_of(&self, node: usize) -> &[[f32; 2]] {
        self.leaves
            .iter()
            .find(|l| l.node == node)
            .map_or(&[], |l| &l.hull)
    }

    /// ⚠️ Só para o gate: quantas folhas têm região própria.
    #[doc(hidden)]
    #[must_use]
    pub fn probe_leaves(&self) -> usize {
        self.leaves.len()
    }
}

impl RegionCompiler {
    /// ⭐⭐ **A folha `i` é especializada, e com que mapa?** — a regra de quem consome o casco, escrita
    /// UMA vez.
    ///
    /// ⚠️ Os dois leitores são a compilação e [`Self::hulls`]. *Uma regra escrita em dois sítios ainda
    /// não é uma regra* — e aqui a cópia que divergisse serviria uma fita sobre uma folha que a
    /// compilação cortou por outro critério.
    pub(crate) fn specialised_leaf(&self, node: &Node, i: usize) -> Option<(Affine, &ProfileIndex)> {
        self.maps
            .get(i)
            .copied()
            .flatten()
            .filter(|_| !node.mods.iter().any(crate::remaps_coordinates))
            .zip(self.idx.get(&i))
    }

    /// ⭐⭐⭐ **Os cascos de uma região** — exactamente os que [`Self::compile_hulled`] vai consumir.
    ///
    /// ⚠️ Os cantos são os do tubo, **crus** (a folga da sonda da normal é somada pelo `hull_uv`, que
    /// a lê da caixa) — a mesma convenção do [`Self::compile_at`].
    #[must_use]
    pub fn hulls(
        &self,
        doc: &FieldDoc,
        lo: [f32; 3],
        hi: [f32; 3],
        corners: &[[f32; 3]],
    ) -> RegionHulls {
        let mut leaves = Vec::new();
        for (i, node) in doc.nodes().iter().enumerate() {
            // ⚠️ Só o EXTRUDE e o POLÍGONO cortam por casco: o `u` do torno é `√(x² + z²)`, e a região
            // dele em `(u, v)` é um rectângulo por construção — ver `specialised_profile`.
            if !matches!(
                node.kind,
                NodeKind::Leaf(Primitive::Extrude { .. } | Primitive::Polygon { .. })
            ) {
                continue;
            }
            let Some((m, _)) = self.specialised_leaf(node, i) else {
                continue;
            };
            let (llo, lhi) = m.box_of(lo, hi);
            let (lo2, hi2) = ([llo[0], llo[1]], [lhi[0], lhi[1]]);
            leaves.push(LeafRegion {
                node: i,
                lo: lo2,
                hi: hi2,
                hull: hull_uv(&m.points_of(corners), lo2, hi2),
            });
        }
        RegionHulls { leaves }
    }

    /// ⭐⭐⭐ **A especialização com os cascos JÁ CALCULADOS** — a porta de quem os guarda ao lado da
    /// fita.
    ///
    /// ⚠️ Os cascos têm de ser os desta caixa ([`Self::hulls`] com o mesmo `lo`/`hi`): um casco de
    /// outra região não fica lento, ele deita fora a aresta mais próxima e a marcha **atravessa a
    /// peça**. O [`Self::compile_at`] é esta porta com os cascos calculados na hora.
    #[must_use]
    pub fn compile_hulled(
        &self,
        doc: &FieldDoc,
        lo: [f32; 3],
        hi: [f32; 3],
        hulls: &RegionHulls,
    ) -> Tree {
        crate::compile_in_region_with(self, doc, lo, hi, hulls)
    }
}

#[cfg(test)]
#[path = "region_hulls_tests.rs"]
mod tests;

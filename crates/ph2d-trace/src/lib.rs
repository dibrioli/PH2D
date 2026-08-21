//! **O TRAÇADO E A DECOMPOSIÇÃO EM PATCHES** — a fase **F3** do plano.
//!
//! Clean-room a partir de Pietroni et al., *Reliable Feature-Line Driven
//! Quad-Remeshing* (SIGGRAPH 2021), **§6**. ⚠️ Nenhuma linha traduzida de fonte
//! GPL — ver **ADR-0161**, Trilha A.
//!
//! # O que ela fecha
//!
//! O **F2** ([`ph2d_crossfield`]) entrega o campo e diz **onde as singularidades
//! ficam**. O **F4** ([`ph2d_quantize`]) já sabe consumir um [`Layout`] — patches,
//! lados, arcos, alvos — e resolvê-lo com ótimo demonstrado. ⭐ **Faltava quem
//! produzisse o layout sem o oráculo, e é esta fase.**
//!
//! # A ideia, em três frases
//!
//! De cada **singularidade** partem curvas que seguem o campo — as
//! *separatrizes*. Elas correm até bater noutra singularidade ou noutra
//! separatriz. O que sobra da superfície, recortada por elas, são os **patches**.
//!
//! ⚠️ **Aqui as separatrizes correm sobre ARESTAS da malha**, vértice a vértice,
//! e não como polilinhas a cortar triângulos. Não é aproximação por preguiça: é o
//! que faz a fronteira de cada patch ser um conjunto de arestas existentes, e por
//! isso o patch é um conjunto de **faces inteiras** — que é exatamente o formato
//! que o oráculo também exporta (medido: as fronteiras dele, reconstruídas em
//! 2026-08-20, caem todas em arestas da malha remalhada).
//!
//! # ⚠️ O que decide um CANTO
//!
//! Não é *"o vértice é um nó do traçado"*. Um nó em T é canto para os dois
//! patches do lado da haste e **meio de lado** para o patch do outro lado — e é
//! por isso que um lado pode ter mais de um arco. O que decide é o **ângulo
//! interno daquele patch naquele vértice**: perto de 90° é canto, perto de 180°
//! não é. *A pergunta é do par (patch, vértice), nunca do vértice sozinho.*
//!
//! # O que esta fase ainda NÃO faz
//!
//! - ⛔ **Feature lines** (arestas vivas) não entram: o QuadWild traça também a
//!   partir delas, e sem isso uma quina dura não vira fronteira de patch.
//! - ⛔ Uma separatriz que não fecha é **descartada**, e o [`TraceReport`] conta
//!   quantas. Ligá-la à mais próxima é trabalho do dia em que o número doer.

#![forbid(unsafe_code)]

/// **A DECOMPOSIÇÃO** — do traçado para patches, lados e arcos — ver [`patches`].
pub mod patches;
/// **O ANEL de um vértice e as SEMENTES** — ver [`ring`].
pub mod ring;
/// **O PASSEIO** que segue o campo — ver [`walk`].
pub mod walk;

use std::collections::BTreeMap;

use ph2d_crossfield::{CrossField, Dual};
use ph2d_mesh::Mesh;
use ph2d_quantize::{ArcSpec, Layout, PatchSpec};

pub use patches::PatchLayout;
pub use ring::Ring;
pub use walk::Walker;

/// O que o traçado mediu — o relatório que diz se ele fez sentido.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TraceReport {
    /// Quantos vértices o campo marcou como singulares.
    pub singularities: usize,
    /// Quantas separatrizes fecharam e viraram parede.
    pub separatrices: usize,
    /// ⚠️ Quantas foram **descartadas** por não fecharem dentro do limite de
    /// passos. Cada uma é uma parede que não existe, logo um patch maior do que
    /// devia — o número tem de ser lido junto com o histograma de valências.
    pub dangling: usize,
    /// Quantos patches saíram.
    pub patches: usize,
    /// Histograma **valência → quantos patches**. ⭐ É a linha que se compara com
    /// o oráculo: ele entrega 3 a 6, e um patch de 12 lados é o sintoma de uma
    /// separatriz que faltou.
    pub valence: BTreeMap<usize, usize>,
    /// ⚠️ Quantos patches **não são disco** (a fronteira deles não é um laço só).
    /// O F4 recusa layout aberto; um patch anelar não tem lados definidos.
    pub non_disk: usize,
    /// Quantos arcos o layout tem.
    pub arcs: usize,
    /// Quantas paredes foram **dissolvidas** para curar patches degenerados.
    /// ⚠️ Cada uma é uma separatriz que o traçado não devia ter posto ali — o
    /// número é o custo da limpeza, e lê-se junto com o das valências.
    pub dissolved: usize,
    /// Quantas rondas de limpeza correram.
    pub rounds: usize,
    /// ⚠️ **Quantos vértices têm o anel ABERTO** — não-manifold ou de borda.
    ///
    /// ⭐ Ele está aqui para separar *"o traçado é mau"* de *"a malha que
    /// chegou é má"*, e as duas coisas leem igual num histograma de valências.
    /// Um anel aberto não tem índice de singularidade definido, e o campo
    /// devolve `0` ali — o que faz a soma dos índices deixar de bater `4·χ` sem
    /// que nada no campo esteja errado.
    pub open_rings: usize,
}

/// **TRAÇA e DECOMPÕE** — a porta desta fase.
///
/// ⚠️ **A malha tem de estar TRIANGULADA e fechada**, que é o que o
/// [`ph2d_crossfield::Dual`] já exige.
#[must_use]
pub fn trace_patches(mesh: &Mesh, dual: &Dual, field: &CrossField) -> PatchLayout {
    let walker = Walker::new(mesh, dual, field);
    let (mut walls, base) = walker.trace_all();
    let mut out = patches::decompose(mesh, &walls, base.clone());
    // ⭐ **A limpeza é iterativa porque dissolver uma lasca pode fazer nascer
    // outra**: as faces dela passam para o vizinho, e o vizinho ganha cantos que
    // não tinha. Ela pára sozinha quando não há degenerado nenhum — e o teto
    // existe só para o caso de duas lascas se curarem uma à outra em ciclo.
    let mut dissolved = 0usize;
    let mut rounds = 0usize;
    for _ in 0..MAX_CLEANUP_ROUNDS {
        let victims = out.degenerate();
        if victims.is_empty() {
            break;
        }
        if !patches::dissolve(&mut walls, &out, &victims) {
            break;
        }
        dissolved += victims.len();
        rounds += 1;
        let mut next = base.clone();
        next.dissolved = dissolved;
        next.rounds = rounds;
        out = patches::decompose(mesh, &walls, next);
    }
    out.report.dissolved = dissolved;
    out.report.rounds = rounds;
    out
}

/// ⚠️ **O teto de rondas de limpeza.** Ele não escolhe qualidade nenhuma: a
/// limpeza pára sozinha quando não há patch degenerado. Existe para o caso
/// patológico de duas lascas se recriarem uma à outra, e quem o atinge sai com
/// degenerados ainda na lista — que o F4 recusa, alto e claro.
const MAX_CLEANUP_ROUNDS: usize = 32;

impl PatchLayout {
    /// **O LAYOUT que o F4 consome**, com os alvos derivados de `target_edge`.
    ///
    /// ⚠️ **`target_edge` é o comprimento de aresta desejado no quad final**, na
    /// mesma unidade do mundo. O alvo de cada arco é o comprimento geométrico
    /// dele a dividir por esse número — é a mesma régua que a bancada usa para
    /// medir o oráculo, e é o que torna as duas colunas comparáveis.
    ///
    /// # Errors
    /// Devolve [`ph2d_quantize::LayoutError`] quando a decomposição não fecha —
    /// um arco de bordo, um patch de menos de 3 lados. ⚠️ **É a mesma cerca que o
    /// F4 já tinha**, e ela é aqui que passa a proteger o nosso traçado e não só
    /// o do oráculo.
    pub fn to_layout(&self, target_edge: f32) -> Result<Layout, ph2d_quantize::LayoutError> {
        let scale = if target_edge > 0.0 { target_edge } else { 1.0 };
        let arcs = self
            .arc_length
            .iter()
            .map(|&l| ArcSpec::new(f64::from(l / scale)))
            .collect();
        let patches = self
            .sides
            .iter()
            .map(|sides| PatchSpec {
                sides: sides.clone(),
            })
            .collect();
        Layout::new(arcs, patches)
    }
}

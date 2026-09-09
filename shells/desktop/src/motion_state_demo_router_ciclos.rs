//! **AS CENAS DE SMOKE DOS CICLOS** — irmãs do [`super::demo_router`] pelo tecto de 600 LOC
//! (HR-18), e o corte é por RESPONSABILIDADE.
//!
//! ⚠️ **Elas não são «mais uma família» do roteador:** uma cena de família é **uma chamada**
//! (`conferencia::field_family(..)`); uma cena de ciclo **constrói, pousa a LEGENDA no canvas e
//! ANUNCIA os passos no terminal**, porque ela é o **smoke que o dono segue** do princípio ao
//! fim ([doc 103 §1](../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md), passo 7). São três
//! coisas, e é isso que as junta aqui.
//!
//! ⚠️ **A lista vive numa porta só** ([`e_de_ciclo`] e [`build`] leem a MESMA tabela): um
//! roteador que soubesse o número e um construtor que soubesse outro dariam uma cena muda, que é
//! exactamente a forma de defeito que o gate `no_two_smoke_scenes_claim_the_same_level` não vê.

use super::announce;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Os níveis que são cena de ciclo. ⚠️ **Uma tabela, dois leitores** — ver o cabeçalho.
const CICLOS: &[&str] = &["111", "112"];

/// Este nível é uma cena de ciclo?
pub(super) fn e_de_ciclo(n: &str) -> bool {
    CICLOS.contains(&n)
}

/// Constrói a cena, pousa a legenda e anuncia os passos.
pub(super) fn build(n: &str, doc: &mut MotionDoc, reg: &NodeRegistry) -> Vec<NodeId> {
    match n {
        // ⭐ **EM TORNO DE QUÊ** (ciclo 3, doc 106). O pano nasce LONGE da origem de propósito:
        // com ele centrado, os três modos de pivô dão a mesma imagem e a cena ensinaria que a
        // escolha não importa.
        "111" => {
            let sinks = super::pivot_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::pivot_demo::captions());
            announce::pivot();
            sinks
        }
        // ⭐ **NEM TODOS AO MESMO TEMPO** (ciclo 4, doc 107). O campo nasce FORA do centro do
        // pano: centrado, mexer nele só o faria sair, e a cena ensinaria que um campo é um
        // interruptor.
        "112" => {
            let sinks = super::foco_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::foco_demo::captions());
            announce::foco();
            sinks
        }
        // ⚠️ Inalcançável: a [`e_de_ciclo`] gateia esta função com a MESMA tabela.
        _ => Vec::new(),
    }
}

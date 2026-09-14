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
const CICLOS: &[&str] = &["111", "112", "113", "114", "115", "116"];

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
        // ⭐ **DEIXAR A FÍSICA DECIDIR** (ciclo 5, doc 108). O colisor é um `Box` e não o
        // chão, porque num plano infinito a alça de tamanho escreve num param que a forma
        // não lê — e um tutorial não aponta para um controlo inerte.
        "113" => {
            let sinks = super::sim_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::sim_demo::captions());
            announce::sim();
            sinks
        }
        // ⭐ **PEÇAS QUE NÃO SE ATRAVESSAM** — o `motion.collide` dentro de uma simulação a
        // correr. ⚠️ Ela não é de um ciclo: entra aqui porque é o que esta tabela sabe fazer
        // (construir, pousar a LEGENDA e ANUNCIAR os passos), e uma cena que o dono segue
        // precisa das três.
        "114" => {
            let sinks = super::pilha_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::pilha_demo::captions());
            announce::pilha();
            sinks
        }
        // ⭐⭐⭐ **DE QUE A PEÇA É FEITA** (doc 109 §7) — o material. A rampa é `sim.collide` e
        // não uma peça pousada: o obstáculo do nó é o único plano INCLINADO deste catálogo, e
        // uma rampa feita de peças pedia um pino, um ângulo e três nós a mais para dizer o
        // mesmo. A pergunta da cena é o cartão da BOLA.
        "115" => {
            let sinks = super::material_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::material_demo::captions());
            announce::material();
            sinks
        }
        // ⭐⭐⭐ **UM NÚMERO QUE MANDA EM TUDO** (ciclo 6, doc 110 §6) — 102 400 peças e um fio.
        // ⚠️ Ela não ensina uma LEI, ensina um CUSTO: até à W1a, aquela única ligação trocava o
        // caminho de `3,85 ms` pelo de `195,9 ms`. Por isso é grande — um custo de `50×` sobre
        // dez peças não se vê.
        "116" => {
            let sinks = super::fio_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::fio_demo::captions());
            announce::fio();
            sinks
        }
        // ⚠️ Inalcançável: a [`e_de_ciclo`] gateia esta função com a MESMA tabela.
        _ => Vec::new(),
    }
}

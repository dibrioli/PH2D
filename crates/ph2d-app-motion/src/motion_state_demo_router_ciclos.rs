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
const CICLOS: &[&str] = &[
    "111", "112", "113", "114", "115", "116", "117", "118", "119", "120", "130", "131", "132",
    "133", "134", "135", "136", "137", "138", "139", "140", "141", "142",
];

/// Este nível é uma cena de ciclo?
pub(super) fn e_de_ciclo(n: &str) -> bool {
    CICLOS.contains(&n)
}

/// ⚠️ **A MESMA tabela, para quem não é o roteador** — o gate dos anúncios varre as cenas de ciclo
/// e tem de as descobrir por aqui, senão ele leva a própria lista e as duas divergem no dia em que
/// nascer a sétima.
#[cfg(test)]
pub(crate) fn e_de_ciclo_pub(n: &str) -> bool {
    e_de_ciclo(n)
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
        // ⭐ **PEÇAS QUE NÃO SE ATRAVESSAM** — o colisor DA FORMA dentro de uma simulação a
        // correr. ⚠️ Ela não é de um ciclo: entra aqui porque é o que esta tabela sabe fazer
        // (construir, pousar a LEGENDA e ANUNCIAR os passos), e uma cena que o dono segue
        // precisa das três.
        //
        // ⛔⛔ **Esta linha dizia «o `motion.collide` dentro de uma simulação» e era FALSA** —
        // corrigida em 2026-09-17 por um censo DERIVADO (doc 115 §14.1), que monta cada nível
        // pela porta do produto e pergunta ao grafo: a `=114` tem **zero** nós de colisão, e há
        // um gate dela a afirmá-lo desde que nasceu (`the_scene_has_no_collide_node…`). O
        // próprio anúncio dela diz *«não há cartão `Collide` nenhum na linha da simulação»*.
        // ⚠️ *Um comentário de roteador e um gate a dizerem o contrário um do outro é exactamente
        // como uma nota envelhece: ninguém corre o comentário.*
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
        // ⭐⭐⭐ **UM NÚMERO QUE MANDA EM TUDO** (ciclo 6, doc 110 §12) — quatro quadrantes, e em
        // cada par muda **UM NÓ**. ⚠️ Ela é irmã da `=116` e não a repete: aquela ensina o CUSTO
        // (102 400 peças, a rota no terminal), esta ensina a LEI (trinta e seis peças, o que um
        // nó a mais faz ao número e ao instante que já lá estavam).
        "117" => {
            let sinks = super::valor_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::valor_demo::captions());
            announce::valor();
            sinks
        }
        // ⭐⭐⭐ **A COR E O RASTO** (ciclo 7, doc 112 §4-octies) — três fileiras, uma pergunta por
        // fileira (de que cor · que marca deixa · quando chega). ⚠️ A de cima é PARADA de
        // propósito: a cor não precisa de tempo, e pô-la a mexer misturaria duas perguntas.
        "118" => {
            let sinks = super::aparencia_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::aparencia_demo::captions());
            announce::aparencia();
            sinks
        }
        // ⭐ **DE ONDE VÊM AS COISAS** (ciclo 8, doc 113). ⚠️ A fileira de baixo é a única que
        // pede a MÃO do dono (o zoom) — sem ele os dois panos dela são iguais, e o anúncio
        // di-lo no passo.
        "119" => {
            let sinks = super::fontes_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::fontes_demo::captions());
            announce::fontes();
            sinks
        }
        // ⭐ **COISAS QUE SE SEGURAM** (ciclo 9, doc 114). ⚠️ A fileira do MEIO é a única em que
        // os dois panos correm a MESMA lei por caminhos opostos (FK escreve ângulos, IK
        // escreve-os a partir de uma posição) — o anúncio tem de o dizer, senão o par lê-se
        // como *«dois nós que fazem o mesmo»*.
        "120" => {
            let sinks = super::rig_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::rig_demo::captions());
            announce::rig();
            sinks
        }
        // ⭐ **A CORDA COM COLISOR** (ciclo 16, doc 125 §1): a única diferença entre as duas é o
        // `Collide` do Shape ligado à porta `shape` da corda.
        "130" => {
            let sinks = super::corda_com_colisor_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::corda_com_colisor_demo::captions());
            super::corda_com_colisor_demo::announce();
            sinks
        }
        // ⭐ **O `Collide` DEPOIS DA CORDA** (ciclo 16, doc 125 §2): o cartão do Shape diz o que fazer.
        "131" => {
            let sinks = super::corda_com_colisor_demo::build_depois(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::corda_com_colisor_demo::captions_depois());
            super::corda_com_colisor_demo::announce_depois();
            sinks
        }
        // ⭐ **O RITMO** (ciclo 17, doc 126): duas galáxias iguais menos o `Order` — fila a fila
        // contra anéis do centro para fora.
        "132" => {
            let sinks = super::ritmo_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::ritmo_demo::captions());
            super::ritmo_demo::announce();
            sinks
        }
        // ⭐ **A COR QUE O OLHO VÊ** (ciclo 18, doc 127): duas galáxias iguais menos o `Color
        // Space` do `Tint` — cinza no meio contra cor viva.
        "133" => {
            let sinks = super::cor_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::cor_demo::captions());
            super::cor_demo::announce();
            sinks
        }
        // ⭐ **O JOGO MANDA** (ciclo 19, doc 128): quatro entradas publicadas mandam numa barra
        // (tempo dirigido), num cruzamento de estados e numa explosão sobre a galáxia simulada.
        "134" => {
            let sinks = super::jogo_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::jogo_demo::captions());
            super::jogo_demo::announce();
            sinks
        }
        // ⭐ **O ACABAMENTO** (ciclo 20, doc 129): a galáxia passa por um `Lens` — a vinheta
        // ligada ao abrir, e o roteiro liga o resto secção a secção.
        "135" => {
            let sinks = super::acabamento_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::acabamento_demo::captions());
            super::acabamento_demo::announce();
            sinks
        }
        // ⭐ **VIDA E DANO NUM OBJECTO** (doc 131 §3, D.13): o dano sai da vida (`On Change` Fall),
        // e a cor das estrelas liga-se num gesto.
        "136" => {
            let sinks = super::dano_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::dano_demo::captions());
            super::dano_demo::announce();
            sinks
        }
        // ⭐ **O COLISOR PELO NOME, NA PLACA** (doc 131 §3, D.14): duas galáxias de discos, a única
        // diferença é o nome que o `Drive` Custom escreve.
        "137" => {
            let sinks = super::colisor_nome_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::colisor_nome_demo::captions());
            super::colisor_nome_demo::announce();
            sinks
        }
        // ⭐ **O BRILHO EM CADA FORMA** (doc 132 §1, P1): 4 096 discos a cair num campo, `Outer Glow`
        // em `Each Shape`.
        "138" => {
            let sinks = super::cada_forma_demo::build_discos(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::cada_forma_demo::captions());
            super::cada_forma_demo::announce_discos();
            sinks
        }
        // ⭐ **A FITA QUE DOBRA** (doc 132 §2, P2): a galáxia simulada com dezasseis gatilhos a
        // disparar em todo tique; com o tecto pequeno a faixa da Timeline começa depois do 0.
        "139" => {
            let sinks = super::fita_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::fita_demo::captions());
            super::fita_demo::announce();
            sinks
        }
        // ⭐ **O AVISO QUE SE LÊ INTEIRO** (doc 132 §5, P5): o fio de uma corrente por-elemento
        // num param escalar dá o aviso mais longo do Motion.
        "141" => {
            let sinks = super::aviso_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::aviso_demo::captions());
            super::aviso_demo::announce();
            sinks
        }
        // ⭐ **IMAGENS E FORMAS NUM GRUPO, NA PLACA** (doc 132 §4, P4): duas galáxias de estrelas e
        // quadrados de imagem que misturam em grupo (`Screen`, `Add`) sobre um fundo.
        "140" => {
            let sinks = super::imagens_grupo_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::imagens_grupo_demo::captions());
            super::imagens_grupo_demo::announce();
            sinks
        }
        // ⭐ **O `Scene` COM IMAGENS, NA PLACA** (doc 132 §4.1 (3)): imagens na porta 0 e estrelas na
        // 1 de uma galáxia; as estrelas tapam (a ordem das linhas) e só o grupo clareia o fundo.
        "142" => {
            let sinks = super::imagens_scene_demo::build(doc, reg).unwrap_or_default();
            crate::motion_demo_legend::publish(super::imagens_scene_demo::captions());
            super::imagens_scene_demo::announce();
            sinks
        }
        // ⚠️ Inalcançável: a [`e_de_ciclo`] gateia esta função com a MESMA tabela.
        _ => Vec::new(),
    }
}

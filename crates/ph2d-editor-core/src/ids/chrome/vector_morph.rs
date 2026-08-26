//! **As SETAS do Morph** — os `NodeId` da seção *Morph States* (plano 32 W4/W7/W8).
//!
//! ⛔⛔ **SEÇÃO PRÓPRIA, e a lição é de produto** (Enio, 2026-08-25: *"os states de morph deveriam
//! ter sessão exclusiva"*). A W4 pendurou estas linhas dentro da seção **States** — a das poses de
//! UI e do Smart Animate — com o argumento de que *"o Inspector mostra o que o objecto TEM"* e de
//! que *"um objecto raramente é as duas coisas"*. O argumento é verdadeiro e **não era a pergunta**:
//! partilhar a seção fez o cabeçalho de uma feature **já entregue** passar a aparecer por causa de
//! outra, e o dono leu isso como contaminação — que é exactamente o que era.
//!
//! ⚠️ *A lei do ADR-0166 diz o que MOSTRAR, nunca ONDE.* Duas features com donos diferentes,
//! histórias diferentes e gates diferentes partilhando um cabeçalho é uma porta a mais na seção de
//! quem chegou primeiro.

use super::{NodeId, fnv_node_id_runtime, hash_node_id};

/// **O cabeçalho da seção MORPH STATES** — a máquina de estados do Morph selecionado.
///
/// Vizinha da [`super::vector::VECTOR_SECTION_MORPH`] e **abaixo dela**, porque a ordem é o
/// assunto: a seção Morph diz o que o objecto **é**, esta diz **como ele decide** em que forma
/// está.
pub const VECTOR_SECTION_MORPH_STATES: NodeId = hash_node_id("vector.section.morph_states");

/// **O botão que faz o conjunto** — escolhe-se N formas no canvas e ele cria o objecto que as
/// governa, com **todas** as transições possíveis já ligadas (plano 32 W8).
pub const VECTOR_MORPH_STATES_MAKE: NodeId = hash_node_id("vector.morph.states.make");

/// **QUANTAS FORMAS um conjunto de estados aceita** — e o recurso é o **relógio de pintura do
/// painel**, medido.
///
/// ⚠️ **O número que manda é este, e não o de setas:** o artista escolhe FORMAS, e o grafo é o
/// completo dirigido sobre elas ⇒ [`MAX_MORPH_ARROWS`] é **derivado**, nunca escrito à mão. Um par
/// de constantes independentes seria duas respostas à mesma pergunta.
///
/// # A medição (2026-08-25, `MockPanelHost` a pintar o painel Vector inteiro, release)
///
/// O painel **sem** esta seção custa `0,746 ms`. Cada linha de seta acrescenta `0,0104 ms`, linear:
///
/// | formas | setas `n(n-1)` | painel | delta da seção | % de um quadro de 16,7 ms |
/// |---:|---:|---:|---:|---:|
/// | 7 | 42 | `1,181 ms` | `0,435 ms` | 7,07 % |
/// | 8 | 56 | `1,330 ms` | `0,584 ms` | 7,97 % |
/// | **9** | **72** | **`1,497 ms`** | **`0,752 ms`** | **8,97 %** |
/// | 10 | 90 | `1,699 ms` | `0,954 ms` | 10,18 % |
/// | 11 | 110 | `1,899 ms` | `1,154 ms` | 11,37 % |
///
/// ⇒ **9 é o último `n` em que esta seção sozinha não custa mais do que TODO o resto do painel
/// junto** (`0,752` contra `0,746`). Em 10 ela passa a custar mais que todas as outras seções
/// somadas, e o painel inteiro existe para responder a mais perguntas do que esta.
///
/// ⛔ **O pool de ids NÃO é o recurso** — foi a primeira hipótese e a medição refutou-a: registar
/// 132 linhas × 25 widgets custa `0,293 ms` **uma vez**, no `populate`, e nunca por quadro.
///
/// ⛔ **A régua não fica como gate.** Ela divide dois relógios, que é exactamente a família de
/// flakes sob fan-out do `CLAUDE.md` §5.0 — a tabela acima é o registo, e re-medir é rodar a sonda.
pub const MAX_MORPH_STATES: usize = 9;

/// **Quantas setas a seção mostra** — `n(n-1)`, o grafo completo dirigido sobre
/// [`MAX_MORPH_STATES`] formas. **Derivado**, e é isso que impede as duas de divergirem.
pub const MAX_MORPH_ARROWS: usize = MAX_MORPH_STATES * (MAX_MORPH_STATES - 1);

/// **Quantas acções o menu da condição oferece.** É o pool de ids do popover.
///
/// ⚠️ Um mapa com mais acções continua a funcionar; o menu mostra as primeiras. O número acompanha
/// o que um projecto real tem (o mapa de fábrica traz **seis**).
pub const MAX_MORPH_ACTIONS: usize = 24;

/// O chip da CONDIÇÃO da seta `row` — abre o menu das acções do Input Map.
#[must_use]
pub fn morph_arrow_when_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.morph.arrow.when.{row}"))
}

/// A opção `action` no menu da condição da seta `row`.
///
/// ⚠️ **O índice `0` é o «—»** (sem condição): ele existe porque *tirar* a condição tem de ser um
/// gesto — e, desde a W8, é **a única maneira de desligar uma transição**. O grafo é completo por
/// construção, então uma seta sem acção é uma passagem que existe e **nunca acontece**.
#[must_use]
pub fn morph_arrow_when_option_id(row: usize, action: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.morph.arrow.when.{row}.{action}"))
}

/// O cabeçalho da sub-lista das setas, dentro da seção.
pub const VECTOR_MORPH_ARROWS_LABEL: NodeId = hash_node_id("vector.morph.arrows.label");

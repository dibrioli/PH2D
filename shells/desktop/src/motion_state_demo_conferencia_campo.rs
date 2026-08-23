//! **A cena do CAMPO QUE ERA UM NÚMERO** (`=83`) — o anúncio, e só ele (Grupo Y, doc 90 §5).
//!
//! ⚠️ Mora sozinha pela mesma razão da irmã `gates`: o corte é por ASSUNTO. As cenas da
//! `utilidade` respondem a uma FOLHA do doc 89 (*que controle falta contra a referência*); esta
//! responde a um DEFEITO — *o nó tinha o controle e não o ouvia*.

use super::*;

/// **A CENA `=83` — O CAMPO QUE ERA UM NÚMERO** (Grupo Y).
///
/// ⚠️ **O oráculo é a figura VARIAR ao longo de si mesma.** Duas portas do domínio `Instances`
/// — campos, por tipo — eram lidas por `.first()`: o nó inteiro recebia o elemento zero, que num
/// degradê é `0.0`. A metade esquerda desenha o que o nó de facto fazia com a porta **ligada**;
/// a direita, o campo a valer por elemento.
pub(crate) fn field_port_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_campo::build_campo_demo_document(doc, registry).unwrap_or_default();
    let (n, count) = conferencia_demos_campo::authored();
    eprintln!(
        "[cena 83] {n} linhas, {c} pecas nas duas primeiras. Esta cena julga-se PARADA -- nao
  carregue Play.

  ESQUERDA = o que o no' fazia com o fio LIGADO (um numero so' para todos).
  DIREITA  = o campo a valer PECA A PECA.",
        c = count as u32,
    );
    for (i, label) in conferencia_demos_campo::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) LINHA 1, EMBRULHO: a` esquerda a fileira fica RETA -- o degrade' estava ligado e o
  no' leu `0` dele, entao a curva inteira nao aconteceu. A` direita a fileira ENTRA na curva
  progressivamente: reta de um lado, embrulhada do outro.
  (!) LINHA 2, TRELICA: a` esquerda o favo esta' PERFEITO (jitter zero, com o fio ligado). A`
  direita ele esta' arrumado de um lado e DERRETIDO do outro -- o degrade' a valer.
  (!) LINHA 3, ONDA: a` esquerda as pecas so' ENGORDAM -- crista e vale desenham a mesma
  bolha, e metade da onda e' invisivel. A` direita a altura vai para o Y: a onda SOBE e
  DESCE, e da' para ver onde ela e' negativa.

  DEU ERRADO se alguma metade da direita sair IGUAL a` da esquerda (o campo nao chegou), se a
  linha 1 da direita estiver reta, ou se a linha 3 da direita nao tiver peca nenhuma abaixo
  da linha do meio."
    );
    sinks
}

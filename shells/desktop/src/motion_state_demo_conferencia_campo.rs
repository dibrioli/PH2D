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
    // As fichas no canvas — cada uma pousa sobre a metade que explica (Enio 2026-08-23).
    crate::motion_demo_legend::publish(conferencia_demos_campo::captions());
    let (n, count) = conferencia_demos_campo::authored();
    eprintln!(
        "[cena 83] {n} linhas, {c} peças nas duas primeiras. Esta cena julga-se PARADA — não
  carregue Play. Cada figura tem uma ficha em cima a dizer o que ela é.

  ESQUERDA = o que o nó fazia com o fio LIGADO (um número só, para todos).
  DIREITA  = o campo a valer PEÇA A PEÇA.",
        c = count as u32,
    );
    for (i, label) in conferencia_demos_campo::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  DEU ERRADO se:
    · alguma metade da DIREITA sair igual à da esquerda (o campo não chegou lá);
    · a linha 1 da direita estiver reta (devia entrar na curva aos poucos);
    · a linha 3 da direita não tiver peça nenhuma abaixo da linha do meio (a onda
      tem de descer, e não só engordar).
  E se as fichas não aparecerem sobre as figuras, o defeito é da legenda, não da cena."
    );
    sinks
}

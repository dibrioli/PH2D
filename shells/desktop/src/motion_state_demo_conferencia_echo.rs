//! **A cena do ECO QUE VÊ O FUTURO** (`=88`) — o anúncio, e só ele (doc 89,
//! folha 07, o P1).

use super::*;

/// **A CENA `=88` — O ECO QUE VÊ O FUTURO.**
///
/// ⚠️ **A linha 2 é a que autoriza a linha 3.** Ela desenha a MESMA cauda que a
/// 1 por um caminho completamente diferente (re-cozinhar em vez de lembrar), e é
/// essa igualdade que diz que o modo novo não é outra física — é a mesma cauda,
/// obtida de outra maneira, e por isso ela pode olhar para o outro lado.
pub(crate) fn echo_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_echo::build_echo_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_echo::captions());
    let (n, length, spacing) = conferencia_demos_echo::authored();
    eprintln!(
        "[cena 88] {n} bolinhas a percorrer o MESMO caminho, cada uma com um rastro de
  {length} pecas, uma a cada {spacing} quadros. Deixe correr (ou carregue Play). Cada uma
  tem uma ficha em cima."
    );
    for (i, label) in conferencia_demos_echo::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  O QUE OLHAR: as duas de cima tem de ficar IGUAIS — mesma cauda, no mesmo lugar. A de
  baixo tem o rastro do outro lado: as pecas vao NA FRENTE da bolinha, mostrando por
  onde ela ainda vai passar. Nenhum programa de animacao faz isso sem re-renderizar.

  QUER MEXER? Clique numa peca da linha de baixo. O painel tem uma secao «Source»
  com dois numeros:
    · «Source» — Remembered (lembrar) ou Resampled (recalcular);
    · «Forward Steps» — quantas pecas vao na frente. Arraste de {fwd} para 0 e a
      cauda vira para tras; ponha no meio e ela fica dos dois lados.

  ⚠️ So' funciona sobre um caminho CALCULADO (uma formula, uma curva). Sobre uma
  simulacao (uma corda, uma gelatina, um bando) o programa recusa em vez de inventar:
  ninguem sabe onde uma simulacao vai estar daqui a meio segundo sem a rodar ate' la'.

  DEU ERRADO se:
    · as linhas 1 e 2 nao ficarem iguais;
    · a linha 3 tiver o rastro atras da bolinha, como as outras;
    · alguma cauda piscar, encolher sozinha ou aparecer so' depois de uns segundos;
    · arrastar a regua do tempo para tras deixar a linha 2 ou a 3 diferente do que
      estava (elas nao guardam nada — tem de rebobinar exatas).",
        fwd = length - 1
    );
    sinks
}

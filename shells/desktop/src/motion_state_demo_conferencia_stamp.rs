//! **A cena do VOCABULÁRIO DO CARIMBO** (`=98`) — o anúncio (folha 08).

use super::*;

/// **A CENA `=98` — A COR QUE SUMIA, E A ORDEM DENTRO DE GRUPOS.**
pub(crate) fn stamp_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_stamp::build_stamp_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_stamp::captions());
    eprintln!(
        "[cena 98] Tres pares. ⚠️ ESTA CENA E' PARADA -- nao precisa de Play.
  Cada bloco tem uma ficha em cima a dizer o que ele e'.

  ESQUERDA = como era.  DIREITA = o controlo novo.
  As duas primeiras fileiras sao a MESMA cena: uma peca carimbada 16 vezes ao longo de
  uma fila, e a fila e' que tem a cor. A terceira e' uma fila baralhada."
    );
    for (i, label) in conferencia_demos_stamp::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER?

    · Clique num bloco das DUAS PRIMEIRAS fileiras e procure «Transfer» no painel do
      «Duplicator»: Shape Wins (a cor do arranjo e' deitada fora -- o que sempre
      aconteceu) · Point Wins · Add · Multiply.
    · No bloco de baixo a' DIREITA, o que muda e' que a entrada «group» do «Sort» esta'
      ligada. Mexa em «Step» no «Quantize» para partir a fila em mais ou menos grupos.

  DEU ERRADO se:
    · a direita de alguma linha ficar igual a' esquerda;
    · a fileira 1 a' ESQUERDA mostrar uma rampa (ali as 16 copias tem de sair IGUAIS);
    · a fileira 3 a' DIREITA nao mostrar quatro faixas de cor;
    · as duas bandas «Point Wins» (linhas 2 e 3, a` direita e a` esquerda) sairem
      diferentes uma da outra;
    · algum bloco invadir o vizinho ou sair do sitio;
    · alguma peca desaparecer ou ficar do tamanho do ecra."
    );
    sinks
}

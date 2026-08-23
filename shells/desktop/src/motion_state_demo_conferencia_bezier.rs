//! **A cena da FRONTEIRA CURVA** (`=86`) — o anúncio, e só ele (doc 89, folha 04).

use super::*;

/// **A CENA `=86` — A FRONTEIRA CURVA.**
///
/// ⚠️ **A primeira linha é a que decide**, e é por isso que ela vem primeiro: os dois nós
/// recebem os MESMOS quatro cantos, e o que os separa aparece só no MIOLO do bloco. Uma
/// fileira de peças (a fixture das cenas irmãs) não teria miolo, e as duas metades sairiam
/// iguais.
pub(crate) fn bezier_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_bezier::build_bezier_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_bezier::captions());
    let (n, side) = conferencia_demos_bezier::authored();
    eprintln!(
        "[cena 86] {n} linhas, um bloco de {s}×{s} peças em cada metade. As duas se julgam
  PARADAS — não carregue Play. Cada figura tem uma ficha em cima a dizer o que ela é.

  ESQUERDA = o deformador que já existia.
  DIREITA  = o novo.",
        s = side as u32,
    );
    for (i, label) in conferencia_demos_bezier::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  O QUE OLHAR NA LINHA 1: as QUATRO PONTAS das duas metades estão no mesmo lugar — só um
  canto foi puxado, e igual nos dois. Olhe as FILEIRAS DE DENTRO do bloco: à esquerda elas
  continuam retas, à direita elas se curvam. É essa a diferença inteira entre os dois, e é
  por ela que são duas ferramentas e não uma com mais botões.

  QUER MEXER? Clique numa peça da direita para o painel mostrar o nó. Ele tem cinco
  seções: «Corners» e uma por borda («Top Edge», «Right Edge», «Bottom Edge», «Left
  Edge»). Arraste um número de «Top Edge» e a borda de cima entorta sozinha.

  ⚠️ O painel dele é ALTO — 24 números. Ele abre já precisando da roda do mouse, e isso é
  esperado: são os 12 pontos que a curva da borda tem.

  DEU ERRADO se:
    · na linha 1 as fileiras de dentro estiverem retas nas DUAS metades;
    · as quatro pontas não coincidirem entre as duas metades da linha 1;
    · na linha 2 a borda de cima da direita não estiver curvada;
    · o painel do nó novo não tiver as cinco seções."
    );
    sinks
}

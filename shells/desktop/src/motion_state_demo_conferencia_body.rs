//! **A cena do CORPO QUE NÃO É UM RETÂNGULO** (`=87`) — o anúncio, e só ele
//! (doc 89, folha 03, o P1).

use super::*;

/// **A CENA `=87` — O CORPO QUE NÃO É UM RETÂNGULO.**
///
/// ⚠️ **A primeira coluna é o CONTROLE e vem primeiro de propósito:** ela é a
/// gelatina de sempre, com a porta nova vazia. Se ela balançar diferente, a wave
/// partiu o que já existia — e é a única das três que se pode comparar com o que
/// o Enio já viu.
pub(crate) fn body_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_body::build_body_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_body::captions());
    let (n, side) = conferencia_demos_body::authored();
    eprintln!(
        "[cena 87] {n} gelatinas penduradas do mesmo mastro, que varre sozinho. Carregue Play
  (ou deixe correr) — elas balançam e VOLTAM à forma. Cada uma tem uma ficha em cima.

  A da ESQUERDA é a de sempre: um retângulo de {side}×{side}.
  As outras duas têm a forma que o grafo lhes deu."
    );
    for (i, label) in conferencia_demos_body::col_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  O QUE OLHAR: as três balançam do mesmo jeito, mas o ANEL continua um anel (com o
  buraco no meio) e a CRUZ continua uma cruz, por mais que chacoalhem. É essa a
  novidade — antes, uma gelatina só podia ser um retângulo.

  QUER MEXER? Clique numa peça da cruz para o painel mostrar o nó. Ele tem três
  seções: «Mesh» (a forma que ele usa quando ninguém lhe dá uma), «Physics» e
  «Pin». Baixe o «Stiffness» em Physics e ela fica mole; suba e ela endurece.

  ⚠️ Os números de «Mesh» não fazem nada na coluna do anel nem na da cruz — nessas
  duas a forma vem de fora, e é por isso que eles ficam onde estão em vez de sumir:
  desligar o fio devolve o retângulo que eles descrevem.

  DEU ERRADO se:
    · as três forem retângulos;
    · o anel perder o buraco, ou a cruz virar um bolo;
    · alguma ficar pendurada por um ponto só, em vez de pela borda de cima;
    · alguma cair e não voltar (isso é uma nuvem de pontos, não um corpo mole)."
    );
    sinks
}

//! **A cena do EMISSOR QUE DEIXA RASTO** (`=89`) — o anúncio, e só ele (doc 89,
//! folha 01, o P1).

use super::*;

/// **A CENA `=89` — O EMISSOR QUE DEIXA RASTO.**
///
/// ⚠️ **A primeira linha é o CONTROLE e é o que sempre houve** — e ela continua a
/// ser uma resposta legítima, não um bug com nome bonito: um efeito ANEXADO (a
/// chama que anda com a tocha) quer exactamente que o penacho ande junto.
pub(crate) fn plume_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_plume::build_plume_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_plume::captions());
    let (n, rate, life) = conferencia_demos_plume::authored();
    eprintln!(
        "[cena 89] {n} fontes iguais, varridas de um lado ao outro pelo MESMO relogio.
  Cada uma cospe {rate} particulas por segundo, que vivem {life:.1} s. Deixe correr."
    );
    for (i, label) in conferencia_demos_plume::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  O QUE OLHAR: a de CIMA carrega o penacho inteiro para os lados — as particulas velhas
  andam junto com a fonte, como se estivessem coladas nela. A do MEIO deixa um rasto no
  ar: cada particula fica onde nasceu, e o varrimento desenha uma faixa. A de BAIXO faz
  o mesmo E inclina o jacto para o lado da marcha, como agua a sair de uma mangueira que
  se move.

  QUER MEXER? Clique numa peca e procure «Emitter Motion» na secao Velocity. Troque
  entre Carry / Leave / Inherit e veja as tres na mesma linha. Com Inherit aparece
  «Inherit Strength» ao lado — baixe para 0 e o jacto volta a sair a prumo.

  ⚠️ So' vale quando a FONTE SE MEXE. Com o emissor parado os tres modos desenham a
  mesma coisa, porque nao ha' historia nenhuma para lembrar — e e' por isso que esta cena
  varre.

  DEU ERRADO se:
    · as tres linhas ficarem iguais;
    · a do meio carregar o penacho junto, como a de cima;
    · a de baixo nao inclinar;
    · alguma piscar, ou as particulas aparecerem todas amontoadas num ponto so'."
    );
    sinks
}

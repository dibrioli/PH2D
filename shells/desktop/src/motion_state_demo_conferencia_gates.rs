//! **A cena da CURA DOS KNOBS MORTOS** (`=82`) — o anúncio, e só ele (doc 90).
//!
//! ⚠️ **Mora sozinha, e o corte é por ASSUNTO — o critério que a irmã
//! [`utilidade`](super::utilidade) já declara no cabeçalho dela.** As cenas de lá respondem
//! todas a uma FOLHA da conferência (doc 89: *que controle falta a este nó contra a
//! referência*); esta responde a outra pergunta — *que controle o painel PINTA e que não faz
//! nada* — e a resposta dela é o doc 90.
//!
//! ⚠️ O corte foi FORÇADO pelo teto de 600 LOC (HR-18) quando o anúncio entrou na irmã, e a
//! cura de um teto é um split. Mas *onde* cortar não é do teto: um split por «o que entrou
//! hoje» envelhece na semana seguinte, e um por assunto não.

use super::*;

/// **A CENA `=82` — O PAINEL QUE ENCOLHE** (doc 90: a cura dos dezanove knobs mortos).
///
/// ⚠️ **O oráculo é CONTAR LINHAS.** Cada célula desenha o MESMO controle duas vezes ao mesmo
/// tempo — no mínimo e no máximo da faixa. Onde ele é mudo as duas cópias coincidem ao bit e
/// vê-se **uma** linha; onde ele age, separam-se e vêem-se **duas**. É a única pergunta que a
/// cura faz, e ela não precisa de apreciação nenhuma para ser respondida.
///
/// ⚠️ Ela mora aqui, com a `=81`, e não com as cinco do `..._grafico.rs`, pela mesma razão da
/// irmã: **o que se lê não é a altura de uma curva, é quantas figuras há**.
pub(crate) fn gate_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_gates::build_gates_demo_document(doc, registry).unwrap_or_default();
    // As fichas no canvas — uma por metade, em cima da figura que ela nomeia.
    crate::motion_demo_legend::publish(conferencia_demos_gates::captions());
    let (n, count) = conferencia_demos_gates::authored();
    eprintln!(
        "[cena 82] {n} linhas, {c} peças por figura. Esta cena julga-se PARADA — não carregue
  Play. (A linha 6, do TREMOR, é a única que anda se o relógio andar; parada, ela lê-se
  como as outras.) Cada figura tem uma ficha em cima a dizer o que ela é.

  O QUE CONTAR: cada metade desenha o MESMO controle DUAS vezes — no mínimo e no máximo.
  À ESQUERDA o controle está num modo onde ele NÃO FAZ NADA, então as duas cópias caem uma
  em cima da outra e vê-se UMA linha. À DIREITA ele AGE, e vêem-se DUAS.",
        c = count as u32,
    );
    for (i, label) in conferencia_demos_gates::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  E HÁ UMA METADE QUE NÃO SE DESENHA, que é a cura: clique no nó de cada linha e olhe o
  painel. Do lado ESQUERDO a linha daquele controle tem de estar AUSENTE — foi isso que se
  consertou. Do lado DIREITO ela tem de estar lá.

  DEU ERRADO se:
    · alguma metade ESQUERDA mostrar duas linhas (o controle age e foi escondido — é o
      pior caso de todos);
    · alguma DIREITA mostrar uma só (o controle continua mudo);
    · o painel do lado direito não tiver a linha do controle."
    );
    sinks
}

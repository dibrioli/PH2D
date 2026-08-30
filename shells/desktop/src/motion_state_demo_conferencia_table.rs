//! A cena `=109` — a tabela, lida das duas maneiras que a indústria distingue.

use super::*;

pub(crate) fn table_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_table::build_table_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[cena 109] Uma TABELA de dados, lida das DUAS maneiras. Ficheiro escrito em:
    {}

  EM CIMA -- cada linha do ficheiro e' UM PONTO. Doze meses, doze pontos, e a ALTURA de
  cada um vem da coluna «vendas». E' um grafico, feito com dois nos: `source.table`
  entrega as linhas, e `value.attribute` escolhe a coluna pelo NOME.

  EM BAIXO -- cada linha do ficheiro e' UM INSTANTE. ⚠️ DE' PLAY. O quadrado muda de
  tamanho seguindo a coluna «nivel» ao longo da coluna «tempo». Parado, ele nao diz
  nada -- e' o playhead que o faz ler a tabela.

  ⭐ A coluna «mes» tem PALAVRAS, e por isso nao entra: um leitor que a convertesse
  daria uma coluna de zeros sem avisar. Ela e' saltada e nomeada.

  ⭐ PARA RECARREGAR: edite o ficheiro acima, guarde, e escolha-o outra vez no botao
  «Table File» do painel. Escolher e' recarregar.",
        conferencia_demos_table::fixture_path().display()
    );
    sinks
}

//! **A cena de TODO O DUPLICATOR** (`=110`) — o anúncio (pedido do Enio, 2026-09-05).

use super::*;

/// **A CENA `=110` — O CARIMBO INTEIRO, TREZE BANDAS.**
pub(crate) fn dup_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_dup::build_dup_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_dup::captions());
    eprintln!(
        "[cena 110] TUDO O QUE O DUPLICATOR FAZ, numa tela. ⚠️ ESTA CENA E' PARADA -- nao
  precisa de Play. Cada bloco tem uma ficha em cima a dizer o que ele e'.

  O Duplicator CARIMBA uma forma em cada ponto de um arranjo: a esquerda do no' recebe
  O QUE desenhar (um «Shape»), a direita recebe ONDE (um «Grid»). As quatro fileiras
  sao quatro perguntas.

  ⭐ CADA BLOCO DA TELA TEM A SUA PROPRIA CADEIA DE NOS, e elas nao se tocam: no grafo
  sao TREZE ILHAS empilhadas, cada uma a ler-se da esquerda para a direita. A ficha de
  cada bloco comeca pelo NUMERO dele, que e' o mesmo da lista aqui em baixo.

  PARA OLHAR UMA DE CADA VEZ: clique num no' da cadeia e aperte F -- o grafo enquadra
  o que esta' selecionado. Sem selecao, F volta a mostrar tudo."
    );
    for (i, label) in conferencia_demos_dup::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  AS QUATRO FILEIRAS

    1  O QUE O CARIMBO E'.  A forma inteira pousa em cada ponto: a posicao do ponto
       SOMA-SE a' da forma (por isso o cometa chega inteiro) e o giro tambem.
    2  PICK -- qual forma pousa em qual ponto. Sem ele, TODAS pousam em TODOS.
    3  POINT SCALE -- quanto da escala do PONTO entra no tamanho da copia.
    4  TRANSFER -- de quem e' a cor quando a forma E o ponto tem cor.

  QUER MEXER?

    · Clique num «Duplicator» e escreva/arraste os controlos dele. Em «Pick», clicar
      troca de opcao; a «Seed» so' aparece em Random, que e' o unico modo que a le^.
    · As tres bandas do Pick carimbam AS MESMAS tres formas, e as quatro do Transfer a
      mesma forma cinzenta -- mas cada uma tem a SUA copia dos nos. Se mexer numa forma
      e quiser comparar de novo, faca a mesma mexida nas irmas da fileira.
    · Clique num «Shape» e troque o campo «Kind»: sao 43 silhuetas, e a forma nova e'
      carimbada em todos os pontos na hora. A geometria e' VETORIAL -- aproxime o zoom
      e a borda continua nitida.
    · Puxe o fio do «points» de qualquer banda: a forma passa INTACTA -- um carimbo sem
      onde carimbar deixa a corrente passar, e e' isso que torna seguro enfiar um
      Duplicator no meio de uma cadeia. Puxe o do «shape» e a banda esvazia (sem o que
      carimbar nao ha nada). Nos dois casos o no' ganha um aviso, e ele EXPLICA: as duas
      entradas sao pedidas, e o aviso diz qual falta -- ele nao bloqueia a cena.

  O QUE E' PRECISO PARA CADA CONTROLO FALAR (medido)

    · Pick        precisa de DUAS ou mais formas na entrada de cima. Com uma forma so',
                  as tres opcoes sao a mesma coisa -- nao e' defeito, e' a definicao.
    · Point Scale precisa que os PONTOS tragam escala propria. Sem isso nao ha o que
                  compor, e o controlo nao move nada.
    · Transfer    precisa da MESMA cor nos dois lados. Uma cor que so' o ponto tem ja'
                  chega em todos os modos -- e' o Transfer que decide quem ganha quando
                  os dois a tem.

  (i) A banda 2 e a banda 4 sao a MESMA lei vista de dois lados: «uma forma feita de tres
      pecas» e «tres formas alternativas» sao a mesma corrente para este no'. O que as
      separa e' o Pick, que trata cada peca como uma candidata.
  (i) A numeracao das copias corre CONTINUA pelo conjunto todo (nao reinicia a cada
      copia), e e' isso que faz uma rampa posta DEPOIS do carimbo atravessar as 15 pecas.

  DEU ERRADO se:
    · alguma das tres bandas do Pick sair igual a' vizinha;
    · a banda «Pick = Off» nao tiver 15 pecas (3 alturas x 5 colunas);
    · as tres bandas do Point Scale sairem do mesmo tamanho;
    · «Shape Wins» mostrar a rampa, ou «Point Wins» mostrar cinco copias cinzentas;
    · «Multiply» nao ficar mais escuro que «Point Wins», ou «Add» mais claro;
    · a fila da banda 3 nao torcer da esquerda para a direita;
    · algum bloco invadir o vizinho, sair do sitio, ou ficar vazio."
    );
    sinks
}

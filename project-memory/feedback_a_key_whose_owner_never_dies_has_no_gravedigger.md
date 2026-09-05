---
name: feedback_a_key_whose_owner_never_dies_has_no_gravedigger
description: "Limpeza que corre na DESTRUIÇÃO não cobre quem nunca é destruído — a chave fica nem viva nem sepultada, e isso lê-se como ausência"
metadata:
  type: feedback
---

Um sistema que guarda estado por-chave costuma ter **duas** saídas para uma chave: ela continua
**viva** (alguém a resolve todo passe), ou é **sepultada** (uma limpeza a arquiva no instante em que
o alvo morre). Quando um alvo **nunca morre**, a segunda saída não corre — e uma chave que perde a
correspondência fica **numa terceira posição que ninguém desenhou: invisível**.

Medido em 2026-09-05 (`line/components`, F5). Trocar o componente de uma cópia re-chaveia as
excepções do artista por um mapa `peça velha → peça nova`. As peças que não emparelham são apagadas
pelo passe estrutural, e o `entomb` serializa a excepção de cada uma **no instante em que a peça
morre**. Mas a **raiz** de uma instância nunca morre numa troca — o que muda é o elo dela. ⇒ deixar
a raiz fora do mapa produzia uma chave que:

- **não é viva** — o passe compara com a raiz do mestre NOVO, e a chave aponta para a do velho;
- **não é sepultada** — o sepultador só corre sobre quem vai morrer;
- e **bloqueia** — o passe salta o que a instância «possui», então a receita nova nunca mais
  alcançava aquele componente da cópia, sem uma linha em painel nenhum a dizê-lo.

**Why:** o desenho da limpeza estava certo e a POPULAÇÃO dele é que era menor do que a dos donos de
chave. Um censo de «quem morre» não é um censo de «quem pode perder a correspondência», e a
diferença é exactamente a entidade que sobrevive à operação. O sintoma para o utilizador não é um
erro: é uma edição da receita que deixa de chegar, que se reporta como *«mudei o componente e nada
aconteceu»* — a mesma frase de quatro defeitos diferentes deste módulo.

**How to apply:** ao escrever um re-chaveamento, liste os donos de chave e pergunte de cada um *«se
ele não emparelhar, QUEM enterra a chave dele?»*. Se a resposta for «ninguém, porque ele não morre»,
ele tem de emparelhar **sempre** — e isso é uma lei do mecanismo, não uma escolha de gosto (aqui ela
contradizia o nome do modo, *«não leves nada»*, e ganhou). ⚠️ O gate tem de medir o **produto**: a
excepção sobrevive **e** continua endereçável pela chave nova; medir só a forma do mapa deixa passar
a metade que o painel lê. Ver [[feedback_a_gate_that_asks_the_producers_state_is_blind_to_a_second_producer]].

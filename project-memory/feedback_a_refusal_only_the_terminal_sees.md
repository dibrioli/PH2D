---
name: a-refusal-only-the-terminal-sees
description: Uma recusa cujo doc diz «a frase que o artista lê» e que sai por eprintln! é um botão mudo — o terminal não é superfície do produto
metadata:
  type: feedback
---

⛔⛔ **O doc de `RemeshRefusal::explain` chamava à frase dele *«a FRASE que o artista lê»*, e os três
sítios que a mostravam eram `eprintln!`.** O botão `Quad Retopology` recusava com a cura escrita
dentro da frase (*ACHATE a pilha antes*, *baixe o Detail*) e no ecrã não acontecia nada —
indistinguível de um botão partido.

**Why:** o dono corre o smoke a partir de um terminal, logo *quem escreve o código vê a frase* e
conclui que ela chega. O artista não tem terminal. Nenhuma sonda deste repo pergunta *«esta frase
chega a um PIXEL?»* — os gates de registo provam que o clique chega à ferramenta, nunca que a
resposta dela chega ao olho.

**How to apply:** quando um doc-comment disser que um texto é *lido pelo artista*, siga o texto até
à superfície. Se ele acaba num `eprintln!`, a cura é uma **caixa de saída** (a cena publica, a shell
drena para a fila de avisos — ADR-0075: publicar, não chamar), com o `eprintln!` a ficar **dentro da
porta**: o terminal é de quem bisseca, o ecrã é de quem trabalha. Uma porta só, senão uma das duas
metades envelhece. Gate: `a_refusal_the_artist_can_read_leaves_the_terminal` (2026-09-16), que mede
*nenhum `.explain()` fora da porta* e *a shell drena*. Ver
[[a-badge-is-painted-text-and-the-key-of-its-own-colour]].

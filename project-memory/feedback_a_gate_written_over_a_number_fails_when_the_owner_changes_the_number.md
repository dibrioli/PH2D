---
name: a-gate-written-over-a-number-fails-when-the-owner-changes-the-number
description: «O rótulo é a menor das duas colunas» era a lei da fracção 0,348 disfarçada de invariante — e reprovou sobre o desenho CERTO no dia em que o dono partiu a linha ao meio.
metadata:
  type: feedback
---

Um gate que afirma uma propriedade **derivada de um número escolhido** não defende a lei: defende o
número. No dia em que alguém com autoridade muda o número — o dono, uma medição nova, um redesenho —
o gate reprova sobre o produto CERTO, e o reflexo é relaxá-lo em vez de o reescrever.

**Caso medido (`line/UIUX`, 2026-09-14).** A coluna do rótulo era `0,348` da linha, e escrevi um
gate com duas metades. A primeira era a lei (*o controlo nunca fica abaixo do piso nomeado*) e
sobreviveu a tudo. A segunda dizia *«numa linha larga o rótulo é a MENOR das duas colunas — senão a
fracção deixou de ser uma fracção»* — e isso era verdade **porque `0,348 < 0,5`**, não porque uma
linha de propriedade tenha de ser assim.

O dono pediu a partição ao meio (*«alinhar no meio do painel»*). O rótulo passou a `120` e o controlo
a `114` (a coluna de animação sai do lado do controlo), e a segunda metade reprovou sobre exactamente
o desenho pedido. Substituída pela lei que o dono enunciou: *o controlo começa no meio da linha*.

**Why:** ao escrever um gate é fácil colher uma consequência aritmética do valor de hoje e lê-la como
invariante — ela passa, mata mutações, e parece uma lei. O discriminador é perguntar **quem pode
mudar isto legitimamente**: se a resposta é *o dono* ou *uma medição*, então a propriedade é do
número, não do desenho.

**How to apply:** escreva a asserção com as palavras com que a LEI se diz (*«o controlo começa no
meio»*, *«o controlo nunca fica abaixo do piso»*), nunca com uma comparação entre duas grandezas
cujo sinal depende do valor corrente. E quando um gate reprovar logo a seguir a uma decisão do dono,
a primeira pergunta é *«que propriedade é que ele estava mesmo a defender?»* — não *«como é que eu o
afrouxo?»*.

Vizinhos: [[feedback-a-gate-that-compares-two-constructions-is-blind-to-a-shared-mutation]] ·
[[feedback-a-wave-that-changes-a-surface-invalidates-the-designs-that-sat-on-it]] ·
[[feedback-an-inequality-accepts-a-whole-interval-only-an-oracle-accepts-an-answer]] ·
[[reference-topic-gate-discipline]]

---
name: feedback-a-gate-that-reproduces-the-formula-pins-the-formula-not-the-product
description: "Um gate que RECALCULA o que o produto calcula continua verde (ou continua a acusar) depois de o produto mudar de fórmula — pergunte ao produto, ou o gate mede código que já não corre"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T22:52:37.403Z
---

Quando um gate precisa de um número que o produto também calcula, ele tem duas formas: **perguntar
ao produto** ou **reproduzir a conta**. A segunda parece equivalente e não é: no dia em que o
produto muda de fórmula, o gate continua a medir a **antiga** — e, pior, continua **verde**.

**Medido na `line/UIUX`, 2026-08-31 (entrega 32):** o gate do orçamento de ecrã
(`the_chrome_never_eats_more_of_a_tablet_than_this`) calculava a altura da fila de ferramentas
chamando `tool_bar_lines(...)` e multiplicando, tal como o `frame_layout` fazia. A wave seguinte
fixou a faixa em **uma** linha (o excesso foi para um `⋯`) — e o gate continuou a imprimir
`40,8 %` e `37,6 %`, os números de **duas** linhas, depois de a cura estar no produto. O ganho de
`+3,3` pontos ficou invisível.

> *Um gate que reproduz a fórmula pina a fórmula, não o produto.*

⭐ **E o que o destapou foi o TECTO da catraca:** ao apontar o gate ao produto, duas células
saltaram `3` pontos e a metade de **obsolescência** reprovou, obrigando a descer os pisos em vez de
os deixar como folga silenciosa. *Uma catraca só com piso teria absorvido o ganho e continuado a
defender um número que já não descrevia nada.*

**Why:** o gate e o produto partilham a intenção, não o código. A intenção não compila.

**How to apply:** ao escrever um gate que precisa de uma geometria/tamanho/limite que o produto
resolve, chame **a função do produto**. Se ela for privada, torne-a pública com o motivo escrito —
é mais barato do que uma cópia. E se a cópia for inevitável, ponha um assert a comparar as duas.

Relacionadas: [[feedback_a_gate_that_compares_two_constructions_is_blind_to_a_shared_mutation]] ·
[[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]] ·
[[feedback_a_permanent_band_must_return_more_screen_than_it_eats]]

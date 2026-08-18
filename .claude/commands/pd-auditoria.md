---
description: Lentes independentes + varredura de seam.
argument-hint: [Alvo (crate/wave/diff)]
---
Faça auditoria de 2 lentes sobre $1.

Lente 1 — CORREÇÃO: o código faz o que o doc/commit afirma? Procure especificamente
  por dois lugares que devem concordar sobre um fato e discordam.
Lente 2 — COSTURA DE UI: todo widget é pintado, populado E clicável? Varra os seams
  (o clique tem de chegar ao barramento) e a SEQUÊNCIA tem de levar a algum lugar.

Para cada achado, entregue: mecanismo · como reproduzir · o gate que faltava.
Não conserte antes de listar. Se um gate existente estava verde sobre o achado, diga
qual e por que ele era verde (fixture sem o fenômeno? oráculo que usa a função sob
teste? razão entre dois doentes?).

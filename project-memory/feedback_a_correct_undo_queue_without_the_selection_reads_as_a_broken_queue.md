---
name: feedback-a-correct-undo-queue-without-the-selection-reads-as-a-broken-queue
description: Uma fila de undo exacta sem a SELEÇÃO lê-se, de fora, exactamente como uma fila partida — a marca da mão pertence ao PASSO, não ao transporte.
metadata:
  type: feedback
---

Enio, 2026-09-04: *«o undo/redo está completamente destruído»* — o quarto report seguido sobre o
mesmo módulo, depois de três jornadas terem curado três defeitos **reais** do registo de passos.
A sonda auto-dirigida mostrou a fila **exacta** em todos os gestos (arrasto do gizmo, criação pela
paleta, modificador por chip: 2 passos, 2 `Ctrl+Z`, 1 `Ctrl+Shift+Z`, todos certos) — e mostrou
isto:

```text
f=30 criar        nos=4→5  sel=Some(..)  setas=3
f=58 Ctrl+Z       nos=5→4  sel=None      setas=3
f=60                                     setas=0   ⛔ o gizmo morreu
f=66 Ctrl+Shift+Z nos=4→5  sel=None      setas=0   ⛔ e nao volta mais
```

A selecção era **transportada** através do restauro (ler o que está escolhido, restaurar, devolver o
que sobreviveu). Isso é correcto para uma **edição** e falso para uma **criação**: desfazer apaga a
selecção porque o objecto deixou de existir, e **refazer transporta a selecção de agora, que está
vazia**. A peça volta sem gizmo, sem painel e sem alças — e daí em diante todo `Ctrl+Z` *parece não
fazer nada*, porque não há nada visível para mudar.

**Why:** um utilizador não observa a fila; observa o que ele consegue fazer a seguir. Uma fila
perfeita cujo restauro não devolve a MÃO é indistinguível de uma fila partida, e manda a
investigação para o sítio errado — três jornadas, no meu caso.

**How to apply:** a selecção **pertence ao passo** e viaja ao lado dele na fila, em identidade
durável (`StableId`), nunca dentro da unidade comparada — pô-la no estado faria cada clique de
escolha registar um passo e o save guardar quem estava escolhido. E a marca do baseline substitui-se
**exactamente** onde o baseline se substitui: quando um passo nasce **e** no restauro.
Ver [[feedback_a_decision_log_that_omits_one_key_explains_every_choice_but_the_one_that_matters]].

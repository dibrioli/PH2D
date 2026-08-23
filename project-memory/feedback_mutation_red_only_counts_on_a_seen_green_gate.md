---
name: feedback-mutation-red-only-counts-on-a-seen-green-gate
description: "Mutação-RED só prova algo sobre um gate JÁ VISTO VERDE no código corrigido — um gate vermelho nos DOIS mundos 'sangra' toda mutação e não testa nada; o ciclo é verde(fix) → red(mutação) → verde(restore)"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 92714982-3cf5-48f6-96d6-acbdbe13b4f5
---

Na fase D do Painter (2026-07-15), escrevi o gate do handoff GPU→CPU (`a_gpu_lane_drain_leaves_no_partial_lane_behind`), apliquei a mutação A e vi RED — e quase registrei a prova. O gate estava vermelho **nos dois mundos**: a fixture era um stack TRIVIAL, onde `bbox=Some` é o comportamento correto da via zero-copy (o Arc é o próprio canvas, nunca stale) — o vermelho não tinha relação com a mutação. Eu nunca tinha rodado o gate com o fix em pé.

**Why:** a prova de mutação é um CONTRASTE (verde com o fix, vermelho sem). Rodar só o lado vermelho colapsa o contraste: qualquer gate quebrado, mal-fixturado ou impossível "sangra" toda mutação e vira prova falsa. É o dual de [[feedback_check_the_oracle_is_achievable_before_writing_the_gate]] — lá o prescrito podia ser impossível; aqui o vermelho podia ser incondicional.

⚠️ **REINCIDIU em 2026-08-23 (`line/3DModeling` W50), com um arnês automatizado — e é aí que está a
lição nova.** O gate de costura chamava `note_safe` de dentro de um `with_smoke`, e essa função entra
pelo mesmo `with_smoke`: `RefCell` re-entrante, pânico **sempre**. O arnês imprimiu
`RED (recompilou: True · correu 1 teste: True)` — os **dois** controles positivos que as waves
anteriores tinham pago (o binário é novo · um teste correu) passaram, e mesmo assim o veredito era
falso. ⇒ **a regra tem de estar NO ARNÊS, não na cabeça de quem o corre**: ele corre o teste antes de
mutar, exige verde, e imprime `verde antes: True`. Com isso, uma mutação que de facto sobrevivia
apareceu — e virou um ⏸️ declarado em vez de uma prova inventada.

**How to apply:** o ciclo completo, sempre nesta ordem: (1) rode o gate com o fix — VERDE (se vermelho, a fixture não contém o fenômeno ou o oráculo está errado; no meu caso a fixture precisava de stack não-trivial); (2) aplique a mutação — RED; (3) restaure — VERDE de novo (pega restore quebrado, que já aconteceu via `git checkout` reflexo). Só então o placar conta a mutação. Irmãos: [[feedback_mutate_the_code_not_just_the_test]] (mute o código), [[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] (o inverso: verde-sob-mutação).

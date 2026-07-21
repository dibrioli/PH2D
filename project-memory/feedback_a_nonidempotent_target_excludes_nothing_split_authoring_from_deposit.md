---
name: a-nonidempotent-target-excludes-nothing-split-authoring-from-deposit
description: "Erro de avaliação no Wet Paint — \"depósito não-idempotente ⇒ só métodos incrementais\" era falso; o momento do depósito é escolha de design"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 614b83f1-60a4-4540-9888-0d66e196d6cb
  modified: 2026-07-21T23:17:22.940Z
---

No Wet Paint (2026-07-21) eu escondi DragDot/Anchored/Line e os shape editors do modo fluido
com a lei *"o depósito não é idempotente, então métodos que re-stampam por frame empilhariam
tinta"* — e o Enio derrubou: **o re-stamp é AUTORIA (preview flat, estático); o depósito no
alvo não-idempotente pode acontecer UMA vez, no commit** (mouse-up / Enter). A
não-idempotência do alvo não exclui nenhum método de autoria — ela só fixa ONDE o funil
deposita.

**Why:** eu colapsei duas fases com cadências diferentes (autoria re-emite a lista inteira por
frame; depósito consome a lista final uma vez) numa propriedade só, e "escondi por
incompatibilidade" (lei #3) o que era compatível com um deslocamento de momento. O esconderijo
tinha gates de presença/ausência verdes — gate não valida a PREMISSA da decisão de produto.

**How to apply:** antes de declarar um método/controle incompatível com um alvo
não-idempotente (sim, acumulador, fluido), pergunte: *a incompatibilidade é da AUTORIA ou do
DEPÓSITO?* Se só do depósito, a cura é um funil de commit único (deposit-at-commit), não
esconder. Relacionado: [[feedback_inherited_affordance_must_be_rederived]],
[[feedback_ergonomics_verdict_is_a_design_bug]].

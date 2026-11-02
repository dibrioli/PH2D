---
name: feedback_always_hand_the_full_continuation_prompt
description: "Dono 25/10 — ao trocar de janela entregue SEMPRE o prompt de continuação completo, sem ele pedir — mas só DEPOIS do smoke (handoff leva o veredito)"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 682227a6-5cf9-4a9e-82a2-c9600f44313f
  modified: 2026-10-25T10:14:09.885Z
---

Quando a janela deve trocar (fim de onda, conversa grande, aviso do `tamanho-da-conversa.sh`), a resposta ao dono
TERMINA com o **prompt de continuação completo** num bloco pronto a colar — o modelo de
`docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md` preenchido com a linha, a worktree, o handoff a ler
e a tarefa seguinte. Nunca só «abra uma conversa nova».

**Why:** dono 25/10: *«Guarde como regra: quando for a hora de trocar de janela sempre forneça o prompt de
continuação completo sem que eu precise solicitar»* — ele teve de pedir o prompt depois de lhe dizerem para trocar.

⛔ **Mas SÓ DEPOIS DO SMOKE** (dono 25/10, F29: *«o handoff/prompt para nova janela só deve ser escrito após o
smoke»*): o fim de onda entrega o smoke e PÁRA; o handoff de continuação e o prompt saem quando o dono disser o
resultado — o handoff leva o veredito do smoke (e os reports dele), que é o que a janela seguinte precisa.

**How to apply:** depois do «smoke OK»/report do dono, escreva/comite o handoff de continuação com o resultado, e
cole o bloco no fim dessa resposta. Ver [[feedback_one_feature_one_line_minimum_windows]].

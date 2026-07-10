---
name: harness-reproduces-mechanism-not-context
description: "Smoke \"não mudou nada\" após fix provado no harness = o harness reproduz o MECANISMO, não o CONTEXTO do app; 1 eprintln por-evento no app real fecha o que auditoria estática não fecha"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 4d0d1aab-131f-444b-9cc0-a338b3b72ecf
---

Saga aquarela 2026-07-09: TRÊS fixes de junção (retângulo, laterais, painel 3-lentes), todos
provados refutáveis no harness — e o smoke do Enio: "nenhuma diferença", três vezes. A raiz: os
fixes eram todos INTRA-sessão molhada; no app real o timer de secagem expirava na pausa natural
de olhar o resultado, o traço seguinte abria sessão nova e GLAZEAVA com borda dura by-design —
um caminho que o harness (ticks sintéticos, sem pausas reais) nunca exercitava.

**Why:** teste unitário reproduz o mecanismo que você IMAGINA estar ativo; se o estado-portão
(sessão/modo/cache/timer) difere no app real, você mata mecanismos inocentes em série. Painel
de 3 lentes de auditoria estática também não pegou — todas as lentes olharam o composite,
nenhuma questionou SE o caminho rodava.

**How to apply:** smoke contradiz fix provado no harness ⇒ PARE de iterar mecanismo; instrumente
a DECISÃO-PORTÃO no app real (1 eprintln por evento com cada condição do guard, ex.
`[wet-diag] continua=false | wet_rect=false ...`), peça pro usuário colar o terminal, remova o
diag depois. Uma linha do terminal real vale mais que outra rodada de lentes. Ligado:
[[tool-unit-green-integration-dead]], [[visual-bug-debug]].

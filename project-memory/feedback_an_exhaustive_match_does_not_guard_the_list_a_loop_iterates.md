---
name: feedback-an-exhaustive-match-does-not-guard-the-list-a-loop-iterates
description: "Variante nova de enum + lista escrita à mão que um laço itera = braço de match INALCANÇÁVEL, sem warning; e agulha de busca com espaço nunca casa"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 0912ace0-ae98-4484-b7e2-3961027f28dd
  modified: 2026-07-29T19:24:37.574Z
---

Quando um laço itera uma **lista escrita à mão** de variantes (`const ORDER: [Kind; 5]`)
e faz `match kind` lá dentro, acrescentar um variant **satisfaz o compilador no
`match`** (que precisa ser exaustivo de qualquer forma) e **deixa a lista intocada**:
o braço novo vira código morto silencioso — parece tratado, e nunca roda.

**Why:** custou três rodadas de smoke no PH2D (as alças de roldana não desenhavam
NEM registravam hit; havia gate provando o publicador e gate provando o pintor, e
nenhum afirmando que a saída de um chega à entrada do outro). Duas hipóteses
anteriores eram fatos medidos e nenhuma era a causa.

**How to apply:** a cura não é acrescentar itens à lista — é **apagar a lista**.
Derive a ordem de um `match` exaustivo (`fn rank(k) -> u8`) e o limite dos DADOS
(`iter().map(rank).max()`); aí o próximo variant **não compila** até se declarar.
Antes de aceitar "o publicador está certo, então o bug é do consumidor", teste a
**costura** entre os dois.

⚠️ **Corolário em gates de busca textual:** uma agulha com **espaço** (`"aro "`)
**nunca casa** sob comparação por palavra inteira, e plural faltando (`"alca"` vs
`"alcas"`) esvazia a classe — o gate passa sobre o vácuo. Toda lista de agulhas
precisa de um controle: nenhuma contém espaço, e cada lista casa algo no corpus
real. Ver [[reference_topic_gate_discipline]],
[[feedback_a_condition_that_enumerates_its_readers_rots]],
[[feedback_a_negative_search_needs_a_positive_control]].

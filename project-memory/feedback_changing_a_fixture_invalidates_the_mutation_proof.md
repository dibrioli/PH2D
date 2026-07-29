---
name: feedback_changing_a_fixture_invalidates_the_mutation_proof
description: "Mexeu na fixture por QUALQUER motivo (matar flake, LOC, velocidade)? re-rode a mutação — o gate pode ter perdido os dentes em silêncio"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: bd77bd42-faf0-4adf-80e0-f9c2cd20a5ad
  modified: 2026-07-29T22:33:56.676Z
---

Uma mutação provada VERDE→RED prova o gate **contra aquela fixture**. Trocar a
fixture depois — mesmo por um motivo legítimo e não relacionado — pode tornar o
defeito inobservável, e o gate segue passando, agora sobre nada.

Caso real (`line/Painter`, 2026-07-29, sim off-thread do Wet Paint): um gate de
`want` obsoleto sangrava na poça de 4096². O gate irmão flakou sob a suíte
carregada, então encolhi a fixture para 512² — e a mutação passou a **passar**,
porque o defeito só existe quando o worker responde mais devagar que a espera do
frame, o que a tela pequena torna impossível. A lentidão relativa **era** o
fenômeno.

**Why:** a prova de mutação é sobre o par (gate, fixture), nunca sobre o gate
sozinho — e o motivo pelo qual se muda uma fixture (flake, custo, teto de LOC)
não tem relação nenhuma com o que ela precisa CONTER.

**How to apply:** depois de tocar qualquer fixture de um gate mutação-provado,
re-rode a mutação. Se ela parar de sangrar, a fixture nova não contém o fenômeno
— separe os gates (o de MECANISMO pode ser barato; o de TEMPO precisa da escala
que produz o defeito) em vez de compartilhar uma. Ver
[[reference_topic_fixture_discipline]] e
[[feedback_a_mutation_that_does_not_bleed_may_indict_the_oracle_not_the_finding]].

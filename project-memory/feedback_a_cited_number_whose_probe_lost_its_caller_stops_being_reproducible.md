---
name: feedback-a-cited-number-whose-probe-lost-its-caller-stops-being-reproducible
description: "Função de medição que ficou sem chamador não é só código morto — ela leva junto a reprodutibilidade do número que o doc cita; a cura é devolver a chamada, nunca silenciar o lint"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e6737742-4fe5-47ef-a299-f60a403fc03e
  modified: 2026-07-31T01:36:47.587Z
---

Quando um doc/ADR/handoff justifica uma decisão com um número MEDIDO ("o LSQ ingênuo é
PIOR: 0,1675 contra 0,0782"), esse número vive numa sonda. Se a sonda deixa de **chamar**
a função que o produz, o clippy acusa `never used` — e o reflexo errado é
`#[allow(dead_code)]` ou apagar. Os dois destroem a mesma coisa: a afirmação do doc deixa
de ser reproduzível, e vira folclore.

**Why:** o repo trata rota rejeitada como **referência congelada** (o `warp_axis` do
Painter, o `serial_side` do undo) justamente para que a comparação continue executável. O
valor dela é ser CHAMADA e IMPRESSA lado a lado com o que shipa — é isso que impede a
próxima LLM de "melhorar" o código por um caminho que já foi medido e recusado.

**How to apply:** ao encontrar uma função de medição sem chamador, primeiro **procure o
número dela num doc**. Se existe, devolva-a ao readout da sonda e **confira que o número
bate** com o que o doc afirma (na integração de 2026-07-30 o `lsq_fit` da `line/Vector`
voltou e imprimiu exatamente 0,1675 vs 0,0782). Se não existe número citado em lugar
nenhum, aí sim ela é código morto e sai.

Relacionadas: [[feedback_stale_comment_and_dead_code_lie]] · [[reference_topic_oracle_discipline]].

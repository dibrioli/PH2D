---
name: feedback-first-case-rescued-by-side-effect-test-repetition
description: "Quando o 1º caso é salvo por um efeito colateral (invalidação de cache vinda de outra mudança), ele passa e os seguintes falham — teste a REPETIÇÃO, não a primeira ocorrência"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1bd49227-b849-4388-97f3-4bf2e54fab65
---

Bug do Impasto (2026-07-12): o commit do traço trocava o relevo cru pelo **assentado** e não marcava nada
sujo — o composite seguia mostrando a iluminação do relevo velho. Mas o **1º traço funcionava**, porque ele
virava o `has_relief` da camada e *essa* troca de flag invalidava o composite **de efeito colateral** (código
de outra fase, escrito por mim dias antes). Enio: *"o primeiro traço aplica; do segundo em diante só quando
mexo no slider"*.

A fixture que escrevi tinha **um traço só** — logo era exatamente o caso salvo pelo acidente. Passou **verde
por cima de um defeito vivo**, e eu reportei fechado.

**Why:** é a 7ª vez nesta linha que um teste ficou verde pelo motivo errado, e a variante mais traiçoeira: o
fenômeno não estava ausente da fixture *por descuido* — estava **suprimido por uma mudança minha recente**.
Nenhuma revisão do teste o denunciaria; só rodar o caso 2 vezes.

**How to apply:** para qualquer bug de **estado que persiste entre operações** (cache, dirty-rect, flag,
sessão, undo), a fixture pinta/executa **≥ 2 vezes**, não 1. Se o gate só exercita a primeira ocorrência,
pergunte explicitamente: *o que na primeira vez é diferente da segunda?* (flag virando, cache frio, alocação
nova). E dirija no **ritmo real do app** — no Painter, um `take_preview_arc()` por evento de ponteiro; um
teste que nunca drena começa `preview_dirty` e full-recompõe no fim, apagando o bug.

Relacionadas: [[feedback_harness_reproduces_mechanism_not_context]] · [[feedback_nonreproduction_is_not_proof_of_fix]] · [[feedback_tool_unit_green_integration_dead]]

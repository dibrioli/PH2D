---
name: feedback_an_identity_gate_cannot_see_a_defect_in_the_shared_body
description: "Gate \"rota A == rota B\" sobre um kernel COMPARTILHADO só prova o walker — defeito no corpo aparece nas duas e passa; o oráculo do corpo é outro"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: bd77bd42-faf0-4adf-80e0-f9c2cd20a5ad
  modified: 2026-07-29T23:39:11.227Z
---

Quando duas rotas (serial×paralela, CPU×GPU, rápida×lenta) chamam **o mesmo
corpo**, um gate que compara os resultados prova só o que DIFERE entre elas — o
*walker*, o mapeamento índice↔fatia, o agendamento. Um defeito **dentro** do
corpo aparece identicamente nas duas e o gate fica **verde**.

Caso real (`line/Painter`, 2026-07-29, ADR-0145 — os passes row-paralelos do
solver do Wet Paint): o desenho é *um corpo, dois walkers*, e é isso que torna a
paralelização byte-idêntica por construção. Quatro gates comparavam todo plano
byte a byte entre as rotas. A mutação *"o laço 2 do `project` lê a linha
errada"* — um bug real e grave — **sobreviveu aos quatro** e sangrou o
**fingerprint de sessão**. Correto nos dois casos: a identidade não era o
oráculo daquele fato.

**Why:** o poder de um gate diferencial é exatamente a diferença entre os dois
lados. Compartilhar o kernel é a decisão certa (elimina a 2ª implementação que
divergiria), e o preço é que o kernel precisa de um oráculo **externo** —
golden/fingerprint/referência congelada — que nenhuma comparação entre rotas
substitui.

**How to apply:** ao shipar um gate de identidade entre rotas, escreva no doc
dele **o que ele NÃO pode ver**, e nomeie o gate que cobre isso. Na prova de
mutação, separe as duas classes: mutações de **ROTA** (só num walker) têm de
sangrar o gate de identidade; mutações de **CORPO** têm de sangrar o oráculo
externo — e se não houver um, o corpo está descoberto. Ver
[[reference_topic_gate_discipline]] e
[[feedback_layered_defenses_need_per_layer_gates]].

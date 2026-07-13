---
name: feedback-absence-gate-needs-a-presence-sibling
description: "Gate que afirma \"X não aparece onde não deve\" fica VERDE quando X não é renderizado de todo — sempre escreva o irmão \"X aparece onde deve\""
metadata: 
  node_type: memory
  type: feedback
  originSessionId: d0c3d83b-dca0-4107-b60e-6a89aff4818d
---

Um gate de **ausência** ("a cor não vaza da linha", "nenhum artefato fora da região", "zero
alocação") é **falso-zero** quando a coisa medida simplesmente não existe: não há o que vazar.

**Medido no Flip (BUGS #17, 2026-07-13):** o fill de um traço aberto era descartado pelo `pack` →
invisível na tela. O gate de vazamento marcou `spill = 0` (verde!) enquanto o interior da forma
tinha **16.240 px de FUNDO**. Só a asserção de **cobertura** ("o interior não tem fundo") sangrou
na mutação. O mesmo padrão já tinha mordido a linha do áudio (o 5º gate por-efeito é literalmente
"false-zero").

**Why:** um gate verde que você não sabe derrubar não é um gate — e a mutação que o derruba é
justamente a que APAGA a feature, não a que a estraga.

**How to apply:** todo gate de "não aparece onde não deve" nasce em PAR com "aparece onde deve"
(cobertura/presença), e a mutação obrigatória é **remover a feature**, não só perturbá-la. Corolário
para varredura (zoom/escala/faixa): confira que a coisa medida **está em quadro** em cada ponto da
varredura — no Flip, o zoom 5× pôs a câmera dentro da forma, a tela virou cor lisa e a costura saiu
de quadro: verde vacuoso. Asserte o enquadramento (ex.: `interior` dentro de uma faixa esperada).

Relacionadas: [[feedback_mutate_the_code_not_just_the_test]] ·
[[feedback_render_and_look_when_a_green_gate_is_contradicted]] ·
[[feedback_zero_alloc_gate_capacity_not_global_counter]] · [[feedback_gate_the_edges_of_the_domain]]

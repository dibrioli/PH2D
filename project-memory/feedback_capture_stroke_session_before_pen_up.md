---
name: feedback-capture-stroke-session-before-pen-up
description: "O pen-up carimba os dabs de CAUDA (janela de suavização) e mata a sessão — gate que compara estado-da-sessão vs resultado captura os DOIS antes do Up, senão mede fantasma"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 92714982-3cf5-48f6-96d6-acbdbe13b4f5
---

No gate do suporte do Inflate (2026-07-15), capturei `amount` antes do Up (a sessão morre no commit) mas
li `heights` DEPOIS. O anel "fora do suporte" media 2,87 loads — pânico — até ver que `paint_end` faz
`paint_extend(ev)` + `stroke.finish()` + `stamp_dabs`: **o Up carimba os dabs que a janela de suavização
do traço segurava**, estende o `amount` além da captura, e o "anel" estava DENTRO do suporte real,
medindo a física correta da secante como se fosse bug.

**Why:** a máscara derivada de um estado de sessão e o resultado comparado contra ela precisam descrever
O MESMO instante. O pen-up não é um "fim" passivo — é um batch a mais + destruição da sessão; qualquer
gate que atravessa o Up com metade da captura feita compara épocas diferentes.

**How to apply:** gate que relaciona estado-de-sessão (amount/disp/pre) com efeito no canvas: capture
estado E efeito **antes do pen-up**, no mesmo ponto do fluxo, e mande o Up depois (só higiene). O
invariante testado é propriedade de CADA render — medir no meio do traço não é concessão. Vale para
sculpt, deform e qualquer traço com janela de suavização. Irmão de
[[feedback_derived_coordinate_seed_must_match_sample]] (mesma doença: duas leituras, dois relógios).

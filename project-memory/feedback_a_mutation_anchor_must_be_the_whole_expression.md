---
name: feedback_a_mutation_anchor_must_be_the_whole_expression
description: "Âncora de mutação que é PEDAÇO de uma expressão (tr(\"x\") dentro de ph2d_i18n::tr(\"x\")) gera mutante que não compila — e o cargo-test-narrow.sh devolveu 1 (\"teste vermelho\"), não 2; leia o NOME do teste que reprovou"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e990a2f5-7d16-405a-8ddf-54393edf203d
  modified: 2026-09-13T21:35:21.205Z
---

Medido 2026-09-13 (`line/UIUX`, prova de mutação M7): a âncora `tr("chrome.done")` casava UMA vez dentro
de `ph2d_i18n::tr("chrome.done")` (a asserção de contagem passou), o mutante ficou
`ph2d_i18n::"Done"` e não compilava. O `scripts/cargo-test-narrow.sh` imprimiu
`0 falharam · 0 passaram` e saiu **1** — o contrato escrito dele diz **2** para erro de compilação. Eu
contei a mutação como morta pelo código de saída e depois atribuí o `0/0` à carga (`load 79`); a
segunda corrida a `load 31` deu igual, e só a corrida CRUA mostrou o `expected identifier`.

**Why:** a asserção «a âncora existe uma vez» prova que a mutação APLICOU, não que o resultado é o
código que se queria; e um script que resume a saída pode trocar «não compilou» por «vermelho».

**How to apply:** a âncora é a expressão inteira com o caminho (`ph2d_i18n::tr("…")`); uma mutação só
conta como morta quando o NOME do teste visado aparece a reprovar; `0 passaram · 0 falharam` é
inválida, nunca morta. Ligado: [[reference_topic_mutation_proofs]],
[[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]].

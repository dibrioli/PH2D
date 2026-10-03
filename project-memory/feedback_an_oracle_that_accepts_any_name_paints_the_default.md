---
name: feedback-an-oracle-that-accepts-any-name-paints-the-default
description: "O Krita aceita QUALQUER id de modo de camada e pinta Normal em silêncio — um oráculo que ecoa o nome não prova que o modo existe; o controlo é comparar com a corrida do modo padrão"
metadata:
  type: feedback
---

Medido 2026-10-03 (ADR-0177, `docs/Painter/ferramentas/oraculo_camadas_gimp/`): no `kritarunner`, `node.setBlendingMode("nao_existe")` é aceite, `blendingMode()` devolve `"nao_existe"`, e a projecção sai igual à do Normal. Dois ids reais da lista que eu escrevi de memória (`linear_light`, `clear`) eram desses; o certo é `linear light` (com espaço) e `erase`. O GIMP faz o mesmo de outra forma: `BEHIND` numa camada volta a `NORMAL` sem erro.

**Why:** um oráculo que ECOA o pedido parece uma confirmação, e a fixtura ficaria com «o Krita faz LinearLight assim» quando era o Normal.

**How to apply:** em todo arnês de oráculo que escolhe um modo/operação por NOME, corra o modo padrão primeiro e recuse a corrida cuja saída é idêntica à dele (e leia de volta o que o app ACEITOU, não o que se pediu). Família: [[reference_topic_oracle_discipline]].

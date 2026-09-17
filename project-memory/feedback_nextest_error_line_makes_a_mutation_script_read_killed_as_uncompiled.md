---
name: feedback-nextest-error-line-makes-a-mutation-script-read-killed-as-uncompiled
description: O nextest imprime «error: test run failed» quando um teste REPROVA — um script de mutação que procura ^error: classifica mutações MORTAS como «não compilou»; procure error[E…] ou «could not compile»
metadata:
  type: feedback
---

Medido em 2026-09-13 (`line/3DModeling`, W148, `docs/3DModeling/ferramentas/w148_provas_de_mutacao.sh`):
a 1.ª redacção do script separava «não compilou» de «morta» com `grep -qE '^error(\[|:)'`. O nextest
termina toda corrida com teste vermelho com a linha `error: test run failed` — e **quatro de cinco**
mutações MORTAS saíram como «NÃO COMPILOU». Só ao abrir os logs apareceram os `FAIL`.

**Why:** a palavra `error:` no início da linha é do cargo **e** do nextest, com sentidos opostos para uma
prova de mutação. Um classificador que confunde vermelho de teste com vermelho de compilação não prova
nada nos dois sentidos — e «não compilou» tende a ser aceite sem abrir o log.

**How to apply:** o sinal de compilação partida é `^error\[E[0-9]+\]` ou `could not compile`; o de teste
vermelho é `^ +FAIL ` no log do nextest. Todo veredicto do script tem quatro estados que não se leem uns
pelos outros (MORTA · SOBREVIVEU · NÃO COMPILOU · FILTRO VAZIO), e o primeiro «não compilou» de uma
mutação que só troca um valor pede abrir o log antes de acreditar. Ver [[reference_topic_mutation_proofs]]
e o irmão [[feedback_a_mutation_harness_needs_a_positive_control_that_a_test_ran]].

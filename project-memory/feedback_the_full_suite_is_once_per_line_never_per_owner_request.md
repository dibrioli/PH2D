---
name: feedback_the_full_suite_is_once_per_line_never_per_owner_request
description: Dono 13/10 — «se a cada mínima implementação temos testes de 10 min, nunca acabaremos»: por pedido só testes por NOME; o nextest-impacted 1× no fecho da linha
metadata:
  type: feedback
---

Numa janela em que o dono vai pedindo itens pequenos em sequência (smoke → ajuste → smoke), corri o
`nextest-impacted` (~10 min) depois de CADA item, e esperei por ele antes de responder. O dono:
*«se a cada mínima implementação temos testes de 10 min, nunca acabaremos»*.

**Why:** a regra já estava no CLAUDE.md §2 («teste/clippy/auditoria 1× no fecho»; «gate batched 1× sobre o diff
acumulado») e eu a tratei como «1× por commit». O relógio do dono é o recurso — cada espera de 10 min é dele.

**How to apply:** por item do dono — `cargo-check-narrow` + os testes que tocam o item, POR NOME (segundos), e o
commit. A suíte inteira (`nextest-impacted`, clippy amplo) corre UMA vez no fecho da linha, sobre o diff acumulado;
se algo quebrar, conserta-se tudo num bloco. Se uma corrida longa já começou, deixe-a no fundo e não a espere para
responder. Ver [[feedback_plan_every_measurement_upfront_and_batch_it]].

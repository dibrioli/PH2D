---
name: feedback_an_unmeasured_rule_in_a_briefing_is_confirmed_by_the_agents_who_follow_it
description: Uma regra escrita num briefing SEM medição é «confirmada» pelos agentes que a seguem — cada achado feito pelo atalho que ela manda passa a contar como prova da causa que ela atribui
metadata:
  type: feedback
---

Na W2 (partir a shell) o integrador escreveu no bloco da Fase B2 (2026-09-12): *«o `nextest-impacted`
NÃO alcança `shells/desktop/tests/it/` — corra `--test it` à parte»*. **Não mediu.** A regra viajou
para os blocos das Fases C e D e para o `ESTADO_W2` §4 como lei nº 4.

Três linhas (`physics` A e C, `components`) acharam vermelhos reais em `tests/it/` — **a correr
`--test it` à parte, porque o bloco lhes mandava** — e escreveram nos handoffs que o
`nextest-impacted` *«filtra»* aquela suíte. A regra ficou com três «confirmações» independentes.

Medido depois: `cargo nextest list -E 'rdeps(ph2d-app-vec)'` selecciona os **793** testes da suíte
`it` da shell (a shell depende de toda família, logo `rdeps(<família>)` contém-na), e o `BASE` por
omissão é `origin/main`, que estava 212 commits atrás. **O script nunca a filtrou.**

**Why:** um vermelho achado pelo atalho que a regra manda prova que o atalho acha vermelhos — não
prova a CAUSA que a regra dá para o atalho ser preciso. Quem segue a instrução não tem como separar
as duas coisas, e quem a escreveu lê as confirmações como independentes quando todas vêm da mesma
fonte: ele próprio. É o *verde por vácuo* ([[reference_topic_gate_discipline]]) no plano da
instrução, e o irmão de [[feedback_an_instruction_doc_can_order_what_its_readers_own_fence_forbids]]
(lá o briefing colide com a cerca do leitor; aqui ele fabrica a própria evidência).

**How to apply:**
1. Uma regra de briefing que afirma **a causa** de um defeito («o script X não faz Y») leva a medição
   **ao lado**, com o comando que a reproduz. Sem ela, escreva-a como **prática** («corra Y à parte,
   é barato»), nunca como **diagnóstico**.
2. Ao ler handoffs que «confirmam» uma regra, pergunte: *este achado seria diferente se a regra
   fosse falsa?* Se não, não é confirmação.
3. Quando refutar uma regra que já circulou, corrija o documento **canónico** e ponha um aviso nos
   blocos históricos — não reescreva os handoffs das linhas: eles são o registo honesto do que se
   acreditava.

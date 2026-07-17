---
name: reference-topic-mutation-proofs
description: "Provas de mutação — as 5 regras do placar"
metadata:
  node_type: memory
  type: reference
---

- [[feedback_mutate_the_code_not_just_the_test]] — verde na mutação = gate frouxo ou comentário errado
- [[feedback_mutation_red_only_counts_on_a_seen_green_gate]] — vermelho nos 2 mundos prova nada; ciclo verde→red→verde
- [[feedback_check_the_oracle_is_achievable_before_writing_the_gate]] — o prescrito pode ser impossível
- [[feedback_an_optimization_needs_a_gate_that_proves_it_fires]] — o fallback silencia o bug
- [[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] — explique por que é inofensiva ALI

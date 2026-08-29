---
name: feedback_first_accepted_and_best_are_two_numbers_and_a_give_up_rule_must_use_the_first
description: «Rondas desde a MELHOR» e «rondas sem aceitar NADA» são regras diferentes — a primeira corta trabalho real e quase apagou o maior ganho da jornada
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-28T23:05:54.772Z
---

Medido 2026-08-28 (acabamento do quad remesh): a rede que corta o desperdício dizia
*«desistir ao fim de `128` rondas sem melhoria»*. Numa peça a **primeira** ronda aceite é a
`312` e a **melhor** é a `830` — a regra matava a corrida à ronda `128` e a peça saía
**intocada**, quando ela chega a `1,04 / 2,0° / p99 22,8` com zero faces péssimas.

**Why:** as duas grandezas parecem a mesma («há quanto tempo não melhora») e não são. Uma
busca que tem de atravessar um vale só produz a primeira aceitação **muito depois** do
começo; contar a janela a partir da melhor mede um regime que ainda não começou. ⭐ *Desistir
enquanto NADA foi aceite é barato; desistir depois corta trabalho real.*

**How to apply:** numa busca com desistência, guarde **duas** contagens — a primeira
aceitação e a melhor — e faça a rede correr **só enquanto a primeira é zero**. Depois escolha
o valor da janela como um múltiplo da **maior primeira aceitação medida no corpus** (aqui
`768` = `1,8 × 418`), e ponha a tabela ao lado. ⚠️ E o instrumento tem de imprimir as duas: a
confusão só se vê quando as duas colunas estão lado a lado. Relacionado:
[[feedback_a_relative_stopping_threshold_is_repriced_by_whatever_runs_before_it]] ·
[[feedback_an_unlabelled_probe_column_gets_read_backwards]] ·
[[feedback_a_claim_no_mutation_can_kill_is_a_claim_about_nothing]]

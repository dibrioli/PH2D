---
name: feedback-a-tail-is-a-window-not-a-verdict
description: Li «0 FAILED» de um `grep … | tail -25` sobre 212 suites e quase declarei verde — e o gate que tinha reprovado estava fora da janela; a contagem sai do ficheiro inteiro, nunca da cauda.
metadata:
  type: feedback
---

Medido em 2026-09-05 (W122). Corri a suíte do shell com `--no-fail-fast` e canalizei a saída por
`grep -E "^test result|FAILED" | tail -25`. Li **zero FAILED** e escrevi que estava verde. ⛔ Havia
**duas** reprovações — uma delas o gate de LOC do shell —, e as duas ficaram fora das últimas vinte
e cinco linhas de **212 suites**.

**Why:** um `tail` não é um veredito, é uma **janela**. E o pior é que ele *parece* um veredito
quando o que está na janela é a última suíte, que quase sempre passa. É a mesma família do `| head`
que encolhe o conjunto de candidatas em silêncio
([[feedback_a_swallowed_panic_silently_shrinks_the_candidate_set]]) e do pipe que mascara o exit
code ([[feedback_pipe_masks_script_exit_code]]) — só que aqui **eu próprio escrevi o corte**.

**How to apply:** grave a corrida num ficheiro e derive as três contagens **dele**:

```sh
cargo test … --no-fail-fast > $LOG 2>&1; echo "exit=$?"
grep -c '^test result' $LOG          # quantas suítes correram
grep -c '^test result: FAILED' $LOG  # quantas reprovaram
grep -A3 '^failures:$' $LOG          # e QUAIS
```

⚠️ **O `exit=` é a metade que não se pode perder num pipe** — e num `until`/`[` do zsh, `pgrep -c`
devolve `0` **e** sai com `1`, então `pgrep -c x || echo 0` imprime `0\n0` e o teste de inteiros
rebenta (foi o que matou o laço que devia correr a prova de mutação, sem que ela chegasse a correr).
Ver também [[feedback_an_automatic_tools_exit_code_says_nothing_about_what_it_produced]].

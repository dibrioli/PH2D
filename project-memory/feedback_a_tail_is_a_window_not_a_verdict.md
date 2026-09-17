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

---

⛔⛔ **E em 2026-09-17 ele fez-me DIAGNOSTICAR MAL a ferramenta, não só perder uma linha.** Duas
corridas seguidas do `nextest-impacted.sh` devolveram **sete** reprovadas cada, em conjuntos
**disjuntos** de crates — e eu li isso como uma propriedade do script (*«o conjunto impactado cresce
com o diff»*) e escrevi-o num doc. ⛔ Era o `| tail -8` que eu próprio pusera: os ficheiros das duas
corridas têm **10 linhas e nenhuma linha de `Summary`**, e as «sete» eram só as que cabiam na
janela.

⭐ **Duas leis ficam:**
1. **O arnês já guarda a saída inteira num ficheiro** — um `tail` no comando destrói-a *antes* de lá
   chegar. Corra o portão sem pipe e faça o `grep` no ficheiro depois.
2. **Quando uma família de falhas aparece corrida a corrida, pare de a descobrir e VARRA-A.** Uma
   varredura estática que cruzava os textos migrados com as linhas que comparam `labels` deu a lista
   completa de uma vez — e provou que não sobrava nenhum caso, com três dos quatro «candidatos» a
   serem **doc-comments**.

⚠️ E a corrida **completa** da workspace acusou **quatro** reprovadas que as três corridas
impactadas nunca tinham alcançado, três delas na shell: *quando o diff atravessa uma crate-folha que
toda a gente usa, o conjunto impactado deixa de ser mais barato do que a verdade.*

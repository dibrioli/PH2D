---
name: feedback_a_shell_clock_in_a_comma_locale_truncates_to_whole_seconds
description: "`$EPOCHREALTIME` no locale pt_BR sai com VÍRGULA e o `awk` corta nela — o cronómetro do script lê segundos INTEIROS e imprime tabelas plausíveis (`1,00`/`4,00`)"
metadata:
  type: feedback
---

⛔ **Um cronómetro de script em bash nesta máquina lê SEGUNDOS INTEIROS e parece certo.** O
`$EPOCHREALTIME` do bash 5 respeita o `LC_NUMERIC`, e esta máquina corre `pt_BR.UTF-8`: ele sai
`1789324065,130207`, com **vírgula**. O `awk` lê o número até à vírgula, então `b − a` dá a diferença
de segundos inteiros. Medido 13/09: `sleep 0.3` entre duas leituras → o `awk` imprimiu **`0.00 s`**.

Na medição de velocidade de compilação antes×depois da refatoração final, a tabela do script dizia
`1.00`, `4.00`, `12.00`. O `Finished … in` do próprio cargo, no log da mesma corrida, dizia `0.88`,
`4.63`, `11.91`. Com o `check` incremental a custar ~0,9 s, a régua de 1 s leria
**«1,00 antes, 1,00 depois ⇒ sem diferença»**, e a diferença real era de 8 %.

**Why:** a falha é muda e produz números **redondos e plausíveis**. Nenhum erro, nenhum `NaN`, e a
coluna tem as duas casas decimais do `printf`, que fingem uma precisão que não existe. É a família
[[reference_topic_measurement_discipline]]: *o número que eu li mede o que eu penso que mede?*

**How to apply:**
- **Prefira o relógio da própria ferramenta** (o `Finished … in X.XXs` do cargo, o `--timings`, o
  `time` do nextest) e extraia-o do log. O relógio do script serve só para confirmar a ordem de
  grandeza.
- Se o script precisar do próprio relógio, **`export LC_ALL=C`** no topo, ou use `date +%s.%N`
  (sem locale).
- Antes de acreditar numa coluna de tempos, **imprima UM valor cru** e procure a vírgula. Uma coluna
  inteira de `.00` é a assinatura deste defeito.
- A mesma armadilha vale para qualquer número que atravesse o shell até ao `awk`/`python` com o
  locale pt_BR (`printf "%.2f"` do bash também escreve vírgula).

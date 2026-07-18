---
name: feedback-the-ceiling-is-the-hardwares-never-the-fallbacks
description: Meça antes de limitar; o teto é do dispositivo, não do caminho lento — e "inalcançável" é uma afirmação sobre um número que alguém pode mudar
metadata:
  type: feedback
---

Antes de escrever qualquer limite (`MAX_*`, cap, faixa de slider, "por ora"),
**meça** — e escreva o número que a medição deu, com a tabela ao lado dele.

**Why:** o Enio cobrou (2026-07-17): *"não estamos levando o motion nodes para o
GPU para alcançarmos resultados extraordinários?"*. Ele estava certo. A sim de
partículas fazia **4,19 M partículas em 3,6 ms na GPU** (22% de um frame de 60
fps) e eu tinha posto o teto em **16.384** — 256× abaixo — porque a **CPU** seria
lenta a 262k. O caminho mais lento definindo o teto do mais rápido, no módulo
cuja razão de existir é o mais rápido. E eu tinha escrito um parágrafo bem
argumentado justificando, e chamado o resultado de "decisão do Enio".

Duas armadilhas, as duas neste mesmo caso:

1. **O fallback não define o produto.** O caminho de referência (aqui a CPU) só
   precisa **computar a mesma resposta** — ele pode demorar o que um teste
   aguentar. Quem manda no teto é o dispositivo que o usuário roda.
2. **"Fora de escopo porque é inalcançável" é uma afirmação sobre um número que
   outra pessoa pode mudar.** O doc dizia *"id é `f32`, teto 2²⁴ ≈ 4,8 dias a
   rate 40 — fora de escopo"*: **verdade** enquanto o slider parava em 200 (23
   horas), **4 segundos** depois que o slider subiu. Quem move o número que
   tornava algo inalcançável tem de **reconferir a nota**.

**How to apply:** um limite legítimo diz **de que recurso ele é** (memória,
banda, precisão de representação) e traz a medição — `MAX_ALIVE` virou "≈370 MB
de residência GPU", não "a CPU fica lenta". Um limite que só diz "por segurança"
é palpite esperando um smoke. Ao subir um limite, **grep pelas notas que o
chamavam de inalcançável**. E não confunda isto com otimização prematura
([[project_m5_perf_validated]]): a exigência é **medir antes de limitar**.

Relacionados: [[feedback_measure_perf_symptom_scale]] ·
[[feedback_documented_decision_chesterton_fence]] ·
[[feedback_two_doors_to_the_same_question_diverge]] ·
[[feedback_perfection_no_deferrals]]

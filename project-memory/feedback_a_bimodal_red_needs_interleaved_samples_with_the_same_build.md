---
name: feedback_a_bimodal_red_needs_interleaved_samples_with_the_same_build
description: Vermelho bimodal (0,3 s ou 30 s) no portão: 6/6 verdes no main «provou» a culpa da linha — era sorte e outro build (-p vs workspace); só 12×12 intercalado com o MESMO -p decidiu
metadata:
  type: feedback
---

24/10, integração da `line/Velocidade`: `a_cena_4_conduzida_passo_a_passo` (CodeEditor, corrida com o `luau-lsp`)
vermelho na suíte. Main sozinho 6/6 verde × linha 5/15 vermelho ⇒ «a linha causou». Falso: o main corria com
`-p ph2d-host-code-scenes` (features unificadas só desse pacote), a linha com a workspace inteira; e 6 amostras
de um vermelho de ~25 % dão 6/6 verdes em ~18 % das vezes. Intercalado, `-p` dos dois lados, 12 × 12: main 3/12 ·
linha 1/12. O mesmo dia: «cresceram ~10×» saiu de testes medidos 5 em paralelo; sozinhos eram 3,6–5,5×.

**Why:** a régua (amostra pequena, build diferente, concorrência) quase bloqueou um merge e culpou a linha errada.

**How to apply:** vermelho que não é carga (falha calmo) ⇒ A × B intercalado, mesmo comando de build nos dois lados
(`-p` ou workspace, nunca misturado), ≥ 10 amostras cada; tempo de um teste «sozinho» = UM teste, `-p`, sem vizinhos.
Família: [[reference_topic_measurement_discipline]] · [[feedback_a_tail_is_a_window_not_a_verdict]].

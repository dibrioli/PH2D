---
name: feedback-heuristic-needs-false-positive-rate
description: Detector/heurística só se prova com TAXA de falso-positivo sobre muitas realizações de ruído — um fixture verde é sorte do seed
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7af6ea3e-743d-44de-88f2-6062de6f1639
---

Ao construir um **detector, classificador ou heurística** (achar quina, pico, transiente,
outlier, "isto é um cusp?"), um teste verde num fixture sintético **não prova nada** — o
resultado é refém do seed de ruído específico. Antes de enviar, meça a **taxa de
falso-positivo sobre ≥100 realizações de ruído** dos casos que ele NÃO deve disparar, com
amplitude de ruído **realista** (tremor de mão ≈ 2% do range para mocap de mouse).

**Why:** na ETAPA 5 da `line/anim` (2026-07-12) construí um detector de quina cujos 3
fixtures estavam verdes. A taxa sobre 200 seeds mostrou **100% de falso-positivo** em
gravações suaves com tremor realista — ~12 quinas fantasmas por gesto. Os fixtures passavam
por sorte. Quatro modelos diferentes (ângulo de giro · crescimento de força em 2 escalas ·
dobra do resíduo em 2 escalas · retidão relativa ao ruído) morreram todos no mesmo lugar: na
escala da amostra, **um gesto suave rápido e um cusp diferem só de um jeito que o ruído
mascara**. Enviar teria trocado um erro de 2,8% que o Enio já aceitara por uma regressão
visível.

**How to apply:**
1. **Sempre escreva o guarda de falso-positivo primeiro**, e com o adversário REAL, não uma
   versão fraca dele (a senoide *rápida* e o tremor *alto*, não a lenta e o baixo).
2. Meça FP **e** TP como taxas, não como um caso. Um harness de 30 linhas resolve.
3. **Assimetria decide o limiar:** se a falha do detector é uma regressão visível e o que ele
   consertaria já está dentro do envelope aceito, ele erra para "não detectou".
4. Contar as reconstruções do modelo: a **regra two-strikes** da DIRETIVA §5 vale aqui —
   na 3ª formulação, PARE e prove o modelo em vez de tentar a 4ª.

Relacionado: [[feedback_tool_unit_green_integration_dead]] · [[feedback_nonreproduction_is_not_proof_of_fix]] ·
[[feedback_measure_perf_symptom_scale]] · [[feedback_oracle_must_model_appearance_not_implementation]]

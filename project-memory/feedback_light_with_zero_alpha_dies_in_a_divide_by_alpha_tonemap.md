---
name: feedback_light_with_zero_alpha_dies_in_a_divide_by_alpha_tonemap
description: "Luz somada com alfa 0 (um halo) morre num tonemap que divide pela cobertura — e só num alvo TRANSPARENTE (o quadro por faixas): o fx.glow não brilhava com formas na placa desde a W2, sem gate nenhum o ver"
metadata:
  type: feedback
---

Doc 121 §9.14 (04/10, line/motion-value): a foto da `=70` mostrou o `fx.glow` SEM halo sempre que o
passe de formas estava ligado — em qualquer rota. Mecanismo: o passe de formas força o quadro por
FAIXAS (ADR-0154), em que o `game_rt` começa transparente; o composite do brilho somava luz com alfa
`0`; o tonemap faz `cor ÷ alfa → curva → × alfa` e a luz fora das peças saía a zero. Sem faixas o
`game_rt` é opaco e a luz sobrevivia — por isso o defeito nasceu com a W2 e ninguém o viu.
Cura (`2c16ead81`): o composite escreve `a = min(máx(rgb), 1)` com a mistura de alfa
`src·(1 − dst) + dst` — byte a byte o de antes sobre o opaco.

**Why:** «aditivo pré-multiplicado» (`rgb > 0, a = 0`) é válido para a mistura e INVÁLIDO para todo
passe que divide pela cobertura; um gate de pixel sobre alvo opaco nunca o apanha.

**How to apply:** um efeito de LUZ que acaba num alvo que pode ser transparente tem gate sobre o
TRANSPARENTE (com o controlo do opaco); e o smoke de um efeito fotografa-se com a placa de formas
ligada e desligada — o diagnóstico que separou o defeito foi limpar o RT do halo a vermelho.
Ver [[reference_topic_code_pattern_gotchas]].

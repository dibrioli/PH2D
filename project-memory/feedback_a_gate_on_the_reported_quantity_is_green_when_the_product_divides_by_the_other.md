---
name: feedback-a-gate-on-the-reported-quantity-is-green-when-the-product-divides-by-the-other
description: Quando a sonda calcula duas grandezas e o produto usa uma delas, o gate tem de derivar a que foi USADA do efeito — senao a mutacao sobrevive.
metadata:
  type: feedback
---

Se a função de diagnóstico devolve **duas** grandezas (a certa e a que o defeito usava) e o
produto escolhe uma, um gate escrito contra o que o **relatório publica** fica verde com o
defeito de volta: ele mede a grandeza **calculada**, e o defeito está em **qual delas o
código divide**.

**Why:** medido no `ph2d-gridmap` (2026-08-27). A `tie_normal` devolve `(g, H, H_fingida)`.
O gate afirmava `H ≥ curvatura_efectiva` — verdade **sempre**, calcule-se o passo com qual
das duas se calcular. A mutação que repunha o denominador fingido **sobreviveu**. A cura foi
derivar o denominador **usado** do efeito observável: `andado = gradiente / denominador` ⇒
`denominador_usado = gradiente / andado`. Aí a mutação morre (`8,10` contra `73,80`).

**How to apply:** quando o gate puder ler a grandeza pela API **ou** derivá-la do que o
produto de facto fez, **derive-a do efeito**. E corra sempre a mutação: aqui ela foi a única
coisa que distinguiu um gate verdadeiro de uma tautologia.
Parente de [[feedback-a-gate-on-the-mark-i-chose-is-green-when-the-marks-premise-is-false]]
e [[feedback-counting-the-work-done-is-not-counting-the-work-delivered]].

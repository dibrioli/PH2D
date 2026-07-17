---
name: feedback_a_sequential_accumulation_is_sampling_dependent
description: "Acumulação sequencial (produto sobre incrementos) depende da amostragem e da FASE — telescope-a"
metadata:
  node_type: memory
  type: feedback
---

A mordida do Push era `take = (g + p)·Δm` por dab, então `q = g + p` evoluía como `q ← q·(1 − Δm)` e o
total virava `g·(1 − Π(1 − Δm_k))` — um **PRODUTO sobre os incrementos**. Produto depende de *em quantos
passos* o envelope foi alcançado e da **FASE de cada texel contra a grade de dabs** ⇒ o piso do canal
ondulava no período exato do dab (a coria/mola do smoke do Enio, 2026-07-15). Dois corolários que só a
medição mostrou: a lei convergia pra `g·(1 − e^{−m})` ≈ **63%** do chão (então "Push = 1" removia 63% da
tinta, e esse número era **acidente do espaçamento**), e o **Smooth também era afetado** (−0,649 @1px vs
−0,670 @2px) — só não gritava.

**Why:** o **depósito** já tinha essa lei resolvida — *"o relevo é propriedade do pincel e do CAMINHO,
nunca de quão fino o motor amostrou o caminho"* — e é imune porque toma um **ENVELOPE** (`max` = função
pura da distância ao caminho). A mordida era **acumulação sequencial**, então herdou a doença que a
cápsula tinha curado um plano acima. Um falloff de borda macia ESCONDE (Δm pequenos e parelhos); um de
tangente vertical (`Sphere = √(1−t²)`) EXPÕE. Ver [[reference-topic-impasto-physics]].

**How to apply:** se um efeito acumula por-dab lendo o estado que ele mesmo escreve, pergunte *"isto é
função do envelope ou da SEQUÊNCIA?"*. Se for da sequência, **telescope**: normalize o incremento pela
SOBRA (`Δm/(1 − m_prev)`), porque `Π (1−m_k)/(1−m_{k−1}) = 1 − m_final` — o resultado vira função pura do
envelope, em qualquer espaçamento e qualquer ordem, **sem transcendental** (HR-5; `e^{−Δm}` também
telescopa e é proibido). O gate é a lei, não o sintoma: **o MESMO caminho amostrado a 1 px e a 2 px tem
de dar o mesmo campo** — e a mutação sangra no falloff que ESCONDE, não só no que grita. Corolário de
diagnóstico: [[feedback_measure_perf_symptom_scale]] vale pro visual — **inocente o suspeito por
medição** (renderizei a coria com a âncora VELHA e ela estava lá: o fix acusado era inocente, e o bug
tinha vindo com a feature anterior).

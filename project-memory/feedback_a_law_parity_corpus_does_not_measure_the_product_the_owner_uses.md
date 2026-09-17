---
name: feedback-a-law-parity-corpus-does-not-measure-the-product-the-owner-uses
description: "Um corpus de oráculo com valores escritos à mão para a LEI ser recuperável não mede o PRODUTO que o dono usa — a lei do pincel de plano batia a 1e-8 e o pincel deixava a superfície 1,1× mais rugosa"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 9f820704-0d7e-4d96-847e-9cd720cbf178
  modified: 2026-09-16T21:32:37.390Z
---

O pincel de plano (`line/sculpt3d`, 16/09) reproduzia o oráculo em 42 traços a `≤ 8,9e-8`, e o dono
disse *«resultado inferior ao blender para produzir superfícies planas»*. Estava certo: o corpus da
1.ª missão escrevia os valores **à mão para a lei ser RECUPERÁVEL** (força 1, curva constante), dava
os dabs **por script** e punha o cursor **fora** do relevo. O artista tem os valores **de fábrica**,
um traço **arrastado** (com espaçamento e atenuação) e o cursor **na** superfície. Corridos no próprio
alvo, os NOSSOS valores de fábrica pioravam o relevo `1,114×`; faltava uma memória do plano (a
«firmeza da normal») que nenhuma fixtura da 1.ª missão exercitava. ⭐ E o caminho por script do alvo
nem sequer aplica a atenuação do traço — o interruptor dela lia «inerte».

**Why:** uma barra de paridade verde afirma a LEI sob as condições da fixtura; as condições do
produto (valores, traço, cursor) são outra população, e a diferença que o dono vê mora lá.

**How to apply:** quando um oráculo fecha a lei, peça ao E uma **2.ª missão de PRODUTO**: os valores
com que a ferramenta NASCE no alvo, o gesto REAL (arrastado, por eventos de rato), o cursor onde o
artista o põe, e uma régua do resultado que o dono julga (aqui, a planura) — com uma **ablação** que
troca um valor de cada vez para nomear a alavanca. Ver [[reference_topic_oracle_discipline]].

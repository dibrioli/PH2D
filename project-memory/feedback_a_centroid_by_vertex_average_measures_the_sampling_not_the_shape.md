---
name: feedback-a-centroid-by-vertex-average-measures-the-sampling-not-the-shape
description: "Régua ancorada no centroide POR VÉRTICE mede a amostragem, não a forma — e num remalhador ela defende a candidata culpada"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-31T19:04:18.528Z
---

Toda régua «distância ao centroide» tem de tirar o centroide da **área** (ou de outra
quantidade da superfície), nunca da **média dos vértices**. A média dos vértices é uma
propriedade de *onde estão os vértices*; qualquer passo que os redistribua — que é
literalmente o que um remalhador faz — move-a sem que a forma mude.

Medido 2026-08-31 (`line/quadextract`, escultura do dono): centroide por vértice deriva
`0,2129` entre entrada e saída e o alcance lê **`−6,5 %`**; por área deriva `0,0037` e lê
`+0,0 %`; a verdade (referencial comum) é `−0,1 %`. Duas densidades da mesma peça diferem
**`1,06 %`** na régua contaminada contra uma banda de decisão de `2 %`.

⚠️ **E o sinal foi o pior possível:** a régua estava no `worse` do selector do botão — uma
candidata que **corta** a ponta perde vértices longe do corpo, o centroide afasta-se, e o
alcance medido **sobe**. *A régua defendia exactamente a candidata que devia acusar.*

⭐⭐ **O que expôs isto foi um relatório com DUAS réguas do mesmo fenómeno a discordar:** o
suporte por ponta dizia `0 de 4 cortadas, pior −0,4 %` e o alcance dizia `−6,5 %`, na mesma
página, havia semanas. *Quando um relatório imprime duas medidas da mesma grandeza e elas
discordam, isso É o achado — reconcilie antes de acreditar em qualquer uma.*

**Why:** um número contaminado pela amostragem parece uma medição e não é; e num
comparador ele decide.

**How to apply:** antes de escrever `distância ao centroide`, pergunte *de que é a média?*
Se a resposta for «dos vértices», o gate a escrever é «a mesma forma amostrada de duas
maneiras dá o mesmo número», com a régua velha ao lado como **controlo** (ela tem de errar
na mesma fixtura). Ver [[feedback-a-ruler-anchored-in-the-world-measures-the-gesture-not-the-shape]]
e [[feedback-a-normalising-law-needs-a-quantity-invariant-to-free-motion]].

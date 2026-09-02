---
name: feedback-smoothing-a-field-that-feeds-a-sharp-feature-trades-the-feature-away
description: "Suavizar um campo para o estabilizar dilui-o exactamente onde a feição fina o consome — meça pela régua da FEIÇÃO, nunca pela dispersão"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-31T23:27:58.039Z
---

Quando um campo de densidade/peso alimenta uma **feição fina** (uma agulha, um vinco, uma
ponta), a regra dura que o espalha — *«toma o valor mais exigente da vizinhança»* — não é um
acidente de implementação: **é ela que alimenta a feição**. Qualquer mistura suave entrega
*menos* onde a feição precisa de mais.

Medido 2026-08-31 (`line/quadextract`): para curar a instabilidade, o `min` duro de 27
células foi substituído por um campo contínuo, em três passos, cada um melhor no
instrumento — dispersão `4,4 % → 3,0 % → 1,4 %`. ⛔ **E o produto piorou:** no botão, a peça
na origem passou de `0` de `4` pontas cortadas para `1` de `4`, e uma posição deu **`−40,8 %`**
numa ponta. Revertido.

⇒ *a estabilidade e a ponta pediram coisas opostas, e a ponta é o que o dono vê.*

**Why:** as duas réguas apontavam em sentidos contrários, e a única que aparecia no ecrã da
cura era a da estabilidade. Um progresso monótono no instrumento errado lê-se como progresso.

**How to apply:** antes de suavizar/regularizar um campo, pergunte *quem CONSOME o extremo
dele?* Se houver uma feição fina a viver do extremo, a régua da cura é a **régua dessa
feição** (aqui, pontas cortadas), e a dispersão é só uma coluna ao lado. Ponha as duas na
mesma tabela desde a primeira medição. Ver
[[feedback-a-better-instrument-can-make-the-product-worse-and-that-is-the-finding]] e
[[feedback-a-phase-measured-alone-can-improve-and-make-the-pipeline-worse]].

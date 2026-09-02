---
name: a-perfect-input-producing-a-worse-output-localises-the-damage
description: Quando uma fase entrega um input PERFEITO e o resultado final piora, o defeito está a jusante — por aritmética, sem precisar de mais nenhuma sonda
metadata:
  type: feedback
---

Se uma fase passa a entregar um input **perfeito** na régua dela e o produto final fica **pior**,
o dano está **a jusante**. Não é hipótese: é aritmética, e não precisa de instrumento novo.

**Why:** medido em 2026-08-30 (`line/quadextract`). A resolução injectiva levou o mapa contínuo de
`120` para **`0`** dobras — a propriedade que a literatura promete, verificada. E o A/B ponta a
ponta pelo botão deu **pior** em todas as colunas de forma (enviesamento p50 `6,4° → 21,3°`,
`>60°` `2 → 1 191`, e **`22 → 415`** faces dobradas na extracção). ⇒ *a escada gulosa a jusante é
a culpada*, e o passo seguinte não precisou de sonda nenhuma para o nomear.

⚠️ **E há quase sempre um SEGUNDO mecanismo, de desenho, escondido no mesmo resultado:** a fase
substituída otimizava `‖∇f − R/h‖²`, que fixa a escala **e a ORIENTAÇÃO** contra um campo; a nova
energia fixava escala e conformidade e **não tinha termo nenhum a amarrar o mapa ao campo**. ⇒ uma
energia nova que **SUBSTITUI** a antiga tem de responder por **cada propriedade** que a antiga
fixava — enumere-as antes de trocar, e some em vez de substituir se faltar alguma.

**How to apply:** ao medir uma fase melhorada, corra sempre o A/B **pela porta do produto** e não
só na régua da fase ([[a-phase-measured-alone-can-improve-and-make-the-pipeline-worse]]). Se a
fase ficou perfeita e o produto piorou, **pare de instrumentar a fase** e escreva a conclusão: o
dano é a jusante. E não apague as colunas que MELHORARAM — aqui foram duas (pontas cortadas
`2 → 1` de `12`, a queixa do dono; e fidelidade `4,4×`), e são elas que dizem que a direcção está
certa.

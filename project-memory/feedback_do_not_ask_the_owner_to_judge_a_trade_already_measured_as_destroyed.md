---
name: do-not-ask-the-owner-to-judge-a-trade-already-measured-as-destroyed
description: Só leve ao dono um troco que esteja PERTO; se a sua própria medição já diz que um lado é catastrófico, a decisão é sua e o smoke custa-lhe tempo por nada
metadata:
  type: feedback
---

Um smoke pede ao Enio o que **só ele** pode decidir: um troco genuinamente **próximo**. Se a sua
própria medição já diz que um dos lados está destruído, **a decisão já está tomada** — leve-lhe o
resultado, não a pergunta.

**Why:** medido em 2026-08-30 (`line/quadextract`). O A/B da resolução injectiva deu **6 de 8
colunas piores**, incluindo `torção máxima 180°`, `125` faces auto-intersectadas (contra `0`) e
`415` faces dobradas (contra `22`). Eu li isso como *«pior no corpo, melhor nas pontas — troco para
o dono»* e escrevi-lhe um smoke de 4 passos. A resposta dele foi *«destruiu completamente a malha e
demorou minutos»*, com foto de uma peça rasgada de alto a baixo.

⛔ **O erro não foi a medição — foi a LEITURA da severidade.** Uma percentagem de faces defeituosas
(`4,83 %`) lê-se como moderada e os defeitos **concentram-se em bandas**, não espalhados: no ecrã
são rasgões contínuos. E `torção 180°` não é «um quad torto», é um quad **virado do avesso**, que
renderiza preto.

**How to apply:** antes de escrever um smoke, pergunte *«se eu fosse o dono, isto seria uma escolha
ou uma reclamação?»*. Sinais de que **não** é troco: uma coluna com **zero natural** de um lado
(gravatas, não-manifold, componentes a mais), um extremo saturado (`180°`), ou uma razão acima de
`2×` numa coluna de qualidade. Nesses casos escreva a recusa medida e siga. Guarde o smoke para
quando as colunas discordarem **pouco** — é aí que o julgamento dele é o instrumento que falta.
Ver [[a-perfect-input-producing-a-worse-output-localises-the-damage]] para o achado técnico da
mesma medição, e [[communication_simplicity]] para o formato.

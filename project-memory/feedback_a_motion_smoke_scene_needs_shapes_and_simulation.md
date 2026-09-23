---
name: a-motion-smoke-scene-needs-shapes-and-simulation
description: Toda cena de smoke do Motion tem FORMAS e SIMULAÇÃO com campos — sem forma só se vêem gizmos; esquecido duas vezes em 23/09
metadata:
  type: feedback
---

Toda cena de smoke do Motion (tutorial OU ciclo de optimização) tem de ter o uso real:
**FORMAS** (`source.shape` / `source.object`) **e SIMULAÇÃO com campos**. Uma corrente só de
posições desenha só **gizmos** desde 19/09 (lei do dono, `SinkStyle::so_com_forma`), e uma cena
parada não exercita o cozimento, que é onde o custo e os defeitos moram.

**Why:** 2026-09-23, duas vezes no mesmo dia. De manhã dei a `=6` (grelha sem forma) para ver uma
cura de mistura — ele viu *«apenas gizmos do grid»*; a lição foi escrita, mas num ficheiro de
tópico (2 saltos) que ninguém lê ao escrever um smoke. À tarde dei a `=93` (Poisson/Voronoi/emissor
sem forma) para o ciclo 11 e o dono: *«você esqueceu de usar shapes novamente. Melhor atualizar os
docs para não esquecer»* · *«e não colocou campos de simulação»*. Nas duas vezes escolhi a cena
pela ROTA (o censo punha-a na placa), que é uma propriedade interna e não diz se o fenómeno se vê.

**How to apply:**
- Antes de mandar um smoke, **fotografar** a cena (`docs/Components/ferramentas/fotografa_cena.sh`)
  com o CONTROLO ao lado; fotos iguais ⇒ a cena não ensina nada.
- ⚠️ Tensão: `source.shape` hoje RECUSA a placa (vai à CPU). Para mostrar a PLACA usar
  `source.object`, ou dizer no roteiro que aquela cena corre na CPU e porquê.
- Se nenhuma cena do catálogo serve, **construí-la é trabalho da wave**, não improviso.
- A regra está também no `CLAUDE.md` §0.8 e no doc 103 §1 (os sítios lidos ao escrever um smoke).

Relacionado: [[a-smoke-for-the-owner-explains-what-each-thing-on-screen-is]].

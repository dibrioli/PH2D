---
name: feedback-a-clock-that-runs-does-not-prove-the-scene-moves
description: "Uma FOTO é um instante de um ciclo, e uma janela de 120 quadros é a média dele — nenhuma das duas diz como está o instante denso; meça por janela CURTA ao longo do ciclo, em release"
metadata:
  type: feedback
---

4.ª onda, 05–06/10 (doc 121 §9.19 (5)): a foto da `=114` mostrou a pilha na grelha de partida e eu concluí que a
cena NÃO caía no roteiro e que as tabelas do app mediam a cena parada — errado: a demo entra em Play sozinha e a
foto tinha apanhado o RECOMEÇO do ciclo (a queda recomeça a cada `3,6` s). Ao mesmo tempo, a janela de `120`
quadros do `[frame]` cobria um ciclo inteiro e a média escondia o instante da pilha, e o smoke de desempenho foi
ao dono em `--profile smoke`: ele viu `≤ 10` fps onde as minhas tabelas diziam `60`.

**Why:** tratei um instante (a foto) e uma média (a janela longa) como estado da cena; o fenómeno vive num
instante curto e periódico, e o perfil de build multiplica-o (`smoke` ~`4×` mais lento que `release` aqui).

**How to apply:** para um custo que varia ao longo de um ciclo, meça por janela CURTA ao longo dele (a régua
`[motion-quadro]`, `PH2D_MOTION_RELOGIO=1`) e leia a PIOR janela; antes de concluir «a cena parou», registe o estado
(posição de uma peça) em vários instantes; smoke de desempenho ao dono sempre em `--release` (`CLAUDE.md` §2).
Família [[reference_topic_measurement_discipline]].

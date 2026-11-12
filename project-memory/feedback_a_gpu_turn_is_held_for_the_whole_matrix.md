---
name: feedback_a_gpu_turn_is_held_for_the_whole_matrix
description: Matriz de medição no app que lança um app por pincel tem de pedir a vez da placa UMA vez e segurá-la até ao fim; cada app a pedir/soltar deixa outra linha entrar entre dois lançamentos (missão 4K, 10/11)
metadata:
  type: feedback
---

Em 10/11 (missão Painter 4K, `docs/Painter/missao_4k/medicoes/DEPOIS.md`) a medição «depois» não mediu nada: o
`mede_pen_up_4k.sh matriz` lançava um app por pincel e cada app pedia e soltava a vez da placa sozinho. Entre dois
lançamentos a `line/InputMap` reentrou na placa; 5 de 6 lançamentos sairam `win NENHUMA` e a 2.ª corrida esperou 20 min
sem lançar.

**Why:** a vez por processo é a certa para UMA medição; para uma matriz, o intervalo entre processos é uma janela aberta
para outra árvore, e a medição «antes × depois» só vale se todas as células forem do mesmo estado da máquina.

**How to apply:** um roteiro que lança vários apps pede `scripts/vez-da-placa.sh` ANTES do Xvfb, com o próprio roteiro como
dono, e só a solta no fim (modo «uma medição, do princípio ao fim» de `docs/DevOps/A_VEZ_DA_PLACA.md`). Antes de lançar,
procure «⏳ a placa está com outra árvore» no log do `ph2d-run.sh`; nunca force duas linhas na placa (14/09 pendurou o driver).

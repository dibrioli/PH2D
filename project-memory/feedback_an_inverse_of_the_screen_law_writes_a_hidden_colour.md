---
name: feedback_an_inverse_of_the_screen_law_writes_a_hidden_colour
description: Guardar a cor de uma camada pela INVERSA da lei do ecrã sob outra que a esconde escreve uma cor fora da gama, invisível até a de cima sair — a régua do ecrã no momento não a vê
metadata:
  type: feedback
---

Quando uma transferência (assentar, secar, re-molhar) resolve a cor GUARDADA pela inversa da lei que o ecrã
usa para compor «camada de cima sobre a de baixo», e a de cima quase esconde a de baixo, a inversa pede uma
cor impossível (reflectância > 1) e o `clamp` escreve-a: ciano/lilás preso a 255, ESCONDIDO. O ecrã não muda
nesse instante — todo portão «assentar não muda o que se vê» passa — e a cor aparece quando a camada de cima
sai (o escorrimento a leva). Medido no Wet Paint (prompt 16, 11/10): o «Glaze layering» (lei `Camada`, que
ainda por cima tratava o assentado como fundo OPACO a qualquer cobertura, com salto em `a_s = 0`) dava
9 001 células fora da gama e os quatro sintomas do dono; onde ATERRA é a advecção, o que PROVOCA é a inversa
chamada pela secagem.

**Why:** a régua certa não é «o ecrã agora», é «a cor guardada está dentro do casco das tintas que ali
pousaram» (por canal, em luz linear) — e uma cena em que a camada de cima SAI depois.

**How to apply:** toda inversa de lei de ecrã leva um tecto na caixa das cores que se juntam (`assenta::na_caixa`)
e um portão de varredura sobre estados de filme grosso / massa ínfima; com a camada de cima VAZIA, a única cor
é a que vem (a cor guardada de um vazio é a de uma tinta que já saiu). Ver
[[feedback_a_per_cell_gate_on_entering_dry_cells_prints_the_grid]] e o 05 §4b «Mistura II».

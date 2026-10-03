---
name: feedback-an-oracle-parameter-on-a-half-step-tie-measures-float-noise
description: "Parâmetro de oráculo cujo resultado cai em x,5 degrau mede o ruído de vírgula flutuante dos dois programas, não a lei — e o controlo do oráculo parte do composto em FLOAT, não do quantizado"
metadata:
  type: feedback
---

Oráculo dos ajustes (P3, ADR-0177, 03/10): a curva `0,1 → 0,9` dava bytes `25,5 + 0,8·k` — empate
exacto em 1 de cada 5 entradas; o arredondamento decidia-se pelo f32 nosso × o double do GIMP: 87 de
4 896 canais a 1 sem defeito nenhum. Pontas `0,1037 → 0,9113` (sem empate): 3. E o controlo «em luz»
construído sobre o composto JÁ QUANTIZADO a bytes errava 6 degraus no escuro (a luz amplia um degrau);
o GIMP não quantiza entre os passos — o controlo tem de partir do composto em vírgula flutuante.

**Why:** a régua tem de medir a lei; um empate de meio degrau ou uma quantização intermédia que o
oráculo não faz medem outra coisa e parecem desvio.

**How to apply:** ao escolher os parâmetros de uma corrida de oráculo, verifique que nenhuma saída
exacta cai em `x,5`; e todo controlo/modelo replica a cadeia do oráculo sem quantizar onde ele não
quantiza. Ver [[reference_topic_oracle_discipline]].

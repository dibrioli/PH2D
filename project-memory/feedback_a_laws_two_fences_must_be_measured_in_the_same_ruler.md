---
name: a-laws-two-fences-must-be-measured-in-the-same-ruler
description: Uma lei com duas cercas ("isto conta?" e "isto ainda é válido?") só está medida quando a CÉLULA é medida — cada metade sozinha parece certa e o par mente
metadata:
  type: feedback
---

Uma lei que decide em duas etapas tem **duas cercas**, e elas parecem independentes: *«isto entra na
população?»* e *«dentro da população, isto ainda vale?»*. Não são. **O valor da segunda depende da
primeira**, porque a primeira define o conjunto em que a segunda foi medida.

**Medido no testbed do Cascadeur (2026-09-19)**, na opção que prende os pés na física dele:

* cerca 1 — *«o pé está no chão?»*. A régua do produto usa **5 mm** («no chão para o olho»); o motor
  usa **3 cm** (a janela em que ele decide prender). Emprestar a do motor faz a lei prender um
  **calcanhar que ainda está 3 cm no ar**.
* cerca 2 — *«isto é um apoio ou um pé a atravessar o chão?»*, pela **velocidade**, com a barra tirada
  do lado aprovado (capturas de pessoas reais). E ela **muda com a cerca 1**: `3,0 cm → 1,39 cm/quadro`
  · `1,5 → 1,75` · `0,8 → 3,05` · `0,5 → 2,20`.

A grade, com a queixa do dono (deriva / tranco piores, em cm):

| cerca | barra | o que o dono vê | capturas de laboratório |
|---|---|---|---|
| — (como veio) | — | 16,2 / 6,5 | 5,4 / 1,6 |
| 3,0 cm | 1,39 | **16,2 / 6,5** — não cura nada | 15,4 / 15,4 |
| 3,0 cm | 2,20 | 0,0 / 0,0 | **15,4 / 15,4** — estraga |
| **0,5 cm** | **2,20** | **0,3 / 0,3** | **1,9 / 1,1** |

As duas linhas do meio são **cada metade certa com a outra errada**, e as duas shipariam: a primeira
com a queixa do dono intacta, a segunda com um defeito novo numa população que ele não vê.

**Why:** eu shipei a linha `3,0 · 1,39` e ela estava internamente consistente — a barra *fora* medida
na cerca que a lei usava. O que faltava não era rigor em cada metade: era correr a **célula**. E a
célula que ganha só aparece quando as duas são medidas na régua do **produto** (aqui, a do
`medir_deslize.js`), não na régua do mecanismo que por acaso está à mão.

**⛔⛔ E o rabo da lição, que é o mais caro:** depois de corrigir a cerca que estava errada, **volte a
perguntar se a outra ainda faz trabalho**. Aqui não fazia: com a cerca de contacto certa, apagar a
barra da velocidade dá saída **byte a byte igual em 6 dos 8** exemplos. Os `13,8 cm` de teletransporte
que justificavam escrevê-la eram artefacto da janela de 3 cm — *a cerca certa, escrita pela razão
errada*. Quem o disse foi uma **mutação sobrevivente** no portão que a nomeava. A cerca ficou, mas
declarada como **seguro** e gateada onde de facto morde (uma fixtura construída), e não onde eu
julgava que mordia.

**How to apply:** quando escrever uma lei com duas cercas, **corra a grade** antes de escolher, e
guarde-a ao lado do número (aqui: `node sonda_ajustes_do_cascadeur.js --grade`). Se a segunda cerca é
uma barra tirada de um corpus, meça-a **com a primeira cerca aplicada** e escreva as duas juntas — uma
barra que veio de outra população é [[a-quality-bar-copied-from-another-doc-loses-the-density-it-was-measured-at]]
um nível acima. E nomeie a célula que piora: aqui é uma captura onde o tranco sobe 3 mm, e
[[every-ruler-measured-the-goal-and-none-measured-the-cost]] diz porque é que ela tem de estar escrita.

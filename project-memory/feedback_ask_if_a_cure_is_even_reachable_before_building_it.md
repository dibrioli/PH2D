---
name: ask-if-a-cure-is-even-reachable-before-building-it
description: Antes de construir a cura de um mecanismo nomeado, meça se ela tem SUJEITO — quantos casos ela de facto alcança.
metadata:
  type: feedback
---

Nomear um mecanismo plausível e **ir construí-lo** salta a pergunta mais barata:
***quantos casos essa cura alcança?*** Uma cura sem sujeito compila, corre, e não muda
número nenhum — e o tempo já foi gasto.

⛔ **Medido em 2026-08-27 na `line/quadextract`, quatro vezes seguidas sobre o MESMO
`NaN`:** «é a lei da Obra A» → «é o `relax_tie`» → «é o G3 contínuo» → «é a condição do A3
em falta». As quatro plausíveis, as quatro mortas por um controlo de minutos. A última
custou um solver inteiro para depois a coluna «quantas entraram» dizer **`0` de `10`** na
peça que diverge — *o dono que ela queria já tinha dono.*

**Why:** o padrão não é ignorância do mecanismo, é a ORDEM. Construir vem antes de contar
o alcance, e a contagem é sempre mais barata que a construção.

**How to apply:** antes de escrever a cura, escreva a coluna que conta **quantos casos ela
possui** e corra-a com a cura desligada. `0` responde a wave inteira. E quando a cura
entrar, essa coluna já existe para separar *«não fez nada»* de *«não correu»* — dois bytes
iguais sem ela. Irmão de [[counting-the-work-done-is-not-counting-the-work-delivered]] e de
[[a-measured-refusal-answers-one-question-recheck-it-when-yours-is-another]].

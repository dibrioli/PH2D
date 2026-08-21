---
name: feedback-a-conserved-invariant-cannot-grade-quality
description: Uma invariante CONSERVADA (Σ índice = 4·χ, soma de fluxo, contagem que fecha) não pode medir a qualidade daquilo sobre que ela é conservada — ela é verde por construção
metadata:
  type: feedback
---

Uma quantidade que a **topologia (ou a conservação) força** é um excelente gate de
*integridade* e um gate de *qualidade* **vazio**. Ela não pisca quando a resposta
piora, porque os erros entram em pares que se cancelam.

**Why:** medido duas vezes na mesma linha (quad remesher, F2).

1. A primeira tentativa de campo cruzado passava no gate `Σ índice = 4·χ` com
   **duas** singularidades de índice `+4` — um ponto onde a cruz dá uma volta
   inteira, que nenhuma grade de quads contorna. *A invariante prova que o campo
   FECHA, não que ele presta.*
2. ⚠️ **E a lição não pegou, porque a régua certa nunca virou gate.** Um ano de
   trabalho depois, o título de uma fase inteira dizia *"o CAMPO chegou ao ótimo
   teórico"* com base em `Σ = 8` e uma contagem de 8 medida **numa grade `uv`** —
   o caso mais fácil que existe. Na mesma esfera com distribuição irregular a
   contagem era **194**, e a soma continuava `8`. Um par `+1/−1` espúrio
   cancela-se, e podem existir 93 deles sem a soma mexer.

**How to apply:** quando um gate afirma uma soma/conservação, pergunte **o que ela
não pode ver** e escreva o segundo gate ao lado — quase sempre a **CONTAGEM** ou a
**dispersão**, contra um chão teórico conhecido (aqui: 8 numa esfera). E ⚠️ **varie
a fixtura na direção do caso difícil**: a contagem só se partiu quando a malha
deixou de ser uma grade regular, e nenhuma quantidade de esferas `uv` de tamanhos
diferentes teria mostrado isso. Irmã de
[[feedback_a_ratio_between_two_sick_channels_is_green_by_construction]] (verde por
construção pelo outro lado) e de
[[feedback_a_ruler_that_deduplicates_cannot_report_duplication]]; família em
[[reference_topic_gate_discipline]].

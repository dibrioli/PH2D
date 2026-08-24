---
name: feedback_a_house_param_name_carries_a_contract_pick_another_word
description: "Um nome de param da casa carrega um CONTRATO com gate atrás; reusá-lo para outra pergunta faz o gate reprovar sobre produto correcto — a cura é o nome, nunca afrouxar o gate"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-08-23T12:22:55.030Z
---

Ao dar ao `motion.wave` um param que escolhe **para onde a altura vai**, chamei-lhe `channel`.
A suíte reprovou num gate de arquitectura:

> `no_param_of_a_channel_driven_node_is_declared_a_fixed_length` —
> *"`motion.wave::spacing` is a fixed Length on a node whose magnitudes depend on `channel`"*

⚠️ **O gate estava certo e o meu produto também.** No vocabulário da casa, `channel` significa
*"o canal em que as MINHAS magnitudes se exprimem"* (`motion.stagger`, `motion.wiggle`), e daí a
regra: um nó com `channel` não pode declarar `ParamUnit::Length` fixo, porque em `Rotation` isso
escalaria **graus** por `pixels_per_meter`. O `spacing` da onda é uma distância de mundo em
qualquer canal — o meu param escolhia o **destino** da altura, não a unidade de nada.

**Why:** um nome de param neste catálogo não é um rótulo, é uma **declaração de contrato** que
outros sistemas leem — gates, o bridge de unidades, o painel. Reusá-lo para outra pergunta faz
o leitor aplicar o contrato errado, e o sintoma aparece longe da causa (aqui: uma asserção sobre
`spacing`, um param que eu não tinha tocado).

**How to apply:**
1. Antes de nomear um param, **grepe o nome no catálogo**: `grep -rn 'name: "<nome>"' crates/`.
   Se ele já existe noutro nó, ou significa a mesma coisa ou escolha outra palavra.
2. Cura = **renomear** (aqui: `height_channel`), com a razão escrita ao lado da declaração.
   ⛔ Nunca afrouxar o gate: ele protege um bug medido (`±90` mostrado como `±10`).
3. É a mesma lei do **`substeps`**, que esta linha já pagou: um sub-passo local a usar a palavra
   do relógio do grafo fazia o app correr as duas leis, e a corda caía 4,8× menos.

*A palavra tem dono. Quem chega escolhe outra.*

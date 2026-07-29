---
name: feedback-enforce-the-invariant-at-the-derivation-not-at-each-gesture
description: Quando um número AUTORADO tem de satisfazer um invariante geométrico, imponha o invariante onde a geometria é computada — e MEÇA qual LADO dele machuca, porque clampar o lado inofensivo clobba a autoria
metadata:
  type: feedback
---

O `L0` de uma corda de polia é semeado da rota que as roldanas desenham e depois **congelado** para a
corrida (*autorar re-deriva, o runtime congela*). Crescer um raio cresce a rota ⇒ a restrição
`L(rota) ≤ L0` nascia **violada** e o solver comia a diferença num tique (**+2,4 m ⇒ salto de 50 m**). A
1ª cura foi uma **PORTA** que os gestos de autoria chamam — e ela fecha o que ela conhece.

Aí a pergunta certa: **quantos gestos mudam a rota?** Medidos, com o `L0` parado: acrescentar uma roldana
(+2,88 ⇒ 13,97 m de salto) · mover o centro dela (+4,19 ⇒ 25,27) · digitar um comprimento curto (+6,97 ⇒
46,58) · e **apagar** uma roldana, que **nunca poderia passar por uma porta**: o delete da Hierarquia não
sabe o que é uma corda.

**Why:** a porta é uma **enumeração dos gestos** ([[feedback_a_condition_that_enumerates_its_readers_rots]]),
e a lista não pode ser completa — parte dos gestos chega por caminhos genéricos que não conhecem o domínio,
e o gesto N+1 nasce descoberto e **em silêncio**. O invariante, ao contrário, tem **um** lugar natural: onde
a grandeza derivada já é computada.

**How to apply:**
- **Conte os gestos antes de escolher o remédio.** Se algum deles chega por um caminho genérico
  (delete, load de arquivo, undo, um commit de campo compartilhado), a porta já perdeu.
- Ponha o invariante **onde a derivação já roda** — ali ele não é uma 2ª resposta e não pode discordar do
  semeio (*a mesma fórmula, um só lugar*).
- ⚠️ **MEÇA qual LADO do invariante machuca.** Aqui a violação POSITIVA explodia e a NEGATIVA media
  exatamente o salto do controle (0,0785 vs 0,0817) ⇒ a cura é um **PISO**, não uma re-derivação: para o
  lado inofensivo ela clobbaria um número que o artista digita. Uma cura simétrica sobre um defeito
  assimétrico apaga autoria de graça.
- **As duas camadas coexistem e cada uma quer gate:** a porta dá o número EXATO nos dois sentidos para os
  gestos que conhece; o piso garante o invariante para os que ninguém enumerou
  ([[feedback_layered_defenses_need_per_layer_gates]]).
- Sem porta de relógio **só se a grandeza derivada for função do estado AUTORADO** — verifique lendo o
  produtor, e **pine com um gate que roda o relógio**: a mutação "derive da pose VIVA" faz a corda ESTICAR
  durante a corrida e um dispatch pausado não distingue as duas poses.
- Escreva no máximo quando o valor MUDA (gravar o mesmo `f32` que se comparou ⇒ converge sem épsilon):
  componente que muda por frame é **um passo de undo por frame**.

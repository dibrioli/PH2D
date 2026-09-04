---
name: an-order-that-only-coincides-with-the-right-one-is-not-the-right-one
description: Uma mutação sobrevive quando a fixtura produz, por acaso, a ordem certa — a fixtura que a mata é a que produz a ordem INVERSA
metadata:
  type: feedback
---

O passe de instâncias percorre as cópias por `StableId`, e a ordem de criação **coincide** com a de
dependência (a nota da F5.1 já dizia *«coincide — não é derivada»*). Uma cura que escrevia o valor em
cada degrau intermédio parecia redundante: apagá-la deixou **sete** gates verdes.

A fixtura que a matou não é mais rigorosa — é **invertida**: uma segunda cópia interna metida dentro
do mestre externo **depois** de ele já ser receita nasce com identidade mais alta, e o passe passa a
avaliar o de fora primeiro. Aí a mutação mede-se: o valor aplicado volta ao antigo **durante um
quadro** antes de reaparecer.

**Why:** quando a correcção depende de uma ORDEM, todo corpus construído pelo caminho normal tende a
produzir a ordem normal — que é a certa. O gate mede então a coincidência, não a lei. É o irmão do
*«um corpus no ponto neutro de um knob não testa esse knob»*.

**How to apply:** ao ver uma mutação sobreviver sobre código que existe por causa de ordem
(topológica, de criação, de identidade, de z), não conclua «redundante» sem construir a fixtura que
produz a ordem **inversa** — e construa-a pelo MECANISMO (aqui: criar a peça interna depois do
mestre externo), nunca escrevendo ids à mão. Relacionado:
[[a-surviving-mutation-can-mean-the-code-is-redundant]],
[[where-new-objects-are-born-is-the-fixture-your-gates-are-missing]].

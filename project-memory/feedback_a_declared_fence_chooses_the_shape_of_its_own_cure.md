---
name: feedback_a_declared_fence_chooses_the_shape_of_its_own_cure
description: "Quando um doc-comment declara POR QUE um knob é como é, a cura da célula que se queixa dele tem de caber dentro dessa razão — a cerca não é o obstáculo, é o enunciado do problema"
metadata:
  type: feedback
---

Célula da folha 10: *"softness SEPARADA para a borda angular e a radial — uma
cunha fina com borda radial macia é inexprimível, `soft = 0.9` amacia as DUAS"*.
A leitura óbvia é *acrescente um segundo `soft`*.

Mas o `field.radial_sweep` já **declarava** por que tinha um só: *"uma knob
adimensional, porque as duas bordas vivem em unidades diferentes — graus vs
mundo"*. Um segundo `soft` absoluto reabriria exactamente a pergunta que essa
cerca fechou (*em que unidade?*).

A cura que cabe nas duas coisas é um **multiplicador adimensional**: `1` = as duas
iguais (byte-idêntico por aritmética), `0` = borda angular dura com a radial
macia — o caso que a célula nomeia —, `>1` = a angular mais macia.

**Why:** uma cerca documentada é uma decisão ([[feedback_documented_decision_chesterton_fence]]),
e a célula da conferência tinha razão sobre a CONSEQUÊNCIA sem a ter sobre a
FORMA. Tratar a cerca como obstáculo produz a segunda resposta que ela existia
para impedir; tratá-la como enunciado produz uma cura mais pequena e melhor.

**How to apply:**
1. Antes de acrescentar o knob que uma célula pede, **leia o doc-comment do knob
   de que ela se queixa**. Se ele declara um porquê, a cura tem de o honrar.
2. ⚠️ **A célula é sobre o SINTOMA; a forma da cura é sua.** A conferência
   escreve *o que não se consegue exprimir*, não *que param acrescentar* — e as
   duas coisas divergem sempre que existe uma cerca.
3. O irmão desta lei no mesmo bloco: a célula do `field.remap` pedia
   `Clamp Min`/`Clamp Max` **separados**, e um param novo teria mudado o sentido
   de `Clamp = 0` em toda cena já salva. A cura foi um **enum no param que já
   existia**, com `0`/`1` a manter o que significavam.
   [[feedback_convention_vs_inertia]]

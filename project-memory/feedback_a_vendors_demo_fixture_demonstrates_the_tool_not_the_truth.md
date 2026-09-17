---
name: feedback_a_vendors_demo_fixture_demonstrates_the_tool_not_the_truth
description: A animação de exemplo que vem com um app existe para MOSTRAR a ferramenta — medir a lei nela ensina o que a ferramenta faz com um caso quebrado, não qual é a lei
metadata:
  type: feedback
---

A amostra que um app traz embutida foi feita para **demonstrar** o que ele conserta. Medir a lei do alvo sobre ela mede o conserto, não a lei.

Medido no testbed do Cascadeur (14/09). O `Backflip_animation.casc` que vem com o programa parecia a fixtura perfeita: um movimento feito por quem fez o app. Medida ali, a física dele mudava as **pernas 22,8°** no único apoio da cena, contra ~3° das nossas — e daí saiu a hipótese *«no impacto forte a perna absorve, e a nossa lei é só-braços»*, com fixtura, régua e plano.

**A hipótese estava errada, e o dado que a derrubou foi captura de movimento de pessoas reais** (base da CMU, três saltos): **no pouso, as pernas dele mudam 2,8° e as nossas 2,8°**. Já éramos iguais.

**Why:** a animação de exemplo deles é **deliberadamente não-física** — a altura dela no voo seguia uma parábola de gravidade **−3,35 m/s²**, e depois da física **−9,80**. Ela existe para mostrar o AutoPhysics a consertar um salto ruim. Os 22,8° eram o tamanho do conserto daquele defeito, não uma lei sobre impactos. *Um movimento humano real já é física, e por isso a física quase não lhe toca* — é essa a linha de base, e uma demo nunca a dá.

**How to apply:** ao colher um oráculo, pergunte **para que a fixtura foi feita**. Amostra de app = demo (mede o conserto). O que mede a LEI é entrada **neutra**: movimento real capturado, ou o seu próprio movimento. Tenha as duas: as minhas animações à mão e a demo deles diziam a mesma coisa errada, e só o terceiro tipo de dado — pessoas reais — as separou. ⭐ E o mesmo corpus achou a diferença que EXISTE: todo o excesso das nossas pernas vem do «pés apoiados não escorregam» — um pé humano real percorre **11,5 cm** durante um apoio, o Cascadeur deixa 11,1, e nós cortamos para **6,6**. Ver [[project_teste_cascadeur_2d_bones_testbed]] e [[reference_cascadeur_oracle_door_measured]].

---
name: feedback-a-component-missing-its-contract-suspect-the-caller-first
description: "Quando um componente não entrega o que a própria doc dele promete, suspeite do CHAMADOR antes de trocar o componente — trocar esconde a causa e costuma introduzir um segundo defeito."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 23a3b93f-6014-49af-9da3-10003c04bb8c
  modified: 2026-07-19T22:04:21.183Z
---

Um componente cuja **docstring afirma uma propriedade** e que **não a entrega em produção** está, na
maioria das vezes, sendo **mal alimentado** — não mal escrito. Antes de substituí-lo, verifique se cada
entrada dele significa o que a doc dele diz que significa.

**Caso que pagou a lição duas vezes** (Painter, `heading::advance`, 2026-07-19): o filtro é um EMA
ponderado por comprimento e a doc dele promete *"behaviour is independent of how the path was chopped
into steps"*. O `walk_space` alimentava com o **resto dentro da corda de 3 px** (e cordas que não emitiam
dab não o avançavam), então ele rodava com passo até **16× pequeno demais** — suavização efetiva ~240 px
onde 12 era a intenção — e o Rake atrasava **52°**. Eu **troquei o estimador** por um secante de um
spacing: o atraso sumiu (52° → 1,7°) e o heading ficou **3-5× mais RUIDOSO**, o que num padrão repetido
lê como entrelaçado. Mesmo screenshot, bug diferente, e o usuário disse *"continuo achando que rake está
errado pois funcionava antes"* — estava certo. Consertar o **chamador** deu as duas metades: lag 3,7° no
pior caso **e** independente do spacing.

**Why:** um componente com contrato escrito é uma hipótese testável barata. Substituí-lo é caro, apaga a
evidência e troca o modo de falha por outro que ninguém está procurando.

**How to apply:**
1. Leia a doc do componente e **liste as premissas sobre as entradas**.
2. Instrumente/meça **uma entrada por vez** no caminho real (não no harness).
3. Só depois considere trocar o componente — e, se trocar, meça as **duas** dimensões (aqui: atraso *e*
   ruído), porque otimizar uma às cegas piora a outra.
4. O gate certo pina a **propriedade do contrato** (*"é fato do caminho, não do espaçamento"*), não um
   número: [[feedback_test_with_product_numbers_not_convenient_ones]], [[reference_topic_oracle_discipline]].

Parente de [[feedback_measure_perf_symptom_scale]] e [[feedback_remeasure_a_documented_residual_before_curing_it]].

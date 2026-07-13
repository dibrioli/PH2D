---
name: feedback-a-gate-only-proves-what-its-fixture-contains
description: Mutation-testar dentro de um universo de fixtures convenientes só prova coisas sobre aquele universo. A fixture é parte do gate — e é a parte que ninguém audita.
metadata:
  type: feedback
---

**Um gate só prova o que a FIXTURE dele contém.** Você pode mutar o código, ver vermelho, e
ainda assim não ter provado nada sobre o produto — se a fixture não for o que o produto
produz.

**Como aconteceu (Shape Builder, 2026-07-13):** escrevi 16 gates para o arranjo planar. Mutei
o código de propósito (removi a subtração, achatei a componente) e vi 3 e 2 ficarem
vermelhos. Fiquei confiante. O Enio smokou e **nada funcionava**.

Os números, medidos depois:

| | |
|---|---|
| Gates com forma do **catálogo** (curvas) | **0** |
| Gates com `Transform` **não-identidade** | **0** |
| Gates passando pela **ordem real do frame** | **0** |

Todos os 16 usavam **quadrados eixo-alinhados, construídos à mão, na identidade**, chamando a
lógica pura direto. Mas o usuário desenha com a Shape tool, e toda forma dela é uma **Live
Shape**: geometria **curva**, **centrada no local 0**, com a pose num **`Transform`**. Meus
testes nunca viram um único desses.

A mutação foi real. Ela só provou coisas **sobre quadrados**.

**Why:** mutation-testing dá uma sensação forte de rigor, e ela é merecida — mas ela audita a
ASSERÇÃO, não a ENTRADA. Um gate é `fixture → código → asserção`, e a mutação só exercita os
dois últimos. A fixture continua sendo aquilo que foi conveniente escrever.

**How to apply:**
1. Antes de aceitar uma suíte, pergunte: **de onde vem a fixture?** Se ela foi construída à
   mão com um literal, e o produto a constrói por outro caminho (um cook, um import, um
   gerador), a suíte tem um buraco do tamanho da diferença.
2. **Pelo menos um gate tem que nascer da PORTA REAL.** Se o produto chama
   `cook(ShapeKind::Polygon, …)`, o teste também chama. Se o produto passa por
   `Transform`/`bake_xform`, o teste também passa. Se o produto atravessa a ordem do frame, o
   teste atravessa.
3. O sinal de alerta é a **conveniência**: um quadrado eixo-alinhado na identidade é o que se
   escreve quando se quer que a conta feche na cabeça. É exatamente por isso que ele esconde
   erro de unidade, de espaço e de curvatura. (Irmã de
   [[feedback_test_with_product_numbers_not_convenient_ones]].)
4. E: **"simular o smoke no papel" não é smoke.** Ele percorre o roteiro que você IMAGINOU —
   o usuário percorre o que existe. Serve para achar bugs; **não** serve para ganhar
   confiança de que a feature funciona.

Relacionadas: [[feedback_mutate_the_code_not_just_the_test]] ·
[[feedback_tool_unit_green_integration_dead]] ·
[[feedback_painted_is_not_populated_paint_gate]]

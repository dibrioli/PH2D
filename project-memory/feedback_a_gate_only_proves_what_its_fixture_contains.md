---
name: feedback-a-gate-only-proves-what-its-fixture-contains
description: Mutation-testar num universo de fixtures convenientes só prova coisas sobre aquele universo — e um gate que mede o RETORNO de uma função não vê o que o usuário fica.
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

---

## A outra metade, descoberta no conserto (2026-07-13)

A fixture não era o único buraco. **Nenhum dos 16 gates olhou a CENA depois do gesto** — todos
mediam o **retorno** de `resolve()`. E o bug que o Enio viu era um **efeito colateral**: o
`build_up` dissolvia as três formas dele num blob único ao aplicar aquele retorno no documento.
Um gate que mede o valor devolvido por uma função **nunca vê a destruição que a função seguinte
comete**.

5. **O oráculo de uma ferramenta de edição é o DOCUMENTO, não o retorno.** Extraia a mutação de
   estado numa função pura (`commit(scene, sources, result) -> scene`) e faça o gate asserir
   sobre a cena resultante: *o que sobrou, com que id, com que estilo, com que geometria*. É a
   mesma disciplina do [[feedback_painted_is_not_populated_paint_gate]], um nível acima.

6. **Instrumente o app antes de teorizar.** O handoff da reprovação apontava um suspeito nº 1
   (xform stale) com código e tudo. **Estava errado** — o gate novo do arranjo, com formas do
   catálogo e rotação, nasceu **VERDE**. O que resolveu foi montar a cena do print DENTRO do
   app (env `PH2D_BUILD_SMOKE`), dirigir o gesto no frame real, imprimir os números e olhar a
   tela: 20 minutos, duas causas, nenhuma delas onde a teoria dizia.

7. **E o gate novo também pode ser frouxo.** Escrevi o gate do estilo, mutei o código — e ele
   ficou **VERDE**: media *"algum path que contém o ponto"* e pegava a forma NOVA em vez da
   sobra. Verde na mutação é **sempre** gate frouxo; não racionalize
   ([[feedback_mutate_the_code_not_just_the_test]]).

Relacionadas: [[feedback_mutate_the_code_not_just_the_test]] ·
[[feedback_tool_unit_green_integration_dead]] ·
[[feedback_painted_is_not_populated_paint_gate]]

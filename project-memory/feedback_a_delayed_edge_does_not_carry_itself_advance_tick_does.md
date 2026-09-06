---
name: feedback_a_delayed_edge_does_not_carry_itself_advance_tick_does
description: Um nó com estado (aresta `delayed`) medido ou desenhado sem `Cook::advance_tick` devolve o PRIMEIRO tique dele, que é a identidade por desenho — e lê-se como um nó morto.
metadata:
  type: feedback
---

No `ph2d-nodegraph`, a convenção sequencial é uma aresta **`delayed: true`** de `out` para a
porta `state` do próprio nó (`motion.spring`, `motion.delay`, e a família das simulações). Ela
**não se carrega sozinha**: quem passa a saída de um tique para a porta `state` do seguinte é
**`Cook::advance_tick(&graph, &registry, t)`**, chamado depois de cada `cook`.

Um laço que só coze em `t` crescente — sem `advance_tick` — faz o nó responder **para sempre o
primeiro tique**, que nesses nós é a **identidade por desenho** (não há passado ainda).

**Why:** o modo de falha é silencioso e tem duas caras, medidas as duas em 2026-09-06 no ciclo 2
dos Motion Nodes:
- uma **sonda de custo** cronometrou a mola e o atraso no tique em que eles não fazem nada, e
  devolveu o preço de outro nó;
- um **gerador de figuras** produziu, para o `motion.delay`, uma imagem **byte-idêntica** à da
  cadeia sem o nó — *ele parecia um controlo morto quando o morto era o meu laço*.

Nenhum gate acusa: o grafo está bem-tipado, a aresta existe, o `connect` não falha.

**How to apply:** em toda sonda, gerador ou fixtura que corre `cook` em vários instantes, chame
`advance_tick` a seguir a cada um — o precedente vivo é `the_ease_kills_the_twitch_and_keeps_the_motion`
em `shells/desktop/src/motion_delay_gate_tests.rs`. E ao medir CUSTO, meça **dois** números: o
tique **frio** (`Cook` novo — o que o nó custa quando tem de correr) e o de **regime** (o n-ésimo
tique do mesmo `Cook`), porque um nó `Pure` cujo resultado não muda com o tempo lê `0,00 ms` em
regime — o memo responde, e isso é a resposta, não um erro. Ver [[feedback_a_ruler_placed_after_the_tidying_step_measures_the_tidying]].

---
name: feedback_a_defect_the_tool_emits_and_cannot_read_is_a_feedback_loop
description: "Um defeito que a ferramenta EMITE e não sabe LER realimenta-se: feche os dois lados — não emitir, e reparar o que já existe"
metadata:
  type: feedback
---

Medido 2026-08-29: a retopologia emitia **doublets** (um vértice preso entre duas faces que
partilham três cantos) nas pontas finas — `19` na peça do artista. Sozinhos eram quase
invisíveis. Mas a **fase zero** da mesma cadeia, que só sabe remalhar superfície, não sabe o
que fazer com um vértice de duas arestas: ao voltar a entrar, a peça saía com `χ = 2 → 6` e
uma aresta não-manifold, e o solver a jusante entrava em `index out of bounds`. *Cada volta
piorava a anterior, e o artista via «o remesh amputou pontas».*

⭐ **A forma geral:** quando o **produtor** e o **consumidor** são a mesma ferramenta, um
defeito que ela emite e não sabe ler não fica onde nasceu — ele **amplifica**. E o sintoma
aparece na volta seguinte, longe da causa.

**Why:** curar só o lado da produção deixa **toda peça já gravada** a partir a ferramenta para
sempre; curar só o lado do consumo deixa o defeito a nascer. As duas metades são obrigatórias,
e a terceira — *uma tentativa que estoura PERDE, em vez de derrubar tudo* — é o que impede o
artista de perder o trabalho enquanto as outras duas não existem.

**How to apply:** quando a saída de uma ferramenta pode voltar a ser a entrada dela,
(1) meça a saída com as réguas da **entrada**, (2) não emita o que a entrada não sabe ler,
(3) **repare** na porta o que já existe, e (4) ponha uma rede à volta da tentativa. ⚠️ E
desconfie de fixturas sintéticas: uma bola de espinhos varrida de `σ = 0,30` a `0,05` **não
reproduziu nada** — *não era a espessura sozinha, era a espessura mais a mordida que já lá
estava*, e só a peça real a continha. Relacionado:
[[feedback_a_closed_surface_can_contain_a_second_one_count_the_components]] ·
[[feedback_a_knob_whose_range_is_derived_from_the_object_it_rewrites_is_not_idempotent]] ·
[[feedback_where_new_objects_are_born_is_the_fixture_your_gates_are_missing]]

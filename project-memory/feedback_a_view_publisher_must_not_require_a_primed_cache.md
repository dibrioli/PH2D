---
name: feedback_a_view_publisher_must_not_require_a_primed_cache
description: Exigir cache primed num publicador de view é contrato de ordem escondido; quem AUTORA num instante pode exigir, quem PUBLICA não
metadata:
  type: feedback
---

`TimelineViewSnapshot::rebuild` passou a precisar do scratch da pilha (para o relógio do
clip). Eu segui o contrato existente do módulo — "o caller PRIMA, e um `debug_assert` cobra" —
porque o `key_home` já era assim. **4 gates ficaram vermelhos na hora.** Fix: o `rebuild`
prima sozinho (`&mut TimelineState`), pulando quando não há pilha (custo zero no caso comum).

**Why:** o contrato é certo para quem **autora** (o K: um instante específico, código que sabe
o tempo, e prima explicitamente). É errado para quem **publica uma view**: `rebuild` é chamado
por qualquer coisa que queira olhar a timeline, todo frame. No shell o apply roda antes por
acaso — e "por acaso" é precisamente o que torna o acoplamento invisível até alguém reordenar
o frame e a régua apontar, calada, para o instante anterior. É a MESMA classe que já quebrou
o módulo três vezes ([[feedback_derived_coordinate_seed_must_match_sample]]).

**How to apply:** ao dar a uma função uma dependência de cache, pergunte **quem chama**. Poucos
callers que conhecem o instante → pode exigir prime. Um publicador/getter/view → ele mesmo
prima. E note o sinal: o `debug_assert` do módulo **fez seu trabalho** — o teste que fica
vermelho quando você estende um contrato está te dizendo que o contrato não estica até ali,
não que os testes estão errados. Ver [[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]].

---
name: feedback-a-probe-that-arms-a-module-by-env-var-measures-another-program-than-the-pill
description: Quatro jornadas de sondas do undo 3D passaram VERDES sobre um produto partido porque armavam o módulo por `PH2D_FIELD_SMOKE` e o dono arma-o pelo pill — a ordem "cena nasce vs. primeira captura" invertia-se, e o cache incremental nunca vigiava as colunas do módulo.
metadata:
  type: feedback
---

Enio, 2026-09-04, quinto report: *«o undo é um bosta, não melhorou nada — auditoria completa»*.
Todas as sondas anteriores (`PH2D_FIELD_UNDO_PROBE` v1–v5) armavam o modelador por
`PH2D_FIELD_SMOKE=1`. A cena de demo é a mesma nos dois caminhos; a **ordem** não é: com a variável
ela nasce **antes** da primeira captura de undo, pelo pill nasce **depois**. O cache incremental
resolvia a lista de colunas vigiadas **uma vez**, na primeira captura, a partir de
`world.component_id::<T>()` — `None` para um tipo que o mundo ainda não usou. Pelo pill a primeira
captura vê a cena vazia ⇒ `FieldPose` nunca entrava na lista ⇒ mover com o gizmo, arrastar um
slider, digitar um número (escritas **no lugar**) eram invisíveis; só spawn/despawn/troca de
archetype viravam passo. Era, letra por letra, *«não obedece cada etapa, principalmente se
transformação»* e *«um Ctrl+Z apaga tudo»*.

**Why:** uma sonda que arma o sistema de outra maneira que o utilizador **mede outro programa** —
e quanto mais verde ela passa, mais convence. O que muda não é a cena, é a **ordem** de nascimento
relativa a um cache que se prime uma vez.

**How to apply:** a sonda entra pela **mesma porta** que o dono (aqui: o pill, `ask_open_panel`,
sem a variável). E todo cache que se "prime" sobre *o que existe agora* tem de responder à pergunta
*«e o que nascer depois?»* — aqui a cura é re-resolver quando `world.components().len()` cresceu.
Gate headless que reproduz o pill: `a_component_type_born_after_the_first_capture_is_still_watched`.
Ver [[feedback_a_correct_undo_queue_without_the_selection_reads_as_a_broken_queue]] e
[[feedback_where_new_objects_are_born_is_the_fixture_your_gates_are_missing]].

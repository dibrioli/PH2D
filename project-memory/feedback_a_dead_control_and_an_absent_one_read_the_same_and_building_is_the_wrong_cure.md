---
name: feedback-a-dead-control-and-an-absent-one-read-the-same-and-building-is-the-wrong-cure
description: "Antes de dar casa nova a um comando, procure o controlo que JÁ faz a pergunta — construir por cima de um morto dá dois sítios para o mesmo verbo"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-09-01T19:57:31.536Z
---

Em 2026-09-01 mudei os verbos do gizmo 3D (`Move`/`Rotate`/`Scale` + `Global`/`Local`) do painel
para um **pulldown novo** na fila de ferramentas. O Enio devolveu uma foto do trilho: *«esses
botões de mover, rot e scale já existiam. só não estavam ligados a cada modo.»*

Estavam. Medido depois do report: `TOOL_TRANSLATE`/`TOOL_ROTATE`/`TOOL_SCALE`/`TOOL_PIVOT` e o
`tool_space_local` do `SPACE` eram um rádio exclusivo que escrevia **só a própria luz** — fora o
pintor do chip, **zero leitores em toda a árvore**, nos dois modos do editor.

**Why:** o sintoma de um controlo **morto** e o de um **ausente** é o mesmo — *«não consigo fazer
isto pela tela»*. E o diagnóstico natural (*«falta um controlo»*) leva a **construir**, que é a
cura do ausente aplicada ao morto. O resultado é o app com **dois** sítios para o mesmo verbo, e o
que apodrece é o que ninguém relê. ⚠️ Custou uma entrega inteira: construir o pulldown, apagá-lo,
ligar os chips e apagar as famílias de id duplicadas.

**How to apply:** antes de dar casa nova a um comando, faça a pergunta que eu saltei —
***quem mais, na tela, já faz esta pergunta?***

- Procure pelo **verbo**, não pelo módulo: os chips do gizmo 3D chamavam-se `TOOL_TRANSLATE` e
  viviam no `ph2d-editor-core`, a três crates de distância do painel que eu estava a esvaziar.
- O teste é `grep` do **consumidor**, não do controlo: *este `ButtonState` / este campo do store é
  LIDO por alguém que decide?* Nenhuma sonda deste repo faz essa pergunta
  ([[feedback_a_dead_knob_has_two_species_no_probe_catches]]).
- **Ligar e apagar é UMA obra.** Ligar o que existe e deixar o duplicado de pé troca um morto por
  um órfão ([[feedback_an_orphan_id_and_a_dead_knob_read_the_same_and_their_cures_are_opposite]]).
- ⚠️ **Espaço que cabe não é razão para o gastar.** A medição dizia que a fila aguentava 3 chips de
  área; usá-los foi o que fez o segundo parecer barato.
- Escreva o gate que impede a reconstrução (*«a área não oferece uma segunda porta para o
  gizmo»*) — a mutação que o mata é o meu próprio erro repetido.

Irmãos: [[feedback_the_design_being_asked_for_may_already_be_law_in_another_half_of_the_app]] ·
[[feedback_a_parameter_that_changes_nothing_is_discarded_downstream]] ·
[[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]

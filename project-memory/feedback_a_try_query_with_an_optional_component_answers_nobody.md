---
name: feedback_a_try_query_with_an_optional_component_answers_nobody
description: O `try_query` do bevy devolve `None` quando QUALQUER componente da consulta é desconhecido do mundo — e um `Option<&T>` conta: a porta responde «ninguém» e nada o diz.
metadata:
  type: feedback
---

Medido em 2026-09-14 (`line/components`, W4 das Tags). A porta `ph2d_ecs::tags::tagged` perguntava
`world.try_query::<(Entity, &Tags, Option<&StableId>)>()` — o `StableId` só para **ordenar** a
resposta. Num mundo onde nada nunca teve `StableId` (ele é atribuído pelo caminho do NOME e pela
captura do undo, **não** pelo spawn), o `try_query` devolve `None` e a porta responde **vazio**: um
sinal dirigido a uma tag não alcançava nada, e **nada na tela o dizia**.

**Why:** o `try_query` é a variante que não regista componentes novos; ela falha assim que um dos
tipos da consulta é desconhecido do `World`, e a opcionalidade é sobre a ENTIDADE ter o componente,
não sobre o MUNDO o conhecer. *Um `Option<&T>` numa consulta lê-se como «não faz mal se faltar» e
significa «o mundo tem de saber que ele existe».*

**How to apply:** um componente que a consulta só usa para **decorar** a resposta (ordenar, rotular)
lê-se **por acerto** (`world.get::<T>(e)`), fora da query — o custo é uma leitura por ACERTO, e os
acertos são poucos. ⚠️ E a fixtura do gate que mede a ORDEM **não contém o fenómeno**: ela spawna
*com* o componente, porque é ele que ela mede. O gate novo spawna *sem*, com o controlo explícito
(`all(|e| e.get::<T>().is_none())`). Irmãs:
[[feedback_a_gate_that_compares_two_constructions_is_blind_to_a_shared_mutation]] ·
[[reference_topic_fixture_discipline]]

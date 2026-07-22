---
name: feedback_two_quantities_that_should_differ_can_coincide_by_fixture_phase
description: "A gate proving max≠last (or peak≠endpoint) is green-over-nothing if the fixture makes the two coincide; pick a fixture where they differ by PHYSICS, not luck."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 4d33a1a5-4cde-45a3-94d1-ae7125cd56bf
  modified: 2026-07-22T23:27:26.806Z
---

Um gate que prova que uma grandeza é o **máximo/pico** de outra (não o último/endpoint) fica
**verde sobre nada** se o fixture faz as duas coincidirem por acaso de fase.

**Caso real (W-ImpactForce, física, 2026-07-22):** `impact` = `max` do impulso sobre os
sub-passos; `impulse` = o último sub-passo. Num tick de POUSO o corpo é pego mais forte NA
fronteira do tick, então `impact == impulse` — e a mutação `impact = impulse` **sobrevive**.
Medido: no caminho da ponte o pouso dá `load == impact == 3,00`. As duas grandezas SÓ diferem
num tick de RASPÃO (o corpo saiu antes do último sub-passo: endpoint ~0, pico não). O fixture
*parecia* conter o fenômeno (havia contato, havia impacto) — mas não continha a **diferença**.

**Why:** "o fixture contém o fenômeno" ([[reference_topic_fixture_discipline]]) tem um degrau
a mais aqui: as DUAS grandezas existem, mas coincidem, então o oráculo não vê a mutação. É a
mesma classe de "verde por acidente" da [[reference_topic_gate_discipline]].

**How to apply:** antes de confiar num gate `A é o max/pico/união de B`, MEÇA se A e B
diferem no fixture. Se coincidirem, ache um fixture onde a diferença é **estrutural** (física,
garantida), não incidental — no caso, bola que QUICA + chão FINO, onde um raspão (endpoint ~0,
pico grande) é garantido pela física do quique, e a mutação colapsa `impact` no endpoint
pequeno → RED. Vale para qualquer max-vs-last, sum-vs-endpoint, união-vs-último.

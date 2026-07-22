---
name: feedback-a-gesture-report-needs-a-fixture-containing-the-gesture
description: "Report sobre um GESTO (\"clico e não vira passo de undo\") não se fecha com fixture que chama a mutação direto — ela pula a costura input→bus→drain→hook"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 85e38f84-1b86-49d2-aee2-91da101e1fd7
  modified: 2026-07-22T00:20:39.786Z
---

Quando o usuário reporta que um **gesto** não funciona, a fixture tem de conter o gesto. Uma que
chama a mutação direto (`fx_bridge::add(...)`) prova que o **estado** ida-e-volta e **não toca** a
pergunta que ele fez: *"o meu CLIQUE virou um passo?"*. Não-reprodução sobre essa fixture não é
não-reprodução do report.

**Why:** entre o clique e o efeito há uma máquina que a fixture direta não atravessa — o Click nasce
no `Up`, viaja pelo bus, é aplicado dentro do `render_frame`, e o hook de undo decide por flags que
vivem no ritmo dos EVENTOS, não no do drain. Caso real (PH2D, efeitos vetoriais, 2026-07-21): dois
agentes e quatro varreduras não reproduziram e a sessão encerrou com *"Não afirmo que fechou"*; um
probe que clica pelo hit-index (Down e Up em frames **separados**, como um dedo) respondeu em uma
corrida — e a mutação canônica reproduziu o sintoma relatado exatamente.

**How to apply:** dirija a ENTRADA real, não a função. Dê ao probe uma **tabela de expectativas**
(frame → os números que importam) e **um veredito** no fim — parede de telemetria é lida errada por
humano cansado. E prove que ele sabe ficar VERMELHO: sem a mutação que reinstala o bug descrito, um
probe verde não distingue *"funciona"* de *"não observa"*. Ver [[reference_topic_repro_discipline]],
[[reference_topic_fixture_discipline]] e [[feedback_try_to_build_the_harness_before_declaring_it_impossible]].

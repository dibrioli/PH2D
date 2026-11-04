---
name: project_animation_section_like_godot_2026_11_04
description: "Dono 04/11 — seção Animation simples como a Godot: overrides e Direction fora da vista; o visual novo do Inspector é de outra linha e a GEF não alinha agora"
metadata:
  node_type: memory
  type: project
  originSessionId: a3939cb5-3050-4af7-a16f-54772307d66c
  modified: 2026-11-04T21:01:56.684Z
---

Dono 04/11: «Ping-pong tem múltiplas ocorrências», «toda seção precisa ser revisada», e «veja como a Godot faz, pois ela faz bem feito e de forma simples». Aprovou tirar Direction override, Loop override e Direction; «pode fazer como deseja, smoke só depois de corrigir tudo».

**Why:** medido na Godot 4.7.2: o objecto só tem velocidade; a animação tem fps + repetição (nenhuma, normal, ping-pong); ao contrário = velocidade negativa.

**How to apply:** Direction e overrides só aparecem quando há um valor gravado que senão ficaria invisível; não os devolver à vista. Na mesma conversa o dono disse que outra linha cuida do visual novo do Inspector, por isso a `line/GameEngineFuncional` NÃO alinha ao main por agora nem mexe no visual dos cartões. Plano: `docs/GameEngineFuncional/22_animacao_revisao_plano.md`.

---
name: project-ray-sensor-improvements-owner-decisions-2026-11-15
description: Dono 15/11 (GEF 07 RS-1..RS-12) — material do acerto = tags da forma atingida; Shape em dropdown Line·Thick·Cone; cartão por pergunta; Also ignore = lista por conta-gotas; desligar com o raio vendo dispara On Exit
metadata:
  type: project
---

Decisões do dono ao smoke das melhorias do Ray Sensor (smoke OK 15/11).

**(a) «Material» do acerto = as TAGS da forma de colisão atingida + atrito/restituição; nenhum tipo de material novo.**
O estado da arte (Unreal Surface Type) foi apresentado; o dono escolheu tags.
**Why:** a tag já existe, o artista já a usa no filtro; um tipo novo duplicaria a fonte da verdade.
**How to apply:** o script lê o acerto pelas tags da forma filha + friction/bounce; não crie `Material`.

**(b) Shape do raio = DROPDOWN Line · Thick · Cone; só o número da forma ativa aparece** (Thickness para Thick, Cone para Cone; Line sem número). O cone não usa a thickness.
**Why:** campo que não decide é controlo morto.
**How to apply:** `largura_activa`/`cone_activo` decidem; não reintroduza os dois números sempre visíveis.

**(c) Cartão arrumado POR PERGUNTA:** Enabled | Angle · Turns with facing · Reach | Aim at · Shape | número · grupo «Layers» (Layer + Sees juntos, depois Rename/Open in tab) · Filter · Signals.
**Why:** o dono pediu Layer e Sees numa seção própria e próximos; Reach|Aim at e Shape|número na mesma linha.
**How to apply:** nos dois modos (cartões e clássico); só muda lugar, ids e canais ficam.

**(d) «Also ignore» = LISTA de objetos por conta-gotas** (ordem do dono, acima da caixa de tags da crítica).
**Why:** o artista aponta o objeto na cena em vez de digitar tags.
**How to apply:** referência durável por StableId/nome, nunca bits.

**(e) Desligar um raio que está vendo dispara On Exit** (regra do Unreal).
**Why:** quem ouve o sinal não pode ficar com «ainda vejo» preso.
**How to apply:** o desligar fecha o estado como um «deixou de ver».

Relacionado: [[project_ray_sensor_is_a_child_object_2026_11_15]] · [[project_ray_family_ignore_up_to_field_2026_11_15]].

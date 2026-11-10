---
name: project_collision_shape_look_2026_11_10
description: Dono 10/11 — a forma de colisão tem o Look da Sprite (Color que substitui, Opacity ≥ 25 %, resets, Pivot com X/Y sempre à vista) e a ferramenta Pivot do canvas
metadata:
  type: project
---

Dono 10/11 (GEF02 onda 8): a Collision Shape (objeto filho, ADR-0199) ganhou o cartão **Look** e a ferramenta **Pivot** do canvas.
- A forma cresce em volta do PIVÔ dela quando Width/Height mudam (o pivô é fração da meia-extensão, `ShapeLook`).
- Retorno do smoke: Opacity mínima **25 %**; a Color **substitui** a cor da camada (não multiplica); ícone de **reset** na Color e na Opacity; sem o «…», «Pivot from centre» sempre à vista, X e Y empilhados ao lado da grade.
- «Animar o pivot fez a shape balançar» curou-se com a lei da Sprite (GEF 23: chave da trajetória em canto, ritmo linear) — o pivô da forma conta como âncora.

**Why:** pedido do dono no smoke; Self Tint e cantos não entram (forma sem imagem).
**How to apply:** todo leitor do centro da forma passa por `ph2d_physics_ecs::forma_resolvida` — a foto apanhou um que não passava (a caixa do gizmo). Handoff: `docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_LOOK.md`. Ver [[project_physics_fluid_under_animation_2026_11_10]].

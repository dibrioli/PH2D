---
name: project_jump_arc_on_canvas_2026_11_10
description: Dono 10/11 — a curva do pulo (versão C da crítica do Character) vai para o CANVAS; no cartão só o botão que a liga e o seletor Floaty·Snappy·Heavy
metadata:
  node_type: memory
  type: project
  originSessionId: 82471797-dc12-4536-bcaa-c8a8049b8321
  modified: 2026-11-11T00:16:56.599Z
---

Dono 10/11, depois do smoke OK do Character versão B (line/GameEngineFuncional, ficha `docs/GameEngineFuncional/componentes/03_character.md`):

- A ideia C da crítica («Puxe o pulo») entra, mas **no canvas, não no cartão**. No cartão Platform Player: só o **botão que liga a curva do pulo no canvas** e o seletor **Floaty · Snappy · Heavy** (as sensações ficam no CARTÃO).
- O canvas mostra **todas** as curvas: pulo inteiro (com a alça), pulo curto (botão solto cedo, Cut Gravity), pulos no ar (Air Jumps > 0) e as duas larguras, parado e correndo à velocidade máxima.
- A alça do topo muda **altura E distância**: para cima = Jump Height; para o lado = Speed (para o pulo chegar até ali).

**Why:** brincar com a sensação do pulo antes de ler números; o cartão já foi enxugado na B e não deve voltar a crescer.

**How to apply:** a curva tem de ser a da LEI (simular o `ph2d-platformer` com os números do player, não uma parábola ilustrativa); os valores de Floaty/Snappy/Heavy são decisão técnica delegada (estado da arte) e vão ao smoke. Ver [[project_game_engine_refounded_item_by_item_2026_11_05]].

Entregue 10/11, smoke do dono OK: versão B com linhas coloridas (Full ObjPlayer forte; Short Warn, Standing Info, Air ObjMacro tracejados); a sensação conserva o ápice; ficha 04, handoff CURVA_DO_PULO.

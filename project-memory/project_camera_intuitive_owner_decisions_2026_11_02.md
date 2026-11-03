---
name: project_camera_intuitive_owner_decisions_2026_11_02
description: "Dono 02/11 — câmera intuitiva: alvo do Follow por arrastar da Hierarchy, escrever ou Eyedropper (nunca lista); zona morta no MEIO da moldura; gizmo rico como o das shapes do Vector"
metadata:
  node_type: memory
  type: project
  originSessionId: 51be4b96-d7c0-4d91-b0d3-e59e4b17efb6
  modified: 2026-11-02T15:36:27.733Z
---

Dono, 02/11 (linha GameEngineFuncional, auditoria de intuitividade da câmera antes das moedas/J5):

- **Alvo do Follow:** arrastar da Hierarchy para o Inspector, escrever o nome, e um **Eyedropper** no canvas. ⛔ Lista/dropdown de objetos NÃO — a cena pode ter muitíssimos objetos.
- **Zona morta:** a caixa fica **no meio da moldura** (como Godot/Unity), o alvo anda dentro dela. Revoga o desenho anterior (caixa centrada no alvo, `centro_da_zona = mira`).
- **Gizmo da câmera:** o dono gosta de gizmos ricos e intuitivos como os das shapes do Vector — o da câmera deve subir a esse nível (alças distintas por função, nomes, realce).
- «Vamos corrigir tudo»: os 7 achados da auditoria (dicas nos campos, alvo, três retângulos indistintos, zona morta, rótulos cortados/cobertos, limites padrão ±10 m, cena 4 abrindo sem o jogador) numa onda só.

**Why:** a câmera tem de ser entendida sem manual antes de seguir o jogo.
**How to apply:** plano `docs/GameEngineFuncional/20_*`; nenhuma lista de objetos em campo de alvo; ver [[project_game_engine_plan_owner_decisions_2026_10_25]].

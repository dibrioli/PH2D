---
name: project-collides-with-override-per-shape-2026-11-08
description: "Dono 08/11 (smoke opção B) — «Collides with» de uma forma vale só para ela, como exceção sobre a tabela da camada (Unity include/exclude)"
metadata:
  node_type: memory
  type: project
  originSessionId: e7c8bb8c-4f13-473e-9631-ed6d2dead57d
  modified: 2026-11-09T11:09:00.084Z
---

Dono 08/11, smoke da opção B (`line/GameEngineFuncional02`): com 3 Collision Shapes no Crate, desmarcar Ground no «Collides with» de uma desmarcava as três — o cartão editava a linha da TABELA de camadas (uma regra por camada). Escolheu **exceção por forma** (o idioma do Unity, `includeLayers`/`excludeLayers` do Collider2D): a tabela da aba Collision Layers continua a ser o padrão de cada camada; o «Collides with» no cartão de UMA forma vale só para ela; o cartão mostra que ela difere da camada, com volta ao padrão. Recusados por ele: lista própria por forma sem tabela (Godot) e deixar como estava.

**Why:** cada forma é agora um objeto (opção B, [[project-outline-polygon-owner-decisions-2026-11-08]]); o dono espera ajustar cada uma sozinha.

**How to apply:** a exceção mora na FORMA (lado-forma, migra/Remove a levam), muda o formato (degrau de schema); a tabela e o «Sees» do Ray Sensor não mudam de sentido.

**09/11 (dono):** a aba NÃO mostra a exceção na matriz por desenho (é a regra da camada) — mas tem de a DENUNCIAR: (1) no cartão, a frase «Differs from layer» e o nome dos quadros fixados vão em destaque (`Warn`); (2) na aba, sob «Objects on this layer», «Differ from this layer» com um botão por forma (seleciona) e o que difere («Ground off · Props off»); (3) um ponto `Warn` na célula da grade onde alguma forma tem regra própria; (4) os nomes dos objetos da aba mostram o dono («Crate › Collision Shape»). E **nomes únicos**: forma nova (e o corpo do Make Independent Body, e a forma copiada pelo Duplicate) nasce «Collision Shape (n)» pela lei do app (` (n)`, `ph2d-unique-name`) — o dono pediu «sufixo 002»; manteve-se a lei única do app e disse-se-lhe.

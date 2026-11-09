---
name: project_physics_cards_owner_decisions_2026_11_07
description: "Dono 07/11 (GEF02): Physics Body em dois cartões Body e Shape, alças no canvas, Touch Solid|Sensor, polígono depois, Static nasce com pedra, forma solta aparece no canvas; colisão: matriz + Collides with no objeto, 32 camadas"
metadata:
  node_type: memory
  type: project
  originSessionId: 18b42431-425d-4ebc-a825-63896692c04e
  modified: 2026-11-07T19:34:44.049Z
---

Dono, 07/11/2026, aceitou as cinco recomendações da proposta dos cartões de física:

1. Dois cartões, Body e Shape (opção B), com dobras com nome.
2. Alças da forma no canvas entram nesta revisão.
3. Rótulo Touch, opções Solid | Sensor; «Trigger» segue a ser o componente da tecla.
4. Polígono pelo contorno da imagem é item próprio, logo depois.
5. Static Body continua a nascer com o bloco de pedra.

No smoke do mesmo dia (cena `PH2D_PHYSICS_SMOKE=120`): 12 de 13 passos aprovados. Ao achar a lista Layer, disse que não via
com quem o objeto colide. Decidiu que a forma solta **aparece no canvas**.

Depois de pesquisa (`docs/GameEngineFuncional/componentes/f1_pesquisa_camadas.md`), decidiu, por «faça como sugere»:

- **Matriz global + fila «Collides with» de quadros clicáveis no objeto**, que edita a mesma regra da matriz, vale para a camada
  inteira e diz isso. Ele gosta do Godot, mas aceitou este desenho. Exceção por objeto seria item próprio.
- **32 camadas**, mostrando só as que importam, com botão «+ Layer». O teto de 8 não tinha medição.

Depois do cartão próprio «Collision Layers», ainda 07/11: **a matriz abre num MODAL central**, como o editor de animações do Animated Sprite (aspecto visual igual ao da `line/GameEngineFuncional` atual): só ali cabem as 32. **Renomear por duplo clique** no nome, no Inspector e no rótulo à esquerda da matriz. **Camada criada chama-se «Layer n»** (ex.: Layer 20).

Ainda 07/11: contornos do canvas na cor da camada, tipo do corpo pelo traço (Static grosso, Dynamic fino, Kinematic pontilhado), e **sombra escura por baixo dos contornos** (sim). Cor da camada: contorno na moldura dos elementos, menos na lista «Layers» da aba, que tem o quadradinho antes do nome. Botões e caixinhas da aba no tamanho do editor de animações. Smoke da onda 9g OK; próximo: fechar a linha para integração; polígono depois, como item novo.

**Why:** ganhar facilidade sem perder ajuste, conforme [[project_physics_cards_keep_all_config_power_2026_11_07]].

**How to apply:** o desenho, o mapa campo a campo e as ondas vivem em
`docs/GameEngineFuncional/componentes/f1_physics_static_shape.md`, com a maquete ao lado.

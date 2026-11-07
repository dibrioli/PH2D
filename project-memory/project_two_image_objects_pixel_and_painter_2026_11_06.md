---
name: project_two_image_objects_pixel_and_painter_2026_11_06
description: "Dono 06/11 — Pixel e Painter são ferramentas diferentes, logo 2 tipos de objeto; 2 ícones (canvas e pixel), sai o do Painter; cada objeto só com o botão da sua ferramenta; sem modos Draw novos"
metadata:
  node_type: memory
  type: project
  originSessionId: 2a24e826-c122-4d27-9ce6-f954979089ce
  modified: 2026-11-06T19:41:02.772Z
---

Dono, 06/11 (smoke da Sprite): *«as ferramentas Pixel e Painter são totalmente diferentes e se prestam para artes e
objetos e animações diferentes. Logo não podemos ter um objeto só.»*

1. **Dois tipos de objeto**, como já é: imagem (Painter) e Pixel Sprite. ⛔ Não unificar.
2. **Dois ícones na Hierarchy: canvas e pixel.** Sai o terceiro, o do Painter.
3. **O objeto Pixel tem só o botão de editar no Pixel; o de imagem, só o do Painter.** Revoga a decisão 4 da ficha da
   Sprite (05/11, «Edit in Pixel completo») para objetos de imagem.
4. O mesmo pedido, minutos antes, propunha um modo novo por ferramenta («Draw em Pixel» e «Draw em Painter»): caiu com
   os dois objetos. Cada objeto segue com o Draw da sua ferramenta e o `Tab` como está.

5. **As camadas saem da Hierarchy:** vivem só no Painter e no Pixel (06/11). O resto do «conteúdo = sub-linha» de
   [[project_game_engine_refounded_item_by_item_2026_11_05]] não foi revisto.
6. **Abrir no Painter uma imagem em Repeated mostra-a sem a repetição** (06/11).
7. **A miniatura do cartão Image não muda ao estreitar o painel, e o tamanho não sobe ao título** (06/11).

8. **O Pixel Sprite ganha um cartão com o botão Pixel, igual ao da imagem** (miniatura, nome, tamanho, botão), que antes
   não existia (06/11).

9. **Imagem no Painter (06/11, aceitou as recomendações):** o retângulo verde fica na largura do objeto; a cópia abre no
   centro, mesmo sem coincidir com uma cópia em número par; a Region abre a imagem inteira, com o salto na tela.

**Why:** cada ferramenta serve a arte, objetos e animações diferentes; um objeto só esconderia essa diferença.

**How to apply:** a janela de criação ganha tamanhos comuns de pixel art e Custom (pedido do mesmo dia). Ficha:
`docs/GameEngineFuncional/componentes/01_sprite.md`. Ver [[project_sprite_card_owner_decisions_2026_11_05]].

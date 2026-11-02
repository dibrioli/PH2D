---
name: project-image-is-a-project-asset-reference-2026-10-26
description: Dono 26/10 (smoke J4) — trocar a imagem de qualquer objecto como na Unity: a imagem é um asset do projeto escolhido por referência; vale para Sprite, Background Layer, Animated Sprite/Player, Flip, Vector e Pixel.
metadata:
  type: project
---

Smoke do J4 (GameEngineFuncional, 26/10): o dono aprovou a Background Layer e apontou uma falha de DESENHO da
engine — não consegue mudar a imagem da sprite do fundo («a Godot resolve de forma muito apropriada»). Pediu o
padrão Unity (somos mais parecidos com ela): a imagem é um ASSET do projeto (painel com miniaturas); o objecto
guarda só a REFERÊNCIA (campo «Sprite» do Sprite Renderer: arrastar do Project, ou o seletor com todas as
imagens e busca; arrastar para a cena cria o objecto). E acrescentou: **o Player e as sprites animadas precisam
do equivalente (a folha), e o mesmo para Flip, Vector e Pixel** (Pixel = linha paralela).

**Why:** a arte de fábrica vive embutida no programa (chaves reservadas do atlas), não como imagem do projeto
que se escolhe; cada objecto novo (moedas, inimigos, portas) vai precisar de trocar imagem.

**How to apply:** é a onda seguinte da line/GameEngineFuncional, ANTES do J5. Medir primeiro o que já existe
(biblioteca de assets, campo de imagem no Inspector, importação, `.ase`) e desenhar UMA porta de «qual
conteúdo este objecto mostra» que Sprite, folha animada, Flip, Vector e Pixel usam; a linha do Pixel encaixa
pela mesma porta (contrato escrito no handoff, sem tocar o código dela). Liga com
[[feedback-the-content-of-an-asset-is-shared-only-which-asset-is-per-object]] e
[[project-pixel-art-module-standalone-aseprite-spine-2026-10-25]].

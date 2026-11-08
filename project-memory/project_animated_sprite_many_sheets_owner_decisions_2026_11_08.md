---
name: project_animated_sprite_many_sheets_owner_decisions_2026_11_08
description: "Dono 08/11: Animated Sprite com várias folhas — lista Sheets, uma folha por animação, Pivot por folha, «+» cria campo vazio, tirar leva as animações"
metadata:
  type: project
---

Dono, 08/11, sobre «Animated Sprite · várias folhas» (GEF 26):

1. **Lista de folhas:** o cartão Image vira «Sheets (N)», uma linha por folha; cada animação usa UMA folha (etiqueta na linha dela); o editor ganha seletor de folha. Mistura de folhas dentro de uma animação (à Godot): NÃO.
2. **Pivot por folha:** ao trocar de animação os pés ficam no mesmo lugar e o pixel tem o mesmo tamanho na tela; a folha nova copia o Pivot da primeira.
3. **Folha nova = um botão que cria um segundo campo de imagem VAZIO** (o «Drop a sprite sheet or click to choose»). Não foi a opção «depende de onde soltar». Soltar numa folha existente continua a trocar essa folha (interpretação, confirmar no smoke).
4. **Tirar uma folha leva as animações dela**, um passo de desfazer, com aviso no botão.

**Why:** Godot (corrido, MIT) deixa cada quadro de qualquer atlas e centra o quadro: 32→48 px o boneco salta. LibreSprite (corrido) só junta ficheiros num atlas ao exportar.

**How to apply:** plano `docs/GameEngineFuncional/26_animated_sprite_varias_folhas_plano.md`; oráculos em `docs/GameEngineFuncional/oraculos/varias_folhas/`. Continua [[project_animated_sprite_card_owner_decisions_2026_11_06]].

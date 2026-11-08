---
name: project_animated_sprite_many_sheets_owner_decisions_2026_11_08
description: "Dono 08/11: Animated Sprite com várias folhas — o cartão Image fica, miniatura vira dropdown, Load/Delete sheet, todas ativas, animação MISTURA folhas, Pivot por folha"
metadata:
  type: project
---

Dono, 08/11, sobre «Animated Sprite · várias folhas» (GEF 26), **revisto no mesmo dia** sobre a captura do cartão Image:

1. **O cartão Image fica.** Em Animated a miniatura vira um **dropdown das folhas** (item = moldura com a imagem inteira); a escolhida é a folha VISÍVEL do painel. Botões **Load sheet** e **Delete sheet** (apaga a visível).
2. **Todas as folhas carregadas ficam ativas**, seja qual for a visível.
3. **Uma animação MISTURA quadros de folhas diferentes** (pergunta feita 08/11: «Misturar quadros»). No editor escolhe-se a folha e clica-se; trocando a folha continua-se a mesma animação.
4. **Uma lista só** de animações, na ordem que o artista quiser (editor e cartão Animation).
5. **Pivot por folha** (pés no mesmo sítio, mesmo tamanho de pixel; a nova copia o da 1.ª) e **apagar leva o que é dela** — da 1.ª rodada, mantidos.

Recusado pelo dono: a lista «Sheets (N)» com uma linha por folha (1.ª maquete) e «uma folha por animação».

**Why:** Godot (corrido, MIT) deixa cada quadro de qualquer atlas mas centra o quadro: 32→48 px os pés saltam 8 px. LibreSprite (corrido) só junta ficheiros num atlas ao exportar.

**How to apply:** plano `docs/GameEngineFuncional/26_animated_sprite_varias_folhas_plano.md`; maquete `docs/GameEngineFuncional/maquetes/26_hoje_e_proposta.png`; oráculos em `docs/GameEngineFuncional/oraculos/varias_folhas/`. Continua [[project_animated_sprite_card_owner_decisions_2026_11_06]].

---
name: project-sprite-closed-accept-as-is-2026-11-06
description: "Dono 06/11: Sprite FECHADA («aceitar como está. Smoke OK»); Mirror vira a imagem como as engines, Keep in place, Mirror com Pivot animado aceito sem cura, undo da agulha adiado para a Timeline"
metadata:
  node_type: memory
  type: project
---
O dono fechou a Sprite do GEF 23 no smoke de 06/11: «Parece que tudo está funcionando perfeitamente. Vamos aceitar como está. Smoke OK». Decisões finais (ficha `docs/GameEngineFuncional/componentes/01_sprite.md`, decisões 20–23): **Mirror object vira a imagem**, como Godot e Unity; com *Keep in place* vira sem sair do lugar · **Mirror num objeto com Pivot já animado** vale dali em diante e a imagem se desloca ao tocar: aceito, sem cura (medido: Mirror H, desenho salta de x 8,5 para 11,5 entre t=1,5 e t=1) · moeda abaixo da plataforma só com o Mirror vertical · Keep in place na linha dos botões do Mirror. Censo aceito: 13 campos à vista, 31 no total. **Cada ida da agulha cria passo de undo**: fica como está, tratar na etapa da Timeline.

**Why:** o dono testou os casos reais (moeda presa, Pivot animado, Ctrl+Z) e vê o resultado como o desejado; os desvios que restam são medidos e inofensivos para o uso dele.

**How to apply:** não reabra a Sprite nem «cure» o Mirror com Pivot animado sem ele pedir. O que sobra vai para a Timeline (undo da agulha) ou para o integrador (handoff `HANDOFF_CONTINUACAO_2026-11-06_23_SPRITE_FECHADA.md`). O próximo item da fila é o Animated Sprite, a propor ao dono antes de construir.

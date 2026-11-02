---
name: project_painter_panel_e10_owner_decisions_2026_11_01
description: "Dono 01/11 (smoke da maquete E10 do Wet Paint): o painel novo NÃO pode reduzir nem sair do padrão — tools na barra vertical, secções comuns aos 4 modos, UI do app, avançado preservado; respostas às 9 perguntas"
metadata:
  node_type: memory
  type: project
  originSessionId: 5007c508-75cd-4b63-816e-9d4c1b01715c
  modified: 2026-11-01T12:15:30.748Z
---

Smoke de 01/11 sobre a maquete F3 (`docs/Painter/Arquitetura/fotos/f3/maquete_painel.png`) — o dono REPROVOU a forma:

1. As ferramentas (Blend, Smear, Blow, Dry, Water, Erase, Knife) **ficam na barra vertical da esquerda** (ao lado da
   Hierarchy); abas no painel só como complemento.
2. Os 4 modos (Digital · Watercolor · Impasto · Wet Paint) **partilham o máximo de UI**: as secções comuns (Paper,
   Grain, Shape…) ficam — integram o módulo e não exigem aprender cada modo de novo.
3. A UI segue o **padrão do app** (o Inspector de hoje: rótulo + campo numérico com setas + ponto de keyframe, secções
   colapsáveis) — a maquete com sliders próprios estava fora do padrão.
4. **Não desfazer vitórias**: Spacing com «Continuous» (custo baixo sem perder qualidade); o **Tilt e o seu visor
   ficam como estão**.
5. **A configuração avançada é importante**: as propriedades de hoje (Pigment, Pickup, Dry Speed, Edge Darkening,
   Gravity, Leveling, Level Clamp, Capillary, Brake, Brush dust, Backruns, Granulation, Diffusion, Wet bloom, Tip
   pickup, Re-wet, Drag, Solubility, Staining, Glaze, Cover, Dry cover, Wet soften…) NÃO se perdem.

Respostas: (2) **um só «Dilution»** em vez de Pigment + Water ✓ · (3) **Dry speed da tela inteira** · (4) Paper press
por omissão: escolha técnica · (5) meios Fan Brush 0,5 · Express Oil 1 · Oil Pastel/Wax 1 ✓ · (6) **quer um controlo
de sombra** · (7) riscas/cor por cerda → a secção **Shape** · (8) camada fina: o que der MENOS artefactos · (9) **Body
fica como controlo**.

**Why:** o dono quer um módulo integrado e uma ferramenta de artista avançado; «um controlo = uma grandeza» vale
para a LEI (o que cada knob escreve), não para cortar o que o artista pode afinar.

**How to apply:** E10 = os controlos principais (Medium, Dilution, Volume…) por cima, e TODO knob de hoje que tenha
leitor numa secção avançada no estilo do Inspector; nenhuma secção comum aos 4 modos sai; nunca redesenhar o Tilt, o
Spacing/Continuous nem o padrão visual. Relacionado: [[project_wetpaint_refactor_and_illustrated_help_2026_10_31]],
[[feedback_app_ui_english_only]], [[project_paper_press_is_a_brush_control_2026_10_28]].

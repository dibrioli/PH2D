---
name: project_large_brush_look_both_in_panel_2026_11_07
description: "Dono 07/11: acima de Size 60, as duas leis no painel, «agua/alisar» por padrão; até 60 ao byte. Trava: a grade é da sessão, trocar acaba a tinta viva"
metadata:
  type: project
---

Dono, 07/11, depois de ver as folhas de `docs/Painter/handoffs/material_2026-11-07/`: acima de Size 60, as duas leis ficam disponíveis no painel, «agua/alisar» (a tinta calculada em resolução menor) como padrão, e «tex+fios» (só o desenho ampliado) como a outra. Smoke do erro no desfazer e do relevo no refazer: OK.

**Why:** a «agua/alisar» é a mais próxima do Size 60 na maioria dos pincéis e pinta 3–4× mais rápido; o dono quer a escolha à mão para os casos em que ela alisa demais (Thick Impasto).

**How to apply:** a simulação usou `grid_ratio` por tela, com um tamanho só. No produto a razão é da SESSÃO: `set_wet_grid_ratio` acaba a sessão e o acordar recusa grade diferente (`wetpaint_settings.rs:523`, `adormecida.rs:351`). Trocar o Size entre traços mataria a tinta viva, contra [[project_live_paint_survives_2026_11_06]]. Prototipar primeiro a mesma cara com a sessão viva; foto ao dono antes de ligar. Até Size 60 idêntico ao byte; «tex+fios» estria o Wet Round: tratar as aguadas. Faltam: foto em zoom 100 % e os saltos de resolução (90, 150).

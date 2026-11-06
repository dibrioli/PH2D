---
name: project_live_paint_survives_2026_11_06
description: "Dono 06/11: a tinta do Wet fica viva ao trocar de modo/ferramenta sem pesar nos outros modos; opção ao salvar; a Borracha é inteligente (real na viva, imita na morta) e leva juntos os traços por cima"
metadata:
  type: project
---

Dabber + trocar de modo mata a tinta e a Relief Eraser vira corte seco. Decisões do dono (06/11): (1) manter viva ao trocar de modo e ferramenta, sem reduzir a velocidade dos outros modos; (2) traço de outro modo por cima: a Borracha leva os dois; (3) **opção** ao salvar o projeto, não sempre; (4) a Borracha age de verdade na viva e imita o comportamento certo na morta.

**Why:** a borracha viva é a que ele acha realista; arquivos sem a opção continuam a existir, por isso a imitação.

**How to apply:** plano, custos medidos e pesquisa em `docs/Painter/TINTA_VIVA_2026-11-06.md`. Depois disto a fila volta a `docs/Painter/PROXIMA_FASE_2026-11-06.md`. Ver [[project_next_phase_painter_2026_11_06]].

Onda 2 pronta (dentro do Wet, Borracha na morta, compressão zstd): [handoff](../docs/Painter/handoffs/HANDOFF_CONTINUACAO_2026-11-06_tinta_viva_onda2.md); aguarda o veredito do dono.
Lição: `bytes_retidos` conta 0 com o motor no worker (`EngineSlot::Away`): mede-se depois de o trazer (577 MB eram 5 GB).

**Dono 06/11: a Borracha na tinta morta está APROVADA pela foto** (`material_2026-11-06/foto_borracha_morta.png`). As constantes de `borracha_morta.rs` ficam; os passos 2, 3 e 5 do smoke ainda sem veredito.

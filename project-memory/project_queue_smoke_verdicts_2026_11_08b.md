---
name: project_queue_smoke_verdicts_2026_11_08b
description: Dono 08/11 (2.º smoke): busca no dropdown «feia mas funciona»; o efeito do Paper na tinta não segue o Relief + pede um multiplicador próprio no Paper; o resto OK
metadata:
  type: project
---

Smoke de 08/11 da onda «fila pós-smoke» (`ac3afbdad5`): OK Composite em px, Pointillism/Pastel/Chalk de volta, prévia
dos esbatedores, Paper morde com Grain/Grain Colors. Ditos: «o aspecto do campo de busca ficou feio. Mas funciona.» ·
«Independente da intensidade do relevo de paper, o efeito sobre a tinta é o mesmo. isso não está correto. além de
corrigir isso precisamos de um novo parâmetro multiplicador em paper para poder ajustar o efeito no pincel de forma
que não seja determinado apenas pelo valor do relevo.» · «Coloque os apontamentos na fila de implementação.»

**Why:** o dente do papel na tinta Digital usa só o Tooth do pincel (`paper_depth / PAPER_TOOTH_MAX`, `dente.rs` e o
ramo antigo de `poe_o_papel_como_grain`); o Relief (`substrate_depth`) só liga/desliga (> 0).

**How to apply:** o efeito na tinta tem de crescer com o Relief do papel, × um controlo NOVO no Paper (multiplicador
do efeito no pincel) — os dois caminhos (papel-como-grain e o slot `dente`). O campo de busca refaz-se no idioma dos
nossos painéis (maquete + foto lado a lado antes do smoke). Fila: `docs/Painter/handoffs/HANDOFF_ONDA_line_PainterWatercolor_2026-11-08_FILA_POS_SMOKE.md`.

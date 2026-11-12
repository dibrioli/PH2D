---
name: project_timeline_easing_is_rhythm_along_the_path_2026_11_12
description: Dono 12/11 (plano 17 item 13) escolheu o modelo After Effects — forma do caminho no canvas, uma linha «Position» na Timeline cujo easing é só o ritmo ao longo do caminho (MotionPath, ADR-0141)
metadata:
  type: project
---

Dono 12/11, item 13 do plano 17 da Timeline («mudar o easing danifica a curva do canvas»): escolheu **A**. A forma do caminho vive só no canvas; na Timeline X e Y viram UMA linha **Position**, e o easing dela muda só a velocidade ao longo do caminho, nunca a forma. Base: o `MotionPath` do ADR-0141 (`ph2d-timeline/src/path.rs`), que estava «proposto, aguardando aceite».

**Why:** com X e Y separados, cada eixo guarda forma e ritmo no mesmo `Interp` (`BezierW`); o easing da linha substitui a curva inteira e a forma entortada no canvas perde-se. A opção B (seta só no ritmo de X e Y juntos) dava presets aproximados.

**How to apply:** curva entortada no canvas ⇒ objeto com caminho (Position = distância no caminho, easing próprio). Não voltar a guardar forma espacial no easing por eixo. Ver [[project_timeline_curves_live_on_canvas_2026_11_10]] e [[project_jump_arc_on_canvas_2026_11_10]].

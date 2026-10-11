---
name: project_wet_paint_is_the_only_painter_mode_to_evolve_2026_10_07
description: Dono decidiu (07/10): Digital, Impasto e Watercolor NÃO mudam; só o Wet Paint evolui e passa a simular TODAS as mídias («Wet Painter»), com o Rebelle 8 como oráculo medido
metadata:
  type: project
---

Depois da avaliação paralela Painter × Rebelle 8 (`docs/Painter/NewFeaturesWetPaint/`, docs 01–03: as sete
leis medidas do traço dele, as lacunas, os presets por mídia lidos de 281 presets de fábrica), o dono fixou o
alvo: *«não tenho intenção de mudar os modos Impasto, Digital ou Watercolor. Apenas aperfeiçoar Wet Paint»*.
O prompt da linha (`04_PROMPT_linha_wet_painter.md`, `line/WetPainter`) re-aponta o plano ao
`PaintMedia::WetPaint` (`ph2d-wet-paint` + `wetpaint*.rs`) em ondas W0–W8: bancada, presets em ficheiro +
volumes, fios com física própria, espessura/oleosidade, lei do papel por pressão, lei da tinta (Paint & Mix,
tinta finita, pincel sujo), dinâmicas (piso/tecto de pressão, tilt em 2 eixos, barrel), mídias secas e água
como volume, painel.

**Why:** três motores a evoluir em paralelo triplicam o custo e o smoke; o fluido já tem fios, água, papel e
secagem — é a base que falta menos para «todas as mídias».

**How to apply:** qualquer trabalho de pintura que toque `stamp_dab`, `watercolor_*.rs`, `impasto_*.rs` ou
`dynamics.rs` para «melhorar o traço» está FORA da decisão — a cerca da linha exige os PNG da sonda
byte-idênticos nesses três meios. Oráculo: a porta WebSocket do Rebelle
([[reference_rebelle_trial_installed_under_wine]]); recusas: Mixbox, texturas dele, injecção de rato.

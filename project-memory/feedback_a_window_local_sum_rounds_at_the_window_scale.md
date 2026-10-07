---
name: feedback_a_window_local_sum_rounds_at_the_window_scale
description: "Um composite por janela só é igual à recomposição total se nenhuma soma f32 depender da origem da janela — `lx + dx` e prefixos a partir da origem arredondam diferente"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 2767b257-bc12-4527-b8e5-ac1ef5ad681c
  modified: 2026-10-06T01:31:49.532Z
---

Um composite que corre por JANELA (sujo + pad) só dá os mesmos bytes que a recomposição total se
nenhuma conta em `f32` depender de onde a janela começa. Medido em 2026-10-05 (BUGS #38, aquarela):
o ponto deslocado do Ragged somava-se em coordenadas LOCAIS (`lx + dx`), que arredondam à escala de
`lx`; numa distância assinada íngreme (o aro) `1e-6` px virou `1` nível num texel — 117 × 116. A
hipótese herdada («uma escrita sem marca no tique do Airbrush») estava errada: o oráculo
`recompoe_tudo` (a tela inteira em todo quadro, mesmo processo) mostrou o pixel DENTRO da janela, e a
ablação termo a termo isolou aro × Ragged. Os prefixos de borrão em `f32` a partir da origem da janela
são a mesma família (curados com soma em ponto fixo).

**Why:** um sujo que "não cobre" e uma conta que depende da janela dão o MESMO sintoma (o quadro ≠ a
caixa inteira), mas curas opostas — alargar a janela não cura a segunda.

**How to apply:** some na TELA e subtraia a origem inteira no fim (exacto para `0 ≤ origem ≤ ponto`);
somas de vizinhança em inteiro/ponto fixo. Antes de caçar "a escrita sem marca", confirme se o pixel
divergente está dentro da janela recomposta. Ver [[reference_topic_repro_discipline]].

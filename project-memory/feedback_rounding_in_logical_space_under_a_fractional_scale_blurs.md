---
name: feedback_rounding_in_logical_space_under_a_fractional_scale_blurs
description: Sob uma escala fracionária (125/175 %, HiDPI 1,5) todo .round() feito no espaço LÓGICO cai entre píxeis do ecrã — arredonde no físico (ui_scale::ao_pixel)
metadata:
  type: feedback
---

**Lei:** quando o chrome é pintado numa cena LÓGICA e colado sob `Affine::scale(s)`, um
`.round()` escrito para alinhar ao píxel (linha de base do texto, snap-X, hairlines) alinha à grelha
LÓGICA; com `s` fracionário essa grelha não é a do ecrã. Arredonde por `ph2d_editor_core::ui_scale::ao_pixel`
(a `s = 1` é `round` AO BIT; a grelha só vale `s` dentro do `pintar_no_chrome`).

**Why:** medido em 02/10 (`line/UIUX`, fotos em tela virtual): o preset de texto de fábrica
(`CrispHeavyPlus`) não tem hint, logo a nitidez vertical era só o `y.round()` lógico — a 125 % as
bordas horizontais do texto pequeno ficaram 43 % mais macias (1,07 contra 0,75 px de transição AA);
curado, 0,63. ⚠️ O Vello SOZINHO estaria certo: `Scene::append(…, Some(scale))` compõe a escala no
`run.transform` e o resolve hinta a `font_size × s` e arredonda o y no físico — o erro era só nosso.

**How to apply:** qualquer pintor do chrome que arredonde coordenadas (bordas de 1 px, divisores,
réguas) tem o mesmo defeito a `s` fracionário; meça com uma foto a 125 % antes de o declarar nítido.
Ver [[feedback_a_contrast_cure_carries_the_surface_it_was_calibrated_against]].

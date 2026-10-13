---
name: feedback_a_sweep_by_max_paints_the_overlap_twice_use_the_mean
description: Espalhar um padrão por dab ao longo do passo (as cerdas que arrastam): o MÁXIMO no rasto pinta duas vezes onde dois rastos vizinhos se tocam; a MÉDIA conserva a tinta e alisa
metadata:
  type: feedback
---

Ao transformar o carimbo repetido de um padrão (o tapete de cerdas a cada dab) num RASTO contínuo, a lei
intuitiva — o máximo do padrão ao longo do passo — deixa tracejado: um pelo mais largo que um texel é cruzado
pelos passos de DOIS dabs vizinhos e a sobreposição pinta duas vezes, com o período do passo (CV 0,15 e 2–5× a
tinta, prompt 32, 13/10). A MÉDIA ao longo dos `n` texels do passo é a exposição que `n` dabs a 1 px deixariam:
a tinta de hoje (± 1 %), espalhada (CV 0,014 a Spacing 0,1 pelo produto). E o varrimento é MEIO-ABERTO, ou o
texel da ponta conta duas vezes.

**Why:** «cada pelo desenha o risco que percorreu» lê-se como um máximo, mas o carimbo COMPÕE dab a dab — o que
se conserva é a exposição somada, não o pico.

**How to apply:** antes de escolher a lei de um rasto, meça as duas (no mesmo processo) com a régua ao longo E
com Σ tinta; e resolva o passo em texels 1× por dab (por píxel custou 1,6×). Doc 31 §4.1 do Painter. Ver
[[feedback_forcing_a_property_after_the_optimum_is_what_ripples]].

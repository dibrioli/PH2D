---
name: feedback-to-tell-a-cures-ridge-from-the-shapes-own-curvature-run-the-knob-to-both-ends
description: A régua acusava o filete de deixar uma crista de `7,86` no gyroid — com o filete a ZERO ela lia `36,74`: o filete estava a REDUZIR a quebra `4,7×`, e o que sobrava era a curvatura própria da forma.
metadata:
  type: feedback
---

⭐⭐⭐ Medido em 2026-09-06 (W124, doc 06 §125.5). O gate `the_fillet_leaves_no_curvature_ridge`
acusou o gyroid com `7,86` contra uma barra de `2,0`. A pergunta certa **não é «quanto»** — é **«de
quem»**, e ela responde-se correndo a mesma régua com o knob nos **dois extremos**:

| forma | sem filete | com filete | o filete acrescenta |
|---|---:|---:|---:|
| **gyroid** | `36,74` | `7,86` | **`−28,88`** |
| espiral | `50,43` | `2,16` | `−48,27` |
| **nuvem** | `3,06` | `8,58` | **`+5,52`** |

⇒ no gyroid o filete **reduz** a quebra `4,7×`: ele não deixa crista nenhuma, **tira-a**. O que sobra
é a curvatura própria de uma superfície mínima tripla-periódica, que é curva em todo o ponto por
definição. ⚠️ **A nuvem é o controlo** — nela o filete de facto **acrescenta**, e é por isso que está
declarada.

**Why:** uma régua traz uma **premissa** no doc dela (aqui: *«numa superfície de curvatura contínua
ela é pequena em toda parte»*), e uma forma nova pode violar a premissa sem ter defeito nenhum.
Declarar uma excepção sem esta pergunta escreve *«a forma tem um defeito»* onde a verdade é *«a régua
não serve a esta família»* — e a declaração fica errada para sempre, porque ninguém volta a ela.

**How to apply:** antes de pôr uma forma numa lista de excepção, corra a régua com o controlo
acusado **a zero e no máximo**. Se o número **melhorar** com o controlo, o controlo não é o réu.
⚠️ E a segunda metade da mesma família: **uma régua cujo PASSO não é pequeno em relação à feição lê
a feição como defeito** — a mesma sonda mede com `d = 0,004` e o doc dela exige `d ≪ filete`; com o
filete a `0,011` a normal roda `21°` por passo contra uma barra de `25°`, e ela lia o **próprio
filete** como aresta (`19,4 %` que caíram para `7,6 %` só ao dar-lhe um representante que ela
consegue resolver). Ver [[reference_topic_measurement_discipline]] ·
[[feedback_a_probe_with_the_knob_at_another_point_measures_another_piece]] ·
[[reference_topic_implicit_field_laws]]

---
name: feedback-two-nearly-coincident-pieces-in-a-blend-need-the-angle-not-a-fence
description: Os dois flancos de um bico que fecha deitam-se sobre a MESMA recta e a mistura n-ária conta a superfície duas vezes (`1,183`) — a cura não é uma cerca no controlo, é dizer ao par o ÂNGULO em que ele se encontra (`0,954`).
metadata:
  type: feedback
---

Medido em 2026-09-05 (W122, doc 06 §123.3). O bico do *Display* são dois semiplanos distintos; com
`point → 0` eles ficam sobre a mesma recta. Numa `intersection_round_n` o tecto de `‖∇f‖` é
`√(quantas peças estão ACTIVAS)`, e ali estavam **duas cópias da mesma superfície** ⇒ `passo × ‖∇f‖`
lia **`1,183`** exactamente no ponto em que a forma é mais simples.

⛔ **A cura óbvia — pôr um piso no controlo — é a errada duas vezes:** ela custa metade do curso do
slider, e uma coerção *estaciona NA cerca*
([[feedback_a_coercion_parks_at_the_fence_which_is_where_the_shape_degenerates]]), isto é, entrega
sempre o pior caso.

⭐⭐⭐ **A cura é DIZER O ÂNGULO.** O operador tinha uma porta que aceita `cos_faces` e ninguém a
usava aqui; com `cos = (altura² − base²)/(altura² + base²)` o par degenera **por construção** quando
o bico fecha (`cos → 1` são duas normais iguais, ou seja, canto nenhum):

| bico / parede | 0,00 | 0,30 | 0,70 | 1,00 |
|---|---:|---:|---:|---:|
| n-ário **sem** ângulo | **1,183** | 1,024 | 0,991 | 0,991 |
| par **com** o ângulo | **0,954** | 0,979 | 0,991 | 0,990 |

⇒ o extremo do controlo passa a ser uma **forma** (ali, a primitiva vizinha da paleta) em vez de uma
degeneração.

**Why:** uma mistura sem ângulo assume `90°`. Um canto que se abre para `180°` é, para ela, dois
cantos rectos sobrepostos — e o erro é máximo exactamente onde a geometria é mais benigna. *O sinal
é sempre este: o campo pior no ponto mais simples.*

**How to apply:** quando duas peças de uma mistura podem tornar-se paralelas ou coincidentes ao
varrer um controlo, procure a variante do operador que recebe o ângulo **antes** de escrever uma
cerca. ⚠️ E a cura tem metade: um perfil **já composto** não pode entrar numa junta que compõe outra
vez — ela leva a costura dele para a aresta seguinte (medido: o vinco do aro foi de `5,1°` para
`25,3°`), então **com chanfro as peças entram inteiras**. Ver
[[reference_topic_implicit_field_laws]].

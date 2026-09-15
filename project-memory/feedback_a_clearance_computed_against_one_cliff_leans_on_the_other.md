---
name: a-clearance-computed-against-one-cliff-leans-on-the-other
description: Folga calculada contra UM precipício encosta ao outro — o mapa fica perfeito e a malha que o amostra faceta
metadata:
  type: feedback
---

⛔⛔⛔ **Uma folga calculada contra UM precipício encosta no OUTRO.**

Medido 2026-09-15 (`line/Vector`, a cena do canvas preso a ossos, 3.º report do dono seguido).
A pele de ossos tem **dois** precipícios e eles são vizinhos:

| | onde | o que se vê | régua |
|---|---|---|---|
| 1.º — **órfão** | FORA do raio de todo osso | a arte **RASGA** (salto seco para o osso mais perto) | `fold::measure().orphan` |
| 2.º — **normalização** | **logo DENTRO** da borda do raio | a arte **FACETA** | `poly2d::deviation` |

O 2.º existe porque os pesos são **normalizados**: junto da borda do suporte todos tendem a zero, e
a razão entre números que tendem a zero varia depressa ⇒ a curvatura do campo explode. ⚠️⚠️ **O mapa
continua contínuo e injectivo** (`0,00 %` órfã, `0,00 %` do avesso, `0` de `780` triângulos
invertidos) — e a malha, que pinta **um afim por triângulo**, deixa de o conseguir seguir.

Eu derivei a altura da arte para usar `93,75 %` do alcance — o máximo que a folga contra o 1.º
permitia — e aterrei exactamente no 2.º: **`14,24 px`** de desvio. A `0,469` do alcance: **`0,92 px`**,
com a **mesma malha, a mesma dobra e o mesmo número de triângulos**.

**Why:** uma folga é uma distância a UMA coisa. Quando duas patologias se sentam lado a lado, afastar
de uma é aproximar da outra — e a que eu não tinha régua para ver foi a que o dono fotografou.

**How to apply:** ao escolher uma folga, pergunte **o que está do outro lado dela** e corra uma régua
lá também. ⭐ O sinal de que a patologia não é discretização: a convergência é **`O(h)` e não `O(h²)`**
(halvar a célula deu `1,72×`, não `4×`) — *densidade nunca lá chega, o defeito é a curvatura do
campo*. Ver [[feedback_a_ruler_with_n_columns_answers_n_questions_and_i_cited_the_wrong_one]] e
[[reference_topic_measurement_discipline]].

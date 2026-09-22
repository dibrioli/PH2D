---
name: feedback-a-ruler-whose-parameter-sits-inside-a-clamp-measures-the-clamp
description: "Uma varredura cujo parâmetro cai todo dentro de um clamp mede o clamp, não o eixo — e lê-se como invariância"
metadata:
  type: feedback
---

Uma varredura que move um knob e lê `0,000000` em toda a linha pode estar a medir um **clamp** e não
uma invariância. Medido 2026-09-21 na `W9` do Render3d: a premissa da cache do campo do chão
(*«o campo não depende da câmera»*) tinha sido «medida» varrendo `half_extent` (`0,8` · `1,6` · `3,2`)
e `lado_px` (`64`..`2160`), com **`1024 de 1024` células byte-idênticas** nas duas varreduras — e a
única porta por onde a câmera entra na assadura é `Sharpness::for_frame`, que faz
`hit = min(HIT_EPS, half_extent/(2·lado_px))`. Nas **nove** leituras o `hit` esteve preso em `2e-4`.
Com o clamp solto (`lado_px > 2500 × half_extent`, que a `0,2` de enquadramento são `500` píxeis — o
produto alcança-o) o mesmo eixo move o campo até **`0,91` de um byte de saída.**

**Why:** os zeros de uma varredura são a coisa mais fácil de ler como *«este parâmetro não é chave»*,
e é exactamente a leitura que constrói a cache errada — ela entregaria, num zoom apertado, um campo
quase um byte errado, **em silêncio**. E a cura óbvia — *varrer mais pontos* — não ajuda: toda a
faixa original está do mesmo lado do clamp.

**How to apply:** ao medir invariância a um parâmetro, **imprima a grandeza INTERMÉDIA que ele
produz** ao lado da coluna do desvio (aqui: o `hit`, não o `half_extent`). Se ela não se mexe, a
varredura não é uma medição do eixo — é uma medição do clamp, e a fronteira onde ele solta acha-se
por **conta**, não por mais amostras. Quando a conta prova que o parâmetro é constante em toda a
faixa do produto, o gate afirma a conta e **NOMEIA onde ela deixa de valer**; quando a conta a
alcança, o parâmetro é chave. ⛔ E a versão impressora deste mesmo teste escondia o outro lado do
mesmo defeito: ela escrevia `|Δ|` com **seis casas**, e `6e-9` lê-se `0.000000` — *a precisão da
impressão é parte da régua*. Ver [[reference_topic_measurement_discipline]] ·
[[reference_topic_gate_discipline]] · [[reference_topic_mutation_proofs]].

---
name: feedback-an-absolute-unit-that-should-feel-relative-must-scale-with-the-geometry
description: Uma grandeza guardada em unidade ABSOLUTA (altura em cargas, largura de traço em px) parece "fixa/errada" quando deveria acompanhar o tamanho da ferramenta — o conserto é escalar pela geometria pra preservar a razão de aspecto, não calibrar o número
metadata:
  type: feedback
---

A altura do impasto era `depth` cargas — absoluta. Um pincel de 6 px e um de 60 px picavam no MESMO pico.
Enio: **"pincéis grandes ficam parecendo apenas tinta."** Estava certo: um pico de `depth` cargas sobre 60 px
tem `n_z ≈ 1` (inclinação ínfima), e a luz — que lê `∇h` — o desenha CHATO.

**Why:** o que o artista percebe como "relevo" é a **razão de aspecto** (altura ÷ largura), não a altura
absoluta. Uma altura fixa espalhada sobre um footprint grande achata a razão. O olho compara forma, não
número. (Mesmo espírito de [[feedback_growing_geometry_without_growing_matter_grows_nothing]] e
[[feedback_a_hard_clamp_is_not_a_ceiling_it_is_an_eraser]]: o que importa é o que a LUZ mostra.)

Fix: escalar o pico por `radius / referência` (referência = tamanho default da ferramenta). Então a razão de
aspecto fica **constante** e o efeito lê em toda escala. Um dab na referência é inalterado (arte antiga
intacta).

**How to apply:**
- Se uma feature "parece fixa/errada em tamanhos diferentes", pergunte: *a grandeza está em unidade absoluta
  quando deveria acompanhar a geometria?* Largura de contorno, altura de relevo, raio de vinho, tamanho de
  seta — todos têm essa armadilha.
- Escale pela geometria (raio/tamanho) contra uma **referência**, não por um fator mágico. Referência =
  o valor default, pra a arte existente ficar byte-idêntica ali (e isso é gate).
- O oráculo é a APARÊNCIA (a luz/o render), não o buffer. E cuidado com a fixture do gate: `hardness 1.0` faz
  uma MESA (topo chato), não um domo — amostre onde a INCLINAÇÃO vive (as paredes), e cancele a borda
  papel↔tinta comparando o mesmo dab com o efeito ON vs OFF.

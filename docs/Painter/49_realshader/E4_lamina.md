# E4-motor — a lâmina que gira e o rebordo (Óleo Real W2, 27/10)

Item de motor da E4 (CAP-01 / C38) para as 10 facas do Rebelle. Os VALORES das facas ficam para a E4-presets.

## A lei

1. **Orientação = composição que JÁ existia** (medido no código, não reconstruído): `dab_flatten` × `dab_angle_deg`
   × knob `dabRotation` (`0` fixa na tela · `1` segue o traço · `2` inclinação da caneta). É a lei L3 de
   [`06_LEIS_DAS_LACUNAS.md` §1](../Pinceis/06_LEIS_DAS_LACUNAS.md) — no Rebelle a lâmina é FIXA (Textured, Oily,
   Soft, Dry, Dry 2: largas num sentido, finas no outro, ver `fixtures/rebelle_bateria/OIL_AND_ACRYLIC/knife_*_gestos_A`)
   ou SEGUIDORA de largura constante (Smooth 2/3, Grainy, Stain); inércia e memória REFUTADAS (§1, §10.1–10.2).
   A premissa «a lâmina gira com inércia» do C38 não volta.
2. **O rebordo** (novo, `crates/ph2d-wet-paint/src/lamina.rs`, knob `knifeRidge`, px, `0` = ao bit): a altura
   `Grid::altura` sob cada dab é penteada para `altura · perfil(u)`, `u` = distância ATRAVÉS do rumo do traço ÷ a
   meia-largura que a silhueta do dab tem nessa direcção; `perfil` = 0 no miolo (a faca alisa) e rampa recta na
   última `FAIXA = 0,12` da meia-largura.

## A medição que a decidiu

Folha F42 do Rebelle (`ferramentas/out/F42_base.png`, luz 42°), perfil de luminância através do traço (ecrã):

| faca | largura | declive escuro por dentro da borda de cima | banda clara por dentro da de baixo | FAIXA |
|---|---|---|---|---|
| Knife Smooth 2 | 91 | 66 · **51** · 71 … 104 → planalto 127 (7 px) | 152 · **177** · 170 | 6/45,5 = 0,13 |
| Knife Smooth 3 | 88 | 83 · **53** · 89 … 122 (5–6 px) | 167 … 177 | 0,12 |
| Knife Smooth | 19 | 103 (fraco) | 110 | — |

Crista = declive que sobe para a borda nos DOIS lados (escuro em cima, claro em baixo sob luz de cima).

O nosso (probe `medida_e4_lamina_folha`, raio 27 ⇒ 89 px, `water` 0, Óleo Real de fábrica), mínimo do topo ÷ planalto
e máximo da banda de baixo ÷ planalto (Rebelle 0,40 / 1,39):

| `knifeRidge` | raio | topo | baixo |
|---|---|---|---|
| 0 | 21 | 1,05 (sem declive) | 0,98 |
| 3 | 21 | 0,90 | 1,11 |
| 6 | 21 | 0,86 | 1,17 |
| 8 | 21 | 0,72 | 1,17 |
| 8 | 27 | 0,72 | 1,19 |
| 12 | 27 | 0,72 | 1,17 |

A crista satura a ~8 px: acima disso a banda (≈ 3 células) e o pré-filtro do detalhe do Óleo Real (`raio_detalhe` 1,
`fios *= cobre`) limitam o contraste. Faixa do knob 0–16 (o dobro do ponto de saturação medido).

## Prova

[`out/E4_lamina.jpg`](out/E4_lamina.jpg): L e curva com a lâmina seguidora (em cima `knifeRidge` 8, em baixo 0) ao
lado do Knife Smooth 2 (F42 e bateria). Visto: largura igual nos dois braços e na curva; o rebordo aparece como o
friso nas duas bordas (ausente em baixo); mais fino e mais fraco que o do Rebelle, sem a textura da tela dele.

## Aberto

- Contraste do rebordo 0,72/1,19 contra 0,40/1,39: é do RENDER (ganho e pré-filtro dos `fios` no Óleo Real, a
  cobertura na borda) — próximo passo do Óleo Real, não do motor.
- O canto do L com a seguidora varre um leque (a lâmina roda 90° num ponto); o Rebelle não o mostra porque as facas
  dele com canto vivo são FIXAS.
- Sem crista na frente/cauda do traço (a tinta empurrada à frente) — não medido.
- UI: a linha `knifeRidge` no cartão Wet Paint (knob `Hidden` até lá).

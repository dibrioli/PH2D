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

## §Rebordo — porque o contraste fica curto (E4-presets, 27/10)

A sonda `medida_e4_lamina_folha` aceita agora `PH2D_OLEO_AF="nome:campo=v,…;…"` (variantes do render sobre a
MESMA tinta, no mesmo processo) e mede ela própria o contraste (por coluna do braço horizontal: mínimo do terço
de cima e máximo do de baixo ÷ mediana do miolo, sRGB; esta régua lê a fábrica 0,89/1,29 onde a da E4-motor lia
0,72/1,19 — o que se compara são as variantes entre si). Raio 27, `knifeRidge` 8:

| variante | topo | baixo |
|---|---|---|
| fábrica | 0,89 | 1,29 |
| `raio_detalhe` 0 | 0,79 | 1,24 |
| `fios` × 2 | 0,88 | 1,31 |
| `fios *= cobre` desligado | 0,88 | 1,28 |
| **`fios_na_sombra` 1** (a crista entra na altura da SOMBRA) | **0,74** | 1,29 |
| `fios_na_sombra` 1 + Força do ambiente 0,3 | 0,63 | 1,26 |
| `fios_na_sombra` 1 + Força do ambiente 0,15 | **0,44** | 1,26 |
| Força do ambiente 0 (sem a crista na sombra) | 0,20 | 1,25 |
| **Rebelle F42** | **0,40** | **1,39** |

**Causa medida:** (1) a altura dos fios (`Grid::altura`, onde mora a crista) entra só na NORMAL, nunca na marcha
da luz — a crista não projecta a banda escura de ~6 px que o Rebelle tem por dentro da borda de cima; com ela na
sombra aparece o declive 98 → 124 em 6 px (Rebelle 51 → 104 em 7); (2) dentro da tinta a sombra só tira a parte
da lâmpada (`1 − Força do ambiente` = 0,4): o chão é o ambiente; a 0,15 bate (0,44 × 0,40). `raio_detalhe` e
`cobre` (os candidatos da E3-B) mexem pouco.

**Recusada como cura de fábrica:** `fios_na_sombra` 1 muda também o Oil Bristle (os fios dele moram no mesmo
campo) — a regra é neutro bit a bit para quem não tem `knifeRidge`. Fica a alavanca `Afinacao::fios_na_sombra`
(fábrica 0, bit a bit; gates `oleo_real` 26, `oleo_material` 11, `fenda` 2, `seam_wetpaint_oleo_real` 7 verdes).
A Força do ambiente é controlo do artista (cartão), não da faca.

## Aberto

- **Motor (fila):** a crista da lâmina num campo PRÓPRIO (ou bandeira por camada) que o `CampoAltura` soma à
  altura da sombra — então `fios_na_sombra` só para ela; com a Força do ambiente 0,15 o rebordo dá 0,44/1,26.
- A banda clara de baixo 1,26–1,29 × 1,39: a luz de fábrica vem de 225° (cima-esquerda); o declive de baixo só
  recebe a componente vertical dela.
- O canto do L com a seguidora varre um leque (a lâmina roda 90° num ponto); o Rebelle não o mostra porque as facas
  dele com canto vivo são FIXAS.
- Sem crista na frente/cauda do traço (a tinta empurrada à frente) — não medido.
- UI: a linha `knifeRidge` no cartão Wet Paint (knob `Hidden` até lá).

# 47 — A caneta no pincel (W6): o que se MEDIU (18/10)

> Prompt: [47_PROMPT](47_PROMPT_W6_a_pressao_e_a_inclinacao_do_pincel.md). Lei: `crates/ph2d-painter-brush/src/pen.rs`
> (+ `stroke/pen_input.rs`, `BrushSpec::dab_footprint_of`/`at_dab`/`tilt_scale` em `spec_frame.rs`). Barra: 05 §3, linha W6.

## §1 — O mecanismo (perguntado ao código)

- Os 5 meios carimbam pelo MESMO `Stroke` (`dab_build.rs::dab_from`); o tilt do `CanvasPointer` era deitado fora
  no `paint_begin`/`paint_extend` (zero leitores). A pressão só entrava em `Dynamics::radius_scale`/`coverage_scale`.
- O estabilizador (`stroke/stabilize.rs`) suavizava SÓ a posição; a pressão passava crua.
- A pegada de cada dab saía de `spec.dab_footprint(spec.dab_rotor(d))` em ~10 rotas, e as caches de carimbo constante
  decidiam pelo PINCEL (`has_per_dab_rotation`…): uma variação POR DAB que o pincel não declara seria servida pela
  cache com a orientação de repouso (o «controlo morto» que a cura tinha de evitar).

## §2 — A rampa de pressão (um lote, um processo por variante)

Régua: `ferramentas/pontas_rampa.py` (o `largura_5_50_95` do `mede_traco.py`, a média das duas pontas ÷ o miolo),
portada para Rust em `regua_midia::pontas` (gate `a_regua_das_pontas_e_a_do_python`, sintética 15/41/15 = 0,3659 nos dois).

| pincel | Rebelle (foto) | hoje (17/10) | `pressureSize` 0,8 · 0,6 · 0,5 · 0,4 · 0,3 · 0,2 | escolhido → medido |
|---|---|---|---|---|
| Oil Bristle (`oil`) | 0,81 | 0,34 | 0,40 · 0,55 · 0,61 · 0,70 · 0,75 · 0,80 | 0,2 |
| Express Oil (`express`) | 0,68 | 0,30 | 0,39 · 0,53 · 0,62 · 0,66 · 0,75 · 0,80 | 0,4 |
| Graphite 8B (`pencil`) | 0,56 | 0,32 | 0,39 · 0,57 · 0,55 · 0,73 · 0,82 · 0,83 | 0,6 |
| Soft Pastel (`pastel`) | 0,53 | 0,31 | 0,38 · 0,50 · 0,62 · 0,72 · 0,78 · 0,88 | 0,55 |
| Oil Pastel (`oilpastel2`) | 0,45 | 0,27 | 0,33 · 0,48 · 0,50 · 0,67 · 0,67 · 0,75 | 0,65 |
| Ink Bullet (`ink`) | 0,68 | 0,30 | 0,37 · 0,50 · 0,60 · 0,67 · 0,75 · 0,82 | 0,4 |
| Marker Bullet (`markerbullet`) | 0,65 | 0,31 | 0,36 · 0,51 · 0,60 · 0,66 · 0,72 · 0,78 | 0,4 |
| Marker Liner (`marker`) | 0,71 | 0,29 | 0,35 · 0,53 · 0,56 · 0,65 · 0,76 · 0,82 | 0,35 |
| Wet Wash (`wcwash`) | 0,93 | 0,28 | — | 0,1 |
| Airbrush · Wet Flat · Gouache Flat | 0,80 · 0,90 · 0,87 | 0,63 · 0,76 · 0,72 | — | ficam (dono, P44/P45); no vale |
| Charcoal · Dip Pen · Ink Brush · Wet Liner · Acrylic Round | 0,35 · 0,32 · 0,29 · 0,16 · 0,17 | 0,28–0,37 | — | ficam (o controlo não os distingue) |

Gate: `Metrica::Pontas` no `cada_preset_parece_a_sua_midia` (vale do Rebelle + a barra da W6: Rebelle ≥ 0,6 ⇒ nós ≥ 0,6),
vermelho em 9 antes da receita, verde depois. ⚠️ A barra do 05 («≥ 60 % em todos») era a do óleo: o Rebelle MESMO dá 0,16–0,35
aos de ponta fina — a barra certa é a foto de cada um.

**Piso/tecto/curva da mão (C31)**: são do PINCEL (o `pressure_max` 0,6 dos lápis do Rebelle é do preset), não da mesa —
não duplicam a curva global do KDE. ✅ **C31 medida** (sessão do dono, 18/10, `ferramentas/caneta/medicoes_2026-10-18/`,
`c31_sessao.sh` + `c31_compara.py`): o KDE entrega a pressão do libinput SEM curva — 341 toques casados pelo tempo, 32 775
amostras, |PH2D − libinput| p50 0,0025 · máx 0,0051 (o arredondamento a 2 casas do libinput), recta 0,9995·x + 0,0002.
⚠️ A 1.ª leitura dizia «o KDE MUDA a pressão»: o comparador casava os toques pela ORDEM e um toque a mais de um lado
(341 × 342) desalinhou todos — casa-se pelo tempo do pen-down.

## §3 — O lápis deitado (Rebelle 8B, sessão de 18/10, `fixtures/rebelle_tilt/`)

| tilt | largura | textura | comprimento |
|---|---|---|---|
| de pé | 29 | 8,5 % | 533 |
| ao longo 20° · 35° | 29 · 31 | 9,3 · 8,8 | 528 · 537 |
| ao longo 45°–63° | 35–36 | 5,4–5,8 | 606–619 |
| através 35° · 40° · 45°–63° | 33 · 42 · 95–100 | 9,2 · 9,0 · 8,5–8,6 | 529 |
| diagonal (39, 39) | 75 | 7,3 | 588 |

⇒ a ponta é uma ELIPSE ao longo do azimute: ~3,3 raios ao longo, ~1,2 através; nada abaixo de ~39°, inteira a 45°; ao
comprido o grão cai (a sobreposição), de lado fica. O Rebelle ATRASA a inclinação pelo caminho (~170 px: degrau 0→55 a
x = 200 alarga 235–260 px depois; igual a 2 e a 20 ms/evento e a 3 e 6 px/evento). O degrau 55→0 corta em ~15 px — não
explicado (um 55→30/55→20 decidiria); não replicado.

**Lei** (`pen::Tilt`, `DabTilt::compose` — a SVD 2×2 de `S·R(θ)·diag(1, m)`, exacta para pincel achatado): o raio do dab
cresce até ao eixo maior (a caixa de toda rota é o raio ⇒ nenhuma corta a elipse); a pegada sai de `dab_footprint_of`.
A 2.ª corrida (`sonda_o_lapis_deitado_por_variante`, 20 variantes num processo):

| `tiltWidth` / `tiltSmooth` | ao longo (R 1,24) | textura (R 0,65) | +px (R 79) | através (R 3,45) | diagonal (R 2,59) |
|---|---|---|---|---|---|
| 1,2 / 0,43 | 1,58 | 0,49 | 80 | 3,50 | 2,81 |
| 1,0 / 0 | 1,19 | 0,61 | 74 | 3,42 | 2,50 |
| **1,05 / 0** (fábrica) | **1,27** | **0,62** | **74** | **3,46** | **2,54** |
| 1,05 / 0,1 | 1,31 | 0,60 | 77 | 3,46 | 2,58 |

⇒ o grão alisa-se SÓ pela sobreposição da elipse (como no Rebelle); o `tiltSmooth` alarga o traço e fica a 0 no 8B.
O modelo prevê a diagonal: 30·√((3,3² + 1,2²)/2) = 74,5 px (Rebelle 75). Gate `a_caneta_deitada` (vermelho sem os knobs).

## §4 — A suavização da pressão no oráculo dos 180 s

`medicoes_2026-10-17/pressao_por_toque.txt` (os 105 toques, 3 538 amostras, do `.zst`): um toque dura 111 ms (p5 50),
uma amostra a cada 5 ms (p95 6), o pico chega aos 40 ms (mediana). A lida `prev + (p − prev)(1 − s)` atrasa `s/(1 − s)`
amostras ⇒ **`MAX_SMOOTHING` = 0,9** (45 ms = o tempo da mão ao pico; a 0,95 seriam 95 ms, maior que metade do toque).
Gate `the_smoothing_on_the_owners_strokes_adds_no_step_and_no_lag_beyond_the_law` (s 0,3/0,6/0,9: nenhum salto maior
que os da mão, menos saltos de um nível do libinput, atraso do centróide ≤ a lei).

## §5 — Portões

`pen_tests` (14: os neutros ao bit, cada lei, a corda acaba onde o pincel está) · `caneta_deitada` (lib: a ponta deitada
chega ao carimbo nos 5 meios, e o rato com a lei ligada é o de sem ela — a cache decide pelo LOTE, `pen::any_tilted`) ·
`a_caneta_deitada` (it) · `Metrica::Pontas` · os de identidade verdes: `os_outros_meios_nao_mudam`, as miniaturas, as 22
cenas (9 repinadas pela barra, o resto ao byte; o 8B com o rato ao byte entre as duas versões da inclinação).

## §6 — Aberto

- Os knobs da caneta vivem no pincel do Wet Paint (o mapa `engine` do preset, painel Tuning); os outros 4 meios têm a lei
  (o gate dos 5 meios) mas não têm onde a ligar até os presets chegarem a todos os modos (A15) / o painel da W8.
- Na Aquarela com o «Automatic» a ponta é o carimbo redondo dela (o Flatten também não vale lá): a elipse só com ele off.
- A suavização e a corda não têm pincel de fábrica: o Rebelle tem `pressure_smoothing_strength` 5 nas tintas (escala por
  medir) e nenhum preset com corda.

## ⛔ Recusas MEDIDAS

| proposta | medido | porquê não |
|---|---|---|
| `tiltSmooth` 0,43 no 8B (o grão pela inclinação) | 1,58× de largura ao longo (Rebelle 1,24) | a sobreposição já alisa; o alisamento extra alarga |
| a barra «pontas ≥ 60 % em todos» | o Rebelle dá 0,16–0,35 aos de ponta fina | a barra é a foto de cada um |
| `MAX_SMOOTHING` 0,95 | 95 ms de atraso | o toque mediano dura 111 ms |
| cache decidida pelo pincel (`pen_varies_dabs`) | o rato com a lei ligada mudava bits no Digital | decide-se pelo lote deitado |

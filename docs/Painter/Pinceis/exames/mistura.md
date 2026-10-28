# Exame da família `mistura` (catálogo L — borradores, misturadores, borrachas)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/mistura.rs` (`MEIOS`, `WET_FERRAMENTAS` = a fábrica;
`EM_ESTUDO` = reprovados com receita e portão). Pranchas: `pranchas/mistura/` (`cegas/`, `chaves/`; os presets de
exame fora da fábrica em `presets/`). Régua: `exames/<slug>_regua.txt`. O histórico da J4 (bancada, fábrica das
ferramentas, motor, medições completas): o detalhe está em `ea4e3c5a07` (esta página antes do corte da J5).

| pincel | veredito | onde parou |
|---|---|---|
| L-9 Hard Eraser | **aprovado** (J4) | — |
| L-8 Kneaded Eraser | **aprovado** (J4) | — |
| L-1 Blender Soft | **aprovado** (J4) | — |
| L-6 Water Brush | **curado, não aprovado** (2 rondas) | a base pálida e o «U» na faixa seca raso |
| L-2 Blender Grainy | **curado, não aprovado** (1 ronda + cura sem efeito) | motor: o alcance do misturador e os grumos |
| L-7 Dry/Lift | J6: motor `dryerLevel` + 2 rondas (cega por fazer) | o cruzamento liso; falta a linha escura dos lados |
| L-3 Smudge Finger | J6: motor `smear_fio` (Fine Thread) + 2 rondas (cega por fazer) | o fio contínuo mas pálido; a língua sob a faixa de baixo oca |
| L-5 Blur | reprovado (J4) | falta um oráculo em que o desfoque se veja |
| L-4 Smudge Spiky | fundido com Smudge Finger | — |

## Aprovados (J4; o detalhe está em `ea4e3c5a07`)

- **Hard Eraser** (Digital `EraseAlpha`, `hardness 0,9` + `borda_px 1`, `dab_quina 1`): Rebelle `ERASER/Square Hard`,
  resto · largura · orla 0 · 39 · 2 contra 0 · 40 · 2; régua 48/56. Uso: recortar, borda limpa, ponta quadrada.
- **Kneaded Eraser** (Digital `EraseAlpha`, `hardness 0` `Sharp`, Strength 0,26): Rebelle `ERASER/Round Soft`, deixa
  0,71 da tinta (Rebelle 0,72); régua 72/80. Uso: levantar luz sem fazer buraco.
- **Blender Soft** (ferramenta Wet **Blend**, `felt 0,2`, `blendForce 0,5`, à escala 0,45): Rebelle `BLEND/Soft`,
  alcance acima · abaixo 24 · 21 px contra 26 · 23; r3 escolheu o NOSSO como original nas duas linhas. Custo p95
  0,72 · 1,23 · 7,30 ms, RSS ≤ 97 MiB.

## J5 — L-6 Water Brush, 2.ª ronda — CURADO, NÃO APROVADO

- Preset de exame (`presets/water_brush.ph2d-brush.png`): o Wet Round modo **Wet**, `waterPush 0,87`, **`wetLift 0,2`**
  (era 0,1). Bancada: escala 0,56 (os 38 px do Rebelle), **`PH2D_EXAME_ESCALA_BASE=0,78`**: as faixas da base com a
  largura do Watercolor Round do Rebelle (44–47 px contra 33 na r1 — a queixa da r1). Oráculo `rebelle_mistura/WATER/default_soft`
  (compara-se a B: a A do Rebelle mostra a água a ciano). Prancha `pranchas/mistura/water_brush_r2.png`.
- Régua **38/68** (r1 36/68): a largura das faixas casa (45 × 44/47); os ✗ são a cobertura da base (0,23 × 0,35) e a cor.
  Vizinho Kneaded Eraser d 2,558 (≥ 0,596).
- Revisor cego r2 (sonnet): linha A acertou o Rebelle (a água ciano denuncia-o, média); linha B acertou com confiança
  **baixa** e apontou o NOSSO como imitação: *«a faixa de cima do original é mais escura e densa … os vincos de baixo do
  original são mais fundos, escuros e arrastam mais o vermelho; os do imitador são pálidos e curtos»*. Terceiro (o
  vizinho) claramente outro.
- Veredito (regra de custo, sem 3.ª ronda): **curado, não aprovado**. O que falta, medido: (1) a faixa da BASE do Wet
  Round sai pálida (cobertura 0,23 × 0,35; cor 240,197,201 × 231,164,171) — a bancada, não o pincel: uma base mais
  carregada (`presets/base_escura.ph2d-brush.png`, `pigmentPerDab 160`) aproxima a cor e serve à próxima ronda; (2) o
  escorrido na faixa SECA (o «U» do Rebelle) ainda raso a `wetLift 0,2`.

## J5 — L-7 Dry/Lift — CURADO, NÃO APROVADO (motor)

- Preset (`presets/dry_lift.ph2d-brush.png`): o Wet Round modo **Dry**, `dryerEdge 0,5`, `wetLift 0,25`. Bancada: escala
  0,56, base escala 0,78. Oráculo `rebelle_mistura/DRY/default_sponge`. Prancha `pranchas/mistura/dry_lift.png`.
  Régua **48/80** (os ✗: a cobertura e a cor da base, como no Water Brush). A matriz deu o vizinho `nan` (Airbrush; a
  régua da ferramenta não mede o Dry) — o terceiro da cega foi claramente outro.
- Revisor cego r1 (sonnet): acertou o Rebelle nas duas linhas (confiança baixa) e o NOSSO como imitação. Diferenças:
  *«a imitação é muito mais pálida … nos cruzamentos o original tem um bloco vermelho liso e a imitação um risco fino
  granulado»* (a 3.ª, o azul na A, é a água a ciano do Rebelle).
- Cura (4 variantes, sobre a base carregada): `felt 0,2` · `felt 0,2` + cerdas 4000 · `felt 0,2` + `blendForce 0,2`. A
  base carregada cura a palidez; o cruzamento NÃO muda. Medido no eixo do cruzamento 2 (x 500, B, ÷ a faixa intacta):
  Rebelle na vertical 0,84 · 1,08–1,16 (liso), as linhas escuras VERTICAIS nos lados do traço (1,34 · 1,64 na
  horizontal); o nosso na vertical 1,30 · 1,57 · **1,90** · 1,61 · 1,28 (uma banda escura HORIZONTAL no meio da faixa) e
  1,9–2,16 em toda a largura do traço. ⇒ o `apply_dry_edge` junta o pigmento na direcção do GESTO (ao longo do traço)
  e não nos lados dele — **motor** (fila; o motor Wet é do agente molhado na J5). Sem portão (não aprovado).

## J5 — L-2 Blender Grainy — CURADO, NÃO APROVADO (motor)

- Preset (`presets/blender_grainy.ph2d-brush.png`): a ferramenta **Blend** do Blender Soft + Grain **Noise 24**,
  `intensity 0,2`. Bancada: escala 1,0, base 1,0 (a faixa do Rebelle tem 60 px; a 0,45 a nossa ficava com 27).
  Oráculo `rebelle_mistura/BLEND/default_textured`. Prancha `pranchas/mistura/blender_grainy.png`. Régua **92/146**.
  Vizinho Blender Soft d 2,322 (≥ 0,596).
- Revisor cego r1 (sonnet): nas duas linhas escolheu o **VIZINHO** (Blender Soft) como original, o NOSSO como imitação
  e o **Rebelle como «claramente outro»** (*«curto, denso e muito granulado, com grumos escuros e quase sem degradê»*). ⇒
  o nosso fica entre o Blender Soft e o Rebelle: mais perto do Soft (a coluna em degradê comprida).
- Cura (4 variantes, já com a imagem do revisor): `intensity 0,5` + `paperGate 0,8` · `intensity 1` + `blendForce 1` ·
  Noise 36 + `intensity 0,5` + `paperGate 0,8` + `blendForce 1` · `intensity 0,35` + `paperGate 0,6` + `blendForce 1`.
  Nenhuma dá a mancha curta e grumosa: mais força = topo opaco e redondo (o Soft); menos = pó pálido. Medido (B, x 470..530):
  alcance acima da faixa Rebelle **28** px × nosso 52–66; grumos escuros (verde < 60) a ±40 px 278 × 207 (pálido) ou
  2239 (coluna opaca). ⇒ o que falta é a LEI do transporte do Blend com grão (a janela arrasta a mistura ao longo do
  traço inteiro; o Rebelle mistura só perto da tinta, em grumos opacos) — **motor** (fila). Sem portão.

## J6 — L-2 Blender Grainy — motor `blendReach` + 2 rondas (cega por fazer)

- **Lei** (`Knob::BlendReach`, `trail/transfer/perto.rs`; gate `ph2d-wet-paint` `o_misturador_so_leva_a_tinta_perto`):
  o Blend relaxa cada célula para a média das células da janela a `≤ blendReach × raio` dela (tabela de somas da
  janela), não para a da janela inteira. `0` = a média da janela, ao bit (o mesmo `Media` entra no mesmo corpo do
  relax). Tecto 1: a janela tem ~2 raios + a deriva — a 4 raios o censo do corpo não via diferença. Gate (raio 30,
  `susp > 20`): para trás janela 41 → perto 0,75 28 · 0,375 16; para a frente 15 · 18 (arrasta como a janela).
  Mutações (4) todas VERMELHAS: lei desligada · caixa só para cima · o 0 fora da janela · `r = 1`.
- Receita (preset de estudo `pranchas/mistura/presets/blender_grainy.ph2d-brush.png`, fora da fábrica): o Blender Soft
  + Grain Noise 24 + `intensity 0,2` + **`blendForce 1` + `blendReach 0,75`**. Ronda 1 (alcance 0 · 0,6 · 0,9 · 0,9+força
  1) e ronda 2 (força 1 + alcance 0,75 · +Noise 36/intensidade/gate × 3) — medido (`sobre`, x±30, verde < 200, px):
  | | acima | abaixo | grumos (verde < 60, ±40 px) |
  |---|---|---|---|
  | Rebelle `default_textured` | 30 | 41 | 260 |
  | J5 (janela) | 52 | 37 | 147 |
  | **J6 (0,75 · força 1)** | **36** | **35** | **216** |
  | Noise 36 + intensidade (b/c/d) | 38–47 | 39–50 | 350–860 (mancha opaca larga) |
- Régua 92 → **106/146**. Vizinho Smear Shattered d 2,966 (≥ 0,596). Pranchas `pranchas/J6_Blender_Grainy.jpg` · cega
  `J6_Blender_Grainy_cega.jpg` + `J6_Blender_Grainy_chave.txt`. O que a prancha ainda MOSTRA: o grão do Rebelle são
  grumos grossos e opacos de borda rasgada; o nosso é um pontilhado fino e pálido numa coluna de lados direitos — o
  grão da máscara (o `texv` do Noise), não o alcance. Custo (load 54, o relógio não vale): 200 px rápido p95 283 contra
  284 sem a lei; RSS +17 MiB (a tabela de somas, reutilizada).
- **Smudge Finger** (o fio do arrasto) NÃO é este código: é o `smear_field` do Digital (`EM_ESTUDO`, modo Smear), não a
  janela do Blend do Wet — fica na fila sem custo medido daqui.

## J6 — L-7 Dry/Lift — motor `dryerLevel` + 2 rondas (cega por fazer)

- **O diagnóstico da J5 estava errado** (medido): com `dryerEdge 0` as bandas do eixo ficam iguais. A sonda (foto da
  faixa fresca · depois do secador · depois de simular, cena `sobre` reduzida) mostra o mecanismo: o secador CONGELA os
  sulcos das cerdas da tinta fresca do Wet Round (43 · 199 · 55 a 12 px) que a água nivela nos segundos seguintes (~100
  liso). ⛔ Recusas medidas: assentar o pigmento sob o carimbo (nada muda — não há transporte durante o traço) · nivelar a
  CONCENTRAÇÃO (o filme tem os mesmos sulcos: no-op) · a média sobre toda a água (a franja rasa alarga a faixa 10 px).
- **Lei** (`Knob::DryerLevel`, Tools, `tools/secador.rs`; `0` = ao bit): cada célula DENTRO da tinta que o dab do Dry
  toca pela 1.ª vez toma a média do pigmento suspenso das células dentro da tinta da caixa de meio-lado `L·r` (tabela
  integral, O(área) por dab); dentro = filme ≥ 0,25 × a espessura média da água à volta. Portão
  `familias::mistura::secador::o_secador_seca_a_tinta_ja_nivelada` (desvio no eixo 0,688 → 0,011, massa ± 10 %, franja
  fica, neutro ao bit); mutações (lei desligada · sem piso · caixa de raio 1 · média sobre o papel) todas VERMELHAS.
- Receita (`pranchas/mistura/presets/dry_lift.ph2d-brush.png`, fora da fábrica): J5 + **`dryerLevel 2`** +
  **`dryerEdge 0,25`** (era 0,5). Medido (B, x 500, ÷ a faixa intacta):
  | | eixo (ao longo) | linha dos lados | fora da linha |
  |---|---|---|---|
  | Rebelle `default_sponge` | 0,9–1,14 | 1,31 · 1,35 | 0,97–1,02 |
  | J5 (a mesma base) | 0,35 · 1,63 · 0,35 | 1,22 · 1,11 | 0,73 · 0,82 |
  | r1 (`dryerLevel 2`, edge 0,5) | 0,96–1,10 | 1,17 · 1,10 | 0,71 · 0,78 |
  | **r2 (edge 0,25)** | **0,98–1,10** | 1,09 · 1,04 | **0,87 · 0,93** |
  Ronda 2 entre `dryerEdge` 0 · 0,15 · 0,25 · 0,35: nenhuma dá a linha de 1,3 sem o claro de fora (o anel puxa de
  FORA). Régua 48 → **62/80**. Vizinho Smear Flat d **2,491** (≥ 0,591). Pranchas `pranchas/J6_Dry_Lift.jpg` · cega
  `J6_Dry_Lift_cega.jpg` + `J6_Dry_Lift_chave.txt` (o oráculo só tem a cena `sobre`).
- O que a prancha ainda MOSTRA: o Rebelle risca a linha escura nos dois lados do secador; o nosso deixa um claro fino
  (fila 1).
- **Water Brush**: NÃO é o mesmo mecanismo (a ferramenta Wet e o `waterPush`; o que falta é a base pálida e o «U» na
  faixa seca) — fica.

## J6 — L-3 Smudge Finger — motor `smear_fio` (Fine Thread) + 2 rondas (cega por fazer)

- **O que o fio é** (medido na foto `rebelle_mistura/SMUDGE/default_soft_sobre_B`, massa = Σ (248 − verde)/248 em x0 ± 12,
  igual nos três cruzamentos): da faixa de cima o V desce (17,3 px a y 180 → 5,0 a y 290); o traço «papel» corta-o em
  y 305–312 e puxa um rasto horizontal (1,61 a +30 px · 0,27 a +100 · ~0 a +200); por baixo, o fio CONTÍNUO até à faixa
  de baixo (y 322..440: 0/118 linhas vazias), pálido (verde mín 59 → 147), a massa a decair 3,66 → 0,97 (média 2,14).
  Nosso (J4, o ponto): 35/118 linhas vazias — o fio em contas soltas, o rasto horizontal partido (198/225 colunas vazias).
- **Mecanismo** (medido): o mapa do campo é LISO por texel ao longo do gesto, mas atravessado ele salta dezenas de px
  entre o eixo (a corrente levou o início do traço) e o texel ao lado; a faixa cabe numa lasca sub-píxel e o re-amostrar
  num ponto acerta-a ou falha-a. ⛔ Não é o campo re-amostrado a cada pingo (o diagnóstico da J4): o passo é ao longo do
  gesto e não difunde na transversal.
- **Lei** (`BrushSpec::smear_fio`, «Fine Thread» no Smear do Digital; `warp::session::reamostra_por_area`; `false` = ao
  bit, rota separada): cada texel é a média do `pre` sobre a pegada do texel levada para trás pelo mapa (grelha nx × ny
  pelo salto do `disp` aos vizinhos, só onde ele passa de 1 px). Teto 64 por eixo (convergência contra 256: 0 px com
  |Δ| > 8; 32 → 1 px, 16 → 81). Custo da cena inteira na sonda (load ~24, indicativo): 0,24 s (ponto) → 1,37 s.
  Portão `familias::mistura::o_fio_do_arrasto_do_dedo_e_continuo` (0/118 vazias em x 250 · 500 · 750, massa 0,6..4,
  controlo sem o fio ≥ 10 vazias); mutações M1 lei desligada · M2 teto 1 · M3 só o eixo do gesto · M4 amostras no
  centro · M5 o preset não lê — todas VERMELHAS. Fábrica e censos: verdes (a miniatura vermelha da corrida é a do óleo,
  agente paralelo — só Oil/*).
- **Rondas** (sonda `sonda_o_fio_do_arrasto`, `PH2D_SONDA_BRUSH`): |Δ massa| médio por linha contra o Rebelle (acima ·
  abaixo do «papel») e o V a y 130 · 140 · 160 (Rebelle 21 · 15 · 11):
  | variante | acima | abaixo | V | rasto horiz. +30 · +100 |
  |---|---|---|---|---|
  | **Smooth (a receita J4) + fio** | 2,94 | 0,80 | **22 · 18 · 12** | 0,41 · 0,04 |
  | Root + fio | 2,28 | **0,50** | 34 · 28 · 22 | 1,20 · 0,39 |
  | InvSquare + fio | 2,98 | 0,48 | 30 · 26 · 22 | 0,64 · 0,12 |
  | Sphere + fio | 5,39 | 0,56 | — | 0,62 · 0,28 |
  | dureza 0,3 | 5,68 | 1,02 | — | 0,10 · 0 |
  | Linear + fio | 2,13 | 2,02 (fio some) | — | 0,22 · 0 |
  As quedas cheias dão a massa do fio mas abrem o V a 34 px (o «largura» da identidade cai): fica a Smooth.
- Receita (`pranchas/mistura/presets/smudge_finger.ph2d-brush.png`, EM_ESTUDO): a J4 + **`smear_fio`**. Régua J4
  94/146 → **134/186** (`exames/smudge_finger_J6_regua.txt`; a cena `papel` passou a ter tinta). Vizinho Smear Soft
  d **2,798** (≥ 0,591). Pranchas `pranchas/J6_Smudge_Finger.jpg` · cega `J6_Smudge_Finger_cega.jpg` +
  `J6_Smudge_Finger_chave.txt`.
- O que a prancha ainda MOSTRA: o fio nosso é mais pálido e fino (massa 1,25 × 2,14, plana em vez de decair); abaixo da
  faixa de baixo o Rebelle leva uma língua CHEIA de vermelho, a nossa é um «U» oco (o eixo relaia sem tecto e traz o
  papel do início do traço — o tecto medido e recusado do `smear_field`); as faixas do Rebelle têm estrias.
- **L-5 Blur**: NÃO é o mesmo Smear (rota Blur, outro código) e o que lhe falta é um oráculo em que o desfoque se veja —
  fica.

## Fila (motor, com a medição acima)

1. **Dry/Lift** — a linha escura nos LADOS do secador (1,09 · 1,04 × Rebelle 1,31 · 1,35) sem o claro de fora: o anel
  do `dryerEdge` tira a tinta de fora; a do Rebelle vem de dentro ou da água ao lado (por medir).
2. **Blender Grainy** — o alcance do Blend com grão limitado à tinta (52–66 px × 28) e os grumos opacos.
3. **Smudge Finger** — o fio já é contínuo (J6); falta a MASSA dele (1,25 × 2,14 px) sem abrir o V, e a língua cheia
  abaixo da faixa de baixo (o eixo relaia sem tecto e traz o papel do início do traço).
4. **Bancada**: uma base Wet mais carregada para as ferramentas de água (`presets/base_escura`), antes de rever de novo
   o Water Brush. O vermelho que cobre mais que o azul (alpha-over em sRGB, J4) continua medido e não curado.

## Portões (verdes por nome, J4; nada mudou na J5)

`familias::mistura::cada_ferramenta_da_mistura_se_reconhece_pela_foto` (as 5 da fábrica e em estudo, contra a foto e
os irmãos) · `familias::mistura::o_portao_da_mistura_sangra_com_cada_mutacao` (7 mutações, todas sangram) ·
`familias::mistura::motor::*` (WaterPush, o grão no Blend, DryerEdge; neutro = ao bit).

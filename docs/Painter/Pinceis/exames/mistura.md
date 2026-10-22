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
| L-7 Dry/Lift | **curado, não aprovado** (1 ronda + cura sem efeito) | motor: a orla do secador sai na direcção errada |
| L-3 Smudge Finger | reprovado (J4) | motor: o fio do arrasto |
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

## Fila (motor, com a medição acima)

1. **Dry/Lift** — a orla do secador nos LADOS do traço (hoje sai ao longo dele: 1,90 no eixo × Rebelle 1,16).
2. **Blender Grainy** — o alcance do Blend com grão limitado à tinta (52–66 px × 28) e os grumos opacos.
3. **Smudge Finger** — o fio do arrasto (J4: o campo re-amostrado a cada pingo perde o traço sub-píxel).
4. **Bancada**: uma base Wet mais carregada para as ferramentas de água (`presets/base_escura`), antes de rever de novo
   o Water Brush. O vermelho que cobre mais que o azul (alpha-over em sRGB, J4) continua medido e não curado.

## Portões (verdes por nome, J4; nada mudou na J5)

`familias::mistura::cada_ferramenta_da_mistura_se_reconhece_pela_foto` (as 5 da fábrica e em estudo, contra a foto e
os irmãos) · `familias::mistura::o_portao_da_mistura_sangra_com_cada_mutacao` (7 mutações, todas sangram) ·
`familias::mistura::motor::*` (WaterPush, o grão no Blend, DryerEdge; neutro = ao bit).

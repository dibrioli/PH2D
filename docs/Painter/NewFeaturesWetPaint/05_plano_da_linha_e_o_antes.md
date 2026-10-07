# 05 — Plano da linha «Wet Painter» e o «antes» medido (2026-10-07)

> Linha `line/PainterWatercolor` reaberta sobre `main` 2a925859d (decisão do dono 07/10: só o Wet Paint
> evolui). Este doc é o **tracker** da linha: as ondas do [04](04_PROMPT_linha_wet_painter.md) com o ponto
> de código onde cada uma entra, e a barra de cada gate tirada do VALE entre o nosso «antes» e o Rebelle
> ([03 §2](03_tracos_medidos_rebelle_x_ph2d.md)) — nunca de um número escolhido.

## §1 — O «antes» (W0), reproduzido

`PH2D_SONDA_SIZE=28|40` sobre 2a925859d, duas corridas: **sha256 iguais nos quatro meios** (a sonda é
determinística ⇒ a cerca «byte-idêntico» é possível). A tamanho 28 os números batem **exactamente** com o
03 §2 (mesma régua, `ferramentas/mede_traco.py`).

| Wet Paint hoje | larg. px | taper 5/50/95 | orla px | CV | período (% pot.) | textura % | borda/miolo | cob |
|---|---|---|---|---|---|---|---|---|
| size 28 recto | 28 | 28/29/15 | 6 | 0,036 | 19,4 (5 %) | 14,4 | 0,61 | 0,78 |
| size 28 rampa | 20 | 8/28/8 | 6 | 0,066 | 17,6 (6 %) | 13,0 | 0,65 | 0,79 |
| size 40 recto | **44** | 46/45/41 | **14** | 0,031 | 2,0 (**16 %**) | 12,0 | 0,52 | 0,77 |
| size 40 rápido | 44 | 36/45/24 | 14 | 0,033 | 2,0 (14 %) | 12,3 | 0,52 | 0,77 |
| size 40 rampa | 28 | **9/46/4** | 9 | 0,064 | 2,0 (8 %) | 13,8 | 0,55 | 0,75 |

Digital/Watercolor/Impasto a 40: 58 / 68 / 72 px de largura (o mesmo número, quatro larguras).

## §2 — Onde o código entra (o mapa que decide a cerca)

- O gesto vira carimbos no motor PARTILHADO (`stroke_lifecycle` → `dynamics`/`spec` → `stamp_route.rs:340
  stamp_dabs_inner`); o braço Wet Paint é **`wetpaint/dab_route.rs:24 stamp_dabs_wetpaint(dabs, brush)`**,
  que já recebe `radius_px`, `stroke_radius_px` e a pressão de cada carimbo. ⇒ **largura em px, piso/tecto
  de pressão, smoothing e inclinação em dois eixos entram AQUI** (re-mapeiam o carimbo), nunca em
  `dynamics.rs`/`spec.rs`.
- Fios: `ph2d-wet-paint/src/brush.rs:40 BristleKnobs` + `build_bristle_texture` (tile 128² fixo, semente
  `BRISTLE_SEED`) — o tile é ARRASTADO pela lane do traço inteiro: daí os «carris». `trail.rs` (598 l.) e
  `solver.rs` (553 l.) estão perto do tecto de 700 ⇒ os fios com física vão para ficheiros irmãos
  (`bristles_*.rs`).
- Knobs: `wetpaint_settings.rs:26 WetKnobs` + `ph2d-wet-paint/src/tuning.rs KNOB_DEFS`. Um preset do Wet
  Painter = `WetKnobs` + o sub-conjunto do `BrushSpec` que o braço lê + os campos novos das ondas.
- Luz do relevo: `impasto_light.rs:285 apply_impasto_light(&self, …)` é método do estado do Impasto, não
  função livre ⇒ W3 precisa de uma porta livre append-only num ficheiro irmão (o corpo não muda).
- Ponteiro: `ph2d-editor-core/src/tool.rs:230 CanvasPointer {pos, pressure, tilt:[f32;2], phase}` — tilt já
  tem os dois eixos; **não há barrel**. ⚠️ Pôr o barrel é mexer em contrato congelado (§6) ⇒ decisão do
  dono antes da W6 (ver §4).
- Restrições herdadas: determinismo ao bit (ADR-0134/0147), pior tick ≤ 2 ms (ADR-0134), Gauss-Seidel do
  arrasto fica (doc 44 §1), o vidro da aguada (`wetpaint/composite.rs`, `vidro_por_cima_tests.rs`) continua.

## §3 — As ondas, o que entregam e a barra (do vale)

| onda | entrega | gate (barra = vale «antes» ↔ Rebelle) |
|---|---|---|
| **cerca** | `the_other_media_are_untouched` (diff vs merge-base) + fixture `fixtures/sonda_28/{digital,watercolor,impasto}.ppm` com cabeçalho | sha256 iguais; mutação num dos três meios reprova |
| **W0** | sonda com `PH2D_SONDA_PRESET`; as 12 fotos do Rebelle em `fixtures/rebelle/` com cabeçalho (tool, preset, volumes, semente 7); `mede_todos.sh` sem caminho de scratchpad; tabela dos dois lados gravada | largura medida = pedida ± 2 px em todos os presets do Wet Painter (hoje 44 a 40 ✗) |
| **W1** | crate `ph2d-brush-presets` (`*.ph2d-brush.png`), fábrica em `assets/brushes/WetPainter/`, browser + 5 volumes | round-trip ao byte; presets contados; campo desconhecido não falha; miniatura por hash |
| **W2** | fios com pressão Perlin, largura ondulante, oscilação, Thin/Thick/Edges, Random Span, ruído HSLuv | a 40 px: **5 ≤ textura ≤ 10 %** (hoje 12–14 ✗; Rebelle óleo 5,9, express 9) · **CV ≤ 0,02** (hoje 0,031 ✗; dele 0,011) · **período < 15 %** (hoje 16 % ✗; dele 9 %) · controlo: knobs a 0 ⇒ PNG de hoje ao byte |
| **W3** | campo de altura no grid (fios Add/Remove), oleosidade = viscosidade, luz do Impasto por porta livre | óleo: borda/miolo **0,8–0,9** (hoje 0,52 ✗; dele 0,87) e textura na banda da W2; acrílico: largura A = B |
| **W4** | lei do papel do fluido (força × pressão, contraste, scratch, fundo que fura, `PaperBehaviour` × 5 papéis) | lápis 8B: textura **≥ 8 %**, borda/miolo **< 0,5** (dele 9,1 / 0,32; não temos lápis) |
| **W5** | Paint / Paint&Mix / Paint&Blend / Blend / Erase por pressão e limiar da queda; Loading+Length; Dirty Brush; Glaze | 1.º a bancada do Rebelle (03 §4, redondo a cruzar + 3.º traço fora); depois o nosso na mesma direcção |
| **W6** | piso/tecto/smoothing de pressão, Pulled String, tilt em 2 eixos, rotação {None, Follow, PenTilt[, PenRotation]} | rampa: pontas **≥ 60 %** do miolo (hoje 9/46/4 = 20 % / 9 % ✗; dele 35/42/33) · 8B a 55°: largura **≥ 1,2×** e textura menor (dele 37/30, 5,2 vs 9,1) |
| **W7** | caminho água 0 sem solver (lápis, carvão, pastel, giz, marcador, airbrush); Water como volume | a prancha dos 8 meios dentro do vale nas 7 métricas (barra com os dois lados) |
| **W8** | painel: browser + volumes + Stroke/Bristles/Paint/Paper + Tilt + Blow, i18n, tokens | censo sem controlo morto; cada knob com mutação que «morde» |

## §4 — Decisões do dono (07/10)

1. ✅ **A largura prometida vale para TODAS as mídias, secas incluídas** (dono: «SIM»). O Rebelle, a
   `size_px 40`, entrega 41–42 px nos meios de tinta mas **30 (lápis 8B), 17 (pastel), 18 (marcador), 26
   (airbrush)**: a textura e a orla comem a largura. Nós não: o número no painel é a largura medida ± 2 px.
   ⇒ a comparação da W7 com o Rebelle nos secos faz-se a largura MEDIDA igual, não ao número.
2. ✅ **Sem rotação da caneta (barrel)** (dono: «por enquanto ficamos sem rotação»). O `CanvasPointer`
   (contrato congelado) não muda; a W6 entrega None / FollowTrajectory / PenTilt. A mesa de teste do dono
   é uma **Wacom Intuos5 Touch** (pressão 2048 níveis + inclinação; sem barrel — a única caneta Wacom com
   barrel, a Art Pen, está descontinuada; Huion/XP-Pen não o têm). Reabrir = ADR próprio.

## §4b — Estado

- ✅ **Cerca** (07/10): `tests/it/os_outros_meios_nao_mudam.rs` (FNV dos pixels da sonda a 28; perfis
  `test` e `smoke` iguais; 0,7 s) + `ferramentas/cerca.sh` (ficheiros protegidos só ganham linhas).
  Mutações: `impasto_light.rs:48 DEPTH_UNIT_PX` 16→17 ✗ impasto · `watercolor_render.rs:40 SS0` 0,12→0,13 ✗
  watercolor · `ph2d-painter-brush/src/falloff.rs:115` Smooth 3,0→3,1 ✗ digital · controlo
  `ph2d-wet-paint/src/brush.rs:21 BRISTLE_SEED` +1 ⇒ cerca VERDE com o Wet Paint mudado (fnv
  `0x2c12…` → `0xaa36…`). `cerca.sh`: linha alterada ✗, só acrescentar ✓.
- A sonda passou para o binário único `tests/it/` (era `tests/*.rs` solto); os gestos vivem em
  `tests/it/sonda_gestos.rs`; os bytes dos quatro meios não mudaram com a mudança de sítio.

## §5 — Ritmo

Uma janela por onda (CLAUDE.md §2). Esta janela: cerca + W0 (+ W1 se couber). Medições de cada onda numa
rodada só, intercaladas (§0.10); o oráculo só se reabre para as bancadas novas (W5 cruzamento, W4 papéis,
W6 tilt), todas numa sessão do Rebelle.

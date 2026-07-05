# Curvas do Stroke/Selection — Convert · Merge · Simplify · Offset (as soluções)

> **Estado: FECHADO 2026-07-05** (sessão Enio + Claude). Este doc consolida a arquitetura final do
> pipeline de curvas editáveis do Painter — o que cada botão faz, os algoritmos por trás, as constantes,
> e as lições da saga (3 iterações de Simplify, 3 de Offset). O histórico de bugs/erros de diagnóstico
> vive em [`BUGS_painter.md`](BUGS_painter.md) (Bugs #4 e #5 + codas); aqui é o **desenho final**.

## 0. Mapa dos módulos

| Módulo (`crates/ph2d-tool-painter/src/tool/paint/`) | Responsabilidade |
|---|---|
| `curve_refit.rs` | **Funil de qualidade único** de Simplify + Merge: corner-split + fit Schneider por mínimos quadrados |
| `curve_offset.rs` | Offset CAD (reconstrução adaptativa sub-pixel, Levien 2022) — usado só pelo DESENHO |
| `curve_join.rs` | Quinas do offset: suave funde / convexa = miter exato / côncava DIVIDE (spine cruza → Trim corta) |
| `curve_geom.rs` | flatten (com seam fechado), hit-test, insert, `densify_closed_curve` (de Casteljau exato) |
| `curve_model.rs` | Core compartilhado (points/handles/kinds/selected) — stroke E selection usam o MESMO |
| `selection_edit.rs` | Convert/Merge/Simplify da Seleção (mesmos algoritmos, lista `selection_shapes`) |
| `stroke_boolean.rs` | Composite booleano multi-shape (rasteriza SS×3 → une/subtrai → traça contornos) + Merge do stroke |

## 1. Convert to Curve — per-shape, PRISTINO, denso

- Converte **cada** shape (ativa + parked) na SUA curva editável — **nunca funde** (fusão = botão Merge).
- Elipse → 4-arc exato (K=0.552285); Polígono → vértices sharp; ambos **densificados** via
  `densify_closed_curve` (de Casteljau, forma reproduzida EXATA) a `CONVERT_ANCHOR_SPACING_PX=16`,
  cap `MAX_CONVERT_POINTS=512`. Curva existente passa verbatim.
- **PRISTINO:** o Offset vivo NUNCA é assado nos pontos (era a fonte do "bico V" — ver §4). O offset
  persiste como transform de desenho; slider não reseta.
- Ops preservadas por shape (Add/Remove/Overlay); primeira instala como editor ativo, resto parkeia.

## 2. Merge Curves — traçar a máscara composta → REFIT

- Coleta toda shape **fillable** (Overlay conta como Add — paridade com a Seleção), rasteriza o composite
  booleano supersample 3×, traça CADA componente conectado (`trace_all_contours`), e passa cada contorno
  pelo **refit** (§3) com tolerância `MERGE_SIMPLIFY_TOL_PX = 1.0px`.
- Resultado: uma curva limpa por região — poucos pontos, quinas Free reconstruídas, zero bico.
- Shapes não-fillable (Line/Curve abertas) sobrevivem parked; nada é perdido.

## 3. Simplify — refit por mínimos quadrados (o padrão Inkscape/paper.js)

**Pesquisa:** Schneider 1990 (*An Algorithm for Automatically Fitting Digitized Curves*, Graphics Gems) é
a base do `simplify()` do paper.js e do Inkscape; Levien 2023 (*Simplifying Bézier paths*) confirma o
framing. Simplificar curva = **REFIT**, nunca decimação de pontos (duas tentativas de decimação — DP puro
e Visvalingam-20% — deram curvas "quase", nunca perfeitas).

Pipeline (`curve_refit::refit_closed_spine`):
1. **Flatten** pro spine denso (anel fechado de samples).
2. **Detecção de quinas** (cusps): giro entre as cordas de ±`CORNER_WINDOW_PX=3` de arco;
   quina se cos ≤ `CORNER_COS=0.35` (~70°+); supressão não-máxima com raio `CORNER_SUPPRESS_PX=8`
   (**tem de exceder 2× a janela** — senão UMA quina responde em dois samples e vira duas).
3. **Reconstrução do vértice** (§4): apara a ponta arredondada, re-ancora na interseção das bordas.
4. **Fit Schneider** (`ph2d_painter_brush::fit_curve`) em cada trecho **ABERTO** entre quinas —
   aberto = zero risco do colapso do fit em loop start==end (Bug #4).
5. **Kinds do fit:** junção suave = **Aligned** (braços colineares, comprimentos independentes — o fit
   carrega a forma NOS comprimentos; Symmetric os igualaria e distorceria); quina = **Free** (braços
   fitados independentes; Vector os apontaria pros vizinhos e mataria a curvatura de aproximação).
   É o modelo do Illustrator: smooth points alinhados, corner points livres.
6. Anel com <3 cusps ganha **seams artificiais nos terços** (um cúbico engole meio-anel dentro da
   tolerância → assembly degeneraria com <3 âncoras); seams re-suavizados pra Aligned.
   Círculo → anel mínimo de **3 âncoras** (arcos de 120°), spine a <1.5px do círculo real.

**Progressivo:** cada aperto escala a tolerância (`0.5px ×1.7 …` até `32px`) até derrubar ~20% das
âncoras (`SIMPLIFY_KEEP_FRACTION=0.8`, piso `SIMPLIFY_MIN_POINTS=5`); num shape já mínimo o botão
no-opa (não há o que perder). Curva ABERTA (Free Hand) continua no fit Schneider direto (sempre foi bom).

## 4. Quinas — vértice verdadeiro (o fix do offset arredondado)

O trace da máscara é **suavizado** (média móvel), então a ponta de cada quina chega ~2px arredondada — e
o **offset amplifica o arredondamento por |d|** (ponta raio 2px offsetada 20px = arco visível de raio
22px). Fix em `curve_refit` (não no offset):
- Apara `CORNER_TRIM_PX=3` de arco de cada lado da quina REAL (descarta a ponta suavizada);
- Mede as direções das duas bordas num baseline limpo (3–9px da quina);
- Re-ancora a quina na **interseção das duas retas de borda** — vértice-navalha (fallback pro sample
  traçado se bordas ~paralelas ou interseção implausível >6×trim).

Medido: quadrado de pontas arredondadas → quinas a <1.2px do vértice verdadeiro; offset 12px alcança o
ápice do miter a <1.5px (arredondado ficaria ~5px aquém).
**Lição:** quando um consumidor AMPLIFICA erro (offset × curvatura de ponta), a fonte deve reconstruir a
geometria ideal, não reproduzir fielmente o dado degradado.

## 5. Offset — DRAWING-ONLY (o modelo da Seleção)

- O editor INTEIRO — âncoras, handles, gizmo E a **linha-guia** — fica na curva **PRISTINA**.
  Só o **desenho pintado** (dabs) sofre o offset: `offset_curve_spine` roda `offset_curve_refined`
  (a curva paralela CAD exata) e usa **só o spine achatado**; Trim opcional corta auto-interseções.
- `bake_curve_offset` é **no-op**; Apply & Keep só acumula o valor do slider (geometria intocada).
- Por quê: 3 tentativas — (1) re-flow das âncoras offsetadas = remendo; (2) offset por miter de
  polilinha = rebaixou a curva perfeita; (3) **certo**: a fonte da verdade parada, só o resultado
  renderizado desloca — exatamente como a Seleção offseta a máscara (SDF) e não a curva.
- Quinas no offset: `curve_join` — convexa = **miter exato** (`(n1+n2)/(1+n1·n2)`, clamp 4×), côncava
  **divide em 2 âncoras** → spine cruza → Trim corta a orelha (offset-then-trim CAD).

## 5b. Offset da SELEÇÃO — quinas afiadas (2026-07-05, paridade com o stroke)

O grow/shrink da Seleção era um SDF euclidiano — que **arredonda quinas por construção** (dilatação =
Minkowski com um disco: quadrado crescido por d ganha arcos de raio d nas quinas). Substituído pelo
pipeline do stroke ([`selection_offset_geom.rs`](../../crates/ph2d-tool-painter/src/tool/paint/selection_offset_geom.rs)):

1. Traça o crisp-fonte em contornos fechados — **externos E buracos** (`trace_all_contours_with_holes`;
   o SDF via buracos implicitamente, o offset por contorno precisa ver cada boundary).
2. **Refit** de cada contorno (o mesmo `refit_closed_spine` — quinas re-ancoradas na interseção de bordas).
3. Direção de crescimento **calibrada numericamente** por contorno (probe +2px, compara |área|).
4. Por nível: CAD offset (`offset_curve_refined` — miter/split) — externos `+d`, buracos `−d` (o buraco
   ENCOLHE quando a seleção cresce) — e **fill por winding assinado**: a região principal gira COM a
   orientação da fonte; orelhas dobradas giram CONTRA → `sign(winding) == sign(fonte)` preenche exato,
   **sem trim nenhum** (lição: nenhum peeling iterativo de trim é confiável quando as orelhas superam a
   área do miolo — um composto main+orelhas engana qualquer ranking; o winding é exato por definição).
5. Máscaras por nível cacheadas (anéis pinados + nível vivo); anéis/`materialise`/overlay usam as mesmas
   máscaras (o EDT/SDF foi deletado). Fonte re-capturada no ENGAGE do slider (o crisp é pré-offset
   exatamente aí — fonte "stale" perdia um buraco recém-subtraído).

6. **Fast-path PARAMÉTRICO (precisão stroke-exata):** quando toda entry tem geometria exata (marquee
   Polygon/Ellipse, curvas convertidas), o offset roda POR SHAPE direto na geometria pristina — elipse
   `r ± d` continua elipse perfeita; polígono/curva no CAD offset dos anchors — e compõe pelas ops
   (Add/New crescem, Remove encolhe), espelhando o `stroke_state_to_fill_shape(st, off)` do stroke. O
   trace-de-máscara fica só pra `Raster`/ring mode, onde não existe verdade paramétrica.

Medido: quadrado +16px → quina do miter selecionada (o SDF a excluía — dist √2·13 ≈ 18.4 > 16); shrink
−12 mantém quadrado exato; donut +8 cresce por fora e fecha o buraco sem engoli-lo; retângulo +10 com
bordas ANALÍTICAS exatas nos 4 lados; elipse +8 = círculo perfeito (eixo E diagonal a ±1px). Nada do
stroke mudou (`curve_trim` só ganhou o helper aditivo `loop_sign_positive`).

## 6. UI / fluxo (paridade Stroke ↔ Selection)

| Botão | Stroke | Selection |
|---|---|---|
| Convert to Curve | per-shape, pristino, denso | idem (`selection_convert_to_curve`) |
| Merge Curves | `merge_open_shapes_to_curves` | `selection_merge_curves` |
| Simplify Curve | `curve_simplify` (progressivo; visível em qualquer curva fechada editável) | `selection_simplify_curve` (todas as Freehand da lista) |
| Add/Remove no gizmo | tap no quadrado central cicla (o/+/−) | tap cicla Add↔Remove (`selection_op_tap`) |

## 7. Constantes (fonte: `curve.rs` + `curve_refit.rs`)

| Constante | Valor | Papel |
|---|---|---|
| `CONVERT_ANCHOR_SPACING_PX` | 16 | espaçamento do Convert denso |
| `MAX_CONVERT_POINTS` | 512 | cap do Convert/Merge |
| `MERGE_SIMPLIFY_TOL_PX` | 1.0 | tolerância do refit no Merge |
| `SIMPLIFY_KEEP_FRACTION` | 0.8 | alvo de queda por aperto (~20%) |
| `SIMPLIFY_MIN_POINTS` | 5 | piso do Simplify |
| `REFIT_BASE_ERR_PX` / `REFIT_MAX_ERR_PX` | 0.5 / 32 | escalada da tolerância por aperto |
| `CORNER_COS` / `CORNER_WINDOW_PX` / `CORNER_SUPPRESS_PX` | 0.35 / 3 / 8 | detector de cusps |
| `CORNER_TRIM_PX` | 3 | apara da ponta arredondada + baseline do vértice |

## 8. Undo/redo — coalescing por tipo (2026-07-05)

O undo era granular demais ("cada mínima ação entra na sequência"): N apertos de Simplify = N entradas,
N taps de op = N entradas, e o tap do STROKE nem registrava undo (`active_op` não entrava no snapshot).
Solução (`undo.rs::CoalesceKind` + `record_structural_coalesced`):

- Entradas coalescíveis carregam um `CoalesceKind` (`Simplify` · `OpCycleStroke` · `OpCycleSelection(i)`).
- Um **run consecutivo do mesmo kind** colapsa numa entrada só: `before` do PRIMEIRO, `after` do último —
  um Ctrl+Z volta ao estado pré-run inteiro. Qualquer ação normal (kind `None`) quebra o run; um
  undo/redo nunca funde através da fronteira (redo não-vazio → entrada nova).
- `active_op` agora é capturado no `ModelSnapshot` e restaurado no undo — o tap de op do stroke virou
  undoável (e coalescido).
- Arrastos de ponto / inserções continuam 1 gesto = 1 entrada (padrão dos vector tools — desfazer um
  ajuste de ponto individual é desejável).

Testes: `coalesced_runs_merge_and_break_correctly` · `coalescing_never_merges_across_an_undo_boundary`
(unit) · `simplify_run_is_one_undo_step_back_to_the_dense_curve` · `stroke_op_tap_run_is_one_undoable_step`
· `selection_op_tap_run_is_one_undoable_step` (integração).

## 9. Testes-âncora (em `paint/tests.rs`)

- `simplify_refits_a_circle_to_few_faithful_anchors` — 16→3 Aligned, spine <1.5px do círculo.
- `simplify_refits_a_polygon_to_exactly_its_free_corners` — N vértices = N quinas Free.
- `refit_corner_offset_reaches_the_sharp_miter_apex` — vértice reconstruído <1.2px; offset no ápice <1.5px.
- `merge_produces_a_clean_low_point_curve_at_a_sharp_waist` — peanut sem auto-cruzamento.
- `offset_moves_only_the_painted_drawing_not_the_editor` + `offset_apply_keep_absorbs_the_offset_keeping_the_drawing_put` — drawing-only.
- `convert_to_curve_is_per_shape_not_merged` · `merge_curves_folds_the_boolean_result_into_one_curve` ·
  `edit_with_a_live_offset_converts_the_pristine_circle_keeping_the_offset` · `selection_centre_square_tap_cycles_add_remove`.

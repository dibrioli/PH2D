# HANDOFF — Painter "Stroke" section (Blender clean-room) — landed, but 4 behavioral gaps open

> Status: **a seção Stroke inteira foi implementada e está VIVA** (engine + seam + painel UI),
> mas o Enio testou e apontou gaps de comportamento vs Blender que **NÃO estão fechados**. Tudo
> committado local, **nada pushado**. Seja cético: o engine é fiel ao Blender e bem testado, mas
> "fiel ao paint_smooth_stroke" ≠ "se sente como o Blender" — a diferença é densidade de amostra
> e a UI não esconder parâmetros por método. **Leia a §2 (gaps abertos) antes de mexer.**
>
> Toda etapa: reler [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).
> Plano/algoritmos: [`docs/Painter/02_plano_de_implementacao.md`](Painter/02_plano_de_implementacao.md) +
> [`03_algoritmos_referencia_blender.md`](Painter/03_algoritmos_referencia_blender.md).
> Source Blender vendido (behavioral reference, GPL — clean-room, NÃO transcreva):
> `reference/blender-texture-paint/blender_src/source/blender/editors/sculpt_paint/paint_stroke.cc`.

## §0 — TL;DR

A seção Stroke do Blender (Method/Spacing/Adjust-Strength/Jitter+Unit/Dash/Input-Samples/Stabilize)
foi portada clean-room: engine puro (`ph2d-painter-brush`), seam do tool (`ph2d-tool-painter`),
e painel UI (`ph2d-panel-painter-layers/src/paint_stroke.rs`). Defaults conferidos = Blender DNA.
Mas: ~~(1) o traço não fica fluido com stabilize baixo~~ → **✅ RESOLVIDO 2026-06-21** (§2.3: ranges
do Blender `factor∈[0.5,0.99]`/`radius∈[10,200]` verificados no `rna_brush.cc` + clamp engine/tool/
painel); (2) **Drag Dot** ainda deixa rastro em vez de carimbo único; ~~(3) UI não esconde params
por método~~ → **✅ RESOLVIDO 2026-06-21** (§2.1); (4) Anchored/Line/Curve seguem **DEFER**.
**Abertos agora: 2.2 (Drag Dot carimbo único) · 2.4 (interativo + airbrush on_tick).**

## §1 — O que está PRONTO (mantenha; é fiel + testado)

Commits desta jornada (locais, não pushados):
- `517fb168` — engine: `StrokeMethod`/`JitterUnit` + campos de `BrushSpec` (defaults Blender) +
  pipeline input-samples→stabilize→método→dash+jitter+atenuação. 56 testes.
- `38d63b7d` — seam tool + painel UI (dropdowns Method/Jitter-Unit, sliders, toggles), 7 sites
  fiados + 3 seam tests (`ph2d-ui-testkit`) + 1 teste de roteamento no tool.
- `c33e04a7` — **suavização de trajeto (quadrática-midpoint)** no método Space: o input vira
  curva antes do spacing, então traço curvo não faceta. `Stroke::finish()` drena a cauda (chamado
  em `paint_end`). Teste de quina arredondada.

Arquivos-chave:
- Engine: [`ph2d-painter-brush/src/stroke.rs`](../crates/ph2d-painter-brush/src/stroke.rs)
  (pipeline + `walk_smoothed`/`walk_space`/`finish`), `stroke_method.rs` (enums + predicados
  `is_spaced`/`uses_dash`/`supports_smooth`/`allows_jitter`/`forces_full_pressure`/`emits_on_begin`),
  `spec.rs` (campos + `dash_on`/`space_overlap_factor`), `sampler.rs` (input-samples).
- Tool: [`ph2d-tool-painter/src/tool/paint.rs`](../crates/ph2d-tool-painter/src/tool/paint.rs)
  (`BrushSettings` + setters `set_brush_*` com clamp + consts `BRUSH_*_MAX`),
  `tool/trait_impls.rs` (`handle_panel_event` roteia SetValue/Click/SelectOption).
- Painel: [`ph2d-panel-painter-layers/src/paint_stroke.rs`](../crates/ph2d-panel-painter-layers/src/paint_stroke.rs)
  (a seção), `event.rs` (drains), `populate.rs` (registro), `paint_brush.rs` (helpers
  `pub(crate)` reusados: `ParamRow`/`paint_param_row`/`paint_dropdown_row`/`paint_toggle_row`/
  `paint_dropdown_popover`). Ids: `ph2d-editor-core/src/ids/chrome/painter.rs`.

**Verificado contra o source Blender:** defaults `smooth_stroke_radius=75`/`smooth_stroke_factor=0.9`
(`DNA_brush_types.h:228/230`), spacing 10%, dash 1.0/20, input_samples 1 — todos batem. O
`stabilized()` é byte-a-byte o `paint_smooth_stroke` (`paint_stroke.cc:575-604`).

## §2 — GAPS ABERTOS (o que o Enio reportou) — ataque nesta ordem

### 2.1 — UI não esconde parâmetros por método ✅ RESOLVIDO (2026-06-21)
- **Sintoma (Enio):** "spacing só existe para Method:Space" — mas o painel mostrava TODOS os params
  pra todo método. Spacing num brush Dots não fazia nada → **no-op silencioso (DIRETIVA §2)**.
- **O que foi feito:**
  - Engine: novo predicado `StrokeMethod::uses_spacing()` (= Space|Line|Curve, par do
    `uses_dash()`, separado por intenção). Os outros predicados (`uses_dash`/`allows_jitter`/
    `supports_smooth`) já existiam e bastaram. Lock comportamental novo:
    `stroke_panel_visibility_matches_blender` (a matriz por-método dos 7 métodos vira teste).
  - Painel ([`paint_stroke.rs::paint_stroke_section`](../crates/ph2d-panel-painter-layers/src/paint_stroke.rs)):
    cada bloco agora gateado por `StrokeMethod::from_u8(brush.stroke_method).<predicado>()`.
    Spacing+Adjust-Strength → `uses_spacing()`; Dash → `uses_dash()`; Jitter+Unit →
    `allows_jitter()`; Stabilize(+Radius/Factor) → `supports_smooth()`; Method + Input Samples
    sempre. **Valor persiste:** linha não-pintada não registra hit-rect (não clicável) mas o
    `WidgetStore` mantém o valor → trocar de método e voltar preserva (fiel ao Blender).
  - **Prova (DoD):** 3 testes paint-level novos em `paint_stroke.rs` dirigem o paint real e leem o
    `HitIndex` — `dots_hides_spacing_and_dash_keeps_jitter_samples_stabilize`,
    `space_shows_spacing_dash_and_the_rest`, `dragdot_shows_only_method_and_samples`. Provam que a
    linha SOME (sem hit-rect) por método — o que os seam tests (que injetam `WidgetEvent` direto,
    pulando o hit-test) não cobrem. Gates verdes: panel-LOC (file+fn), wiring-parity
    (`hit_indexed_ids_are_registered`), seam (6), engine stroke (18).
- **Airbrush rate — DEFERIDO de propósito (NÃO é gap aberto):** o `BrushSpec.airbrush_rate_s` existe
  e o engine `Stroke::tick` (timer) é testado, MAS o `PainterTool::on_tick` é **stub vazio**
  ([`trait_impls.rs:378`](../crates/ph2d-tool-painter/src/tool/trait_impls.rs)) — o shell chama
  `on_tick` mas o tool não dirige o `tick`. Logo Airbrush ≈ Dots hoje, e um slider "rate" controlaria
  valor morto = **exatamente o no-op que 2.1 conserta**. A UI do rate entra JUNTO com o wiring do
  timer (mesma classe interativa do 2.4). Por enquanto Airbrush mostra Method/Jitter/Samples/Stabilize
  (honesto, sem no-op). **Não adicione o slider antes de o `on_tick` dirigir o `tick`.**

### 2.2 — Drag Dot e Dots errados vs Blender 🟡
- **Sintoma (Enio):** "Dot e drag dots não funcionam como no Blender."
- **Drag Dot (o claro):** no Blender é **UM carimbo único que segue o cursor** e só commita 1 dab
  (na posição de release) — não um rastro. Hoje
  ([`stroke.rs:139-141`](../crates/ph2d-painter-brush/src/stroke.rs)) o `extend` faz `emit_single`
  por evento → **deixa um rastro de dabs** (errado). É interativo: precisa de preview (o dab segue
  durante o drag) + commit único no Up. Mora parte no tool/shell (preview), parte no engine
  (não acumular). Sugestão: no engine, Drag Dot no `extend` só atualiza a posição (sem emitir); o
  tool pinta 1 dab no `paint_end` na posição final (mirror do que Anchored vai precisar). Pressão
  forçada 1.0 (já em `forces_full_pressure`), sem jitter (`allows_jitter`=false já).
- **Dots:** no Blender é 1 dab por evento de input no cursor (rastro de dabs discretos, sem
  spacing). Hoje o `extend` faz `emit_single` no ponto **suavizado/averaged**. Confirme com o Enio
  o que está "errado" — pode ser só o param-hiding (2.1) + o fato de Dots não dever passar pela
  suavização de trajeto (ela só roda no Space, então Dots já está cru — OK). **Instrumente/peça
  repro** antes de reescrever (lição: não caçar causa errada).

### 2.3 — Stabilize "não fluido com valores baixos" ✅ RESOLVIDO (2026-06-21) — ranges do Blender

- **Sintoma (Enio, 2ª passada):** "Stabilize não funciona tão bem quanto no Blender (o traço não
  fica fluido se os valores são baixos)." → era a **pista do §3**: os ranges dos sliders.
- **Causa-raiz (verificada, não chutada):** PH2D mapeava o track `0..1` em `factor∈[0,1]` e
  `radius∈[0,200]`. Com factor→0, a mola `lerp(cursor, anchor, u)` não puxa nada (segue o cursor
  cru/jittery); com radius→0, sem dead-zone. **O Blender PROÍBE esse regime** — confirmado no
  source live `makesrna/intern/rna_brush.cc`: `smooth_stroke_radius` = `RNA_def_property_range(10,
  200)` e `smooth_stroke_factor` = `RNA_def_property_range(0.5, 0.99)` (são ranges HARD do RNA, o
  Blender nem armazena fora deles). O floor de 0.5 é por que o stabilizer do Blender sempre parece
  liso.
- **Fix (single-source no engine, clamp duplo):**
  - Engine `spec.rs`: consts `SMOOTH_RADIUS_MIN_PX=10`/`MAX_PX=200`, `SMOOTH_FACTOR_MIN=0.5`/
    `MAX=0.99` (citados ao RNA). `stroke.rs::stabilized()` clampa a eles → save/LLM não alcança o
    regime jittery. Teste `stabilize_clamps_to_blender_range_so_low_values_stay_smooth`.
  - Tool `paint.rs`: re-exporta as consts (alias `BRUSH_SMOOTH_*`); setters mapeiam track `0..1`
    → `[min,max]` (radius `10 + t·190`, factor `0.5 + t·0.49`). Teste
    `stabilize_sliders_map_to_blender_floors_not_zero`.
  - Painel `paint_stroke.rs`: o display inverte (value→track) com os mesmos mins. Readouts mostram
    o valor real (0.50..0.99 / 10..200) = o que o Blender mostra.
- **NOTA — o que NÃO foi tocado (e por quê):** o handoff original teorizava densidade-de-amostra
  (FPS/coalescing do macOS) como causa do não-contínuo GERAL. **Isso é uma questão separada** e só
  mede com GUI. O sintoma que o Enio reportou agora ("valores baixos") era o range — fechado. SE
  depois de testar ele achar não-fluido nos valores ALTOS/default, aí sim instrumente a densidade
  de amostra (§ abaixo, preservado). Não toquei no smoother quadrático nem no shell.
- **Doc stale fora da minha pasta:** [`ph2d-editor-core/.../ids/chrome/painter.rs:174,176`](../crates/ph2d-editor-core/src/ids/chrome/painter.rs)
  ainda diz "0..200 px" / "0..1 lag" — comentário, não-funcional; é editor-core (isolamento). Follow-up do owner: trocar p/ "10..200" / "0.5..0.99".

#### 2.3-bis (DEFER) — "não contínuo" GERAL, se persistir nos valores altos 🔴 (MEÇA primeiro)
- **Sintoma (Enio):** "o stabilize ainda precisa de ajustes" + "o traço não é desenhado
  continuamente como no Blender" (mesmo após a suavização que matou as facetas grosseiras).
- **Importante:** o `stabilized()` é **byte-a-byte** o `paint_smooth_stroke` do Blender e os
  defaults batem — então "igual ao Blender" JÁ está feito e **mesmo assim não satisfaz**. O alvo
  real é o FEEL (contínuo/liso), não a paridade literal. Não basta reconferir o port; trate como
  problema de densidade de amostra (abaixo) e, se preciso, AFASTE-se do Blender (ex.: stabilizer
  pull-string estilo Krita/Lazy-Nezumi, que atualiza a cada amostra sem a dead-zone que "trava" o
  pincel) — mas só depois de medir, e idealmente confirmando com o Enio se quer fiel-ao-Blender ou
  mais-liso-que-Blender (eles brigam: o algoritmo do Blender é cru e "laggy" por design).
- **Causa provável (NÃO confirmada — instrumente):** **densidade de amostra de input baixa.** O
  engine é fiel; o Blender se sente contínuo porque processa eventos de mouse em alta frequência.
  Se o shell entrega poucas amostras por segundo (FPS baixo do Painter — ver o FPS-10 histórico em
  `docs/HANDOFF_painter_falloff_curve.md` §4 — e/ou **coalescing de eventos do macOS** no winit),
  o trajeto é construído de poucos pontos → a suavização ajuda mas não é tão fluida quanto denso.
- **Onde olhar:**
  - Entrega de amostra: [`shells/desktop/src/input_dispatch/painter_canvas_input.rs`](../shells/desktop/src/input_dispatch/painter_canvas_input.rs)
    `painter_canvas_move` (1 amostra por `CursorMoved`) — confirme que winit/macOS não está
    coalescendo os `CursorMoved` (NSEvent coalescing). winit tem APIs pra desligar coalescing.
  - FPS do Painter: o composite é bandwidth-bound (memórias `project_painter_composite_perf_*`); se
    está lento, os eventos coalescem pro frame-rate.
- **Como MEDIR (faça antes de teorizar — DIRETIVA §3 / memória "meça a escala do sintoma"):**
  instrumente o nº de amostras por segundo que chegam ao `on_canvas_pointer` durante um traço
  (PH2D_UIDBG-style, revertido depois). Frame(16ms)/amostra vs 100ms/amostra muda tudo. Se for
  esparso: o fix é shell-side (desligar coalescing / entregar todas as amostras / subir o FPS), NÃO
  mais engine. Se for denso e ainda não-contínuo: aí sim revisite o engine (ex.: Catmull-Rom no
  lugar do quadrático-midpoint, ou densificar o flatten <4px).
- **Two-strikes:** já houve 1 reescrita da suavização (quadrática). Se precisar de uma 2ª, MEÇA a
  densidade de amostra e prove a causa antes de uma 3ª.

### 2.4 — Anchored / Line / Curve seguem DEFER (interativo não wirado) 🟡
- Selecionáveis no dropdown, mas **não pintam durante o drag** (engine só faz `advance_anchor`).
  `fill_segment()` existe no engine pra Line/Curve; falta o tool/shell dirigir: preview ao vivo,
  finalização no Up (Line: `fill_segment(down,up)`; constrain 45° com Alt), autoria Bézier (Curve),
  carimbo redimensionável (Anchored). É a mesma classe interativa do Drag Dot (2.2). **Hoje é no-op
  silencioso se selecionado** — idealmente esconda/desabilite no dropdown até wirar, OU wire Line
  primeiro (mais barato, engine pronto).

## §3 — Itens menores / notas

- **Ranges dos sliders são chute meu** (radius 0..200, factor 0..1, count 1..64). O `rna_brush.cc`
  do Blender (min/max da UI) **não está no checkout vendido** — não dá pra verificar. Pista: o Enio
  testou **Factor 0.2**, mas o Blender clampa factor a ~**0.5..0.99**; meu range deixa entrar em
  valores que o Blender proíbe. Provavelmente é o "valores padrão diferentes" que ele reportou. Ache
  uma fonte confiável dos ranges (ou pergunte) e alinhe em `BrushSpec`/`paint.rs` consts +
  `paint_stroke.rs` mapas value↔track. **Não chute e hardcode** (memória `no-industrial-claims`).
- **Space attenuation default ON** (fiel ao Blender `BRUSH_SPACE_ATTEN`): atenua um dab solitário
  abaixo de full opacity. Os testes de pintura do tool optam por `space_attenuation:false` pra
  asserções de cobertura total. Se o Enio achar o brush "fraco" num clique único, é isso — é
  Blender-fiel + ajustável pelo toggle "Adjust Strength".

## §4 — Como validar (DoD = seam test verde + smoke do Enio)

- Engine: `cargo test -p ph2d-painter-brush` (57; inclui spacing/dash/jitter/stabilize/
  input-samples/atenuação + `smoother_rounds_a_corner_instead_of_facets`).
- Tool: `cargo test -p ph2d-tool-painter` (72; `stroke_section_panel_events_route_to_brush_settings`).
- Painel: `cargo test -p ph2d-panel-painter-layers --test seam` (6; 3 Stroke: toggle/slider/dropdown).
- Gates: `architecture_panel_wiring_parity` + panel/workspace LOC + tool-contract-surface (editor-core).
- **Param-hiding (2.1): escreva um seam/behavioral test** que prove que a linha some por método (o
  gate `wiring_parity` é "pintou ⟹ registrado", não cobre "deveria sumir"). Compile-verde NÃO é DoD.
- **Smoke do Enio (obrigatório, é GUI):** rebuild limpo `cargo build -p ph2d-host-desktop`; New
  Canvas (2048); trocar Method e ver os params certos sumir/aparecer; Drag Dot = 1 carimbo;
  traço curvo liso e contínuo. Pergunte o **canvas/zoom** dele (2.3 depende disso).

## §5 — Cadência / CI

Fast mode: `git commit --no-verify -- <paths>` (instantâneo), `cargo check -p` no slot
`target-slots/slot-2` (`CARGO_TARGET_DIR=…/target-slots/slot-2` prefixado). **Implementador NÃO
pusha.** O Enio pediu smoke verde antes de ship. Slot warm em uso.

## §6 — Avaliação honesta (eu, agente anterior)

- Entreguei a seção inteira fiel ao Blender + testada, e matei as **facetas grosseiras** com a
  suavização. Isso é real.
- **Mas** entreguei a UI mostrando params irrelevantes por método (no-op silencioso — eu mesmo
  deveria ter pego: é exatamente o que a DIRETIVA §2 proíbe). E Drag Dot ficou como rastro (não
  pensei no caso "carimbo único interativo"). São gaps que um smoke pega na hora — entreguei sem o
  smoke do Enio.
- O "não-contínuo" (2.3) eu **não consigo medir sem GUI** — não teorize a causa; instrumente a
  densidade de amostra primeiro. Pode ser FPS/coalescing (shell), não o engine.
- Próximo passo sugerido: ~~2.1~~ ✅ → **2.2 (Drag Dot carimbo único) → medir 2.3**.

## §7 — Progresso (agente seguinte, 2026-06-21)

- **2.1 fechado** (commit local `0cdebf1d`, não pushado): gate por-método + `uses_spacing()` +
  matriz Blender testada + 3 testes paint-level (HitIndex). Airbrush-rate deferido (on_tick é
  stub — ver §2.1).
- **2.3 fechado** (commit local, não pushado): ranges do stabilizer alinhados ao Blender
  (`factor∈[0.5,0.99]`, `radius∈[10,200]`, verificados no `rna_brush.cc` live). Single-source no
  engine + clamp no `stabilized()` + track-map nos setters + display invertido no painel. +2
  testes (engine clamp, tool floor-map). Ver §2.3.
- **Ainda falta o smoke do Enio** (é GUI) p/ 2.1 e 2.3: trocar Method e ver params sumir/aparecer;
  Stabilize no fundo do slider ainda deve ficar liso (factor 0.5 / radius 10), não jittery.
- **Abertos:** 2.2 (Drag Dot carimbo único — interativo) · 2.4 (Anchored/Line/Curve + on_tick do
  airbrush) · 2.3-bis (não-contínuo GERAL, só se persistir nos valores altos — MEÇA densidade de
  amostra antes).
- **2.2 (Drag Dot) é o próximo lógico** — interativo (preview + commit único no Up), parte
  tool/shell. A metade UI-honesta já caiu com 2.1 (Drag Dot só mostra Method+Samples); falta o
  COMPORTAMENTO (não acumular rastro). Ver §2.2.

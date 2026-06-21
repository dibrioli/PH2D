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
Mas: (1) o traço não se desenha tão **contínuo** quanto no Blender; (2) **Dots/Drag Dot estão
errados** — Drag Dot deixa um rastro em vez de um carimbo único; (3) a **UI não esconde os
parâmetros por método** (Spacing aparece em Dots, onde não faz nada = no-op silencioso, viola
DIRETIVA §2); (4) Anchored/Line/Curve seguem **DEFER** (sem finalização interativa).

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

### 2.1 — UI não esconde parâmetros por método (CLEAN WIN, faça primeiro) 🟢
- **Sintoma (Enio):** "spacing só existe para Method:Space" — mas o painel mostra TODOS os params
  pra todo método. Spacing num brush Dots não faz nada → **no-op silencioso (DIRETIVA §2)**.
- **Onde:** [`paint_stroke.rs::paint_stroke_section`](../crates/ph2d-panel-painter-layers/src/paint_stroke.rs)
  pinta todas as linhas incondicionalmente (não há `match`/`if` por `brush.stroke_method`).
- **Comportamento Blender (por método mostra):**
  - **Spacing + Adjust-Strength:** só Space/Line/Curve (baseado em espaçamento) → predicado
    `uses_dash()` já existe (Space|Line|Curve) e serve; ou crie `uses_spacing()`.
  - **Dash (Ratio/Length):** só Space/Line/Curve (`uses_dash()`).
  - **Jitter + Unit:** todos MENOS Drag Dot/Anchored (`allows_jitter()`).
  - **Stabilize (+ Radius/Factor):** todos MENOS Anchored/Drag Dot/Line (`supports_smooth()`).
  - **Input Samples:** todos.
  - **Airbrush rate:** só Airbrush (campo existe em `BrushSpec.airbrush_rate_s`; **não tem UI ainda**).
- **Fix:** gate cada linha em `paint_stroke_section` pelo predicado do método (re-exporte os
  predicados de `StrokeMethod` via `ph2d_tool_painter`, ou passe flags no `BrushSettings`). Cuidado
  com a contagem de linhas / LOC do painel (paint_stroke.rs já tem ~286 LOC, cap 600). **Não basta
  esconder** — confirme com seam test que o param escondido não é dispatchável (ou que o tool ignora).

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

### 2.3 — Stabilize / traço "não contínuo como no Blender" 🔴 (mais difícil — MEÇA primeiro)
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
  smoke do Enio. **Não conte 2.1/2.2 como fechados.**
- O "não-contínuo" (2.3) eu **não consigo medir sem GUI** — não teorize a causa; instrumente a
  densidade de amostra primeiro. Pode ser FPS/coalescing (shell), não o engine.
- Próximo passo sugerido: **2.1 (esconder params) → 2.2 (Drag Dot carimbo único) → medir 2.3**.

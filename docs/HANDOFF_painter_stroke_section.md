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

A seção Stroke do Blender (Method/Spacing/Adjust-Strength/Jitter+Unit/Dash/Input-Samples) foi
portada clean-room: engine puro (`ph2d-painter-brush`), seam do tool (`ph2d-tool-painter`), e painel
UI (`ph2d-panel-painter-layers/src/paint_stroke.rs`).

> **⚠️ ESTADO ATUAL (2026-06-21, Enio é coord+impl agora — eu): suavização = UM knob "Stabilize" (0–100%).**
> Histórico: removi o Stabilize do Blender (2 knobs dead-zone+factor) e pus Catmull-Rom **fixo** →
> Enio: "o traço não pode ser suavizado [à força]; substituto do Stabilizer; ajustar o quanto fica
> regular." **Design final (§2.7):** um **slider único "Stabilize" (0–100%)** que escala JUNTOS
> (a) um filtro lazy-mouse de posição (tira tremor da mão) e (b) a tensão Catmull-Rom (curva entre
> amostras). **0% = traço CRU** (cantos retos, exato, real-time); **100% = bem regular/liso.**
> Catch-up no pointer-up (não trunca). Real-time (causal). **§2.3/§2.3-ter/§2.6 são histórico.**

Estado dos gaps: §2.1 (esconder params por método) **válido**; a regularidade é o slider único.
~~2.2 Drag Dot~~ → **✅ RESOLVIDO** (§2.2: carimbo único restore+re-stamp, segue o cursor, sem
rastro). **Abertos: 2.4 (Anchored/Line/Curve interativo + airbrush on_tick).**
**NOTA:** Enio me deu coord+impl (autoridade total: editor-core/shell/contratos sem coordenação).

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
- **Airbrush rate — ✅ RESOLVIDO (2026-06-21):** o `on_tick` agora dirige `Stroke::tick(dt)` **todo
  frame enquanto o traço está ativo** (movendo OU parado), com `dt` = `frame_ms_now` real do shell.
  Correção de fidelidade no engine: o `extend` do Airbrush **não emite mais no movimento** (era ramo
  compartilhado com Dots = errado); fiel ao Blender, airbrush deposita dabs **só no timer**
  (`paint_stroke.cc`: o ramo de motion é gateado `!AIRBRUSH`, o de TIMER `AIRBRUSH`). Logo: parado →
  acumula num ponto; varrendo rápido → spray esparso. Slider **Rate** agora VIVO (gateado por novo
  predicado `uses_rate()` = Airbrush; track `0..1` → `[0.01,1.0]`s, default 0.1, readout em segundos).
  Guard anti-stall no `tick` (cap 8 dabs/frame, dropa backlog — timer async do Blender não dispara
  retroativo). Wiring completo: engine (`extend` fix + `uses_rate` + consts `AIRBRUSH_RATE_*`); tool
  (`BrushSettings.airbrush_rate_s` + `set_brush_airbrush_rate_norm` + `paint_tick(dt)` dirige tick+settle);
  editor-core (id `PAINTER_BRUSH_RATE`); painel (linha Rate + populate + event drain). **Falta smoke
  do Enio:** segurar parado = mancha cresce; varrer rápido = pontilhado; mexer Rate muda a densidade.

### 2.2 — Drag Dot ✅ RESOLVIDO (2026-06-21) — carimbo único que segue o cursor

- **Comportamento Blender (verificado no source vendido `paint_stroke.cc`):** Drag Dot = UM dab que
  segue o cursor (raio fixo, posição = cursor), re-renderizado a cada evento por **restore + re-stamp**
  (o Blender restaura a região 2D e re-aplica o dab), commitado na soltura. **Sem suavização** (o
  Blender desliga smooth-stroke pra Drag Dot — posicionamento preciso). Pressão 1.0, sem jitter.
- **Implementação (espelha o Blender 2D, sem overlay no shell — o preview É o `canvas_rgba` vivo):**
  - Engine: o `extend` do Drag Dot usa o cursor **cru** (movi o `stabilize()` pra dentro do braço
    Space; novo predicado `uses_stabilizer()` = só Space). Emite 1 dab/evento na posição crua.
  - Tool ([`paint.rs`](../crates/ph2d-tool-painter/src/tool/paint.rs)): `stamp_drag_preview` —
    **restaura os pixels sob a posição anterior** (`save_region`/`restore_region` + `dab_bbox`) e
    re-carimba na nova → um dab segue o cursor sem rastro; `commit_drag_preview` no `paint_end`
    larga o restore-record (o dab da soltura fica). Roteado via `stamp_stroke_dabs` (Drag Dot →
    preview; resto → cumulativo). Dirty-rect cobre as duas regiões (apaga a antiga, mostra a nova).
  - Painel: o slider **Stabilize é gateado por `uses_stabilizer()`** = `!{Anchored, DragDot, Line}`
    (= o conjunto smooth-stroke do Blender: Space/Dots/Airbrush/Curve). **Drag Dot e Anchored** não
    mostram (posicionamento exato); **Dots/Airbrush MOSTRAM** (Enio pediu + Blender suporta). Matriz
    de visibilidade atualizada.
- **Testes:** engine `drag_dot_ignores_the_stabilizer_and_sits_at_the_cursor`; tool
  `drag_dot_follows_cursor_leaving_no_trail` (carimba em A→B→C, prova: pixel em C preto, A e B
  brancos = sem rastro, restore-record limpo); painel paint-level atualizado (Drag Dot esconde
  Stabilize). 63 engine / 73 tool / clippy / host compila.
- **NOTA — Dots (revisado 2026-06-21, pedido do Enio):** auditado vs `paint_stroke.cc`. Core CORRETO
  (1 dab/evento no cursor, sem spacing, input-samples + jitter + pressão como Blender). O Enio pediu
  **Stabilize pra Dots** → o Blender suporta smooth-stroke pra Dots, então liguei (`uses_stabilizer`
  agora inclui Dots/Airbrush; o engine estabiliza a posição do dab; teste `dots_use_the_stabilizer_
  like_space`). **Dots NÃO tem Dash/Length** (`uses_dash` só Space/Line/Curve) nem Spacing. Painel
  final de Dots = **Method + Jitter+Unit + Samples + Stabilize**.

#### ~~2.2-orig — Drag Dot e Dots errados vs Blender~~ (resolvido acima; texto original abaixo)
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

### 2.7 — Stabilize = UM knob de intensidade ✅ (2026-06-21) — design FINAL da suavização

- **Pedido do Enio:** "um substituto do Stabilizer. Uma forma de ajustar **o quanto o traço se torna
  regular**." (Não é opacidade — é regularidade ajustável. "Não pode ser suavizado" = não fixo.)
- **Design:** um único campo `BrushSpec.stabilizer: f32` (0..1) + slider "Stabilize" (0–100%) na
  seção Stroke (sempre visível, todos os métodos). O knob escala **dois efeitos juntos**:
  1. **Lazy-mouse de posição** ([`stroke.rs::stabilize`](../crates/ph2d-painter-brush/src/stroke.rs)):
     `stab_pos += (amostra − stab_pos) · blend`, `blend = 1 − intensidade·(1−0.08)`. Filtra o tremor
     da mão (a "regularidade" de verdade). Em 0 → `stab_pos = amostra` (sem filtro).
  2. **Tensão Catmull-Rom** (`walk_smoothed`): as tangentes do Hermite são escaladas por
     `w = intensidade`. Em `w=0` → tangentes zero → **chord reto** (cru, facetado); em `w=1` →
     curva cheia entre amostras.
  - **0% = traço cru** (cantos retos, exato, real-time — honra "não pode ser suavizado"); **100% =
    bem regular/liso**. Real-time (causal, sem lookahead) — o lag é proporcional à intensidade que o
    Enio escolhe. **Catch-up no `finish()`**: com lazy-mouse o traço fica atrás do cursor; no
    pointer-up ele caminha até o ponto de soltura real (Space) → não trunca. Default 0.5.
  - **Catch-up na PAUSA (2026-06-21, pedido do Enio):** com stabilize alto o traço só alcançava o
    cursor no mouse-up. Agora `Stroke::settle()` (engine) avança `stab_pos` rumo ao último cursor a
    cada **tick por frame** e o tool dirige via `on_tick`→`paint_tick()` (antes era stub) — **só
    quando o cursor está PARADO** (flag `moved_this_frame`, pra não enfraquecer a suavização durante
    o movimento). O shell já redesenha contínuo (`ControlFlow::Poll`+`request_redraw`/frame), então
    `on_tick` roda parado. `settle` converge com piso `SETTLE_BLEND_FLOOR=0.3` (alcança em ~⅓s mesmo
    no máximo) + snap sub-pixel. Testes `settle_catches_the_stroke_up_to_the_cursor_on_a_pause` +
    `settle_is_a_noop_without_lag`.
- **Wiring:** engine (campo + `stabilize()` + tensão + catch-up); tool (`BrushSettings.stabilizer`
  + `set_brush_stabilizer` + routing do id `PAINTER_BRUSH_STABILIZE` **reaproveitado como slider**);
  painel (slider + populate como Slider + event drain + FALLBACK). Ids dead `_RADIUS`/`_FACTOR` em
  editor-core seguem órfãos (limpeza trivial, tenho autoridade — follow-up).
- **Testes:** engine `stabilizer_zero_keeps_the_raw_path`, `stabilizer_regularizes_a_jittery_line`
  (zigzag ±6px: 95% achata pra <60% da amplitude crua), `stabilizer_catches_up_to_release_on_finish`
  (60 engine). Tool: routing no `stroke_section_*`. Painel: 3 paint-level (STABILIZE sempre visível)
  + seam `stroke_stabilizer_slider_forwards_setvalue` (7 seam). Host-desktop compila. **Falta smoke
  do Enio:** mexer o slider e ver de cru (0) a liso (100).
- **Se ainda não satisfizer:** o lazy-mouse exponencial é frame-rate-dependente; se o feel variar com
  FPS, trocar por média-móvel-ponderada (janela ∝ intensidade) — sample-count, previsível. E o
  coalescing de `CursorMoved` no macOS (shell) ainda limita a densidade em traços rápidos.

---

### ~~2.6 — Suavização: Catmull-Rom + Stabilize REMOVIDO~~ ⚠️ HISTÓRICO (o Catmull-Rom virou a tensão do §2.7)

- **Sintoma (Enio, com print):** o traço saía **facetado** — segmentos retos com quinas, não curva
  suave. **Causa:** o smoother "causal com w-damping" (§2.3-ter) colapsava o control na reta quando a
  direção mudava (pra não dar overshoot na quina); com input esparso (coalescing do macOS) isso vira
  faceta. Resolvi o problema errado (evitar overshoot) ao custo do que o Enio queria (curva lisa).
- **Ordem do Enio:** "retire a implementação do stabilize e tente outra técnica."
- **O que foi feito:**
  1. **Stabilize REMOVIDO por completo** (engine `stabilized()`+campos `smooth_*`+consts `SMOOTH_*`+
     predicado `supports_smooth`; tool campos/setters/consts/`toggle_brush_smooth_stroke`/routing;
     painel toggle+Radius+Factor+populate+event+FALLBACK; seam test repontado p/ Adjust-Strength).
  2. **`walk_smoothed` reescrito como spline Catmull-Rom** (`stroke.rs`): cada `extend` pinta o
     segmento `a→p` como Hermite cúbico com tangentes Catmull-Rom — em `a` a tangente centrada
     `(p−prev_prev)/2` (junta suave com o segmento anterior), em `p` o chord causal `p−a` (não há
     próximo ponto ainda). Resultado: **interpola POR todos os pontos** → curva suave entre amostras
     esparsas (sem faceta), **segue o cursor em tempo real** (sem cauda segurada), passa pelos pontos
     (sem o overshoot do forward-tangent). Campo `prev_tangent`→`prev_prev`; helper `hermite()`.
- **Trade-off honesto:** é Catmull-Rom **causal** (tangente em `p` = chord, sem lookahead), então há
  uma pequena descontinuidade de tangente nos PONTOS (mini-kink), imperceptível com input denso e
  muito melhor que faceta. A alternativa (Catmull-Rom com 1-ponto de lag) seria C1-perfeita mas
  voltaria a atrasar — o Enio priorizou suave+responsivo, então causal. Se ainda facetar em traços
  MUITO rápidos, a causa-raiz é densidade de amostra (coalescing macOS, shell-side — fora da pasta).
- **Testes:** engine `corner_is_rounded_smoothly`, `gentle_curve_is_rounded_not_faceted`,
  `space_paints_up_to_the_cursor_each_event` (59 engine). Retas/segmentos-curtos intactos.
- **Dead ids p/ o owner do editor-core:** `PAINTER_BRUSH_STABILIZE`/`_RADIUS`/`_FACTOR` em
  [`ids/chrome/painter.rs`](../crates/ph2d-editor-core/src/ids/chrome/painter.rs) ficaram **órfãos**
  (consts pub não-referenciadas) + docs stale. Limpeza = Coord (isolamento; não toquei editor-core).

---

### ~~2.3 — Stabilize "não fluido com valores baixos"~~ ❌ REVOGADO por §2.6 (stabilize removido)

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

### ~~2.3-ter — Stabilize "atualização não é em tempo real"~~ ❌ REVOGADO por §2.6 (smoother trocado)

- **Sintoma (Enio, 3ª passada):** "O traço suavizado parece correto, mas a atualização do traço não
  é em tempo real." → o traço **arrastava atrás do cursor**.
- **Causa-raiz (lida no código, não teorizada):** o smoother quadrático-midpoint (`walk_smoothed`,
  do commit c33e04a7) **segurava meia-aresta por design** — pintava só até `mid(prev, cur)` e
  guardava `[mid(prev,cur) → cur]` pro próximo evento / `finish()` no Up. Resultado: o fim pintado
  ficava sempre ~½ segmento atrás do cursor. Confirmado que o shell entrega 1 amostra por
  `CursorMoved` (per-event, não per-frame — `on_cursor_moved` seta `last_pointer` e entrega na
  hora) e que cada `extend` carimba na hora (`stamp_dabs`); o lag era estrutural no engine.
- **Fix — `walk_smoothed` reescrito p/ smoother causal real-time** (paint até o cursor todo evento,
  sem cauda segurada):
  - Cada segmento é uma quadrática `pen → control → cur`, com `control = pen + prev_tangent ·
    (seg·0.5·w)` e `w = max(0, prev_tangent · chord_dir)`. Curva suave (direções alinhadas) →
    `w≈1`, arredonda o join; quina dura → `w=0`, control colapsa em `pen` → reta até o cursor, **sem
    overshoot** pra fora da quina (o Blender também mantém quina dura sem arredondar). G1-contínuo.
  - **Trade-off honesto:** quinas duras agora **ficam duras** (não pré-arredondadas como antes) —
    fundamental: causal + zero-lag não consegue antecipar a quina pra arredondá-la. Curvas suaves
    seguem suavizadas (mata as facetas do c33e04a7). `finish()` virou no-op (não há cauda).
  - Campo `sp_prev: Option<StrokePoint>` → `prev_tangent: Option<[f32;2]>`; `midpoint()` removido.
  - Testes: `space_paints_up_to_the_cursor_each_event` (real-time), `sharp_corner_stays_sharp_
    without_overshoot` (sem bulge), `gentle_curve_is_rounded_not_faceted` (anti-faceta). 61 no engine.
- **Lag residual (se ainda houver) — NÃO é o smoother:** (a) o **dead-zone do Stabilizer** (≥10px)
  segura o traço até mover 10px — é o lazy-mouse fiel ao Blender, aparece SÓ com Stabilize ON, é
  esperado; (b) **coalescing de `CursorMoved` no macOS** (winit) deixa a amostra esparsa em traços
  rápidos — isso é shell-side (fora da pasta). Se o Enio quiser ainda mais responsivo com Stabilize
  ON, baixar o floor do dead-zone diverge do Blender (decisão dele).

#### 2.3-bis (DEFER) — "não contínuo" GERAL, se persistir nos valores altos 🔴 (MEÇA primeiro)
> Nota: a metade **engine** disso (lag do smoother) caiu no §2.3-ter. O que sobra aqui é só o
> shell-side (densidade de amostra / coalescing do macOS), se ainda incomodar.
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

### 2.4 — Stroke methods interativos ✅ TODOS RESOLVIDOS (+ Circle/Polygon, extensões PH2D)
> **NOTA:** "airbrush on_tick" **caiu** (§2.1). **Anchored** **caiu** (✅ abaixo). **Line** **caiu**
> (✅ abaixo). **Curve** **caiu** (✅ RESOLVIDO abaixo). Os 7 métodos do Blender estão completos
> (Dots, Airbrush, Anchored, Space, Drag Dot, Line, Curve) + **Circle** (8º) + **Polygon** (9º),
> extensões PH2D.
>
> **Polygon ✅ RESOLVIDO (2026-06-22):** segue EXATAMENTE o padrão do Circle (mesma máquina de
> shape: draw centre-out → editar handles → Enter/Esc; verbos `commit/cancel/discard_open_shape`
> cobrem os 3 shapes; preview restore+re-stamp; geometria **sem transcendentais**). `StrokeMethod::
> Polygon` = wire 8. Polígono regular **inscrito na elipse** (rx, ry, orientação `u`) com **N lados
> 3..12**. **7 handles** (vs 6 do Circle): 4 de eixo + rotação + **lados** (índice 5: arrasta ao longo
> de `+u`; a posição codifica a contagem — `rx + 3·tol + (sides-3)·1.5·tol`; inverter a projeção dá o
> N, clamp 3..12) + centro. Vértices via **rotação incremental** por `(cos,sin)` de `2π/N`
> pré-computados como const `POLY_STEP[3..=12]` (só mul/add em runtime; drift sub-`1e-6`,
> determinístico) — 1º vértice no topo (`+y`). Engine: `polygon_perimeter` + `fill_polygon_preview`
> em `stroke/polygon.rs`. Tool: `tool/paint/polygon.rs` (`PolygonEditor`, handle de lados,
> `polygon_overlay()` expõe `sides`). Shell: overlay (contorno fechado + 7 handles, lados em ciano +
> conectores centro→rotação/lados) em `painter_bridge`. Painel: dropdown + name "Polygon" + decode
> `0..9`. Testes: engine `polygon_perimeter_has_n_vertices`/`polygon_side_count_clamps`/`polygon_fills_*`;
> tool `polygon_draw_*`/`polygon_sides_handle_*`/`polygon_axis_*`/`polygon_rotate_*`/`polygon_centre_*`/
> `polygon_commit_cancel_and_undo`; painel `polygon_shows_*` + decode round-trip 9 métodos. **Falta
> smoke do Enio (GUI):** desenha; 4 handles redimensionam; rotação gira; **handle de lados** muda 3↔12;
> centro move; Enter aplica; Esc descarta.
>
> **Circle ✅ RESOLVIDO (2026-06-22):** editor de elipse on-canvas (não existe no Blender — extensão
> PH2D, `StrokeMethod::Circle` = wire 7). Fluxo: (1) **desenha** do centro pra fora (press = centro,
> drag = raio → círculo); ao soltar aparecem os handles. (2) **edita**: **4 handles de eixo**
> (dir/cima/esq/baixo, simétricos ao centro → vira elipse), **1 handle de rotação** (sai do topo),
> **centro** arrastável; contorno pintado ao vivo (restore+re-stamp). (3) **Enter** aplica (1 undo);
> **Esc** descarta. **Sem transcendentais (HR-5):** perímetro = 4 cardinais + subdivisão
> `normalize`-midpoint (só `sqrt`, IEEE-determinístico), orientação = vetor unitário `u` (nunca
> ângulo) → `sin`/`cos` ZERO. Engine: `ellipse_perimeter` (free fn compartilhada) + `fill_ellipse_
> preview` (fill espaçado contínuo do perímetro fechado), em `stroke/ellipse.rs`. Tool: `tool/paint/
> circle.rs` (`CircleEditor`, hit-test por handle, `circle_overlay()`). Verbos de forma unificados:
> `commit_open_shape`/`cancel_open_shape`/`discard_open_shape` (cobrem Curve **e** Circle) — undo
> (1º undo aplica a forma), Enter/Esc, troca de método, deactivate, source novo. Shell: overlay
> (contorno + 6 handles, grabbed destacado) em `painter_bridge`; mesmo footprint + grab-tol que o
> Curve. Painel idêntico a Line/Curve (sem Stabilize). Testes: engine `ellipse_perimeter_*`/
> `circle_fills_*`/`circle_preview_is_deterministic_*`; tool `circle_draw_*`/`circle_axis_*`/
> `circle_rotate_*`/`circle_centre_*`/`circle_commit_*`/`circle_cancel_*`/`circle_undo_*`/
> `circle_discarded_*`; painel `circle_shows_*`. **NOTA de split:** `paint_stroke.rs` (painel) passou
> de 600 LOC com o teste → testes movidos p/ `paint_stroke/tests.rs` (o gate per-file do painel conta
> o `mod tests` inline; o sibling fica sob o cap). **Follow-ups (V2):** snap de rotação (Shift = 15°),
> manter aspecto (Shift no eixo); círculo perfeito vs elipse já coberto. **Falta smoke do Enio (GUI):**
> desenha do centro; 4 handles redimensionam; rotação gira; centro move; Enter aplica; Esc descarta.
>
> **Curve ✅ RESOLVIDO (2026-06-22):** editor de pontos on-canvas **simplificado** — diverge do
> workflow de objeto-Curva do Blender (decisão do Enio: "vamos simplificar"). Fluxo: (1) **traça** uma
> linha (press-drag-release, como Line); ao soltar aparecem **3 pontos de controle** (extremos +
> centro). (2) **edita**: arrasta um ponto pra mover; clica em espaço vazio/perto da curva pra
> **adicionar** um ponto (e já arrastá-lo); **handles auto** = spline Catmull-Rom suaviza entre os
> pontos → curvas de qualquer forma; preview pintado ao vivo (restore+re-stamp, sem rastro). (3)
> **Del** apaga o ponto selecionado (mantém ≥2). (4) **Enter** comita (bake, 1 passo de undo); **Esc**
> descarta. Engine: `flatten_catmull_rom` (free fn compartilhada, reusa `hermite`+`dist`) +
> `Stroke::fill_curve_preview` (fill espaçado contínuo via `walk_space` ao longo do spine) — ambos em
> `stroke/curve.rs` (submódulo filho mantém acesso privado; split por LOC-cap). Tool: `tool/paint/curve.rs`
> (`CurveEditor` state machine: draw→edit→commit; hit-test/insert transcendental-free; `curve_overlay()`
> p/ a chrome; `set_curve_grab_tol_px` out-of-band como `set_line_constrain`). Shell: keyboard
> Enter/Esc/Del (`painter_curve_commit/cancel/delete_selected_point`, downcast ADR-0040 §3); overlay
> (spine + dots, selecionado destacado) em `painter_bridge` via o mesmo footprint AABB da entrega de
> pointer. Painel de Curve = **Method+Spacing+Adjust+Dash+Jitter+Samples** (sem Stabilize:
> `uses_stabilizer` agora **só** Space/Dots/Airbrush — Curve é point-editor, não freehand, então o
> stabilizer é no-op → escondido, DIRETIVA §2). Testes: engine `flatten_catmull_rom_*`/`curve_fills_*`/
> `curve_preview_is_deterministic_*`/`curve_fill_needs_two_points`; tool `curve_draw_creates_three_*`/
> `curve_bend_then_cancel_reverts_*`/`curve_add_delete_and_commit`/`curve_discarded_when_switching_*`/
> `curve_grab_tolerance_*`; painel `curve_shows_*`. ⚠️ **Perf:** restore+re-stamp da curva inteira =
> O(bbox)/edit; canvas grande = follow-up (overlay-guia / dirty sub-rect). **Follow-ups (V2):** drag de
> handles manual + toggle corner/smooth (hoje só auto-handles); botão "novo curve" sem precisar de
> Enter. **Falta smoke do Enio (GUI):** traça linha → 3 pontos; arrasta ponto = curva; clica = adiciona;
> Del apaga; Enter comita; Esc descarta.
>
> **Line ✅ RESOLVIDO (2026-06-22):** linha reta do press point (anchor) ao cursor, preenchida com
> dabs espaçados; **preview ao vivo** (restore+re-stamp, sem rastro) + commit no Up. Difere do Blender
> (que só faz fill no release + guia overlay) — live-preview consistente com Anchored/Drag Dot. Engine
> `extend` Line = `fill_line_preview` determinístico (snapshot/restore do dash+jitter+accum, então
> re-stampar a mesma linha é idêntico); `finish` no-op (o preview É o commit). Tool: `stamp_drag_preview`
> generalizado p/ **lista** de dabs (union bbox), `stamp_stroke_dabs` roteia DragDot/Anchored/**Line**.
> **Alt-constrain 45°** (Blender `constrain_line`): `snap_to_45` tool-side (projeção nos 8 raios,
> **sem transcendentais** — determinismo HR-5), shell forward do Alt via `set_line_constrain` (canal
> out-of-band, `CanvasPointer` é congelado). Painel de Line = **Method+Spacing+Adjust+Dash+Jitter+Samples**
> (sem Stabilize: Blender rejeita LINE em `paint_supports_smooth_stroke`). Testes: engine
> `line_fills_*`/`line_preview_is_deterministic_*`/`line_pivots_*`; tool `line_paints_a_straight_*`/
> `line_alt_constrain_*`/`snap_to_45_*`; painel `line_shows_*`. ⚠️ **Perf:** restore+re-stamp de linha
> longa = O(bbox da linha)/frame; p/ canvas grande, o overlay-guia (estilo Blender) seria o follow-up.
> **Falta smoke do Enio (GUI):** arrastar = linha segue; Alt = trava em 45°.
>
> **Anchored ✅ RESOLVIDO (2026-06-21):** carimbo único fixado no press point cujo **raio = distância
> arrastada** (sobrescreve o Size); **Edge to Edge** (toggle novo, `BRUSH_EDGE_TO_EDGE`) centra no
> ponto médio anchor→cursor com metade do raio (vai de borda a borda). Fiel ao anchored arm de
> `paint_stroke.cc` (`anchored_size = |cursor − initial|`; edge-to-edge → halfway + size/2). Reusa a
> infra de preview do Drag Dot (restore+re-stamp, sem rastro, commit no Up) — `stamp_stroke_dabs`
> roteia DragDot **e** Anchored. **Jitter e Stabilize ficam ESCONDIDOS** para Anchored: o Blender os
> mostra no painel mas são **no-op no código** (`paint_stroke_use_jitter` linha 435 e
> `paint_supports_smooth_stroke` linha 1059 rejeitam ANCHORED) → DIRETIVA §2 (não pintar no-op).
> Painel de Anchored = **Method + Edge to Edge + Input Samples**. Wiring: engine (`extend` Anchored +
> `anchored_dab` + `uses_edge_to_edge()` + campo `edge_to_edge`); tool (`BrushSettings.edge_to_edge` +
> `toggle_brush_edge_to_edge` + roteamento Click + preview); editor-core (`PAINTER_BRUSH_EDGE_TO_EDGE`);
> painel (toggle + populate + event). Testes: engine `anchored_radius_is_the_drag_distance_*` +
> `anchored_edge_to_edge_spans_*` + matriz `uses_edge_to_edge`; tool `anchored_stamps_a_drag_sized_disc_*`
> + routing; painel `anchored_shows_edge_to_edge_*`. **Falta smoke do Enio (GUI):** arrastar = disco
> cresce do anchor; Edge-to-Edge = vai de borda a borda.
- ~~(Line/Curve) Selecionáveis no dropdown, mas **não pintam durante o drag**~~ — **OBSOLETO**: Line
  (✅ 2026-06-22) e Curve (✅ 2026-06-22) agora pintam ao vivo (blocos acima). Todos os 7 stroke
  methods estão wirados e testados.

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
- **2.3-ter fechado** (commit local, não pushado): traço agora **real-time** — `walk_smoothed`
  reescrito de quadrático-midpoint (segurava ½ segmento) p/ smoother **causal** (pinta até o cursor
  todo evento, join arredondado por tangente, sem overshoot). `finish()` no-op. Trade-off: quinas
  duras ficam duras (curvas suaves seguem suaves). +3 testes. Ver §2.3-ter.
- **Ainda falta o smoke do Enio** (é GUI) p/ 2.1 e 2.3: trocar Method e ver params sumir/aparecer;
  Stabilize no fundo do slider ainda deve ficar liso (factor 0.5 / radius 10), não jittery.
- **Abertos:** 2.2 (Drag Dot carimbo único — interativo) · 2.4 (Anchored/Line/Curve + on_tick do
  airbrush) · 2.3-bis (não-contínuo GERAL, só se persistir nos valores altos — MEÇA densidade de
  amostra antes).

- **Airbrush fechado (2026-06-21, commit local pendente):** ver §2.1 RESOLVIDO. Engine: `extend` do
  Airbrush não emite no movimento (timer-only, fiel ao `paint_stroke.cc`); `uses_rate()`; guard
  anti-stall + consts `AIRBRUSH_RATE_*`. Tool: `paint_tick(dt)` dirige `tick`+`settle`; `on_tick`
  passa o dt real; `set_brush_airbrush_rate_norm`. editor-core: id `PAINTER_BRUSH_RATE`. Painel:
  linha Rate gateada. Testes: engine `airbrush_does_not_emit_on_motion_only_tracks_the_cursor` +
  matriz `uses_rate`; tool `airbrush_deposits_on_the_tick_at_the_tracked_cursor_not_on_a_bare_move`
  + routing; painel `airbrush_shows_rate_and_hides_spacing_dash`. 65 engine / 75 tool / painel
  (4 paint + 7 seam) / wiring-parity / clippy / host-desktop compila. **Falta smoke do Enio (GUI).**
- **2.2 (Drag Dot) é o próximo lógico** — interativo (preview + commit único no Up), parte
  tool/shell. A metade UI-honesta já caiu com 2.1 (Drag Dot só mostra Method+Samples); falta o
  COMPORTAMENTO (não acumular rastro). Ver §2.2.

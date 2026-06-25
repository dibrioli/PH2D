# Novo Painter — Plano de Implementação (passo a passo)

> Lê junto: [`01_arquitetura_e_decisoes.md`](01_arquitetura_e_decisoes.md) (decisões) e
> [`03_algoritmos_referencia_blender.md`](03_algoritmos_referencia_blender.md) (algoritmos a portar).
> **Cada passo de implementação:** reler [`DIRETIVA_IMPLEMENTACAO.md`](../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).
> Legenda caminho: (A) drop-crate · (B) scaffold · (C) Coord-only+ADR · (D) modificar crate.
> **DoD de fase** = seam test `ph2d-ui-testkit` verde + paridade de pixel + smoke do Enio (§9 do doc 01).

## Mapa de fases & dependências

```
Fase 0 (C) contrato+shell: ponteiro→tool   ─┐ (gargalo: tudo depende)
Fase 1 (A) engine: BrushSpec+falloff+dab   ─┤  (1 e 2 podem começar em paralelo à 0 — são puras)
Fase 2 (A) engine: stroke (spacing/dyn)    ─┘
Fase 3 (D) tool: colar engine no canvas + undo de stroke   ← precisa 0,1,2
Fase 4 (B) UI de brush settings                            ← precisa 3
Fase 5 (D/A) brushes: eraser/smudge/fill/clone + presets   ← precisa 3,4
Fase 6 (D) GPU dab (só se kill-criterion CPU disparar)     ← condicional
```

Owners sugeridos (3 slots, RAM 8 GiB): **Coord** = Fase 0/4-scaffold; **Impl-A** = `ph2d-painter-brush`
(Fases 1,2,5-engine); **Impl-B** = `ph2d-tool-painter` (Fase 3) + `ph2d-panel-painter-brush` (Fase 4-fill).

---

# FASE 0 — Entrega de ponteiro ao tool (C, Coord-only + ADR) 🔴 gargalo

**Meta:** pointer de canvas (x,y,pressão,tilt, fase) chega a um tool stateful, em coords de imagem.
**Aceitação concreta (congelada):** um Down→3×Move→Up dirigido por `ph2d-ui-testkit` entrega 5
eventos a um tool de teste, com coords de imagem corretas (des-pan/des-zoom) e pressão preservada.

### T0.1 — ADR amendment (contrato)
- Arquivo: `docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md` (amendment §) **ou** ADR novo `01xx-canvas-paint-pointer-delivery.md` que estende 0040.
- Conteúdo: justifica `Tool` 11→12 (`as_canvas_paint_mut`) + novo sub-trait `CanvasPaintTool ≤1`.
  Conjunto de aceitação + kill-criterion. Linka ADR-0099.

### T0.2 — Sub-trait + resolver no contrato congelado
- Arquivo: `ph2d-editor-core/src/tool.rs`.
- Add `struct CanvasPointer`, `enum PointerPhase`, `trait CanvasPaintTool { fn on_canvas_pointer(&mut self, ev: CanvasPointer) -> bool; }`.
- Add ao `trait Tool`: `fn as_canvas_paint_mut(&mut self) -> Option<&mut dyn CanvasPaintTool> { None }`.
- Bump cap em `ph2d-editor-core/tests/architecture_tool_contract_surface.rs`: `Tool 11→12`, novo `CanvasPaintTool ≤ 1`.
- Gate: `cargo test -p ph2d-editor-core --test architecture_tool_contract_surface` verde.

### T0.3 — Roteamento tela→imagem + pressão/tilt (shell + editor-core)
- Arquivos: `ph2d-editor-core/src/interaction/dispatch/` (canvas pointer path) + `shells/desktop/src/input_dispatch.rs` / `render_loop/` (HR-18 ≤600 LOC: arquivo novo `painter_canvas_input.rs`).
- Capturar o pointer **dentro do footprint do canvas do painter** (a view que o preview já desenha) e converter **tela→imagem** invertendo pan/zoom da view; anexar `pressure`/`tilt` do `PointerEvent` (ph2d-input, já first-class). Encaminhar a `active_tool.as_canvas_paint_mut()?.on_canvas_pointer(ev)`.
- **Todas as amostras do frame** (não coalescer a 1) — densidade importa pro stroke.
- **Zero no-op silencioso:** fora do footprint / sem tool de canvas = early-return explícito (não corpo vazio).

### T0.4 — Driver de seam headless (blindagem) + tool de teste
- Arquivo: `ph2d-ui-testkit/` → `drive_canvas_pointer(seq: &[CanvasPointer])`.
- Tool de teste mínimo que grava as amostras recebidas; o teste afirma contagem + coords + pressão.
- **DoD Fase 0:** este seam test VERDE + smoke ("Enio: no painter, arraste no canvas com o Pencil; o log mostra amostras com pressão" — instrumentação `PH2D_UIDBG`, revertida depois).

### Kill / risco
- Se a conversão tela→imagem não tiver fonte única (pan/zoom da view): **PARE** — não duplicar a matriz; achar onde o preview desenha o footprint e inverter ESSA transform (DIRETIVA §1: não inventar constante).

---

# FASE 1 — Engine: BrushSpec + falloff + dab (A, `ph2d-painter-brush`)

**Meta:** carimbar 1 dab radial no buffer RGBA8 do layer ativo, em espaço linear, via blend GPL-free.
**Aceitação concreta:** dab Normal flow=1 cor opaca sobre fundo opaco == `blend::apply(Normal,…)` **bit a bit**.

### T1.0 — `src/lib.rs` primeiro (`#![forbid(unsafe_code)]`), depois `Cargo.toml`
- Deps mínimas: `ph2d-painter-effects`, `ph2d-core`, `ph2d-color`. **Sem** editor-core/contrato.

### T1.1 — `BrushSpec` (`src/spec.rs`)
- Campos clean-room (derivar de §03, **não** copiar `DNA_brush_types.h`): `size_px: f32`, `hardness: f32 (0..1)`,
  `strength: f32 (0..1)`, `flow: f32 (0..1)`, `spacing: f32 (frac do diâmetro)`, `blend: BlendMode`,
  `falloff: Falloff`, `jitter: f32`, `dynamics: Dynamics` (Fase 2). Defaults nomeados + doc da fonte.
- Cap de alocação: tamanho do dab capado (`size_px.min(MAX_BRUSH_PX)`), constante derivada do budget, não mágica.

### T1.2 — Falloff (`src/falloff.rs`)
- `enum Falloff { Smooth, Sharp, Root, Sphere, Constant, Linear, Custom(curve) }` + `fn weight(&self, t: f32) -> f32` (t = r/R ∈ 0..1).
- Curvas exatas em §03 (porte a forma, ex.: smoothstep, `sqrt`, `1-t²` etc.); `hardness` reescala o joelho.
- Ref. visual: `blender_ui_reference/brush_falloff_*.png`.

### T1.3 — Aplicar dab (`src/dab.rs`)
- `fn stamp_dab(buf: &mut [u8], w,u32, h,u32, center:[f32;2], spec:&BrushSpec, color_linear:[f32;4], coverage:f32) -> DirtyRect`.
- Por pixel no bbox do dab: `a = falloff(dist/R) * flow * coverage`; decode sRGB8→linear (LUT `ph2d-color`);
  `Co = blend::apply(spec.blend, premul/straight conforme blend.rs, color, dst, a)`; encode linear→sRGB8.
- Respeitar `alpha_locked`/`clipping`/`mask` do layer (passar flags/áreas válidas).
- HR-3: bbox pré-calculado, sem alloc por pixel; itera slice.

### T1.4 — Testes de mesa (sem GPU/shell)
- Paridade §7 (asserção-vermelha). Falloff monotônico. Dab fora do canvas clampa. Alpha-lock não cria alpha novo.
- **DoD Fase 1:** `cargo test -p ph2d-painter-brush` verde incluindo o teste de paridade de blend.

---

# FASE 2 — Engine: motor de stroke (A, `ph2d-painter-brush`)

**Meta:** transformar uma sequência de `CanvasPointer` numa lista de dabs com spacing/pressão/smooth.
**Aceitação concreta:** reta de comprimento L com spacing s ⟹ `floor(L/(s·D)) (+1)` dabs nas posições certas (±1).

### T2.1 — Stroke state + spacing (`src/stroke.rs`)
- `struct Stroke { last_emit: [f32;2], accum_dist: f32, spec, … }`; `push(point) -> SmallVec<Dab>`.
- **Space** (Blender default): emite dab a cada `spacing·diâmetro` ao longo do segmento interpolado (ref `paint_stroke.cc` `paint_space_stroke`). Interpolar pressão/pos linearmente entre amostras.

### T2.2 — Dynamics pressão/tilt (`src/dynamics.rs`)
- `enum DynTarget { Size, Strength }`; mapeia `pressure` (curva) → multiplicador de size/strength por dab.
- Mouse sem pressão ⇒ pressure=1.0 (sem efeito). Curva de pressão = preset linear + custom (Fase 4).

### T2.3 — Smoothing / stabilizer (opcional nesta fase)
- "Smooth Stroke" (ref `brush_stroke_stroke-panel.png`): média móvel/lag do cursor (radius/factor). Pode ser DEFER nomeado se Fase 2 ficar grande — não bloqueia o stroke básico.

### T2.4 — Airbrush (time-based)
- Emissão por tempo parado: usar o heartbeat `Tool::on_tick` (já existe, ADR-0040-amend-2). Liga na Fase 3.

- **DoD Fase 2:** testes de mesa de spacing/pressão verdes (contagem+posições de dabs).

---

# FASE 3 — Colagem engine ↔ canvas + undo de stroke (D, `ph2d-tool-painter`)

**Meta:** pintar de verdade no layer ativo, com preview ao vivo e undo desfazível.
**Aceitação concreta:** seam test dirige Down→Move…→Up; `canvas_rgba` muda no bbox do stroke;
`dirty_rect` setado; undo restaura os pixels exatos; redo repinta.

### T3.1 — `impl CanvasPaintTool for PainterTool` (`src/tool/paint.rs` novo)
- `on_canvas_pointer`: Down → inicia `Stroke` + snapshot de tiles (T3.4); Move → `stroke.push()` → para cada dab `stamp_dab` em `Arc::make_mut(&mut canvas_rgba)`; acumula `dirty_rect`; `preview_dirty=true`; `bump_layer_pixels(active)`. Up → finaliza stroke + push undo.
- Guard: só pinta se layer ativo é `Raster` (senão `tracing::warn!` + UI "disabled" — DIRETIVA §2, zero no-op silencioso).
- `as_canvas_paint_mut(&mut self) -> Some(self)` em `trait_impls.rs`.

### T3.2 — Preview dirty-rect ao vivo
- Reusar `take_preview_arc`/`take_preview_upload_bbox` (`tool/runtime.rs:99/164`): o bbox do dab vira upload parcial. Stack trivial = fast-path Arc do `canvas_rgba`; multi-layer = `composite_region`.
- Seed do composite no início do stroke (contrato do `composite_region_into_canvas`, `api.rs:111`).

### T3.3 — Airbrush via `on_tick`
- Quando `phase=Hover`/parado com botão down + brush airbrush: `on_tick` emite dabs no tempo.

### T3.4 — Undo de stroke por tiles (`src/undo.rs` + `ph2d-painter-brush/src/tile_undo.rs`)
- `UndoController` ganha variante `Stroke { layer: LayerId, tiles: Vec<TileDelta{coord, before: Box<[u8]>}> }` ao lado de `Structural`.
- Tiles (ex. 128×128): no 1º toque de um tile no stroke, snapshot do "before"; ao fechar o stroke, push 1 entry com os tiles tocados. Undo restaura os tiles; redo reaplica (guardar before E after, ou re-snapshot após). **Two-strikes:** se o modelo de tile precisar 2 reconstruções, bench antes da 3ª.
- HR-14 não se aplica (não é save format), mas o `ModelSnapshot` de save deve continuar válido.

### T3.5 — Seam test comportamental (DoD)
- `ph2d-ui-testkit`: `drive_canvas_pointer` de um traço; assert pixels mudaram no bbox, undo restaura byte-a-byte, redo repinta. **Smoke Enio:** "pinte um traço, Ctrl+Z desfaz, Ctrl+Shift+Z refaz; troque de layer e pinte".

### Kill / perf
- Medir T3 em `--release`. Disparou o kill-criterion (>8ms @2048² após 2 tentativas) → agendar Fase 6 antes de Fase 5.

---

# FASE 4 — UI de Brush Settings (B, `ph2d-panel-painter-brush`)

**Meta:** controlar size/strength/flow/blend/spacing/falloff + cor pela UI canônica, ao vivo.
**Ref. de layout:** `blender_ui_reference/texture-paint_tool-settings_brush-settings_popover.png` + `brush_stroke_stroke-panel.png`.

### T4.1 — Scaffold do painel (Coord, B §3.B.1)
- `crates/ph2d-panel-painter-brush/` + feature `panel-painter-brush` em `ph2d-panel-registry-init` + `EXPECTED_TYPED++`.
- Deps padrão de painel. Stub `impl Panel` verde.

### T4.2 — Seções (Impl, espelhar Widget Gallery — SKILL §11.9 / DIRETRIZ §5.2)
- **Brush:** Radius (slider+chip, `link_slider_number`, `mark_chip_no_stepper`, storage 0..1, display "px"),
  Strength (0..1), Flow (0..1), **Blend** (dropdown → 22 modos, padrão `event.rs` explícito como grid-snap),
  **Falloff** (dropdown de presets + curva custom = widget 2D → `InteractiveState`+dispatch em editor-core, padrão BlenderHit; memória `panel 2D-drag precisa dispatch`).
- **Stroke:** Spacing (0..1), Jitter, Smooth toggle+Radius+Factor.
- **Color:** reusar `BlenderColorPicker` (não reinventar chrome — memória `UI source of truth`).

### T4.3 — Seam tool-side (em `ph2d-tool-painter`)
- `BrushUiEdit`/`apply_ui_edit` (single-source-of-truth de clamps) + `handle_panel_event` rotando NodeIds→edits.
  Snapshot publish por frame (`layers_revision`-style) para o painel espelhar o `BrushSpec` vivo.
- Cada slider que muda pixel-preview já está coberto pelo dirty-rect; size do brush muda o ring do cursor.

### T4.4 — Gates de costura + behavioral test (DoD)
- `architecture_panel_wiring_parity` (todo id pintado registrado) + seam test que **dirige** o slider de size e afirma `BrushSpec.size` mudou e o próximo dab usa o novo size. Painel interativo sem seam test = barrado por `architecture_interactive_crate_has_behavioral_test`.
- Checklist Coord (DIRETRIZ §5.2): link_slider_number ✓, mark_chip_no_stepper ✓, storage 0..1 ✓, bridge se altera pixel ✓, apply_event thin ✓.

---

# FASE 5 — Brushes adicionais + presets (D engine + D tool)

Cada brush = variação do dab/blend, **sem** novo contrato. Aceitação por brush = seam test + paridade.

- **T5.1 Eraser** — blend `Clear`/alpha-out (já em `BlendMode`); respeita alpha-lock.
- **T5.2 Smudge/Smear** — arrasta cor (lê vizinhança, reinjeta deslocado); ref `paint_image_2d.cc` smear. Cuidar custo (kill-criterion vale).
- **T5.3 Soften/Blur** — kernel pequeno por dab; reusar math de blur de `ph2d-painter-effects` onde der.
- **T5.4 Fill/Bucket** — flood-fill por tolerância no layer ativo (one-shot, pode ser sabor-1 tool à parte se preferir pill).
- **T5.5 Clone** — copia de um ponto-fonte (offset); ref `paint_image_proj.cc`/2d clone.
- **T5.6 Brush presets** — `BrushSpec` serializável; prateleira de presets (ref `brush_introduction_brush-asset-shelf.png`). Integrar com asset DB (HR-6) é follow-up.

**Fora de escopo da 1ª entrega (DEFER nomeado):** projection paint 3D (`paint_image_proj.cc`, 7k LOC),
texture brushes/stencil (`mask_panel`/`texture-slots`), PBVH sculpt-mode paint. Abrir handoff quando entrar.

---

# FASE 6 — Dab na GPU (D, condicional ao kill-criterion)

Só se a Fase 3 estourar o budget CPU (§8 do doc 01). Portar o stamp para compute e injetar via
`LayerCompositor::inject_texture` (`api.rs:179`, GPU→GPU sem readback). **Paridade obrigatória**
(DIRETIVA §4): o dab GPU bate o CPU canônico bit-a-bit (teste `#[ignore]` headless Metal, `-- --ignored`).
Plano detalhado quando/se disparar — não construir especulativamente.

---

## Ordem de execução recomendada (Coord)

1. **Coord** abre ADR (T0.1) e fecha Fase 0 inteira (gargalo) — em paralelo, **Impl-A** começa
   `ph2d-painter-brush` Fases 1–2 (puras, não esperam o ponteiro).
2. Fase 0 verde → **Impl-B** faz Fase 3 (colagem) consumindo o engine de Impl-A.
3. **Coord** scaffolda o painel (T4.1) → **Impl-B** preenche Fase 4.
4. Fase 5 em fan-out (cada brush isolável). Fase 6 só se o kill disparar.
5. Ship 1× por jornada (CLAUDE.md §3): `./scripts/ship.sh` → push → babysit. Implementadores **não** pusham.

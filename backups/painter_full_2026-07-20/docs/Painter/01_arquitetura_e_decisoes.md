# Novo Painter — Arquitetura & Decisões

> **Objetivo:** reimplementar a pintura raster (brush/stroke/dab) do PH2D, **clean-room** a
> partir do comportamento do Blender Texture Paint (referência em
> [`reference/blender-texture-paint/`](../../reference/blender-texture-paint/)), plugando no
> **host de Layers + Efeitos já existente** ([ADR-0099](../architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md)).
> Este doc fixa as decisões; o passo-a-passo está em [`02_plano_de_implementacao.md`](02_plano_de_implementacao.md);
> os algoritmos a portar em [`03_algoritmos_referencia_blender.md`](03_algoritmos_referencia_blender.md).

---

## 1. Mandato clean-room (inegociável)

O Blender é **GPL-2.0-or-later**; o PH2D é **proprietário** (`LICENSE.md`). Portar a *expressão*
(estrutura, nomes, ordem de código, trechos literais) cria obra derivada GPL = conflito.

**Regra:** portamos **comportamento/algoritmo**, escrito em Rust idiomático. Cada peça portada
carrega no doc-comment a linha:
`// Behavioral reference (clean-room, no code copied): Blender <arquivo> — <o que faz>`
e o **algoritmo** vem descrito em §03 (não lido do .cc enquanto se escreve a versão PH2D).
Isto é exatamente o que a **DIRETIVA §1** já manda ("porte o algoritmo publicado, não o código").
Não copiar: identificadores do Blender, layout de struct, constantes mágicas (achar a fonte/derivar).

---

## 2. O host que já existe (não reconstruir)

Mapa auditado (file:line). **Tudo isto é reutilizado; o brush só ESCREVE pixels no layer ativo.**

| Capacidade | Onde | Nota |
|---|---|---|
| Tool host (stateful + `RasterEditTool`) | `ph2d-tool-painter/src/tool/mod.rs:67` (struct `PainterTool`), `tool/trait_impls.rs` | `set_source`/`current_preview`/`run_full`/`deactivate` |
| Layer stack (Raster/Mask/Group/Adjustment) | `ph2d-tool-painter/src/layers/mod.rs:78` (`LayerKind`), `:87` (`Layer`), `:147` (`LayerStack`) | z-order top→bottom; cap 999; nest 8 |
| Pixels do layer **ativo** | `PainterTool.canvas_rgba: Arc<Vec<u8>>` (`tool/mod.rs`) | RGBA8 **straight**, canvas-size |
| Pixels dos layers **inativos** | `PainterTool.images: BTreeMap<LayerId, LayerImage>` | `LayerImage{w,h,rgba8}` (`compositor/mod.rs:100`) |
| Compositor CPU (referência) | `ph2d-tool-painter/src/compositor/compose.rs:8` (`composite`), `:30` (`composite_region`) | |
| Compositor GPU (22 modos) | `ph2d-render/src/layer_compositor/compositor/api.rs:10` (`composite`), `:111` (`composite_region_into_canvas`), `:179` (`inject_texture`) | dirty-rect + GPU→GPU injection |
| Blend math (GPL-free, W3C) | `ph2d-painter-effects/src/blend.rs:44` (`BlendMode`, 22), `apply(Cs,Cb,mode)` | **o brush reusa isto, não porta `rectop.cc`** |
| Adjustments | `ph2d-painter-effects/src/adjustments/` | curves/levels/HSB/blur/bloom/S-H/… |
| Undo **estrutural** (snapshot) | `ph2d-tool-painter/src/undo.rs:66` (`UndoController`), `:37` (`ModelSnapshot`) | **NÃO** cobre pixels de stroke |
| Apply/bake | `tool/trait_impls.rs:372` (`run_full`) | composita stack → sprite |
| Dirty-rect preview | `tool/runtime.rs:99` (`take_preview_arc`), `:164` (`take_preview_upload_bbox`); flags `dirty_rect`/`preview_dirty` | brush sinaliza região por aqui |
| Painel de layers + seam | `ph2d-panel-painter-layers/` (+ `tests/seam.rs`) | onde anexa a UI de brush |
| Color picker (widget) | `BlenderColorPicker` (SKILL §11.9, em editor-core) | reusar p/ cor do brush |

---

## 3. As 4 lacunas que o brush precisa preencher (e a 1 que é foundational)

Auditadas como **inexistentes** hoje:

1. **🔴 FOUNDATIONAL — entrega de ponteiro ao tool (pressão/tilt).** O contrato `Tool`
   (`ph2d-editor-core/src/tool.rs`, **congelado** ADR-0040, cap `Tool=11`) **não tem hook de
   ponteiro de canvas**. Pointer de canvas (x,y,pressão,tilt) **não chega** a um tool stateful.
   Sem isto, nenhum brush funciona. → **Fase 0, Coord-only + ADR** (§4).
2. **Motor de dab/stamp** — escrever um carimbo radial no `canvas_rgba` (não existe). → Fase 1.
3. **Motor de stroke** — spacing/jitter/smoothing/airbrush/pressão (não existe). → Fase 2.
4. **Undo por-stroke (pixels)** — só há undo estrutural; pintar não é desfazível. → Fase 3.
5. **UI de brush** — Radius/Strength/Flow/Blend/Spacing/Falloff (não existe). → Fase 4.

---

## 4. Decisão-chave: entrega de ponteiro (Fase 0, contrato)

**Problema:** o `Tool` congelado não recebe ponteiro de canvas. **Não** improvisar via
`EditorAction` (os 4 genéricos são para painel↔tool, não para o fluxo de canvas em tempo real,
de alta frequência, com pressão).

**Opções consideradas**

| Opção | Veredito |
|---|---|
| Adicionar `on_pointer_*` direto no `Tool` | ❌ infla o contrato base que TODA tool herda; 3 métodos no cap |
| Rotecallback por `EditorAction` novo variant | ❌ mexe no canal congelado; bus não é p/ stream de canvas |
| **Sub-trait `CanvasPaintTool` resolvido via `Tool::as_canvas_paint_mut()`** | ✅ **espelha exatamente `RasterEditTool`/`as_raster_edit_mut`** (precedente vivo, `tool.rs:148`) |

**Escolhido — Opção 3 (mínima superfície):**
- Novo sub-trait em `ph2d-editor-core/src/tool.rs` (foundational, Coord-only):
  ```rust
  // Behavioral reference: Blender paint_stroke.cc (apenas a NOÇÃO de stream de pontos);
  // nada de código copiado.
  pub struct CanvasPointer {
      pub pos: [f32; 2],     // coords no espaço da IMAGEM/sprite (já des-pan/des-zoom)
      pub pressure: f32,     // 0..1 (1.0 se mouse sem pressão)
      pub tilt: [f32; 2],    // -1..1
      pub phase: PointerPhase, // Down | Move | Up | Hover
  }
  pub trait CanvasPaintTool {
      fn on_canvas_pointer(&mut self, ev: CanvasPointer) -> bool; // true = consumiu
  }
  ```
- **UMA** adição ao contrato congelado `Tool`: `fn as_canvas_paint_mut(&mut self) -> Option<&mut dyn CanvasPaintTool> { None }`
  → cap `Tool` **11 → 12**; cap novo `CanvasPaintTool ≤ 1`. Exige **amendment de ADR-0040**
  + bump em `architecture_tool_contract_surface.rs`. (Precedente: ADR-0040-amendment-2 já
  subiu `Tool` 10→11 para `on_tick`.)
- **Shell + editor-core (Coord-only):** rotear o ponteiro de canvas já capturado para
  `active_tool.as_canvas_paint_mut()`, convertendo **tela→imagem** (inverter pan/zoom da view do
  canvas do painter) e anexando **pressão/tilt** (já disponíveis no FFI
  `ph2d_event_pointer(x,y,pressure,tilt_x,tilt_y,…)`, SKILL §13). Coalescer por frame, mas
  **sem perder amostras** (stroke precisa da densidade — usar todas as amostras do frame).
- **Seam headless (blindagem):** `ph2d-ui-testkit` ganha um driver `drive_canvas_pointer(seq)`
  para dirigir Down→Move…→Up com pressão e afirmar efeito observável. **É o entregável da Fase 0**,
  não o veredito (DIRETIVA §3/§5).

---

## 5. Layout de crates (onde cada peça vive)

```
crates/
  ph2d-painter-brush/        ★ NOVA — engine pura GPL-free (sem UI, sem contrato)
    src/spec.rs              BrushSpec (size/hardness/strength/flow/spacing/blend/falloff/jitter/dynamics)
    src/falloff.rs           máscara de dab: f(r/R) presets + curva custom  (ref: curve_mask.cc)
    src/dab.rs               aplicar 1 dab no buffer RGBA8 linear via ph2d-painter-effects::blend
    src/stroke.rs            spacing/airbrush/smooth/pressão → lista de dabs (ref: paint_stroke.cc)
    src/dynamics.rs          mapeamento pressão/tilt → size/strength (curvas)
    src/tile_undo.rs         (Fase 3) snapshot de tiles dirty para undo de stroke
    deps: ph2d-painter-effects, ph2d-core, ph2d-color (transfer sRGB↔linear)  ⟂ editor-core, ⟂ contrato

  ph2d-tool-painter/         MODIFICAR (caminho D) — host: cola o engine no canvas
    src/tool/paint.rs        ★ novo: impl CanvasPaintTool; estado de stroke vivo; escreve canvas_rgba; dirty_rect
    src/tool/trait_impls.rs  + as_canvas_paint_mut() => Some(self)
    src/undo.rs              + variante Stroke{layer, tiles} no UndoController (Fase 3)
    deps: + ph2d-painter-brush

  ph2d-panel-painter-brush/  ★ NOVA (caminho B scaffold) — UI de brush settings (Fase 4)
    (espelha ph2d-panel-padding/seam.rs onde for forwarder; dropdown/falloff = event.rs explícito)

  ph2d-editor-core/          Coord-only (Fase 0) — sub-trait CanvasPaintTool + resolver + cap bump
  shells/desktop/            Coord-only (Fase 0) — roteamento tela→imagem + pressão/tilt → tool
```

**Por que `ph2d-painter-brush` separada e não dentro de `ph2d-tool-painter`?** Isolamento +
testabilidade (engine pura → testes de mesa sem GPU/shell), e re-uso futuro (brush-along-path,
vector ink). Mesma família-satélite de `ph2d-painter-effects`. **Não** começa com `ph2d-tool-`
(não confunde o `ph2d-tool-sync`) nem com `ph2d-node-` (não confunde `node-sync`) — vide memória
`node-sync glob prefix`.

---

## 6. Triagem por peça (DIRETRIZ §2)

| Peça | Caminho | Toca contrato? | Razão |
|---|---|---|---|
| Fase 0 — `CanvasPaintTool` + resolver + shell routing | **(C) Coord-only + ADR** | **SIM** — `Tool` 11→12 (amendment ADR-0040) | foundational; serializa; não paraleliza |
| `ph2d-painter-brush` (engine pura) | **(A) drop-crate satélite** | Não | crate nova isolada; testes de mesa |
| `ph2d-tool-painter` (cola engine↔canvas) | **(D) modificar crate existente** | Não | edita só a pasta do tool |
| `ph2d-panel-painter-brush` (UI) | **(B) scaffold central** | Não (usa `PanelEvent` genérico) | painel novo = Coord plumba registry antes |

Sequência obrigatória: **0 → (1,2 no tool) → 3 → 4**. Fase 0 é gargalo (tudo depende do ponteiro).
Engine pura (`ph2d-painter-brush` spec/falloff/dab/stroke) pode ser escrita **em paralelo** à Fase 0
(não depende do ponteiro — são funções puras), e só a *colagem* no tool espera a Fase 0.

---

## 7. Corretude de espaço de cor (DIRETIVA §4 — não fecha sem paridade)

- `canvas_rgba` é RGBA8 **straight, sRGB-encoded**. O compositor decodifica sRGB→linear, blenda
  em linear, re-encoda. **O dab deve fazer o mesmo:** decode sRGB8→linear (LUT canônica
  `ph2d-color`, a mesma já bit-idêntica CPU↔GPU pelo gate `srgb_lut_matches_cpu_transfer`),
  blend em linear via `ph2d-painter-effects::blend::apply`, encode linear→sRGB8.
- **Asserção-vermelha de paridade (Fase 1):** um dab "Normal", flow=1, cor opaca sobre fundo
  opaco ⟹ resultado **bit-idêntico** a `blend::apply(Normal, Cs, Cb)`. Brush blend modes ⊆ os 22
  de `ph2d-painter-effects` (não inventar math nova). Sem este teste, não fecha.
- **Alpha-lock / clipping / mask** do layer ativo (já no modelo, `layers/mod.rs:87`) devem ser
  respeitados pelo dab (pintar só onde alpha>0 quando `alpha_locked`; clipar ao raster de baixo).

---

## 8. Orçamento, kill-criteria e two-strikes (DIRETIVA §5)

- **Budget alvo:** render do canvas no orçamento de "Editor UI overlay" (HR-4, ~1 ms) + dab fora
  do hot-path de render (no `on_canvas_pointer`, fora do `editor_layout`). HR-3: dab usa buffer
  pré-alocado por stroke (sem alloc por amostra).
- **Kill-criterion CPU-dab (decidir ANTES da Fase 6):** se um stroke de pressão cheia não
  sustentar a interação em **2048²** após a **2ª** tentativa de otimização CPU (mediana de
  aplicação de dab-batch > **8 ms/frame**), o dab **migra para GPU compute** (path `inject_texture`,
  `api.rs:179`) antes de adicionar qualquer tipo de brush novo. Mede em `--release` (dev=opt0 mente —
  memória `painter composite perf`).
- **Two-strikes de topologia:** bateu na 2ª reconstrução do modelo de stroke/undo → PARE e prove
  o modelo com bench antes da 3ª.

## 9. Definition-of-Done por fase (DIRETIVA §5)

Cada fase só fecha com: **(a)** teste comportamental de seam VERDE em `ph2d-ui-testkit` (evento
real → efeito observável), **(b)** paridade numérica onde há pixel (§7), **(c)** smoke do Enio
nomeado. Compile-verde/gate-verde = velocidade, **não** "pronto" (DIRETIVA §3). DEFER nomeia a
capacidade faltante + abre handoff; não conta como fechamento.

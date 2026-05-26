# ADR-0048 — Stroke Inspector retroativo (W14)

**Status:** Proposed (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, sessão Painter W0).
**Pré-requisitos:** [ADR-0043 — Painter contract](0043-painter-contract.md), [ADR-0044 — Brush Engine GPU](0044-brush-engine-gpu.md), [ADR-0046 — Stroke Vector History](0046-stroke-vector-history.md), [ADR-0047 — MCP Stroke Engine](0047-painter-mcp-stroke-engine.md).
**Spec normativa:** [`docs/Painter_projeto/01_brush_engine.md`](../../Painter_projeto/01_brush_engine.md) §1.14.5.
**Tags:** painter, wave-0, contract, stroke-inspector, retroactive-edit, compositor-slice

---

## 1. Contexto

Procreate é destrutivo no eixo temporal: depois que o stroke é commitado na layer texture, não há "modificá-lo retroativamente sem perder strokes posteriores". PH2D Painter, por ter [Stroke Vector History full](0046-stroke-vector-history.md), pode oferecer **edição retroativa**:

- Selecionar 50 strokes feitos com `pencil_2b` há 20 minutos.
- Trocar brush para `ink_studio_pen` retroativamente.
- Compositor re-renderiza apenas a slice afetada (não o stack inteiro).
- Strokes posteriores ficam intactos (Procreate não consegue fazer isso).

Sem contrato congelado:

1. **"Selection model" vira ad-hoc.** Lasso temporal pode confundir "stroke parcialmente dentro" com "stroke inteiro dentro".
2. **Compositor re-render-slice protocol vira por-feature.** "Trocar brush" e "Trocar color" reimplementam invalidação separadamente.
3. **Performance budget é wishful.** Sem cap "selecionar 100/10k strokes ≤ 100ms", a feature degrada silenciosamente em projetos longos.
4. **Sobreposição com MCP `painter_modify_stroke` (ADR-0047)** fica ambígua. Quem é fonte de verdade quando ambos modificam o mesmo stroke?

ADR-0043 §2.5 e ADR-0046 §6 cederam território a esta ADR.

---

## 2. Decisão

### 2.1 Crate `ph2d-panel-painter-inspector` (W14)

```
crates/ph2d-panel-painter-inspector/
  Cargo.toml         # deps: ph2d-editor-core (Panel trait), ph2d-painter-stroke,
                     #        ph2d-painter-brush, ph2d-color, ph2d-tokens
  src/lib.rs         # #![forbid(unsafe_code)] PRIMEIRO
  src/panel.rs       # impl Panel<InspectorState> (ADR-0029 trait-driven panel host)
  src/state.rs       # InspectorState + InspectorSelection + InspectorAction
  src/lasso.rs       # Lasso temporal selection — geometric matching
  src/recompose.rs   # Compositor slice invalidation protocol
  tests/             # selection + perf gates + UI invariants
```

**Convivência:** o **panel** vive em crate próprio (ADR-0029 padrão `Panel<State>`); a **lógica de slice recompose** poderia viver no compositor real (TBD em W3/W4 — `ph2d-painter-brush::adjustments` ou crate futuro `ph2d-painter-canvas`). Esta ADR fixa **API + caps**; localização final do recompose é decisão de implementação W4 (ADR-0045 já tem o compositor em `ph2d-painter-brush`; coerente que recompose-slice viva no mesmo lugar).

### 2.2 `InspectorState` — cap **≤ 12 fields**

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorState {
    pub active_canvas: CanvasId,
    pub selection: InspectorSelection,
    pub overlay_visible: bool,               // mostra path drawn como overlay no canvas
    pub overlay_numbers: bool,               // numera pontos do path para debug visual
    pub current_action: Option<InspectorAction>, // qual edit retroativo está sendo previewed
    pub preview_mode: PreviewMode,           // Live | OnRelease
    pub last_query: Option<StrokeFilter>,    // filter usado no último query (cache)
    pub query_result_count: usize,           // tamanho do resultado (para UX feedback)
    pub locked: bool,                        // flag de "edit em progresso", trava nova selection
    pub version: u32,                        // HR-14 — v1 = 1
    // === 2 slots de headroom ===
}
```

### 2.3 `InspectorSelection` — cap **≤ 8 variants** (v1 usa 5)

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum InspectorSelection {
    /// Nada selecionado (estado inicial).
    None,
    /// Single stroke (clicado direto).
    Single(StrokeId),
    /// Múltiplos strokes via lasso freeform (geometric match — §2.4).
    Lasso { strokes: Vec<StrokeId>, polygon: Vec<[f32; 2]> },
    /// Range temporal (`from_seq..=to_seq` no history).
    SeqRange { from: u64, to: u64, strokes: Vec<StrokeId> },
    /// Filtrado via `StrokeFilter` (paridade com MCP query — ADR-0047).
    Filtered { filter: StrokeFilter, strokes: Vec<StrokeId> },
    // === 3 slots de headroom (e.g., LayerStrokes, BrushMatch) ===
}
```

`strokes: Vec<StrokeId>` é **cached resolution** — recomputado quando history muda (delete/insert/recompose). Polygon / filter / seq-range são a **especificação** da seleção (persistente entre sessions).

### 2.4 Lasso temporal — geometric matching (definição rigorosa)

**Critério de inclusão:** stroke S é selecionado pelo polígono P se **qualquer ponto** de S (em coords canvas) cai dentro de P, **AND** seq(S) está dentro da janela temporal aberta (`seq ≤ current_seq`, i.e., nunca seleciona strokes "do futuro" do undo).

```rust
fn stroke_in_lasso(stroke: &StrokeRecord, polygon: &[[f32; 2]], current_seq: u64) -> bool {
    if stroke.seq > current_seq { return false; }
    stroke.points.iter().any(|p| {
        let pos_f32 = [q1616_to_f32(p.x_q1616), q1616_to_f32(p.y_q1616)];
        point_in_polygon(pos_f32, polygon)  // ray-casting algorithm, O(|polygon|)
    })
}
```

**Otimização:**
- **Bbox pre-pass:** primeiro filtra strokes cujo `bbox` (cached em `StrokeRecord`) intersecta polygon bbox. Reduz N strokes a ~O(active strokes), depois O(|points|) por stroke ativo.
- **Spatial index:** R-tree de stroke bboxes mantida lazy em `StrokeHistory`. Reconstruída em background quando >10% strokes mudaram. Não bloqueia query.

### 2.5 `InspectorAction` — cap **≤ 10 variants** (v1 usa 6)

Espelha (mas não duplica) `StrokeMods` de ADR-0047. Compartilha **semântica**; UI tem variants extras para affordances específicas:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum InspectorAction {
    /// Apenas mostra overlay; sem mudança no canvas.
    Preview,
    /// Troca brush. Aplica em todas strokes da selection.
    ChangeBrush(BrushHandle),
    /// Troca color (primary).
    ChangeColor(OklchColor),
    /// Escala pressure por fator. `1.0` = identity.
    ScalePressure(f32),
    /// Delete strokes (cronológico-independente — só remove os da selection).
    Delete,
    /// Re-ordena strokes da selection movendo `delta` posições no `seq` order.
    /// Útil para "mover stroke pra cima/baixo no stack temporal".
    Reorder { delta: i64 },
    // === 4 slots de headroom (e.g., ChangePressureCurve, ChangeBrushParams, …) ===
}
```

**`InspectorAction → StrokeMods` mapping** (quando inspector chama ADR-0047 internamente):

| `InspectorAction` | `StrokeMods` |
|---|---|
| `Preview` | n/a (não chama mod) |
| `ChangeBrush(b)` | `StrokeMods { new_brush: Some(b), .. }` |
| `ChangeColor(c)` | `StrokeMods { new_color: Some(c), .. }` |
| `ScalePressure(f)` | `StrokeMods { pressure_scale: Some(f), .. }` |
| `Delete` | (special — não via ADR-0047; calls history mutator) |
| `Reorder` | (special — não via ADR-0047; calls history.reorder) |

**Inspector é cliente legítimo da API MCP.** Quando usuária clica "Apply ChangeBrush" no inspector, internamente é chamada `painter_modify_stroke` × N (com confirmation_token issued automaticamente pelo flow UI — usuária consentiu via click). Audit log registra origem `agent_id = "inspector_ui"`.

### 2.6 Compositor re-render-slice protocol

**Problema:** trocar brush em stroke seq=100 requer recompose de:
1. Layer texture **antes** de seq=100 (recuperar via snapshot mais próximo + replay).
2. Stroke seq=100 com novo brush.
3. Todos os strokes seq=101..current na **mesma layer** (precisam ser reaplicados sobre o novo background).

Custo naive: O(N strokes-since-100). Para current=10000, N=9900 = ~3 minutos GPU.

**Strategy — `RenderSlice`:**

```rust
pub struct RenderSlice {
    pub layer_id: LayerId,
    pub from_seq: u64,                       // primeiro stroke a re-aplicar
    pub strokes_to_replay: Vec<StrokeId>,    // ordenados por seq
    pub base_snapshot: Option<SnapshotId>,   // se Some, load + skip strokes < snapshot.seq
}

impl RenderSlice {
    /// Calcula o slice mínimo. Usa snapshot mais próximo de `from_seq`.
    pub fn compute_min(history: &StrokeHistory, snapshots: &SnapshotIndex, mod_seq: u64) -> Self;
}
```

**Custos com snapshot a cada 50 strokes (ADR-0046 §2.6):**
- Mod em stroke seq=100, current=10000: load snapshot @ seq=50 (≤ 50 strokes back) + replay 50 + replay 9900 posteriores = 9950 strokes (vs 9900). **Não melhora** — snapshot só ajuda undo (chronological).
- **Insight crítico:** snapshot before-N + replay all não otimiza retroactive edit. Otimização real é **per-stroke composite invalidation**: cache compositional textures **por stroke** (não por seq). Strokes que não tocam pixels modified pelo stroke-mudado **não precisam re-aplicar**.

**Decisão estrutural:**
- v1 (W14): **sem cache per-stroke**. Replay completo de strokes-since-mod na layer. Performance gate ≤ 100 ms para 100 mudados em 10k history (vide §2.8).
- v2 (futuro, **fora desta ADR**): dirty rect propagation — apenas strokes cujo bbox intersecta o bbox do stroke modificado precisam re-aplicar. Estimativa 10-50× speedup em projetos típicos.

### 2.7 Coordenação com MCP (ADR-0047)

**Fonte de verdade:** `StrokeHistory` (ADR-0046). Inspector e MCP são **dois clients** do mesmo storage.

| Conflito | Resolução |
|---|---|
| Inspector está editando + MCP `painter_modify_stroke` chega na mesma stroke | **MCP wins** (servidor é serializado por canvas; inspector preview perde, refresh automático). Audit log registra ambos eventos. |
| Inspector inicia edição enquanto MCP está middle-of-batch | Inspector **bloqueia** novo `InspectorAction` até batch terminar (`InspectorState.locked = true`). UI mostra spinner. |
| Concurrent reads (`query_strokes` MCP + inspector lasso) | Sem lock — reads são `BTreeMap` immutable snapshot por call. |

**Auditoria:** todas mudanças retroativas (inspector OU MCP) entram no mesmo `audit.log`. `agent_id` discrimina (`"inspector_ui"` vs `"claude-..."`).

### 2.8 Performance gates

| Gate | Spec |
|---|---|
| `inspector_lasso_100_of_10k_under_100ms` | Lasso polygon selection. 10k strokes total, 100 matching. Wall-clock ≤ 100 ms p99 incluindo overlay render. |
| `inspector_change_brush_recompose_4k_canvas` | 100 strokes mudados em layer 4K. Recompose ≤ 500 ms p99. Soft em W14; hard em W15+. |
| `inspector_spatial_index_rebuild_lazy` | Rebuild de R-tree não bloqueia UI (background thread). Verifica via thread blocking detector. |
| `inspector_concurrent_with_mcp_modify` | Test runtime: MCP modify + Inspector preview na mesma stroke. MCP wins; inspector refreshes. Sem panic. |

### 2.9 UI overlay invariants

Quando `overlay_visible = true`:

1. **Path visualization:** linha conectando os pontos do stroke selecionado, cor `ColorToken::AccentFocus.resolve(theme)` (ADR-0042 typed color), thickness 2 px (token `StrokeToken::OverlayThin`).
2. **Numbers:** se `overlay_numbers = true`, índice numérico por ponto, font 11 px JetBrains Mono (HR-12 a11y via `Role::Image` + `description`).
3. **No interaction with canvas underneath:** overlay é **passivo** — clicks passam through (canvas tools recebem normalmente). Inspector seleciona via lasso no próprio inspector panel, NÃO no canvas.
4. **Auto-hide:** quando `current_action = None` por 5s, overlay esmaece (fade-out 200ms).

### 2.10 Arch-gate `painter_contract_surface::inspector`

```rust
mod inspector {
    #[test] fn inspector_state_field_count_is_capped()      { /* ≤ 12 */ }
    #[test] fn inspector_selection_variant_count_is_capped() { /* ≤ 8 */ }
    #[test] fn inspector_action_variant_count_is_capped()   { /* ≤ 10 */ }
    #[test] fn render_slice_field_count_is_capped()         { /* ≤ 6 */ }
    #[test] fn inspector_action_to_stroke_mods_mapping()    { /* total: cada InspectorAction tem mapping documentado */ }
}
```

---

## 3. Consequências

### Positivas

- **Edição retroativa estável.** Lasso geométrico + R-tree spatial index = O(log N) prática. UI responsiva mesmo em 10k strokes.
- **Inspector é cliente legítimo da MCP API.** Sem duplicação de "modify stroke" lógica — Inspector orquestra; ADR-0047 executa.
- **Fonte de verdade única (`StrokeHistory`).** Não há "inspector storage" paralelo divergente.
- **Recompose slice protocol documentado.** v1 (replay completo) ship pragmático; v2 (dirty rect propagation) registrado como future-work com escopo claro.
- **Conflito Inspector vs MCP resolvido (`MCP wins`).** Determinístico, sem race conditions.

### Negativas / Custos

- **v1 sem cache per-stroke = perf gate apertado.** 100 strokes mudados em 10k = ~500ms recompose @ 4K. Para projetos extremos (50k strokes), perf degrada. Mitigação: gate soft em W14 (warning) escala para hard em W15+ quando v2 dirty rect cache ship.
- **Lasso point-in-polygon é O(|polygon| × |stroke points|).** Em polígonos complexos (200+ vertices) + strokes longos (500 points), ~100k ops/stroke. Mitigação: bbox pre-pass cap ops a O(active strokes) tipicamente <100 = aceitável.
- **R-tree spatial index custa memória** (~12 bytes/stroke × 10k = 120 KB). Aceito vs ganho de O(log N) queries.
- **MCP wins em conflito = Inspector preview pode "desaparecer" no meio da edição.** UI responde com mensagem clara ("Stroke modified externally; selection refreshed"). Aceito vs alternativa "deadlock entre inspector + MCP".

### Neutras

- **Inspector é panel docado (não modal).** ADR-0029 trait-driven panel host canon. Sem chrome novo.
- **Inspector NÃO é overlay no canvas.** UI lasso é no painel inspector, não desenho-no-canvas (decisão UX: separation evita confusão "where do I click?").

---

## 4. Alternativas consideradas

### 4.1 Cache per-stroke já em v1

**Rejeitada.** ~5 KB/stroke × 10k = 50 MB cache só de composite intermediários. Memory budget aperta. Mitigação v2 dirty-rect-propagation tem ganho similar com 1/10 memória. Adiar para W15+ é racional.

### 4.2 Inspector com storage próprio (snapshot da history)

**Rejeitada.** Bifurcação de fonte de verdade. MCP modifica `StrokeHistory`; inspector lê snapshot stale; UX confusa. Single source canon.

### 4.3 Lasso captura strokes "parcialmente dentro" diferente de "inteiramente dentro"

**Rejeitada.** UX subtler — usuária quer "qualquer ponto que clica entrou" (decisão Procreate-style). "Inteiramente dentro" = lasso seletivo demais; "≥50% dentro" = arbitrary cap. "Qualquer ponto dentro" = robusto + descobrível.

### 4.4 Inspector é overlay no canvas (sem panel docado)

**Rejeitada.** UX confunde "lasso = seleção" com "lasso = path do brush". Panel docado tem lasso interno (mini-canvas read-only com painted strokes), separa contextos.

### 4.5 Inspector edits **bypassam** MCP (chama history mutator direto)

**Rejeitada.** Audit log lacuna; HR-11 violation. Inspector é UI client privilegiada que **opta por** issue-token-internamente (ações via UI = humano consentiu via click). Mas chama API canon, não bypass.

---

## 5. Verificação

```sh
cargo test -p ph2d-panel-painter-inspector
# 5 caps + 4 perf gates.

cargo test -p ph2d-panel-painter-inspector --test inspector_concurrent_with_mcp_modify
# Concurrency test: MCP modify + inspector preview na mesma stroke.

cargo test -p ph2d-painter-brush --test architecture_painter_contract_surface
# Caps cumulativos (ADRs 0043+0044+0045+0046+0047+0048).
```

### 5.1 Definição de "Accepted"

Esta ADR transita `Proposed → Accepted` no mesmo evento T0.9.

---

## 6. Tracking

- Plano operacional: [§15 do plano §3 (T0.6) + §15 (W14)](../../Painter_projeto/15_plano_de_implementacao.md).
- Spec normativa: [`01_brush_engine.md §1.14.5`](../../Painter_projeto/01_brush_engine.md) + [`§1.14.7`](../../Painter_projeto/01_brush_engine.md) (compat MCP).
- v2 dirty-rect propagation (futuro): registrar follow-up no plano §follow-ups quando W14 fechar.
- Próxima ADR na cascata W0: [ADR-0049 — Fluid Brushes Extension](0049-fluid-brushes.md) (T0.7, última).

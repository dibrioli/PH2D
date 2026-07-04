# 02 — Design & arquitetura

## 1. Princípio: um kernel, N geradores de campo (anti-redundância)

Tudo é **inverse displacement warp** (`01` §3):

```
warp(layer, region, D):
  before = save_region(region)               // snapshot pristino da região
  for dst in region:
      src = dst − D(dst)                      // deslocamento inverso
      out[dst] = bilinear_sample(before, src) // gather, sem buracos
  # freeze: mistura com o pristino por cobertura de seleção
  if freeze_active:
      out[dst] = lerp(out[dst], before[dst], selection_coverage_at(dst))
  restore_region(region, out); mark_dirty(region)
```

**`D` (o campo de deslocamento) é a ÚNICA coisa que muda entre modos.** Isso elimina a
duplicação Transform-vs-Liquify do Procreate: são o mesmo kernel com `D` diferente.

| Família | `D(p)` |
|---|---|
| **Push** | vetor do traço (`p_now − p_prev`) com falloff radial no raio do brush |
| **Twist** | rotação `θ·falloff(r)` em torno do centro do dab (θ = ±const·pressão; HR-5: sin/cos **gated**) |
| **Pinch/Punch** | `±k·falloff(r)·(p−c)` (bipolar: negativo suga, positivo infla) |
| **Wrinkle** (Crystals) | Pinch/Punch + ruído determinístico (splitmix64, como o `jitter.rs` do brush) |
| **Fold** (Edge) | Pinch projetado numa **linha** do traço, não num ponto |
| **Reconstruct** | `D → 0` progressivo: reamostra o buffer **original** (pré-deform) por falloff |
| **Transform afim** (Uniform/Free/Distort) | `D(p) = p − M⁻¹·p` de uma matriz 3×3 (afim/homografia) |
| **Warp mesh** | `D` interpolado de uma grade de pontos de controle (bilinear/Coons por célula) |
| **Puppet/MLS** | `D(p) = p − f_MLS(p)` (Schaefer 2006, rígido) dos handles de pino |

## 2. Arquitetura de integração (ancorada no código vivo)

- **1 botão de rail** `Deform` → `PanelEvent::SelectOption(PAINTER_PAINT_MODE, "deform")`
  → `PaintMode::Deform` (novo). Canal genérico, **zero mudança de contrato**.
  Rail vive em `ph2d-editor-core` (foundational, Coord-only).
- **1 sub-tool no painter**: roteado pelo ladder de
  [`canvas_pointer.rs`](../../crates/ph2d-tool-painter/src/tool/paint/canvas_pointer.rs)
  para um novo `warp_pointer(ev)`. Reusa `CanvasPaintTool::on_canvas_pointer` (cap=1,
  **sem** mudança de trait).
- **Buffer**: escreve em `PainterTool.canvas_rgba` (`Arc::make_mut`), lê o pristino via
  `save_region` ([`region.rs`](../../crates/ph2d-tool-painter/src/tool/paint/region.rs)).
- **Undo**: padrão estrutural — `before = snapshot_model()` no `Down`, `commit_structural_edit(before)`
  no `Up` (idêntico a `paint_begin`/`close_stroke` e `selection_down`/`selection_up`).
- **Preview**: já drena por frame via `take_preview_arc` + dirty-rect
  ([`painter_bridge.rs`](../../shells/desktop/src/render_loop/painter_bridge.rs)) — herdado, **sem bridge novo**.
- **Freeze**: reusa `selection_mask` + `selection_coverage_at` + `restore_deselected_region`
  ([`selection.rs`](../../crates/ph2d-tool-painter/src/tool/paint/selection.rs)).
- **UI**: seção mode-exclusiva no inspector `ph2d-panel-painter-layers`, dirigida por flag
  `is_deform` no snapshot `BrushSettings` (padrão idêntico ao `is_selection`).

## 3. Estrutura de informação (IA) — uma ferramenta, dois "temperamentos"

A ferramenta **Deform** tem duas famílias, escolhidas por um segmented no topo do painel:

```
Deform
├─ Reshape (brush-driven)   ← Wave 1  ·  paradigma canvas-pointer puro
│   Push · Twist · Pinch/Punch(bipolar) · Wrinkle · Fold · Reconstruct
└─ Transform (gizmo-driven) ← Waves 2-3 · exige handles interativos (editor-core)
    Uniform · Free · Distort · Warp(mesh) · Puppet(MLS pins)
```

**Por que separar em waves e não em duas ferramentas:** Reshape encaixa 100% no paradigma
existente (pointer → dab → warp → Apply/undo) e mora só no painter. Transform precisa de
**gizmo de handles interativos** (InteractiveState/BlenderHit em `editor-core`) — foundational,
Coord-only, mais caro. Uma ferramenta só mantém a UX unificada; as waves isolam o custo.

## 4. O que nos torna **superiores ao Procreate** (checklist concreto)

1. **Freeze/Protect mask** durante Liquify — Procreate mobile não tem; nós reusamos Selection (feathered, por-texel).
2. **Não-destrutivo re-editável** até Apply (+ Apply & Keep + Amount fade global) — Procreate Liquify é destrutivo-imediato.
3. **Reconstruct** (paridade) + **Amount** pós-stroke.
4. **Puppet/MLS pins** (Wave 3) — Procreate não tem deformação por pinos rígidos.
5. **Warp presets + Mesh editável** (Wave 2).
6. **Warps paramétricos como NÓS** (Wave 4) — polar/spherize/ripple/displace animáveis num grafo; Procreate é incapaz.
7. **Symmetry** aplicada ao Liquify (o painter já tem simetria) — espelhamento de deformação.
8. **UX unificada, tokenizada, com cards** — vs a divisão tool/adjustment do Procreate.

## 5. Decisões e trade-offs
- **Kernel CPU-residente primeiro** (o canvas do painter já é CPU-residente). Migração
  GPU-residente é follow-up (mesmo caminho da nota `project_painter_composite_perf`),
  gated por perf (§kill-criteria em `04`).
- **Bipolar Pinch/Punch = 1 modo, slider central** (não 2 modos) — reduz a superfície de UI.
- **Reconstruct guarda o buffer pré-deform** da sessão de deform (não o pristino por-dab)
  — igual ao Reconstruct do Procreate.
- **Distortion/Momentum** ocultos em Reconstruct (não se aplicam) → sem no-op silencioso
  (DIRETIVA §2: caminho fora de escopo mostra "desabilitado", nunca corpo vazio).
- **HR-5:** Twist e warps radiais usam sin/cos → **gated** (feature `transcendental`), como o brush.

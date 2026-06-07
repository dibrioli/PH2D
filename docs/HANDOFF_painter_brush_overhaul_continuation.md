# HANDOFF — Painter Brush Engine overhaul, CONTINUATION (next implementer, START HERE)

> Você assume a missão do [`HANDOFF_painter_brush_overhaul_impl.md`](HANDOFF_painter_brush_overhaul_impl.md)
> (o brief original — leia o §0/§1/§2/§5 dele). Este doc é o **estado pós-sessão** + o que falta.
> **Você atua como Coordenador sozinho** (sem risco de colisão — o Enio autorizou). Leia `CLAUDE.md`
> inteiro primeiro. Processo: pesquise o gold-standard ANTES de codar; verifique cada efeito contra o
> caminho de render AO VIVO; smoke só no fim. O predecessor anterior alucinou algoritmos — **não invente.**

## §0 — A MISSÃO (recap)

Tornar o motor de pincéis do Painter **o melhor de qualquer app — superior ao Procreate**, fundamentado
em **física real** (aquarela, óleo, carvão, tinta, lápis) + melhores algoritmos publicados. Padrão-ouro
vence cronograma; gaps in-scope fecham na sessão.

## §1 — O QUE JÁ ESTÁ FEITO (esta sessão — tudo VERDE, commits LOCAIS pendentes)

Tudo testado + visualmente confirmado. **Nada foi commitado nem pushado** — o Enio testa manual antes
(fast mode). Decisões registradas em **ADR-0077 D1–D11** (`docs/architecture/decisions/0077-brush-engine-physics-overhaul.md`).

| Bloco | O que | Arquivos-chave | ADR |
|---|---|---|---|
| **Sub-pixel** | Dab amostrado do centro fracionário real (mata serrilhado diagonal). CPU (2 loops) + WGSL `cs_stamp`; `aa = SHAPE_AA_PX/size_px` (não footprint); gate textual `cpu_shader_rotation_pipeline_textual_parity` atualizado + naga | `cpu_render/mod.rs`, `shader/stamp.wgsl`, `cpu_render/tests.rs` | D8 |
| **Paper tooth** | Tooth world-space pressure-aware, default **OFF** (0.0; era 0.4 → "grão quando opção off"). Slider **"Paper"** no Brush Studio | `cpu_render/mod.rs` (`paper_tooth_factor`/`paper_tooth_height`), `params.rs`, panel-brush-studio | D7 |
| **v1.5 Watercolor dry-down** | Pen-up `apply_wash_settle` reescrito: **K-M thickness rim** (substitui gamma `c^e`), **granulação** mass-conserving (vales→masstone, cristas→backdrop), **outward bleed** domain-warp (tendrils, K-M glaze) | **`cpu_render/settle.rs`** (NOVO), `lifecycle.rs` (call site) | D9 |
| **Velocity dynamics** | Liga campos dormentes `dynamics.speed_{size,opacity,spacing}` (bipolar ±1) | `stamp_scheduler/advance.rs` + `mod.rs` | D10 |
| **One-Euro motion filter** | Liga campos dormentes `stabilization.motion_filtering_{amount,expression}` (Casiez 2012, substitui jitter rejection) | `stamp_scheduler/advance.rs` (`one_euro`) | D10 |
| **UI sliders** | 5 sliders no Brush Studio (Motion Filt/Expr + Speed→Size/Opac/Space, bipolar). LOC-decompose: `sections.rs` → data-tables; **`painter_bridge_queries.rs`** (NOVO, split do `painter_bridge.rs` 629→590 por HR-18) | `chrome.rs`, panel-brush-studio (`ids/populate/event/sections`), tool-painter (`params/lifecycle/trait_impls`), shell | D10 |
| **v2 Watercolor diffusion CORE** | **`ph2d_painter_brush::diffusion`** — solver gated diffusion-advection low-res (Curtis 1997 sem momentum; determinístico, CFL `D·dt≤0.2`). `DiffusionGrid` (water+pigment+paper), `step()`, `splat()` | **`diffusion.rs`** (NOVO, 456 LOC) | D11 |
| **W15.1 — `Tool::on_tick`** | Contrato CONGELADO `Tool` cap **10→11** (default no-op → 8 tools satélite inalteradas). Shell chama `active_tool.on_tick(frame_ms)` 1×/frame | `editor-core/src/tool.rs`, gate `architecture_tool_contract_surface.rs`, **ADR-0040-amendment-2**, **CLAUDE.md §6** (`Tool=11`), `shells/desktop/src/render_loop/mod.rs` | D11 |
| **W15.2 — live CPU diffusion** | `PainterTool.wet_field: Option<DiffusionGrid>`; alloc em `begin_stroke` se `fluid_enabled`; dabs **splat no grid** (não no canvas) em `queue_pointer`; `on_tick` step+composite+seca. `fluid_enabled` default false → zero mudança | `tool/mod.rs`, `tool/lifecycle.rs` (`on_tick_diffusion`/`tick_wet_field`/`composite_wet_field`), `tool/trait_impls.rs` | D11 |

**Arquivos NOVOS** (untracked): `cpu_render/settle.rs`, `diffusion.rs`, `panel-brush-studio/src/sections.rs`,
`shells/desktop/src/render_loop/painter_bridge_queries.rs`. (sections.rs aparece como `??` por já existir
no working-tree — confira; pode ser que precise `git add` explícito.)

**Contagem de testes (verde):** `ph2d-painter-brush` 326 · `ph2d-tool-painter` 201 · `ph2d-editor-core` 626 ·
gate de contrato 3 (cap 11) · clippy limpo · desktop shell compila · 4 arch-gates passam
(`file_loc_caps`, `architecture_panel_loc_cap`, `node_id_collisions`, `no_downcast_to_concrete_tool_in_shell`).

## §1.5 — SESSÃO 2 (2026-06-07) — Fluid testável + bug headline consertado

Tudo VERDE, commits LOCAIS pendentes (fast mode — Enio testa antes de push). ADR-0077 **D12**.

| O que | Arquivos | Status |
|---|---|---|
| **Toggle "Fluid" no Brush Studio** (§2.B item 1 — o Enio queria testar) | `chrome.rs` (`PAINTER_STUDIO_FLUID`), panel-brush-studio (`ids/event/populate/sections`), tool-painter (`params` BrushParam::Fluid + snapshot field / `lifecycle` set+snapshot / `trait_impls` map+read) | ✅ |
| **BUG HEADLINE consertado:** o bloom pós-pen-up estava MORTO (`end_stroke` consome `pending_pre_stroke` → `composite_wet_field` sem backdrop → canvas congela). Smoke `_live` não pegava (3 bandas byte-idênticas, **sem assert**). Fix: campo dedicado **`wet_backdrop`** que sobrevive ao stroke | `tool/mod.rs` (campo+default), `tool/lifecycle.rs` (begin_stroke captura / composite usa / tick dropa) | ✅ |
| **Safety undo/redo/set_source** (§2.B item "drop wet_field") — dropam `wet_field`+`wet_backdrop` | `lifecycle.rs` (undo/redo), `trait_impls.rs` (set_source) | ✅ |
| **K-M no composite** (§2.B item): `composite_wet_field` trocou linear "over" → `pigment_mix::mix_prepared` (Kubelka-Munk subtractivo). Wash glazeado sobre cor mistura subtractivo (amarelo sobre azul → verde, medido `[81,138,67]`). Solve spectral fica scoped aos pixels molhados pelo short-circuit `dens<1e-4` existente | `tool/lifecycle.rs` (`composite_wet_field`) | ✅ |
| **Edge quality** (report do Enio "baixa resolução nas bordas"): borda blocky era upsample **bilinear** do campo low-res. Fix: **bicúbico Catmull-Rom** (`sample_pigment_bicubic`) + **`WET_FIELD_SCALE` 4→2** (metade do bloco) + **composite por bbox** (`wet_pigment_bbox`, union com frame anterior) + **`prepare_pigment` amortizado 1×/composite** (stroke é mono-cor). Perf medida (release, 1024², worst-case): 46ms → **12.8ms**/frame | `tool/lifecycle.rs`, `tool/mod.rs` (campo `wet_composite_bbox`) | ✅ |
| **Dark borders em camada transparente** (report Enio "bordas escuras / alpha errado"): composite misturava pigmento sobre o RGB do backdrop; camada nova = `(0,0,0,0)` → bordas (alpha parcial) mistura p/ PRETO → franja escura. Fix: **glaze straight-alpha** — lerp entre porter-duff "over" (cor do pigmento na borda) e K-M, pelo alpha DO backdrop. Borda `[60,0,0,124]`→`[255,179,140,18]` (coral limpo); K-M sobre opaco intacto | `tool/lifecycle.rs` (`composite_wet_field`) | ✅ |
| **Coverage por cor errada** (report Enio: amarelo/magenta opacos cobrem tudo; azul/vermelho ok): `alpha=1-exp(-dens·K)` com `dens=Σ(pigmento linear)` = luminance-weighted (amarelo Σ≈1.4 vs azul ≈0.53) → cor clara satura opaco. Fix: normaliza por Σcor do stroke → `amount` independente de cor, `K=1.06` ancorado no azul/vermelho. Mesma carga → mesma opacidade (centro α=93 p/ todas). `tool/lifecycle.rs` | ✅ |
| **Testes/smokes guardando:** `fluid_coverage_is_color_independent` + `visual_smoke_fluid_color_swatches` (gated) + `fluid_no_dark_fringe_on_transparent_layer` + `visual_smoke_fluid_on_transparent_layer` (gated) + `fluid_wash_keeps_blooming_after_pen_up` + `fluid_toggle_via_brush_studio_checkbox` + `fluid_wet_field_dropped_on_undo_and_set_source` + `fluid_composite_mixes_subtractively_km` + smoke gated `visual_smoke_fluid_edge_quality` | `tool/tests.rs` | ✅ |

Verde: `ph2d-tool-painter` **210** · `ph2d-panel-brush-studio` 7 · gates `architecture_panel_loc_cap`/`architecture_tool_contract_surface` (cap 11)/`node_id_collisions`/`architecture_painter_contract_surface` (81) · clippy limpo · shell compila.

**Falta de §2.B (não feito):** cross-stroke wet-on-wet (exige grid K/S p/ misturar cor subtractivo). **§2.C:** granulação 2-octave, end-taper. **§2.A (W15.3 GPU) intacto** — perf de canvas grande (≥2048²) é limitada pelo `step()` full-grid (não pelo composite, que é bbox), → resolve no port GPU / active-region stepping.

## §2 — O QUE FALTA (priorizado)

### A. **W15.3 — GPU + tiers + det-fallback** (o item grande, ADR-0049 `ph2d-painter-fluid`)
O composite full-canvas em CPU (`composite_wet_field`) é o gargalo a 4K. Plano:
1. Crate `crates/ph2d-painter-fluid/` (ADR-0049 §2.1, **feature `fluid`**): WGSL compute solver (mirror do
   `diffusion.rs` — gated diffusion-advection), `FluidSim`/`FluidParams`/`GravitySource` structs (caps no ADR-0049).
2. `fluid_capable()` gating (device tier ≥ Mid, VRAM ≥ 32MB) → senão **fallback** pro caminho CPU de hoje
   (que já existe + é a referência).
3. Composite GPU + **upload por bbox** (não full-canvas). `Stamp.wet_amount` (dormente, =0) + `FLAG_FLUID_SAMPLE`
   (bit 7, reservado) são os hooks per-stamp.
4. **Det-fallback** CPU 256² pro replay HR-5 (GPU é não-det; ADR-0049 §2.11).
**Pesquisa já feita** (use): o relatório de algoritmo está sintetizado no ADR-0077 D11 + no
`diffusion.rs` docstring (Curtis 1997 + Van Laerhoven CAVW 2005 + MoXi; CFL `D·dt≤0.25`; low-res+upsample).

### B. Refinamentos da v2 live (CPU, baratos, alto valor visual)
- **Toggle "Fluid" no Brush Studio** (clone do toggle Pigment/Accumulate → `brush.rendering.fluid_enabled`).
  Hoje só dá pra testar `fluid_enabled` via código. **Trivial** — provavelmente o 1º passo (o Enio quer testar).
- **Cross-stroke wet-on-wet:** hoje `begin_stroke` RESETA o `wet_field` (v1). Manter o grid vivo entre strokes
  (pintar no molhado de um stroke anterior) = o efeito Fresco completo. Cuidado com undo/backdrop.
- **Composite por bbox** (não full-canvas) — computar o bbox de pigmento do grid → região do canvas; corta custo CPU.
- **K-M no composite** (hoje é "over" density→alpha simples; o `pigment_mix.rs` faz K-M real).
- **Undo/set_source** devem dropar `wet_field` (edge case não tratado na v1).

### C. v1.5 follow-ups (do ADR-0077 D9)
- Granulação 2-octave (clumping mais grosso); constantes de granulação **per-pigmento** (ultramar granula, ftalo não).
- (D5) **end-taper** via re-render no pen-up.

## §3 — COMO BUILDAR / TESTAR / SMOKE (importante — leia)

- **Slot de build warm (CoW):** prefixe TODO cargo com
  `CARGO_TARGET_DIR=/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/target-slots/slot-brushoverhaul`
  (o slot já existe desta sessão). NUNCA use o `target/` default. Inner loop = `cargo check/test -p <crate>`.
- **Crates do painter:** `ph2d-painter-brush` (engine + `diffusion`), `ph2d-tool-painter` (tool/lifecycle/runtime),
  `ph2d-panel-brush-studio` (painel), `ph2d-editor-core` (contrato `Tool` + chrome ids), `ph2d-host-desktop` (shell tick).
- **Smoke visual** (gated por env, dumpa PPM→converte PNG):
  ```
  PAINTER_VISUAL_SMOKE=1 CARGO_TARGET_DIR=…/slot-brushoverhaul \
    cargo test -p ph2d-tool-painter --lib <nome_do_smoke> -- --exact tool::tests::<nome> --nocapture
  sips -s format png /tmp/painter_smoke_<x>.ppm --out /tmp/painter_smoke_<x>.png   # daí Read a PNG
  ```
  Smokes existentes: `visual_smoke_watercolor_v15` (`/tmp/painter_smoke_watercolor.png`),
  `visual_smoke_velocity_and_smoothing` (`_velocity`), `visual_smoke_watercolor_v2_diffusion` (`_diffusion`, grid puro),
  `visual_smoke_watercolor_v2_live` (`_live`, ponta-a-ponta pela tool). `visual_smoke_paper_tooth` (`_paper`).
- **Gates a rodar antes de mexer em contrato/UI:** `architecture_tool_contract_surface` (cap `Tool=11`),
  `architecture_panel_loc_cap` (≤600 LOC/arquivo de panel), `file_loc_caps` (shells), `node_id_collisions`,
  `no_downcast_to_concrete_tool_in_shell`. Rode `./scripts/ship.sh` no fechamento do módulo (paridade CI).
- **rustfmt:** use `rustfmt --edition 2024 <arquivos>` (NUNCA `cargo fmt -p` — reformata WIP alheio; aqui você
  está sozinho mas o hábito vale). LOC-gate de panel/shell conta `lines().count()` (raw, incl. comments).

## §4 — GOTCHAS / LIÇÕES desta sessão (economize tempo)

- **K-M rim é LIMITADO pelo masstone** (não passa de R∞). Logo edge darkening só aparece em wash transparente
  (`opacity<1` / cobertura cai) — o caso físico real. Testes de rim com `opacity=1.0` foram migrados pra 0.5.
- **Granulação dryness/bloom:** use **MAX** água (não mean) pra decidir "secou" — o grid é quase todo papel seco,
  a média lê "seco" instantâneo. (`WET_DRY_THRESHOLD=0.045` ≈ gate `w_lo`.)
- **Lift de crista (granulação) NÃO é `km_deposit`:** depositar pigmento branco é no-op (b≈0). Lighten = lerp
  saturante rumo ao backdrop (remover pigmento). Vide `settle.rs`.
- **Borrows no `queue_pointer` fluido:** `stamps` empresta `self.scheduler`; `self.wet_field` é campo disjunto →
  splat inline coexiste. Mas um MÉTODO `self.x(stamps)` conflita (empresta self inteiro) — faça inline ou drope
  o borrow de `stamps` antes (NLL libera após o último uso).
- **`current_preview()`** retorna `&self.canvas_rgba` direto pra stack trivial (gate em `preview_dirty`). O composite
  fluido escreve `canvas_rgba` — força `preview_dirty=true` antes de ler em teste.
- **One-Euro com `expression>0` PRESERVA jitter rápido** (de propósito — é o ponto). Pra demo de "suaviza", use
  `expression=0`. `MF_MIN_CUTOFF=0.22` (baixo, mas o β adaptativo recupera traços rápidos).
- **Flip-invariance gate** relaxado pra ±1/255 (subpixel + footprint par → uv levemente assimétrico; correto).
- **`oklab_to_linear_srgb`** agora é `pub` em `cpu_render/mod.rs` (o splat da difusão usa a MESMA conversão).
- **LOC HR-18 = 600/arquivo** (raw lines). `sections.rs` (panel) e `painter_bridge.rs` (shell) estavam no limite —
  decompostos (data-tables / split de arquivo). Adicionar linha cega estoura; decomponha, não infle.

## §5 — CONTRATOS CONGELADOS tocados (CLAUDE.md §6 — Coord-only + ADR)

- **`Tool=10→11`** (ADR-0040-amendment-2): `on_tick` heartbeat. Gate `architecture_tool_contract_surface` + CLAUDE.md §6 atualizados.
- Painter caps (`Stamp=96B`, `Brush≤168`, `RenderingMode=6`, etc.) **intactos** — toda a v1.5/v2 ride em campos
  existentes (`paper_grain` em params, `wet_field` em PainterTool, `fluid_enabled`/`wet_amount` já reservados ADR-0049).
- ADR-0049 (`ph2d-painter-fluid`, W15) é o destino da W15.3 — o caminho CPU de hoje é a referência + fallback.

## §6 — Como o Enio decidiu o caminho

Watercolor v2 tinha 3 opções (pen-up / live-via-queue_pointer / full-live-W15). **Enio escolheu full live W15**
(on_tick + GPU). Ordem de engenharia: core (D11) → W15.1 contrato → W15.2 CPU live → **W15.3 GPU** (próximo).

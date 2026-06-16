> ⚠️ **SUPERSEDED por [ADR-0096](architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md) (Enio 2026-06-14):** toda a simulação de aquarela/fluido/wash foi **REMOVIDA** do código (crate `ph2d-painter-wash` deletada, canvas voltou a CPU-residente). Doc mantido só como histórico. Norte atual = **Brush Engine (mixer-brush)**, ver [`docs/Novo Painter/`](Novo%20Painter/). Backups em `backups/wash_2026-06-14`.

# HANDOFF — Watercolor v2: perf landed + simplification/stabilization (continuar aqui)

> Continuação de [`HANDOFF_watercolor_v2_refactor.md`](HANDOFF_watercolor_v2_refactor.md) (o mandato GPU-first).
> Esta sessão: **perf-block resolvido** (R0-R2 + cortes) + **C2 robustez** + a **rede de invariantes** (Fase 2 passo 1).
> ADR vigente: [`0085-watercolor-v2-gpu-first-realtime.md`](architecture/decisions/0085-watercolor-v2-gpu-first-realtime.md) (Accepted).

> **✅ FASE 2 PASSO 2 FECHADA (sessão 2026-06-11, local — sem push).** Commits:
> `9f898c0d` (test-side: deletada a paridade bit-a-bit; 4 gates GPU-only estruturais movidos pro
> `physical_invariants.rs`; `gpu_parity.rs` deletado; `composite_parity.rs`→8 gates GPU-only;
> twin-lock fora; `step_cpu_reference`/`step_grid` deletados), `2c848545` (source-side: **aposentado o
> fallback CPU** — `wet_field` só aloca com GPU (`fluid_hires`); sem GPU = aquarela OFF→wash normal;
> `diffusion.rs` 2808→536 LOC, twin CPU + test-mod + campos órfãos deletados; `composite_wet_field_cpu`
> fora; 10 testes CPU/visual deletados, 4 GPU-path reworkados, +1 teste de degradação), `bc84b86e`
> (clippy). **Verde:** brush 332 + tool 208 + 16/16 GPU gates no Metal; brush/fluid/tool/shell check+clippy
> zero-warning. ⚠️ **Enio: validar visualmente a degradação sem-GPU (aquarela OFF) no app antes do push.**
> **PRÓXIMO:** Fase 1 (C1 equilíbrio da água — precisa olho do Enio, §3) + Fase 3 (decomposição HR-18, §5).

---

## §0 — Estado / commits (tudo local, SEM push)

| commit | conteúdo |
|---|---|
| `cef8e9ef` | **Todo o perf:** R1 single-submit, composite bicúbico fast-path (scale=1), **campo SoA/planar** (7 shaders + transpose Rust), `ss=1` em scale=1, substeps pintando 2→1, **active-region window** (região segue o pincel), MacCormack off por default. 18 arquivos. **32/32 paridade Metal verde** (na época). |
| `77be3ae5` | C2 (no-panic no bake síncrono) + cobertura do dab no low-res (raio ≥1.5 células em scale>1) + knob `PH2D_FLUID_SHARPNESS`. |
| `1ad831c4` | **Rede de invariantes** `tests/physical_invariants.rs` (INV-1/2/4/6, **4/4 verde no Metal**). |

**Resultado de perf medido (Immediate):** tela cheia pintando ~15-22fps → **traço contido 67fps**, **parado 500-600fps**, look intacto. Composite barato agora (`comp_tex` 2.8ms). O resíduo da "tela toda ATIVA simultânea" é genuíno O(células ativas) — ver §3 (C1) + Fase 1.

---

## §1 — Os 6 problemas reais (investigação, ranqueados). A MATEMÁTICA está sólida; a fragilidade é a máquina-de-estados do bridge.

| # | problema | arquivo:linha | sev |
|---|---|---|---|
| **C1** | **Keep Wet não tem equilíbrio físico** — o `∇w` na borda molhada/seca injeta velocidade pra fora pra sempre (evaporação 0); `drag` só amortece momento, não a força. Mascarado por `KEEP_WET_SETTLE_FRAMES=180` (freeze de timeout). **É o defeito "não-profissional" e a raiz do creep.** | `diffusion.rs:855-913` (`add_forces`) + `shallow.wgsl:106-141` (`cs_add_forces`) | **ALTA** |
| **C2** ✅ | panic em falha de map GPU no bake síncrono → crash mid-stroke | `composite.rs:654` — **CORRIGIDO em `77be3ae5`** (os outros `expect("mapped")` são test-only, devem falhar alto) | — |
| **C3** | doc/lógica diz "água é mirror CPU" mas é GPU-residente → desync silencioso ao tirar paridade. `fluid_dry_check_and_drop` (CPU) leria ~0 e dropparia campo molhado | `painter_fluid_bridge.rs:13-17` (doc stale); `lifecycle.rs:741` (CPU dry-check) | MÉD |
| **C4** | undo correto depende de **1 pass síncrono** atrás da torre de catch-up (`texture_mode_dirty`/`catchup_bands`/"2 bandas consecutivas") que o §10 do refactor diz "nunca engatou"; `begin_stroke` clona `canvas_rgba` (undo pode reverter múltiplos traços) | `painter_fluid_bridge.rs:629-641`, `painter_fluid_support.rs:160-229` | MÉD |
| **C5** | leak transitório do preview-slot: `ensure_preview_slot` cria slot canvas-res; em falha de cópia não libera (inconsistente com `copy_preview_into_slot` que libera). Liberado só no teardown | `painter_fluid_bridge.rs:585-589`, `support.rs:98-115` | BAIXA |
| **C6** | quando `dabs.is_empty()` + capilar ativo, a região cai pro `dab_region` = `wet_pigment_envelope` **monotônico** (nunca recua) → sob Keep Wet cresce pra tela toda | `lifecycle.rs:960-963` | BAIXA |

**Limpos na auditoria (sem bug):** divisões nos shaders (NaN/div-zero guardados — `glaze_sample` `mass<1e-4`, `refl_to_ks` clamp, `capillary` denom = lado maior, `splat` `1/r` com `r.max(0.5)`); o fix "crash water/water" (drenar `pending` antes de re-mapear) sobrevive. **A fragilidade NÃO é a física — é o frame-lifecycle do bridge.**

---

## §2 — FASE 2 (encolher: o grande ganho de manutenção). Passo 1 ✅ feito; passo 2 é o próximo.

### 2.1 — Rede de invariantes ✅ (commit `1ad831c4`)
`crates/ph2d-painter-fluid/tests/physical_invariants.rs` — 4 gates GPU-only, tolerância frouxa, **sem referência CPU**: INV-1 (conservação de massa), INV-2 (água finita+limitada+sem runaway), INV-4 (azul⊗amarelo→verde), INV-6 (deposição cresce + seca). Todos `#[ignore]` (precisam de device). Rodar: `cargo test -p ph2d-painter-fluid --features fluid --test physical_invariants -- --ignored --nocapture`.

### 2.2 — Deletar a paridade bit-a-bit (passo 2, PRÓXIMO). Inventário do investigador:
- **`tests/gpu_parity.rs` (19 testes):** 15 são CPU↔GPU `|Δ|` — **deletar**. **PRESERVAR 4 GPU-only** (mover pro `physical_invariants.rs`, NÃO perder): `gpu_combine_equals_flowing_plus_deposited` (L459, total=flowing+deposited), `region_scoped_step_matches_full_grid_inside_region` (L514, guarda o `SOLVER_REGION_PAD`), `gpu_solver_conserves_then_dries` (L652, ≈ INV-1/2 já coberto — opcional), `gpu_backdrop_lift_off_is_byte_identical_composite` (L1696, lift=0 = no-op). Helpers a copiar: `seeded_grid` (L46), `try_headless_gpu` (já no invariants).
- **`tests/composite_parity.rs` (13 testes):** deletar os 5 CPU-mirror (`gpu_composite_matches_cpu_reference`, `gpu_step_then_composite_resident_matches_cpu`, `gpu_preview_texture_matches_cpu_premultiply`, `preview_texture_initialized_to_backdrop`, `wet_sheen_matches_cpu_reference`) + os 6 helpers CPU-mirror (`premultiply_rgba8_local`, `srgb_to_linear_f`, `linear_to_srgb_f`, `smoothstep_f`, `water_bilinear_cpu`, `sheen_px_cpu`). **MANTER 8 GPU-only** (`composite_rows_matches_full_band`, `composite_frame_fast_path_matches_one_shot`, `composite_frame_pipelined_matches_sync`, `gpu_composite_km_signature_and_no_fringe`, `composite_lift_reveals_paper_not_transparency`, `wet_sheen_off_is_byte_identical`, `gpu_straight_texture_matches_out_buf_bytes`, + a sonda green-dominant).
- **`tests/contract_surface.rs`:** deletar `cpu_reference_matches_diffusion_with_default_params` (o twin-lock); MANTER os field-count/naga.
- **`crates/ph2d-painter-fluid/src/params.rs:74-100`:** reescrever 5 doc-comments que falam "bit-exact for the parity gate".
- **CI-neutro:** esses testes são `--ignored`, nunca rodam no `ship.sh`/CI. Deletar não afeta CI.
- **NÃO TOCAR:** `crates/ph2d-vector-fill/tests/diffusion_gpu_parity.rs` (é o solver WoS do Vector Module, ADR-0056..0068, fora de escopo).

### 2.3 — Deletar o twin CPU morto (`diffusion.rs` 2808 → ~700 LOC). Mapa do investigador:
- **DEAD (deletar, ~2050 LOC):** `step` (598-651), `diffuse` (656-692), `advect` (699-768), `advect_maccormack` (780-826), `move_water` (835), `add_forces` (855-913), `project` (923-981), `transfer_pigment` (990-1010), `lift_pigment` (1020-1050), `lift_from_backdrop` (1054-1079), `capillary_flow` (1139-1238), `#[cfg(test)] mod tests` (1417-2808, **1392 LOC**), + campos/accessors órfãos do grid (`deposited`/`vel_*`/`pressure`/`scratch*`/`divergence`/`lift_source`/`lifted_frac` + `set_velocity_from`/`set_pigment_from`/`set_water_from`/`evaporate`/`max_water`/`water_bbox`/`total_*`/`velocity`/`lifted_frac`/`lift_source`).
- **LOAD-BEARING (manter):** consts `PIG_*`/`WetCell`/`DiffusionParams`/`RELAX_ITERS`/`WATER_EPS`/`WET_BBOX_WATER_THRESHOLD`/`CAPILLARY_MIN_SATURATION` (sync de literal com shaders), `backdrop_to_lift_source` (bridge L317), `cell_from_color_mass` + `PIG_STAIN` (DabGpu), e o `DiffusionGrid` slim como container paper+water+pigment (`new`/`generate_paper`/`with_paper`/`dims`/`paper`/`water`/`pigment`/`splat`/`cell_color`/`cell_mass`).
- **PRÉ-REQUISITO (bloqueia a deleção do `step`):** aposentar o **fallback CPU** em `lifecycle.rs` — `tick_wet_field` (L352, chama `grid.step`), `composite_and_settle_fluid`/`composite_wet_field` (CPU composite, L368/783), o branch CPU de `on_tick_diffusion` (L342-348). ADR-0085 graceful-degrade = **aquarela OFF em device sem GPU**, não sim CPU. Isso é uma **mudança de comportamento** (decisão Coord, já ratificada no ADR-0085 §2.2). `wet_composite.rs::composite_wet_field_cpu` (chamado em `lifecycle.rs:822`) sai junto; `cell_from_color_mass`/`cell_color` ficam (DabGpu/GPU usam).
- **Ordem:** deletar testes de paridade (2.2) PRIMEIRO → aí `step_cpu_reference`/`step_grid` (em `solver.rs:219/2250`) ficam sem caller → deletar → aposentar o fallback CPU no `lifecycle.rs` → deletar o `step()` + passes do `diffusion.rs`.

### 2.4 — Simplificar o bridge (a torre de heurísticas). Plano do investigador, ranqueado:
1. **Remover a máquina de catch-up** (`texture_mode_dirty`+`catchup_bands`+"2 bandas"+`IDLE_WARMUP_FRAMES`) bakeando `canvas_rgba` **só no pen-up** (§2-I3) em vez de por-frame. **~70 LOC + 2-3 campos de session + o invariante mais perigoso do arquivo.** Risco: médio (manter `flush_pending_bake` como bracket do undo). É o débito §10 "nunca engatou" (= C4).
2. **Dropar E5** (`drive_fluid_chain` + a lane de straight-texture) se o stack não-trivial-mid-stroke não ocorre no fluxo real → cai pra readback lane (idêntico visual, +1 frame de latência num caminho raro). **~200 LOC bridge + ~110 composite + 2 pipelines WGSL.** Risco: médio (confirmar que a forma não é usada).
3. **Remover `KEEP_WET_SETTLE_FRAMES`** (settle-freeze) substituindo pelo equilíbrio real do C1 (Fase 1). A active-region window já subsume o freeze de timeout.
4. **Limpar knobs de env de debug** (§4) e fundir `composite_frame`/`composite_frame_pipelined` num `encode_composite_band` helper (~−50 LOC, mata copy-paste).

---

## §3 — FASE 1 (estabilidade: os bugs reais que tocam o comportamento)

### C1 — equilíbrio físico da água (o creep / "tela ativa cara")
A raiz: sob Keep Wet (`evaporation=0`), o gradiente água molhada→seca na borda **persiste pra sempre** e `cs_add_forces` (`shallow.wgsl:106-141`) injeta velocidade pra fora continuamente; `drag` só amortece momento (`×(1−drag)`), não remove a força motriz → **sem ponto fixo** → espalha até o timeout `KEEP_WET_SETTLE_FRAMES`. `WATERCOLOR_VELOCITY=1.3` (`solver.rs:78`) acelera.
**Direções de fix (precisa olho do Enio, onda a onda — toca o look):** (a) uma evaporação mínima sob Keep Wet (`PH2D_FLUID_KEEPWET_EVAP` já existe — Enio achou que `0.003` "secou muito"; tunar menor / curva); (b) pinning/tensão-superficial: a água para de wickar abaixo de um gradiente-limite; (c) re-derivar `velocity=1.3`/`drag`/`pressure` (Suspeito nº1, §5 do refactor). **Resolver C1 permite remover o settle-freeze (§2.4-3).** ⚠️ Enio rejeitou "conter a água por knob" — quer equilíbrio por física, mantendo o sangramento por ~1-2s.

### C4 — ver §2.4-1 (bake no pen-up mata a fragilidade do undo).
### C3/C5/C6 — fixes pequenos: atualizar o doc stale + remover o CPU dry-check do caminho GPU (C3); liberar o slot em falha de cópia (C5); a active-region já mitiga C6 (mas o fallback `dab_region` monotônico deveria ser a active-bbox).

---

## §4 — Env knobs vivos (decidir: promover a param vs deletar)
Em `lifecycle.rs::fluid_diffusion_params` + `painter_fluid_bridge.rs`:
- **MANTER:** `PH2D_FLUID_PROFILE` (profiler).
- **PROMOVER a param/tier:** `PH2D_FLUID_SCALE` (resolução do grid — o lever dominante; deveria ser tier-driven), `PH2D_FLUID_KEEPWET_EVAP` (o fix de equilíbrio do C1), `PH2D_FLUID_SHARPNESS` (MacCormack on/off — já default 0).
- **DELETAR (cruft de debug):** `PH2D_FLUID_SUBSTEPS` (wrapper; manter o closure), `PH2D_FLUID_ACTIVE_WINDOW`/`_ACTIVE_PAD` (bakear os consts 90/48), os multiplicadores `PH2D_FLUID_CAPILLARY`/`_DIFFUSIVITY`/`_FLOW`/`_VELOCITY` (são `*=` em sliders já expostos no Brush Studio). Aí o helper `env_f32` some.

---

## §5 — FASE 3 (decomposição HR-18). Padrão: struct em `mod.rs` + `impl X` across-siblings (ZERO churn de visibilidade).
- **`solver.rs` 2263 → `solver/`:** `mod.rs` (struct + consts + `pack_soa`/`unpack_soa` + `step_cpu_reference` + `GpuParams`/`FieldStats`), `dab.rs` (DabGpu), `build.rs` (`new()` — **o único corte difícil, ~800 LOC**; extrair helpers per-subsystem que retornam pipelines+bind-groups), `fields.rs` (upload/clear/buffer-accessors), `params.rs` (set_* + write_params_with_region), `passes.rs` (step/splat/encode_resident_splat_step), `stats.rs` (reduce + os 6 `read_*` — **dedup oportuno: 1 `readback_buffer` helper corta ~480 LOC**).
- **`composite.rs` 1354 → `composite/`:** `mod.rs` (structs + `new` + GpuU/Coeffs), `stroke.rs` (`begin_stroke` + os 2 readbacks — **manter juntos**, o invariante `pending`/`staging` drain-before-reuse "aborta o processo" se violado), `texture.rs` (E4/E5 to-texture), `oneshot.rs` (composite_to_rgba/buffer/rows + build_buffers).
- **`lifecycle.rs` 1853 + `tool/tests.rs` 4188:** split por responsabilidade (stroke/fluid_query/fluid_drive/ui/journal; tests por subject).

---

## §6 — O QUE NÃO MUDA (preservar)
- O **modelo K–M espectral** (ADR-0080, 24 bandas, `PIG_CH=32`) — a mágica do azul+amarelo=verde.
- O **look validado** (blooms, edge-darkening, granulação, franja, sheen, lift). Validar paridade VISUAL onda a onda, não bit.
- Os **ABIs de superfície** (`Stamp=96B`, `RenderingMode=6`, caps `FluidSim/FluidParams/GravitySource`) — congelados (gate `architecture_painter_contract_surface`).
- A **active-region window** + o **SoA** + os fixes de perf desta sessão — são a base nova.

---

## §7 — Âncoras de arquivo
- Bridge: `shells/desktop/src/render_loop/painter_fluid_bridge.rs` (active-region ~400-450, consts ~52-85, env knobs), `painter_fluid_support.rs` (FluidSession, catch-up, ensure_preview_slot), `painter_gpu_preview.rs` (E5).
- Solver: `crates/ph2d-painter-fluid/src/solver.rs` (`step_resident_splat`/`encode_resident_splat_step`, `pack_soa`/`unpack_soa`, os `read_*`).
- Composite: `crates/ph2d-painter-fluid/src/composite.rs` (`encode_frame_to_texture`, `composite_frame*`, o `ss` é setado no bridge L335).
- Shaders SoA: `crates/ph2d-painter-fluid/src/shader/*.wgsl` (todos via `pidx(cell,v)=v*NC+cell`).
- CPU twin: `crates/ph2d-painter-brush/src/diffusion.rs` (a deletar §2.3), `wet_composite.rs`.
- Tool: `crates/ph2d-tool-painter/src/tool/lifecycle.rs` (`fluid_diffusion_params`, `tick_wet_field`, dab radius L935/983).
- Testes: `tests/{physical_invariants (novo), gpu_parity, composite_parity, contract_surface}.rs`.

---

## §8 — Primeira ação da próxima sessão
~~**Fase 2 passo 2**~~ ✅ **FEITA 2026-06-11** (ver nota no topo: commits `9f898c0d`/`2c848545`/`bc84b86e`,
local). A paridade + o twin CPU foram-se; GPU é o único caminho vivo; sem-GPU = aquarela OFF.

**Próxima sessão:**
1. **Enio valida visualmente** a degradação sem-GPU + o look intacto no app (pré-push).
2. **Fase 1 — C1** (§3): equilíbrio físico da água (creep/Keep-Wet). **Precisa olho do Enio onda a onda**
   (toca o look). C3/C5/C6 são fixes pequenos que podem ir junto. Nota: o doc-stale C3 já foi parcialmente
   limpo (o CPU dry-check saiu com o fallback); resta a física do C1.
3. **Fase 3** (§5): decomposição HR-18 (`solver.rs`/`composite.rs`/`lifecycle.rs`/`tool/tests.rs`). `diffusion.rs`
   já caiu pra 536 LOC (não precisa mais decompor). `tool/tests.rs` encolheu ~630 linhas (menos pressão).
4. **§2.4 bridge** (torre de catch-up / E5 / KEEP_WET_SETTLE_FRAMES) — ainda aberto; o fallback CPU já saiu,
   então a máquina de catch-up (C4) é o próximo alvo de simplificação do bridge.

— Handoff aberto 2026-06-11. Tudo commitado local (`cef8e9ef`/`77be3ae5`/`1ad831c4`), sem push. Sistema funcional, mais robusto, com perf resolvido pro caso contido.

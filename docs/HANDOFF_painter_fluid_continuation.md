# HANDOFF — Painter Fluid (Watercolor) — Continuação para nova LLM (2026-06-08)

> **Estado: motor de aquarela GPU validado pelo Enio** ("sim, funcionou! tudo correto!"):
> física real (edge-darkening + granulação), **GPU-residente**, **region-scoped**, **sem delay
> clique→traço**, **~240 FPS**. Norte arquitetural: [ADR-0078](architecture/decisions/0078-watercolor-gold-standard-resident-tiled-shallow-water.md).
> Histórico detalhado (a jornada W15.3 + a saga do delay): [HANDOFF_painter_fluid_gpu_composite.md](HANDOFF_painter_fluid_gpu_composite.md).
> **22 commits locais, NÃO pushados** (fast-mode `--no-verify`). Leia §0 + §3 antes de tocar em perf.

---

## §0 — Inegociáveis do motor (memorize)

1. **Paridade é lei.** O `DiffusionGrid` da CPU (`crates/ph2d-painter-brush/src/diffusion.rs`) é o
   **ground truth + det-fallback (HR-5)**. **TODO passo GPU tem gate de paridade** vs a referência CPU
   (`crates/ph2d-painter-fluid/tests/{gpu_parity,composite_parity}.rs`). **NÃO landar um passo GPU novo
   sem o gate.** Os gates rodam em Metal com `--ignored` (precisam de device real).
2. **Inner loop = `cargo check -p <crate>`.** Gates GPU 1× no fechamento (não por task). RAM 8 GiB → ≤3
   cargos simultâneos.
3. **Você NÃO pusha.** Commits locais; o ship (`./scripts/ship.sh` → push) é decisão do Enio. **Os 22
   commits desta sessão usaram `--no-verify`** → fmt/clippy/machete/deny/nextest **ainda não rodaram**.
   Antes de qualquer push: `./scripts/ship.sh` e corrija os `✗` (provável: fmt + algum clippy).
4. **Contratos congelados intactos** (§5): `FluidParams ≤ 12` (ADR-0049) **não foi tocado** — a deposição
   vive em `DiffusionParams` (interno, não-capado) + consts `WATERCOLOR_*` no solver.

## §1 — O que está PRONTO + validado (não refazer)

| Estágio (ADR-0078) | Entrega | Commits | Gate |
|---|---|---|---|
| **S0** núcleo GPU-residente | `cs_splat` (dab-list, substitui upload full-grid) + `step_resident_splat` (sim residente, zero readback) + `cs_reduce`/`read_field_stats` (max-water+bbox no GPU) | `693b6f3`,`1d31dc5`,`8772132` | `cs_splat_matches_cpu_splat` (8e-8), `step_resident_splat_matches_cpu`, `read_field_stats_matches_cpu` ✓ smoke OK |
| **S1a** sim region-scoped | diffuse/advect/evaporate dispatcham só sobre o envelope molhado (pad `SOLVER_REGION_PAD`, invariante solver⊇composite) → `O(frente)`, 2× em 4K | `0ec2978` | `region_scoped_step_matches_full_grid_inside_region` (0 ULP) ✓ smoke OK |
| **S3a** deposição (ref CPU) | `DiffusionGrid::transfer_pigment`: `rate=(deposition+deposition_dry·(1−gate))·(1+granulation·(1−paper))`; camada `deposited` congelada | `734c30e` | 4 invariantes (dormância, conservação, edge-darkening, granulação) ✓ |
| **S3b** deposição (espelho GPU) | `cs_transfer` (paridade exata 0.000000) | `bbe4446` | `gpu_transfer_matches_cpu_deposition` ✓ |
| **S3c** deposição **VISÍVEL** | `cs_combine` (`total=flowing+deposited`) + `total_buffer()` (composite lê o total) + consts `WATERCOLOR_DEPOSITION_*` ligadas no bridge | `d253b8f` | `gpu_combine_equals_flowing_plus_deposited` ✓ **smoke OK (Enio "funciona!")** |
| **perf** composite pipelined | `composite_frame_pipelined` (readback assíncrono 1-frame-late, sem `poll(wait)` por-frame) → ~240 FPS | `7dea61f`,`d975520` | `composite_frame_pipelined_matches_sync` (byte-exato) ✓ |
| **perf** DELAY clique→traço | **cache do papel** (gerado 1×/canvas, pré-gerado no hover) — era o ~⅓s | `0cd7802` | testes diffusion/fluid verdes ✓ **smoke OK (Enio "delay sumiu")** |
| **S3d-a** shallow-water (ref CPU) | `DiffusionGrid` + `vel_u/vel_v/pressure`; `move_water` = `add_forces` (`−β∇h −λ∇w −drag·u +μ∇²u`, wet-gated, CFL) → `project` (Jacobi incompressibilidade, `RELAX_ITERS=6`, seed-0 determinístico); pigmento advecta por `(u,v)` (dormant quando `velocity=0` → look antigo bit-idêntico) | local (pré-commit) | 5 gates CPU (dormant, det, estável, transporta `+x`, **anel de backrun** 6× centro) ✓ + 340 lib verdes |
| **S3d-b** shallow-water (espelho GPU) | `shallow.wgsl` (`cs_add_forces`/`cs_divergence`/`cs_clear_pressure`/`cs_jacobi`/`cs_project`/`cs_advect_velocity`) + `FluidSolver` (buffers vel/pressure, 6 pipelines, bind-groups ping-pong, `set_shallow_water`, `read_velocity`) + integração em `step_resident_splat` | local | `gpu_shallow_water_matches_cpu_move_water` (**0.000000 vel+pig, bit-exato em Metal**) ✓ + naga ✓ + 17 gates GPU existentes intactos (todos 0 ULP) |
| **S3d-c** shallow-water VISÍVEL | `set_shallow_water` ligado no bridge (`WATERCOLOR_VELOCITY=1.4`/`_VISCOSITY=.1`/`_DRAG=.08`/`_PRESSURE=.4`) + clear de velocidade no epoch | local | compila (`-p ph2d-host-desktop --features fluid`) ⚠️ **SMOKE/visual pendente (Enio)** |

**Medido (Metal `--release`, bench `perf_resident`):** traço típico step+composite — 1408: 1.8ms · 2048: 4.8ms ·
**4K: 6.5ms** (region-scoped). Pós-fixes: sem delay, ~240 FPS. **⚠️ S3d adiciona ~11 passes/substep
(add_forces + divergence + 2 clear + 6 jacobi + project) — perf do traço live AINDA NÃO medida com a
camada ligada; validar FPS + visual (anel de backrun) com o Enio antes de qualquer push.**

## §2 — O que FALTA (priorizado)

0. **S3d — VALIDAÇÃO VISUAL + perf do traço live (Enio).** O motor shallow-water está **completo +
   paridade bit-exata** (CPU ref + espelho GPU + ligado no bridge), mas só foi validado por **gates
   headless** — falta o Enio ver o **anel de backrun/cauliflower** num traço real e confirmar que o FPS
   aguenta os ~11 passes/substep extras. **Rode `./play.command`, traço grande de aquarela, observe o anel
   off-center + fluxo direcional.** Se o FPS cair: (a) reduzir `RELAX_ITERS` (6→4), (b) rodar move_water
   1×/frame em vez de por-substep, (c) pular divergence/jacobi quando `pressure==0`. Se o look pedir tuning:
   ajustar os `WATERCOLOR_VELOCITY/_VISCOSITY/_DRAG/_PRESSURE` (consts em `solver.rs`). **`./scripts/ship.sh`
   + os 22+ commits desta saga ainda não pushados** (§0.3).
1. ~~**S3d — campo de velocidade shallow-water**~~ **FEITO** (S3d-a/b/c, ver §1). MoveWater (add_forces +
   Jacobi project) + advect-por-`(u,v)`; dormant→look antigo; paridade GPU 0 ULP. A física da alma chegou.
2. **S4 — multi-pigmento K–M + multi-camada @4K**: granulação/staining por-pigmento; encadear campos
   residentes no compositor de camadas (ADR-0048). Aqui entra a **emenda do `FluidParams`** (3 slots de
   headroom → `deposition/deposition_dry/granulation` per-brush, sob ADR-0078) + atualizar o gate
   `architecture_painter_contract_surface`.
3. **S5 — refinamento**: advecção BFECC (transporte nítido), supersampling adaptativo, capilar LBM
   (MoXi) p/ percolação realista, tune 120Hz.

**Polish (NÃO urgente — 240 FPS + sem delay já entregue):**
- *drive-owns-slot reorder*: produzir o preview do painter **antes** do `sim_extract` (via
  `replace_individual_pixels_region`, API já existente) → colapsa o +1 frame estrutural → zero-frame-extra.
  Toca o caminho validado Apply/preview/bgremoval → passo testado. Detalhe no commit `acfc98c`/`db8d864`.
- *texture-target puro* (composite escreve a textura direto, zero readback) — só importa pra **banda 4K**.
- *GPU paper-gen* (`cs_paper`) + dropar o `DiffusionGrid` da CPU → resolve a **memória 4K** (hoje o grid
  CPU é alocado por-traço só p/ paper+dims; água/pig/scratch são lazy-zero, ~free no caminho GPU).

## §3 — Aprendizados CAROS (leia antes de mexer em perf/delay)

1. **Meça a ESCALA do sintoma antes de caçar a causa.** O "delay clique→traço" era **~⅓ de segundo**
   (Enio precisou dizer isso explicitamente). Eu queimei vários rounds caçando latência de **frame**
   (4–16ms: pipelined/priming/ordem de upload) — era **100× maior**: trabalho **O(grid) na CPU por-traço**
   (`begin_stroke` regerava o papel `grain_noise` célula-a-célula; num canvas grande/4K = centenas de ms).
   Frame≠⅓s. **Pergunte/meça a magnitude (ms) cedo.**
2. **Auditoria multiagêntica acerta a causa, mas confirme a ESCALA.** A auditoria (7 agentes) achou o +1
   frame estrutural (preview produzido **depois** do `sim_extract`) — real, mas **imperceptível** e NÃO
   era o delay. O delay era o papel (O(grid)). A auditoria não tinha o número "⅓s" no prompt.
3. **`bench-verde ≠ vivo.** O `device.poll(wait)` por-frame do composite drenava a fila GPU inteira (~2.6ms
   stall, 250→140 FPS) — o bench de loop-apertado mediu só a transferência (0.03ms), não o stall de
   pipeline. Instrumentação no app (`PH2D_FLUID_PROFILE`) revelou. Fix: readback **pipelined** (assíncrono).
4. **Envelope monotônico ⊇ pigmento** (do W15.3, ainda vale): compositar sobre a bbox da água (que recua na
   evaporação) corta o traço em retângulo; use a união all-time das bboxes de dab.
5. **"Baixa-res" pode ser o CANVAS, não o sim** (W15.3): o painter edita o sprite na res nativa; demo =
   64×64. Pra testar 4K, arraste um PNG grande.

## §4 — Arquivos-chave

- **Solver GPU:** `crates/ph2d-painter-fluid/src/solver.rs` (`FluidSolver`: `cs_splat`/`step_resident_splat`/
  `cs_reduce`/`cs_transfer`/`cs_combine` + **shallow-water** `set_shallow_water`/`read_velocity`/
  `clear_resident_velocity_gpu`, `total_buffer`, consts `WATERCOLOR_*`) +
  `src/shader/{fluid,splat,reduce,transfer,combine,shallow}.wgsl`. **`GpuParams` cresceu 80→96 B** (os 4
  campos S3d ocupam os bytes que os outros shaders liam como padding → byte-compatíveis, não mexidos).
- **Composite GPU:** `src/composite.rs` (`FluidCompositor`: `begin_stroke`, `composite_frame` [sync, p/ test],
  `composite_frame_pipelined` [vivo], `PendingReadback`) + `src/shader/composite.wgsl`.
- **Referência CPU (paridade + física):** `crates/ph2d-painter-brush/src/diffusion.rs` (`DiffusionGrid`:
  `step`=**move_water**/diffuse/advect/**transfer**/evaporate, `move_water`=`add_forces`+`project`,
  `velocity()`/`set_velocity_from`, `generate_paper`/`with_paper` [cache], `deposited`; `pub RELAX_ITERS`).
- **Hooks do tool:** `crates/ph2d-tool-painter/src/tool/lifecycle.rs` (`begin_stroke` [cache do papel +
  `cached_fluid_paper`], `fluid_take_dabs`, `fluid_dry_check_and_drop_gpu`, `fluid_prewarm_paper`) +
  `tool/mod.rs` (`FluidDab`, `fluid_dabs`).
- **Drive (shell):** `shells/desktop/src/render_loop/painter_fluid_bridge.rs` (`drive_fluid_gpu`: pre-warm,
  epoch setup, region-scoped step, pipelined composite, sporadic dry-check, `PH2D_FLUID_PROFILE`).
- **Lançador:** `./play.command` (release, `--features fluid`, slot `target-slots/slot-brushoverhaul`).

## §5 — Contratos (mexer = ADR, CLAUDE.md §6)

- **`FluidParams ≤ 12`** (ADR-0049 §2.13, gate `architecture_painter_contract_surface`) — **INTACTO** (9
  campos). A deposição NÃO está nele (vive em `DiffusionParams` interno + consts do solver). Per-brush
  deposition = **emenda dos 3 slots de headroom** em S4, sob ADR-0078.
- **`GpuParams`** (interno do solver, 80B) carrega region + deposition — não é contrato congelado.
- **Caminho de difusão** de ADR-0049 = graceful-degrade (GPU incapaz / sem feature). CPU ref = det-fallback.

## §6 — Build / rodar / validar

```bash
# App (release, fluid):
./play.command                              # ou PH2D_FLUID_PROFILE=1 ./play.command (log por-frame)
# Gates de paridade GPU (Metal, --ignored):
cargo test -p ph2d-painter-fluid --features fluid --test gpu_parity --test composite_parity -- --ignored --nocapture
# Bench de perf (release):
cargo test -p ph2d-painter-fluid --features fluid --release --test perf_resident -- --ignored --nocapture
# Física CPU (referência):
cargo test -p ph2d-painter-brush --lib diffusion
# Tool (begin_stroke + fluid hooks):
cargo test -p ph2d-tool-painter --lib fluid
# Inner loop:
cargo check -p ph2d-painter-fluid --features fluid   # ou -p ph2d-tool-painter ; ou -p ph2d-host-desktop --features fluid
```

**Validação visual (Enio):** pincel de aquarela, traço grande → anel escuro na borda (edge-darkening) +
granulado nos vales do papel; sem delay no clique; RAW ~240 no traço.

— deixado por Claude (sessão 2026-06-08: motor de aquarela GPU físico completo S0–S3c + perf, validado).
  Próximo: **S3d (shallow-water velocity → backruns)**, em contexto fresco.

— atualizado por Claude (sessão 2026-06-08, cont.): **S3d COMPLETO** — shallow-water velocity layer
  (CPU ref `move_water`=add_forces+Jacobi-project + advect-por-`(u,v)`, 5 gates físicos incl. anel de
  backrun; espelho GPU `shallow.wgsl` 6 passes, **paridade bit-exata 0 ULP em Metal**; ligado no bridge).
  Tudo dormant quando `velocity=0` → look antigo intacto, 17 gates GPU existentes 0 ULP. **Commits locais
  (`--no-verify`), não pushados.** Próximo: **validação visual do Enio (anel de backrun + FPS do traço
  live)** — vide §2.0; depois **S4 (multi-pigmento K–M + emenda `FluidParams`)**.

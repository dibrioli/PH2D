# Handoff de integração — `line/motion-nodes` (2026-07-22)

> A família de **deformers do Motion foi para a GPU**: o 6º canal do resolver
> (`reduce → broadcast → map`) e cinco nós CPU-only agora cozinham 100% no
> device. Linha FECHADA, aguardando ordem de integração do Enio. **NÃO integra
> nem pusha sozinha** (DIRETRIZ §1.5.9).

*Resumo:* Linha `motion-nodes` pronta (HEAD `5daf12fe3`, 8 commits). Handoff abaixo. Aguardo ordem de integração.

---

## 1. Identidade

- **Branch:** `line/motion-nodes`
- **HEAD:** `5daf12fe3`
- **Base (merge-base com main):** `13a04c7aa`
- **Commits:** 8 (6 de código + 2 de docs), ordem LINEAR e aditiva:
  1. `3aba8ebbf` docs — o plano-mestre parou de mentir + a fila REAL
  2. `3df26029c` feat — o **REDUCE reusável** (o primitivo bit-exato, irmão do `scan`)
  3. `efd4182e5` feat — os **DEFORMERS** na GPU: o 6º canal do resolver + `bend`/`twist`
  4. `0a38e9f74` feat — `spherize` (o `Sum` + DUAS reduções num nó)
  5. `169b6b9cb` feat — `four_point_warp` (4 reduções = o bbox; estreia do `Min`)
  6. `97da3ae80` docs — a fila REAL: o canal de redução FECHOU (4 nós, 3 operadores)
  7. `48a8114bb` feat — `kaleidoscope` (o 1º `StreamOp::SourceRows` que LÊ o template)
  8. `5daf12fe3` chore — fmt + import de `Dim` no teste do `gpu.rs` (achado do close-gate)

---

## 2. Foundational / compartilhado tocado + por quê

**Tudo ADITIVO** — módulos-irmãos novos e pontos de extensão append-only, no
molde do side-metadata do ADR-0126 (o kernel/UI vive no REGISTRY, nunca no
`NodeManifest`). **Nenhum contrato congelado foi tocado** (§4).

- **`ph2d-nodegraph`** (crate FOUNDATIONAL do contrato):
  - **NOVO `reduce_meta.rs`** — `ReduceOp` (enum `Max`/`Min`/`Sum`, com `wgsl_combine`/`wgsl_identity`/`cpu`/`empty`) + `ReduceSpec`. Módulo-irmão isolado.
  - **NOVO `column.rs`** — `ColumnAccess` + `ColumnBinding` extraídos do `gpu.rs` (estava em 713 > 700 LOC). `ColumnAccess` ganhou o variant **`SourceRead`** (APPEND — leitura de porta-template comprimento-desacoplada, ADR-0136 `StreamOp::SourceRows`).
  - `gpu.rs` — método `reduces(&self, _ty) -> &'static [ReduceSpec]` (default `&[]`, o **6º canal** do `KernelResolver`) + re-exports de `ReduceOp`/`ReduceSpec`/`ColumnAccess`/`ColumnBinding`.
  - `lib.rs` — `pub mod reduce_meta;` + `pub mod column;`.
- **`ph2d-node-registry`** (foundational): campo `reduces: BTreeMap<NodeTypeId, &'static [ReduceSpec]>` + `register_reduces()` + `reduces()`. Espelha EXATAMENTE `register_gpu_kernel`/`grid`/`state_select`/`stream_op`/`algorithm`.
- **`ph2d-gpu-cook`** (motor de cook compartilhado — a MESMA crate que a `line/gpu-nodes` integrou em 21/07):
  - **NOVO `reduce.rs`** — o primitivo bit-exato (`Reduce`/`ReduceScratch`/`reduce_into`).
  - **NOVO `reduce_stage.rs`** — a costura `run_reduces` (dentro do laço de sweep; uniforme próprio por passe; coluna ausente = `identity`).
  - **NOVO `accessors.rs`** — a superfície de leitura de `GpuCook` (split do `lib.rs`, 729 > 700).
  - `codegen.rs`/`encode.rs` — parâmetro `reduces: &[ReduceSpec]` acrescido (emite bindings `reduce_buf_<name>` + accessors `reduce_<name>()`).
  - `lib.rs` — campos `reduce`/`reduce_hold`/`reduce_hold_bufs`/`reduce_results_hold` no `GpuCook` (limpos por cook); `gather.rs` — braço `SourceRead` em `column_present`; `stream_op.rs`/`scan.rs` — passam `(&[], &[])` no call de `encode_kernel_stage`.
  - **`[dev-dependencies]`** (NÃO `[dependencies]`) += as 5 crates-nó, só para o gate de paridade. gpu-cook `src/` **não** as usa ⇒ machete-safe.
- **`shells/desktop/src`**:
  - **NOVO `motion_state_gpu_demos_deform.rs`** — cenas =12..15 (490k+ instâncias cada, FULLY GPU).
  - `motion_state.rs` — braços de dispatch `Ok("12"..="15")` + imports.
  - `motion_gpu_coverage.rs` — os 4 docs de deformer entram no corpus do censo (o censo antes era CEGO a eles).

**As 5 crates-nó** (`bend`/`twist`/`spherize`/`four-point-warp`/`kaleidoscope`)
são drop-crates (meu módulo — já existiam da integração `line/motion-value`).
Ganharam `GPU_KERNEL` + `REDUCES` + as chamadas `register_gpu_kernel`/`register_reduces`
DENTRO da `register()` de cada uma. **Não toquei `ph2d-node-registry-init`** — o
fan-out já chama as 5 `register()`, então o side-metadata de GPU entra no registry
REAL do app automaticamente. **Os kernels estão VIVOS no editor, não só no demo.**

---

## 3. Símbolos que podem COLIDIR com outra linha (grepar)

| Símbolo | Onde | Nota |
|---|---|---|
| `ph2d_nodegraph::reduce_meta` (módulo pub) | nodegraph | módulo NOVO |
| `ph2d_nodegraph::column` (módulo pub) | nodegraph | módulo NOVO (split do `gpu.rs`) |
| `ReduceOp`, `ReduceSpec` | reduce_meta | tipos NOVOS (re-exp de `gpu`) |
| **`ColumnAccess::SourceRead`** | column | **variant APPEND** — colide se outra linha viva apendou variant em `ColumnAccess` |
| `KernelResolver::reduces()` | gpu | método de trait NOVO (default `&[]`) |
| `NodeRegistry::{reduces (campo), register_reduces, reduces()}` | node-registry | append |
| gpu-cook: mods `reduce_stage`/`accessors`; `Reduce`/`ReduceScratch`/`reduce_into`; `GpuCook.{reduce,reduce_hold,reduce_hold_bufs,reduce_results_hold}`; `REDUCE_MAP_SALT`; naming `reduce_buf_<name>`/`reduce_<name>()` | gpu-cook | append |
| **`PH2D_GPU_COOK_DEMO=12,13,14,15`** | shell | **cenas de smoke** — a `line/gpu-nodes` tomou 7-11 (JÁ integrada); confirmar que nenhuma linha AINDA-NÃO-integrada reivindicou 12-15 |
| `Cargo.lock` | raiz | só as 5 path-deps de dev do gpu-cook |

**ADR:** nenhum número novo reivindicado. O canal de redução monta no padrão
side-metadata do ADR-0126 (6º canal, exatamente como grid/state_select/stream_op/
algorithm da `line/gpu-nodes`). **Se o Enio quiser um ADR próprio pro canal, é
decisão dele** — a linha não o tomou (não é mudança de contrato).

---

## 4. Contratos congelados encostados

**NENHUM.** `architecture_contract_surface` = **3/3 verde** (`NodeOp=2` /
`OpResolver=1` / `NodeManifest=8` intactos). O canal `reduces` é side-metadata no
registry/resolver, jamais um campo de `NodeManifest`. Não exige ADR de contrato.

---

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **machete:** as 5 crates-nó estão em `[dev-dependencies]` do gpu-cook e são
  usadas por `tests/gpu_cpu_parity_deform.rs` ⇒ **não** devem sangrar. Conferir.
- **deny/audit:** **zero dep externa nova** — só path-deps internas. RUSTSEC
  inalterado.
- **fmt/typos/clippy latente pré-fork:** o padrão ([[project_integration_prefork_lines_ship_drift]]). O close-gate desta linha já rodou `fmt --all --check` (limpo) e `clippy --all-targets` nas crates tocadas (limpo — pegou e corrigiu o `Dim` do teste no commit 8).

---

## 6. Ordem / dependências + o que smoke-testar

**Ordem:** os commits são lineares e cada um constrói sobre o anterior (primitivo
→ canal+bend/twist → spherize → four_point_warp → kaleidoscope → higiene). Sem
dependência cruzada com outras crates além do que já está no `main` do fork.

**GATES verdes no fechamento** (rodados nesta máquina, RTX):
- `cargo check --workspace` · `fmt --all --check` · `clippy --all-targets` (crates tocadas)
- `architecture_contract_surface` 3/3 · `generated_wgsl_validates` 2/2 (os 5 kernels em TODO subconjunto de presença)
- `architecture_workspace_file_loc_cap` · `file_loc_caps` (shell) · `motion_gpu_kernel_budgets`
- **`gpu_cpu_parity_deform` 7/7 no DEVICE** (bend/twist/spherize/four_point_warp/kaleidoscope + excursão + a costura de redução), serial — pior ε medido **9,8e-6** vs bound `EPS_POS = 2e-4`.

**Smoke VISUAL (não rodei windowed — é do integrador/Enio):**
```
env PH2D_GPU_COOK_DEMO=12 cargo run -p ph2d-host-desktop --release   # bend -> twist (cloth)
env PH2D_GPU_COOK_DEMO=13 cargo run -p ph2d-host-desktop --release   # spherize (lens)
env PH2D_GPU_COOK_DEMO=14 cargo run -p ph2d-host-desktop --release   # four_point_warp (flag)
env PH2D_GPU_COOK_DEMO=15 cargo run -p ph2d-host-desktop --release   # kaleidoscope (mandala)
```
GPU ON por default; `PH2D_GPU_COOK=0` bissecta para a CPU (a paridade prova que
as duas concordam). Todas as cenas são 490k+ instâncias, FULLY GPU (`plan.is_fully_gpu()`).

**⚠️ NÃO rodei** a varredura pesada `-- --ignored` completa (boids/voronoi a
milhões) — por restrição de saturação de GPU do Enio, ela fica para a integração.
O gate de paridade dos deformers (cenas leves 16k-70k) rodou 7/7 no device.

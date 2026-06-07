# HANDOFF — W15.3 GPU composite (próximo agente, START HERE)

> Você continua o **W15.3** ([ADR-0049](architecture/decisions/0049-fluid-brushes.md) + amendment-1).
> O solver de aquarela já roda no GPU; o **shader de composite agora também existe e está PROVADO
> bit-a-bit no GPU**. **O que falta é só o WIRING no shell** (caminho zero-readback), que precisa de
> validação VISUAL (Enio na tela). Leia `CLAUDE.md` inteiro. Você atua como **Coordenador sozinho**.

---

## §0 — STATUS (o que mudou nesta sessão, 2026-06-07)

**O CRUX (o maior shader do projeto) está FEITO e PROVADO.** Dois commits locais (sem push):

| Commit | Conteúdo | Prova |
|---|---|---|
| `2d0f5d3` | Shader composite (K–M espectral WGSL) + gate de paridade + CPU single-source | `composite_parity` GPU↔CPU = **0 LSB bit-exato** (Metal) |
| *(este)* | `composite_buffer` lê `pig_a` residente + seam step→composite end-to-end | `gpu_step_then_composite_resident_matches_cpu` = **0 LSB** |

### ✅ Feito + verde (tudo commitado local)
- **`pigment_mix.rs`**: `spectral_basis()`, `PreparedPigment::{color,ks,err}()`, `mix_prepared_exact`
  (variante sem-LUT — o GPU não precisa do cache LUT da CPU; é o ground-truth da paridade).
- **`ph2d-painter-brush/src/wet_composite.rs`** (NOVO): a matemática per-pixel do composite hoisted
  pra UMA definição (bicúbico + K–M straight-alpha glaze). `composite_wet_field_cpu` é a referência.
  O tool `composite_wet_field` agora **delega** pra cá (single source; 12 testes do tool verdes).
- **`ph2d-painter-fluid/src/composite.{rs,wgsl}`** (NOVO): `FluidCompositor` (compute pipeline) +
  o WGSL espelhando `composite_wet_field_cpu` banda-a-banda (NB=24, sem LUT; basis+brush via uniform).
  - `composite_buffer(...)` — **lê um buffer de pigmento EXTERNO** (passe `solver.pigment_buffer()`):
    composita o pigmento GPU-residente **sem readback de pigmento** (o fix do stall).
  - `composite_to_rgba(...)` — sobe um campo CPU (conveniência de teste / fallback CPU).
- **`solver.rs`**: `pigment_buffer()` (expõe `pig_a`) + `dims()`.
- **Testes** (`tests/composite_parity.rs`, `--ignored`, Metal): composite-only **0 LSB**; step+composite
  residente **0 LSB**; discriminantes K–M (amarelo/azul→verde, sem franja preta) verdes no GPU.
  naga valida `composite.wgsl` (`contract_surface.rs`). Shell compila `--features fluid`.

### Comandos
- `CARGO_TARGET_DIR=…/target-slots/slot-brushoverhaul cargo test -p ph2d-painter-fluid --features fluid --test composite_parity -- --ignored --nocapture`
- `… cargo check -p ph2d-host-desktop --features fluid`

---

## §1 — A MISSÃO QUE SOBRA (1 frase)

Hoje (`shells/desktop/src/render_loop/painter_fluid_bridge.rs::drive_fluid_gpu`) o fluxo é
`solver.step_grid` (**upload + step + READBACK pigment+water → grid CPU**) → `composite_and_settle_fluid`
(**composite CPU** → `canvas_rgba`). **O stall = `device.poll(wait)` no readback per-frame** (solver.rs
`read_pigment`/`read_water`). Sua missão: **eliminar o readback per-frame** ligando o `FluidCompositor`
(já pronto + provado) ao display, de forma que o composite leia `pig_a` residente e escreva direto
numa textura de preview (GPU→GPU), com readback **só no pen-up**.

## §2 — O BLOQUEADOR REAL (leia antes de codar — é o que torna isto não-trivial)

**Resíduo do pigmento no GPU vs. dabs adicionados na CPU.** Hoje `queue_pointer` (`tool/lifecycle.rs`
~L585-610) faz `grid.splat(...)` **na CPU** a cada dab enquanto pinta. O `step_grid` re-sobe o grid CPU
inteiro a cada frame — por isso o readback existe (a CPU é a source of truth). Pra cortar o readback, o
**pigmento tem que ficar residente no `pig_a` e ACUMULAR no GPU** (o bloom evolui no GPU). Mas dabs
novos entram pela CPU → se você re-subir o pigmento CPU por frame, **reseta o bloom**. Logo precisa de
**acumulação de dabs no GPU**. Opções (decida no padrão-ouro):
- **(A) Deposit buffer aditivo:** `queue_pointer` splata os dabs DESTE frame num buffer "deposit"
  separado (pequeno); o shell sobe + **soma** em `pig_a` (um pass compute `pig_a += deposit`), depois
  zera o deposit. Diffuse/advect rodam sobre `pig_a` residente. (Recomendado — mínimo de shader novo.)
- **(B) Splat no GPU:** porta `DiffusionGrid::splat` (disco soft) pra um compute pass que escreve em
  `pig_a` a partir dos stamps. Mais shader, mas tira o splat da CPU também.

**Dry-check sem readback:** `composite_and_settle_fluid` decide secar quando `water.max() < THRESHOLD`.
Mova a **evaporação pra CPU** (`water -= evap`, é O(células), trivial — diffuse/advect leem a água
PRÉ-evaporação, então a ordem casa) e **NÃO evapore no GPU**: a CPU rastreia a água (splat sobe,
evaporate desce) → dry-check **sem readback**. O GPU diffuse/advect leem a água que a CPU sobe.
(Isso troca o `cs_evaporate` do GPU por um passo CPU — atualize `step`/`step_grid` no solver.)

## §3 — A ARQUITETURA RECOMENDADA (o caminho zero-readback)

- **begin_stroke:** suba `wet_backdrop` (snapshot do `canvas_rgba`) pra uma **textura GPU** 1× (hoje é
  `Vec<u8>` no tool). [O `composite.wgsl` hoje lê o backdrop de um `array<u32>` storage — pra o caminho
  preview-texture, ou mantenha o backdrop como storage buffer (1 upload/stroke) **ou** mude a binding 2
  pra `texture_2d` + sampler. Storage buffer é menos mudança.]
- **por frame (bloom, GPU-only):** (1) soma deposit em `pig_a` (§2A); (2) `solver.step` diffuse+advect
  (sem evaporate GPU, sem readback); (3) CPU evapora sua água + dry-check; (4) **composite GPU lê
  `pig_a` + backdrop → textura de preview** (faça uma variante `composite_to_texture` do
  `composite_buffer`: troque a binding 3 de `array<u32>` por uma storage texture `rgba8unorm`, OU
  escreva no buffer e copie pro texture). (5) premul + copy pro slot de preview
  (`PainterPreviewGpu`) — espelhe `painter_gpu_preview.rs` (`PreviewPremul` + `copy_texture_into_individual`).
  **Zero readback.**
- **pen-up / dry (1×):** composite final → **readback RGBA da bbox** → blit em `canvas_rgba` (canônico,
  pra Apply/undo). Um readback por stroke.
- **Gating:** caminho GPU-preview só quando (a) 1 layer (caso comum aquarela) e (b) device capaz; senão
  caia no caminho CPU atual (que FUNCIONA). Multi-layer: o preview precisa compositar a stack — comece
  caindo no CPU pra multi-layer e refine depois. **Bound o risco.**

### A "alternativa mais simples" (NÃO remove o stall — evite)
Composite GPU → readback RGBA → `canvas_rgba` mantém 1 readback/frame (o stall continua) e só tira o
K–M da CPU. O `step+composite` já está provado; o ganho real é o caminho zero-readback acima.

## §4 — ONDE TOCAR (arquivos)
- `shells/desktop/src/render_loop/painter_fluid_bridge.rs` — `drive_fluid_gpu` (o drive per-frame).
  **Não infle** (`file_loc_caps`); crie helpers/arquivo novo se passar. `painter_bridge.rs` está em
  ~590/600 — NÃO mexa nele; o slot de preview é `PainterPreviewGpu` (estude `painter_gpu_preview.rs`).
- `ph2d-painter-fluid/src/composite.rs` — adicione `composite_to_texture` (output storage texture).
- `ph2d-painter-fluid/src/solver.rs` — `step` sem `cs_evaporate` (ou um `step_no_evap`); manter `pig_a`
  residente entre frames (não re-upload); pass aditivo `pig_a += deposit` (§2A).
- `ph2d-tool-painter/src/tool/lifecycle.rs` — deposit buffer por-frame; `wet_backdrop` → textura;
  pen-up readback→`canvas_rgba`. Hooks pro shell (mantenha o CPU path intacto p/ fallback).
- Gate: `architecture_no_downcast_to_concrete_tool_in_shell` (o bridge já está allowlisted).

## §5 — GOTCHAS (herdados + novos)
- **Paridade já é 0 LSB** no Metal — o shader está correto. Não "melhore" o shader; só ligue.
- **`composite_buffer` já lê `pig_a`** (`solver.pigment_buffer()`) — é a peça central, provada.
- **brush prep (`prepare_wet_composite`)** é amortizado 1×/composite; a chromaticity vem do TOTAL de
  pigmento. No shell, derive da cor do stroke / do grid CPU (não precisa de readback do pigmento pra
  isso — o `color_sum` vem do `stroke_color_oklab`; o `pcol` pode vir do total do deposit acumulado).
- **`mix_prepared_exact` (sem LUT)** é o que o tool usa agora no composite (era LUT). Mudança <1%,
  imperceptível, mais correto. Os 12 testes do tool + os discriminantes passam.
- **Det/§2.11/§2.14**: composite NÃO entra no replay HR-5 (frame-driven); paridade ~1e-3 basta (e
  conseguimos 0). Caps do crate fluid intactos.
- **NÃO pusha** (fast mode — Enio testa visual antes). Commit local scoped.

— deixado por Claude (sessão brush-overhaul + W15.3 GPU composite, 2026-06-07).

# HANDOFF — W15.3 GPU composite (próximo agente, START HERE)

> Você continua o **W15.3** ([ADR-0049](architecture/decisions/0049-fluid-brushes.md) + amendment-1).
> O solver de aquarela já roda no GPU e foi confirmado no app (smoke OK, Enio 2026-06-07).
> **Sua missão: mover o COMPOSITE pro GPU** — é o único stall de perf que sobra. Leia `CLAUDE.md`
> inteiro. Você atua como **Coordenador sozinho** (Enio autorizou; sem colisão). **Não invente
> algoritmo** — espelhe o CPU bit-a-bit (a paridade é o gate). Pesquise antes de codar.

---

## §0 — A MISSÃO (1 frase)

Hoje o **step** roda no GPU mas o **composite** faz **readback do pigmento por frame + roda no CPU**
(`PainterTool::composite_wet_field`). Esse readback é um **stall** (map + `device.poll` espera o GPU)
que sobra a cada frame. **Mova o composite pro GPU** → remove o readback → perf cheia em canvas grande
(o objetivo original do W15.3). É o **maior shader do projeto** (Kubelka–Munk espectral em WGSL).

## §1 — O QUE JÁ ESTÁ FEITO (o substrato — tudo commitado local, sem push)

Sete commits desta sessão (`git log --oneline` — de `ab6fd0a` a `105166a`):

| Peça | Onde | Estado |
|---|---|---|
| Solver GPU (`cs_diffuse`/`cs_advect`/`cs_evaporate`) | `crates/ph2d-painter-fluid/src/shader/fluid.wgsl` + `solver.rs` (`FluidSolver`) | ✅ **paridade bit-idêntica** ao CPU (testes `gpu_parity.rs`, `#[ignore]`, rodar `--ignored`) |
| `step_grid` (upload grid → step → escreve pigment+water de volta) | `solver.rs` | ✅ drop-in do `step` CPU |
| host tiering (`MemoryBudget::fluid_capable`) | `crates/ph2d-host/src/budget.rs` | ✅ |
| tool hooks (`set_gpu_fluid_driven`/`fluid_grid_mut`/`composite_and_settle_fluid`/`has_wet_field`) | `crates/ph2d-tool-painter/src/tool/lifecycle.rs` | ✅ CPU path intacto |
| shell drive (por frame, após `on_tick`) + graceful-degrade | `shells/desktop/src/render_loop/painter_fluid_bridge.rs` (feature `fluid`) | ✅ smoke OK |

**Como o fluxo roda hoje** (`drive_fluid_gpu`): `solver.step_grid(grid)` (GPU step + **readback** pigment+water →
CPU grid) → `painter.composite_and_settle_fluid()` (**CPU** `composite_wet_field` lê o grid, escreve `canvas_rgba`)
→ `painter_bridge` faz upload do `canvas_rgba` pra GPU preview. **Você vai cortar o readback + o composite CPU.**

## §2 — A REFERÊNCIA CPU pra espelhar: `composite_wet_field`

`crates/ph2d-tool-painter/src/tool/lifecycle.rs` (função `composite_wet_field`, ~L358-500). Algoritmo por pixel
do canvas (já scoped à bbox molhada via `wet_pigment_bbox`):

1. **Amortizado 1× por composite** (stroke é 1 cor): `tot = Σpigment`; `pcol = (tot/Σtot*3).min(1)` (cor); `prepared = prepare_pigment(pcol)`; `color_sum = Σ(oklab_to_linear_srgb(stroke_color_oklab))` (normalizador de cobertura independente de cor).
2. **Bicúbico** (Catmull-Rom, `sample_pigment_bicubic`) do pigmento low-res → `p` (vec3).
3. `dens = p.x+p.y+p.z`; se `< 1e-4` → escreve backdrop, pula.
4. `amount = dens / color_sum`; `alpha = 1 - exp(-amount * WET_COVERAGE_K)` (`=1.06`).
5. `back_a = backdrop.a/255`; `back = srgb→linear(backdrop.rgb)`.
6. **K-M glaze straight-alpha** (o miolo): `km = mix_prepared(prepared, back, alpha)`; `straight = (pcol*alpha + back*back_a*(1-alpha)) / out_a`; `out_a = alpha + back_a*(1-alpha)`; `rgb[k] = lerp(straight, km, back_a)`. Escreve `linear→srgb(rgb)` + `out_a*255`.

**Constantes:** `WET_FIELD_SCALE=2`, `WET_COVERAGE_K=1.06` (`lifecycle.rs` topo). **Backdrop** = `wet_backdrop`
(snapshot do canvas pré-stroke, `tool/mod.rs`) — **precisa ir pro GPU** (upload 1× no begin_stroke).

## §3 — O CRUX: portar o Kubelka–Munk pro WGSL

`crates/ph2d-painter-brush/src/pigment_mix.rs`. **Boas notícias (simplificações que já achei):**

- **`prepare_pigment` é amortizado** (1 cor/stroke). Prepare na CPU 1× por composite e envie como uniform:
  `brush.ks[24]` + `brush.err[3]` + `brush.color[3]` = 30 floats. O shader NÃO faz `prepare` por pixel.
- **O `LUT` da CPU é só cache de perf** — no GPU chame `to_reflectance`/`reflectance_to_rgb` inline. **Sem LUT.**
- **A `BASIS` é CONSTANTE** (computada 1× via `LazyLock`): `base[7][24]` + `m[3][24]` = 240 floats. Envie como
  uniform/storage 1× (compute na CPU via a `BASIS` existente, faça um getter `pub`). Sem recompute no shader.

**O que portar pra WGSL** (tudo loop sobre `NB=24`, sem recursão):
- `rgb_to_weights(rgb)->[f32;7]` (partição W+CMY+primário; L145).
- `to_reflectance(rgb)->[f32;24]` (Σ weights·base; L176) — usa `base`.
- `reflectance_to_rgb(refl)->[f32;3]` (Σ refl·m por canal; L191) — usa `m`.
- `refl_to_ks(r)= (1-r)²/2r` (L126), `ks_to_refl(k)= 1+k-√(k²+2k)` (L133).
- `mix_prepared(brush, a, t)` (L319): `lin=lerp(a,brush.color,t)`; `w=4t(1-t)`; se `w<0.02` → `lin`; senão
  `refl_a=to_reflectance(a)`; `rm[i]=ks_to_refl((1-t)·refl_to_ks(refl_a[i]) + t·brush.ks[i])`;
  `mixed=reflectance_to_rgb(rm)`; `ea = a - reflectance_to_rgb(refl_a)`; `spec = mixed + ea·(1-t) + brush.err·t`;
  retorna `lerp(lin, spec, w)`.

**Determinismo:** `powf`/`sqrt`/`exp` não são bit-idênticos cross-backend — mas o composite **não entra no
replay HR-5** (é frame-driven, não record-driven), então paridade ~1e-3 basta (igual o solver). Mire mean |Δ| < ~2e-3.

## §4 — ARQUITETURA: pra ONDE o composite GPU escreve (a decisão de design)

**O problema:** `canvas_rgba` é o canvas CANÔNICO (Apply baka, undo snapshota). Se o composite GPU só
escreve a textura de preview, Apply/undo perdem o wash.

**Recomendação (remove o readback per-frame, mantém `canvas_rgba` correto):**
- **Por frame (bloom):** composite GPU lê o pigmento (já no GPU, `solver.pig_a`) + o `wet_backdrop` (textura GPU,
  upload 1× no begin_stroke) → escreve numa **textura de preview GPU** (a região da bbox). Isso vira o display
  via o caminho `PainterGpuPreview` (`painter_bridge.rs` / `painter_gpu_preview.rs`). **Zero readback.**
- **No pen-up/dry-out (1×):** composite GPU final → **readback da bbox RGBA** → blit em `canvas_rgba` (canônico,
  pra Apply/undo). Um readback por stroke, não por frame.

Isso exige: (a) `wet_backdrop` como textura GPU; (b) o pigmento ficar GPU-resident (já está em `pig_a` — pare de
fazer readback no `step_grid`; crie um `step_only` sem readback); (c) integrar a textura de composite com o
`PainterGpuPreview` existente (estude `painter_bridge.rs` §5 "GPU preview lifecycle" + `painter_gpu_preview.rs`).

**Alternativa mais simples (ganho parcial, menos integração):** composite GPU → readback **RGBA da bbox** (4B/cell
@ canvas res) → blit em `canvas_rgba`. Mantém 1 readback/frame mas tira o K-M caro da CPU. Bom **primeiro
milestone** se a integração com o preview travar — mas o objetivo é a recomendação acima.

## §5 — GATE: teste de paridade (faça PRIMEIRO, igual fiz pro solver)

Antes de integrar no app, **prove o shader** num teste headless (Metal), espelhando `gpu_parity.rs`:
1. Monte um `DiffusionGrid` + `wet_backdrop` sintético + uma cor de stroke.
2. CPU: rode `composite_wet_field` (extraia a lógica testável, ou compare contra um canvas CPU).
3. GPU: rode o composite shader sobre os mesmos inputs → readback → compare mean/worst |Δ| (< ~2e-3).
4. Caso discriminante: **amarelo sobre azul opaco → verde** (a assinatura K-M; `[133,174,69]` no CPU hoje) e
   **borda sobre camada transparente → coral, sem franja preta** (straight-alpha).

Padrão do device headless: `GpuContext::new(GpuContext::default_instance(), None)` + `#[ignore = "needs a GPU device"]`
(veja `crates/ph2d-painter-fluid/tests/gpu_parity.rs` — copie o boilerplate de buffer/dispatch/readback).

## §6 — BUILD / TEST / ARQUIVOS / COMMITS

- **Slot warm:** prefixe cargo com `CARGO_TARGET_DIR=…/target-slots/slot-brushoverhaul`.
- **Crate:** `crates/ph2d-painter-fluid` (feature `fluid`). Shell: `cargo check -p ph2d-host-desktop --features fluid`.
- **Testes GPU:** `cargo test -p ph2d-painter-fluid --features fluid --test <novo> -- --ignored --nocapture` (Metal).
- **Naga (valida WGSL sem device):** veja `tests/contract_surface.rs::fluid_wgsl_parses_and_validates_via_naga`.
- **Gates:** `architecture_no_downcast_to_concrete_tool_in_shell` (se mexer no shell), `file_loc_caps`
  (`painter_bridge.rs` está em 590/600 — **não infle**, crie arquivo novo como `painter_fluid_bridge.rs`).
- **Arquivos-chave:** `pigment_mix.rs` (K-M), `lifecycle.rs::composite_wet_field` (referência),
  `fluid.wgsl`+`solver.rs` (solver GPU), `painter_fluid_bridge.rs` (drive shell), `painter_gpu_preview.rs` (preview).
- **NÃO pusha** (fast mode — Enio testa antes). Commit local scoped (`git commit --no-verify -- <seus paths>`).

## §7 — GOTCHAS

- **Amortize o `prepare_pigment`** (1×/composite, uniform) — foi a alavanca de perf no CPU (era metade do custo).
- **Sem LUT no GPU** (chame `to_reflectance` inline) — o LUT é só cache CPU.
- **`color_sum`** (normalizador de cobertura) vem da cor do stroke (`stroke_color_oklab`→linear→Σ), NÃO do grid.
  Foi o fix do bug "amarelo opaco cobre tudo" (ADR-0077 D12). Replique no GPU.
- **`back_a` lerp** entre straight-over e K-M — foi o fix da "borda escura em camada nova". K-M só sobre tinta opaca.
- **`mix_prepared` short-circuita** (`w=4t(1-t)<0.02` → lerp linear) — mantenha no shader (barato nas pontas).
- **Pare o readback no hot path**: hoje `step_grid` faz upload+step+**readback**. Pro composite GPU, separe
  `step_only` (sem readback) e mantenha o pigmento GPU-resident; só readback no bake final (pen-up/dry).
- **`wet_backdrop`** precisa virar textura GPU (upload 1× no begin_stroke; hoje é `Vec<u8>` no tool).
- **Det/§2.14/§2.11** seguem niche (caps já enforced no crate fluid; `step_cpu_reference` já é o fallback).

— deixado por Claude (sessão brush-overhaul + W15.3, 2026-06-07).

# HANDOFF — Painter Watercolor: **Capillary Soft Edge (S5)** — para a próxima LLM (2026-06-08)

> **Estado: S5 (borda capilar) IMPLEMENTADO + gated — ⚠️ pendente validação VISUAL do Enio.**
> O motor de aquarela tem agora as **4 camadas físicas**: gated-diffusion + deposição +
> shallow-water/backruns + **capilaridade (a franja macia/penugenta)**. GPU-residente,
> region-scoped, **16 controles** per-brush no Brush Studio, fidelidade de cor/Value, input
> transform-aware. Norte: [ADR-0078](architecture/decisions/0078-watercolor-gold-standard-resident-tiled-shallow-water.md)
> + [ADR-0079](architecture/decisions/0079-watercolor-params-per-brush-exposure.md).
> Histórico do motor: [HANDOFF_painter_fluid_continuation.md](HANDOFF_painter_fluid_continuation.md)
> (S0–S3d) + [HANDOFF_painter_fluid_gpu_composite.md](HANDOFF_painter_fluid_gpu_composite.md) (W15.3).
>
> **S5 ENTREGUE nesta sessão (commit local `dd1fee3`, `--no-verify`, NÃO pushado):** ver §1
> (linha S5) + §2 (o que foi feito + o que VALIDAR). **Próximo após o smoke do Enio:** tuning
> dos consts S5 se preciso, depois **S4 resto** (multi-pigmento K–M + multi-camada @4K) e
> as 5 propostas extraordinárias do [avaliacao_e_melhorias.md](Painter_projeto/avaliacao_e_melhorias.md).
> Leia §0 + §3 antes de tocar.

---

## §0 — Inegociáveis (memorize)

1. **Paridade é lei (HR-5).** A referência CPU (`crates/ph2d-painter-brush/src/diffusion.rs`,
   `DiffusionGrid`) é o ground-truth + det-fallback. **TODO passo GPU novo tem gate de paridade**
   vs a CPU (`crates/ph2d-painter-fluid/tests/{gpu_parity,composite_parity}.rs`, Metal `--ignored`).
   Os passos de shallow-water (S3d) batem **bit-a-bit (0 ULP)** com a CPU — mantenha assim:
   mude CPU e o espelho WGSL **identicamente**.
2. **Inner loop = `cargo check -p <crate>`.** Gates GPU 1× no fechamento, não por task. RAM 8 GiB → ≤3 cargos.
3. **Você NÃO pusha.** Commits locais `--no-verify`; ship (`./scripts/ship.sh` → push) é decisão do Enio.
   fmt/clippy/machete/deny/nextest **ainda não rodaram** nos commits desta saga.
4. **NÃO desfaça o modelo de cor recente** (§3.1). O campo de pigmento é **cobertura cinza**
   (`[dep/3;3]`), a cor vem do `pcol` (uniform), e a opacidade escala com o **Value** (`color_sum`).
   Isso conserta preto/escuro/Value — quebrar isso regride o que o Enio acabou de validar.
5. **Contratos congelados intactos** (§5): `FluidParams ≤ 12` (ADR-0049) **não foi tocado**;
   `WatercolorParams ≤ 16` (ADR-0079, **15 usados, 1 de folga**).

## §1 — O que está PRONTO + validado (não refazer)

| Estágio | Entrega | Commits | Gate |
|---|---|---|---|
| **S0–S3c** | Núcleo GPU-residente (`cs_splat`/`step_resident_splat`/`cs_reduce`) + region-scoped + deposição (`cs_transfer`/`cs_combine`) + perf (composite pipelined, fix do delay) | ver handoff anterior | smoke OK |
| **S3d** | **Shallow-water velocity → backruns.** `move_water` = `add_forces` + Jacobi `project`; pigmento advecta por `(u,v)`; `shallow.wgsl` (6 passes). Dormant quando `velocity=0` → look antigo. | `647c012`,`bd5038b` | `gpu_shallow_water_matches_cpu_move_water` **0 ULP** + 5 gates físicos CPU (anel de backrun) |
| **ADR-0079** | **15 controles per-brush** no Brush Studio (seção "Watercolor"): `WatercolorParams` DTO (cap ≤16) em `RenderingParams.watercolor`; bridge dirige via `FluidSolver::set_from_diffusion`; ids derivados por índice; round-trip panel↔tool | `d161c9c`,`18f9f1e` | 82 contract gates + round-trip ✓ **Enio validou os sliders** |
| **fix cor** | **Value preservado** (`pcol` = cor escolhida, não cromaticidade) + **preto/escuro pintam** (depósito = cobertura cinza independente de cor) + **opacidade ∝ Value** (`color_sum = 0.3 + 0.7·value`) | `185d4fc`,`07b918c`,`a2ecef4` | brush 343 + 17 GPU parity ✓ **Enio validou** |
| **fix params** | **Downhill** religado (preset 0); **Perm Valley/Crest + Viscosity** agora visíveis (perm gateia as forças do `add_forces`, CPU+GPU 0 ULP); ranges alargados | `185d4fc`,`979f22f` | 15 diffusion + parity ✓ **Enio validou** |
| **fix UX** | Pintura **transform-aware** (`ph2d_render::sprite_world_to_uv` inverte o `RenderInstance.basis` → pinta no texel certo com sprite movida/rotada/escalada) + **gizmo de ROTAÇÃO liberado no modo pintura** (só rotate cai pro gizmo; corpo pinta) | `9985bd3`,`f7741ad` | picking + painter_input ✓ **Enio validou** |
| **S5** | **Borda capilar (a franja macia).** `DiffusionGrid::capillary_flow` = difusão conservativa da ÁGUA (forma-divergência, condutância = `½(perm_c+perm_n)`, CFL ≤ 0.24, após evaporar) → a água molha o papel seco além da pintura, o gate abre lá, o pigmento sangra na franja. GPU `capillary.wgsl` (`cs_capillary`+`cs_copy_water`, ping-pong `water_b`). **Envelope cresce** (§2.2): `união(dab-bbox, wet-bbox de `read_field_stats`)+pad` no bridge, só com capilar on. 16º controle `Capillary` (folga do ADR-0079, cap ≤16). Dormant a 0. | local `dd1fee3` | `gpu_capillary_matches_cpu` **0 ULP** + 6 gates CPU (dormant/bounded/conserva/det/sangria/**invariante envelope** pig⊆wet-bbox) + 82 contract + 9 painel ✓ **⚠️ visual pendente (Enio)** |

## §2 — Borda capilar macia (S5) — ✅ FEITO (commit `dd1fee3`), ⚠️ visual pendente

**Objetivo (Curtis 1997, camada capilar):** a água **molha o papel seco ALÉM da área pintada**,
carregando um fio de pigmento → a **franja macia/penugenta** que define o look de aquarela. A borda
antiga era a do `wet-gate` (mais dura). Foi a etapa de maior impacto visual por esforço, **isolada**,
e que **não tocou o modelo de cor** (§0.4).

> **✅ O QUE FOI ENTREGUE (esta sessão):** os 4 passos do plano abaixo (§2.1 modelo CPU,
> §2.2 envelope, §2.3 GPU+param) estão **implementados + gated**. A paridade GPU↔CPU é
> **bit-exata (0 ULP em Metal)**. O envelope cresce pela união com a wet-bbox real. O 16º
> controle `Capillary` (preset **0.1** — franja ON por padrão) está no Brush Studio.
>
> **⚠️ O QUE VALIDAR (Enio) — `./play.command`, pincel de aquarela, traço grande NUM CANVAS
> GRANDE (arraste um PNG; no demo 64² a franja some — §3.5):**
> 1. A **franja macia/penugenta** sangra no papel seco ao redor do traço (a borda deixou de
>    ser dura). **Sem corte retangular** nas quinas (o envelope cresceu — era o bug §2.2).
> 2. O **slider "Capillary"** (Brush Studio → seção Watercolor, último): 0 = borda dura antiga;
>    subindo até 0.24 = franja mais larga/molhada. Tune o preset se 0.1 estiver fraco/forte.
> 3. **FPS:** o capilar adiciona 2 passes region-scoped/substep — confira que o traço live
>    segue fluido (`PH2D_FLUID_PROFILE=1 ./play.command` p/ o log por-frame). Se cair, o pad
>    do envelope (`CAPILLARY_FRINGE_PAD=8` em `painter_fluid_bridge.rs`) é o primeiro suspeito.
> 4. **Não-regressão:** um pincel com `Capillary=0` deve pintar **idêntico** ao S3d validado
>    (o caminho é dormant + o envelope não cresce).
>
> Consts a tunar (se pedir): preset `capillary: 0.1` em `watercolor.rs` (`WatercolorParams::default`);
> range em `CONTROLS` (`Capillary 0..0.24`); `CAPILLARY_FRINGE_PAD` no bridge.

### §2.1 — Modelo (determinístico, espelhável na GPU, HR-5)

Um passo novo `capillary_flow` por-substep (depois de `evaporate`): cada célula molhada **doa** uma
fração `capillary` da sua água pros vizinhos mais SECOS (outward-only wet→dry), limitada pela
capacidade do papel (não inunda). É uma **difusão gateada da ÁGUA** (não só do pigmento):
`water_n += capillary · perm · ½·max(water_c − water_n, 0)`, conservativa (o que sai de `c` entra em
`n`). À medida que a franja molha, o `wet-gate` abre lá → o pigmento existente sangra pra dentro
dela (pode precisar de um termo extra fino se o threshold do gate for abrupto). **Pura aritmética**
→ determinístico + replayável. **Limite a doação** (≤ capacidade + CFL) ou desestabiliza.

### §2.2 — A INVARIANTE QUE QUEBRA (o trabalho de verdade)

- **Hoje:** água **só evapora** → o bbox molhado marcha pra DENTRO → o envelope do composite =
  união monotônica dos **bboxes de dab** é um superconjunto seguro (vide §3.4 do handoff anterior +
  `DiffusionGrid::water_bbox` doc).
- **Com capilar:** a água espalha pra FORA → a área molhada **excede os bboxes de dab** → o composite
  **corta a franja** (a classe de bug "quinas retangulares" do Enio). **Este é o ponto central.**
- **Fix:** o envelope do composite precisa CRESCER pra incluir a área molhada pela capilaridade. Opções:
  - **(a)** rastrear a união do **wet bbox** real (`FluidSolver::read_field_stats` → `cs_reduce`) ao
    longo do traço, usar como envelope. Mas `read_field_stats` sincroniza a fila (esporádico) → o
    envelope atrasa; pad generoso compensa. **Recomendado**, com pad.
  - **(b)** crescer o envelope por um pad de alcance-capilar conhecido por-frame (spread ≤ N cél/frame).
    Mais simples/determinístico, mas super-pad.
  - **MEÇA o alcance real** da franja e dimensione o pad. O `SOLVER_REGION_PAD` (região do dispatch
    scoped, `solver.rs`) também tem de cobrir a franja (senão o solver não passa as células da franja).

### §2.3 — Sequência (igual S3a→S3b)

1. **Ref CPU primeiro.** Estenda `DiffusionGrid` com o passo capilar (difusão outward da água) +
   acessores. **Gates** (`--lib diffusion`): blob molhado em papel seco **espalha o wet-bbox pra fora**
   ao longo dos passos (a franja), **limitado** (capacidade), determinístico, conservativo (água); e o
   pigmento **sangra** pra franja (teste de tint fraco na borda). Dormant quando `capillary=0`.
2. **Espelho GPU.** `cs_capillary` (em `shallow.wgsl` ou shader novo) + wire em `step_resident_splat`
   + **gate de paridade** vs CPU (bit-exato como os outros). Naga-test do shader.
3. **Crescimento do envelope (§2.2).** Faça o envelope do composite acompanhar a área capilar; gate
   headless de "a franja é compositada, não cortada".
4. **Param.** Adicione **`capillary`** ao `WatercolorParams` (16º campo — **cabe na folga ≤16**) +
   entry em `CONTROLS` (label/range) + valor no preset (`Default`) + `to_diffusion` + `DiffusionParams`.
   O bridge já manda tudo via `set_from_diffusion`. Se precisar de 2 controles (rate + capacidade) →
   **estoura o cap 16 → emenda do ADR-0079** (Coord-only). Atualize o gate `WatercolorParams ≤ 16` se
   mudar o cap.
5. **Validação visual (Enio).** Traço de aquarela com **borda macia/penugenta** sangrando no papel.

### §2.4 — Riscos
- **Estabilidade:** limite a doação capilar (capacidade + CFL); franja não pode inundar → tune.
- **Clip da franja (§2.2):** o erro nº1; acerte o envelope ou a borda corta em retângulo.
- **Perf:** capilar adiciona 1 pass + estende a região (a franja cresce o dispatch). Region-scoped
  mantém `O(frente+franja)`; meça com `PH2D_FLUID_PROFILE=1 ./play.command`.
- **Paridade fp:** a CPU usa `f64`? não — tudo `f32`; mantenha a ordem das operações idêntica CPU/WGSL.

## §3 — Aprendizados CAROS (leia antes de mexer em cor/perf/envelope)

1. **O modelo de cor (NÃO desfaça — §0.4).** A composição lê o pigmento **só pela SOMA** (cobertura) e
   tira a cor do `pcol` (uniform). Então: (a) o dab deposita **cinza** `[dep/3;3]` (cobertura
   independente de cor — senão **preto = massa 0 = invisível**); (b) `color_sum = 0.3 + 0.7·value`
   (HSV value = canal max) → **opacidade ∝ escuridão** (cor escura é pigmento denso que cobre o papel,
   senão o K–M **queima** pro quase-preto — pesquisa confirma que K–M é ruim p/ cores escuras); (c)
   `pcol` = a cor escolhida (linear, clamped), não a cromaticidade → o Value lê. Em
   `crates/ph2d-painter-brush/src/wet_composite.rs` (`prepare_wet_composite_from_stroke`) + o depósito
   em `ph2d-tool-painter/src/tool/lifecycle.rs`.
2. **Perm gateia as FORÇAS, não a velocidade residente.** Multiplicar a velocidade residente por perm
   por-step DECAI o momento (composta). Gateie só a injeção de força no `add_forces`. Foi o fix que
   tornou Perm + Viscosity visíveis (a velocidade ganha a textura do papel via perm×paper, que a
   viscosidade então suaviza).
3. **A invariante do envelope monotônico** (§2.2) é load-bearing: a água só evaporava → o envelope era
   superconjunto barato. A capilaridade a QUEBRA — é o coração desta etapa.
4. **Meça a ESCALA do sintoma antes da causa** (handoff anterior §3): o "delay" era ⅓s O(grid) CPU, não
   frame. **bench-verde ≠ vivo.** Use `PH2D_FLUID_PROFILE=1`.
5. **"Baixa-res" pode ser o CANVAS** (64×64 demo): params espaciais (diffusivity, perm, granulação,
   capilar) somem na res pequena — teste arrastando um PNG grande. Foi por isso que Perm parecia "sem
   efeito" antes (mascarado + res). 
6. **Input é transform-aware agora.** Pintura usa `ph2d_render::sprite_world_to_uv` (inverte o
   `RenderInstance.basis`). Qualquer feature nova de input do painter deve passar por aí, não por AABB.

## §4 — Arquivos-chave

- **Ref CPU (paridade + física):** `crates/ph2d-painter-brush/src/diffusion.rs` (`DiffusionGrid`:
  `step`=move_water/diffuse/advect/transfer/evaporate; `move_water`=`add_forces`+`project`;
  `water`/`vel_u`/`vel_v`/`pressure`/`deposited`; `RELAX_ITERS`; `water_bbox`/`max_water`). **A franja
  capilar entra aqui primeiro.**
- **Solver GPU:** `crates/ph2d-painter-fluid/src/solver.rs` (`FluidSolver`: pipelines, `step_resident_splat`,
  `set_from_diffusion`, `read_field_stats`, `SOLVER_REGION_PAD`, `GpuParams` 96B) +
  `src/shader/{fluid,splat,reduce,transfer,combine,shallow}.wgsl`. `cs_capillary` mora aqui.
- **Composite (envelope!):** `crates/ph2d-painter-brush/src/wet_composite.rs`
  (`prepare_wet_composite_from_stroke`, `composite_canvas_region`, `composite_wet_field_cpu`) +
  `crates/ph2d-painter-fluid/src/shader/composite.wgsl` + `src/composite.rs` (`FluidCompositor`). **O
  crescimento do envelope (§2.2) toca o cálculo da região do composite.**
- **Params de brush:** `crates/ph2d-painter-brush/src/watercolor.rs` (`WatercolorParams` + `CONTROLS` +
  `to_diffusion`) + `src/rendering.rs` (`RenderingParams.watercolor`). **Adicione `capillary` aqui.**
- **Drive (shell):** `shells/desktop/src/render_loop/painter_fluid_bridge.rs` (`drive_fluid_gpu`:
  epoch setup, `set_from_diffusion(painter.fluid_diffusion_params())`, region-scoped step, pipelined
  composite, `PH2D_FLUID_PROFILE`). **O envelope que o composite recebe vem daqui.**
- **Hooks do tool:** `crates/ph2d-tool-painter/src/tool/lifecycle.rs` (`begin_stroke`, `fluid_take_dabs`,
  `fluid_diffusion_params`, o **depósito cinza** `[dep/3;3]`, `wet_pigment_envelope`).
- **Input transform-aware:** `crates/ph2d-render/src/picking.rs` (`sprite_world_to_uv`) +
  `shells/desktop/src/input_dispatch/painter_input.rs` (`painter_pointer_uv`).
- **Lançador:** `./play.command` (release, `--features fluid`).

## §5 — Contratos (mexer = ADR, CLAUDE.md §6)

- **`WatercolorParams ≤ 16`** (ADR-0079, gate `architecture_painter_contract_surface` em
  `crates/ph2d-painter-contracts/`): **15 usados, 1 de folga.** `capillary` cabe; um 2º controle estoura
  → emenda do ADR-0079 + bump do cap.
- **`FluidParams ≤ 12` / `FluidSim ≤ 12`** (ADR-0049): **intactos** (a aquarela vive no `WatercolorParams`
  do brush + `DiffusionParams` interno, não-capado).
- **`RenderingParams ≤ 14`**: 13 usados (1 de folga — `watercolor`).
- **`RenderInstance.basis`** (ph2d-render, ADR-0070-amendment-4): contrato de render que o input inverte;
  não mexer sem ADR.

## §6 — Build / rodar / validar

```bash
# App (release, fluid):
./play.command                              # ou PH2D_FLUID_PROFILE=1 ./play.command (log por-frame)
# Física CPU (referência — a franja capilar começa aqui):
cargo test -p ph2d-painter-brush --lib diffusion
# Gates de paridade GPU (Metal, --ignored — rode no fechamento):
cargo test -p ph2d-painter-fluid --features fluid --test gpu_parity --test composite_parity -- --ignored --nocapture
# Contract gates (caps):
cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
# UI/plumbing:
cargo test -p ph2d-panel-brush-studio          # round-trip painel↔tool
cargo test -p ph2d-render --lib picking         # input transform-aware
# Inner loop:
cargo check -p ph2d-painter-brush   # ou -p ph2d-painter-fluid --features fluid ; -p ph2d-host-desktop --features fluid
```

**Validação visual (Enio):** pincel de aquarela, traço grande **num canvas grande** (arraste um PNG) →
borda macia/penugenta sangrando no papel (a franja capilar), sem corte retangular; cor escura cobre o
papel; perm crisp↔fluindo; rotação via gizmo no modo pintura.

— deixado por Claude (sessão 2026-06-08: S3d shallow-water + ADR-0079 controles per-brush + fidelidade
  de cor/Value + perm/viscosity + input transform-aware + gizmo-rotate no modo pintura — tudo validado
  pelo Enio). **Próximo: borda capilar (S5, §2)**, em contexto fresco. O envelope que cresce (§2.2) é o
  trabalho de verdade; o modelo de cor (§0.4/§3.1) NÃO se desfaz.

— atualizado por Claude (sessão 2026-06-08, cont. — coordenador+implementador solo): **S5 (borda
  capilar) COMPLETO** — commit local `dd1fee3` (`--no-verify`, NÃO pushado). CPU ref
  `capillary_flow` (difusão conservativa da água, forma-divergência, CFL ≤ 0.24, após evaporar) +
  espelho GPU `capillary.wgsl` (`cs_capillary`+`cs_copy_water`, ping-pong `water_b`) com **paridade
  bit-exata 0 ULP em Metal** + **crescimento do envelope** (§2.2: união dab-bbox + wet-bbox monotônica
  do `read_field_stats` + `CAPILLARY_FRINGE_PAD`, só com capilar on) + 16º controle `Capillary` (preset
  0.1, folga do ADR-0079, cap ≤16 intacto). Gates: 349 brush lib (6 capilar + invariante envelope) · 12
  GPU parity · 82 contract · 9 painel. **⚠️ Pendente: validação VISUAL do Enio** (§2, a caixa "O QUE
  VALIDAR"). Nota anti-colisão: `shells/desktop/src/input_dispatch/gizmo_drag.rs` tem WIP TEMP do Enio
  (`PH2D_GIZMO_DEBUG`) NÃO commitado — fora do meu escopo, deixado intacto. **Próximo após o smoke:**
  tuning S5 se preciso → S4 resto (K–M multi-pigmento + multi-camada @4K).

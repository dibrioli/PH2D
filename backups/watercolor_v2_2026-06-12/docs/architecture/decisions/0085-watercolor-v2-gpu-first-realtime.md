# ADR-0085 — Watercolor v2: arquitetura GPU-first real-time (supersede a paridade bit-a-bit CPU↔GPU)

- **Status:** Accepted (2026-06-10) — ratificado pelo Enio (R0 smoke: "ratifico! GO"). O mandato do
  Enio (`docs/HANDOFF_watercolor_v2_refactor.md` §0) autorizou a direção; este texto foi o sign-off.
- **Decisor(es):** Enio (dono/decisor) + Claude (Coord, sessão Watercolor v2).
- **Supersede:** a regra de **paridade bit-a-bit CPU↔GPU** do [ADR-0049 amendment-1](0049-fluid-brushes.md#7-amendment-1--solver--gated-diffusion-advection-w153-2026-06-07)
  (§2/§3: "o GPU **espelha** o `diffusion.rs` bit-near, lane-for-lane") e as cláusulas de paridade
  0-ULP de [ADR-0080 §2.5](0080-watercolor-km-multipigment-field.md), [ADR-0081], [ADR-0082]. Ajusta a
  promessa de **fallback CPU em tempo-real** do [ADR-0049 §2.3/§2.8/§2.11](0049-fluid-brushes.md) e a
  **device-matrix** do [ADR-0053](0053-cross-platform-tier.md).
- **Estende / preserva:** [ADR-0078](0078-watercolor-gold-standard-resident-tiled-shallow-water.md)
  (motor difusão-gateada + deposição + shallow-water + capilaridade), [ADR-0080](0080-watercolor-km-multipigment-field.md)
  (campo K–M espectral 24 bandas — **fica**), [ADR-0083](0083-4k-fullres-watercolor-field-residency.md)
  (residência 4K + união monotônica — **fica**).
- **Tags:** painter, fluid-sim, watercolor, gpu-first, perf, contract-surface, real-time, supersedes

---

## 1. Contexto — a paridade bit-a-bit é o imposto que trava a topologia do frame

Um dia inteiro de band-aids de perf (`HANDOFF_painter_fluid_perf_block.md`, SUPERSEDED) perseguiu o
sintoma errado. A medição final com o `[gpu]` GPU-timestamp profiler (Enio, Metal, 8 GB, Immediate,
1408×768) crava a causa REAL de "pintar derruba o FPS" — e **não é o compute**:

```
[frame] total=52.50ms (~19 fps) | cpu-encode(raw)=2.49ms | present/acquire-stall=50.02ms
[gpu] passes/frame=21 | Σ kernels reais < 5ms (advect 2.2 / capillary 1.2 / ...)
       copy.slot=9.67ms  fluid.comp_tex=8.22ms   ← cópia full-canvas + comp por frame
```

Os kernels do sim somam **<5ms reais**. O custo está na **estrutura do frame**, em três pontos:

1. **Sobre-submissão:** ~12–15 `queue.submit()`/frame pintando (vs 4–6 sem fluido). Cada submit é um
   ponto de sincronização no Metal — o driver não sobrepõe trabalho através deles.
2. **Cópia do canvas INTEIRO todo frame:** `copy_texture_into_individual(id, tex, cw, ch)` copia o
   canvas inteiro (1408×768 RGBA16F ≈ 8.6 MB R+W/frame; 4K ≈ 67 MB+) quando só a região molhada mudou.
3. **Readback no hot path do traço:** `composite_frame_pipelined` mantém `canvas_rgba` (CPU) atual
   TODO frame de stroke — só precisa estar atual no **pen-up**.

O `present/acquire-stall` de 50ms é **backpressure** real: a fila de GPU está profunda de tanto submit
+ cópia full-canvas; o `acquire_frame` espera a GPU drenar. Atacar (1)+(2)+(3) encurta a fila → o stall
colapsa.

**O bloqueador para consertar isso limpo são os ~740 gates de paridade bit-a-bit CPU↔GPU**
(`gpu_parity.rs` 99 asserts + `composite_parity.rs` 59 asserts + os 0-ULP de ADR-0080/0081/0082).
Eles foram introduzidos quando o GPU foi adicionado como **espelho** do `diffusion.rs` CPU (ADR-0049
amendment-1) para servir de **fallback determinístico** (HR-5). Mas esse contrato:

- **Trava a topologia do solver e do compositor** numa **forma-de-CPU** (gather kernels, contagem fixa
  de Jacobi par, lane-for-lane, ordem-de-ops idêntica) — exatamente a estrutura que a §1 acima precisa
  reestruturar (merge de encoders, direct-render, sparse).
- **Custa 2808 LOC** (`diffusion.rs`) de twin CPU mantido em sincronia bit-a-bit, hot-path-morto desde
  W15.3 (o caminho vivo já é GPU-residente).
- **Não entrega o que prometia.** O "fallback CPU em tempo-real" (ADR-0049 §2.11) roda a **~5
  strokes/s** a 256² (a própria tabela da ADR-0049 §2.11) — não é tempo-real; é uma promessa de
  device-matrix que nenhum artista usaria.

**Decisão de produto (Enio, mandato §0):** "Aqui tudo será simulado em tempo real." A GPU é a fonte da
verdade do look vivo. O twin CPU bit-a-bit deixa de ser fonte da verdade. Isto é **arquitetura, não
tuning** — e a refatoração GPU-first e o conserto de perf são **o mesmo trabalho**.

---

## 2. Decisão

### 2.1 Paridade bit-a-bit CPU↔GPU → **invariantes físicos** (não bit)

A regra "o GPU espelha `ph2d_painter_brush::diffusion` bit-exact, lane-for-lane" (ADR-0049
amendment-1, ADR-0080 §2.5 0-ULP, ADR-0081/0082) é **superseded**. No lugar, a corretude do sim é
guardada por **gates de invariante físico**, com tolerâncias frouxas, rodando sobre a saída GPU:

- **Conservação de massa de pigmento** (`mass`/`ks_acc`/`err_acc` extensivos): a massa total só decai
  por deposição/evaporação documentadas, nunca cresce do nada (tolerância relativa, não ULP).
- **Limites de campo:** água ∈ `[0, w_max]`, sem NaN/Inf, sem runaway (envelope molhado não cresce
  monotonicamente sob evaporação > 0).
- **Simetria:** um dab radialmente simétrico em papel uniforme difunde simétrico (tolerância de
  device-FP).
- **Monotonia da secagem:** sob evaporação > 0 e sem deposição nova, a água total é não-crescente.
- **Equilíbrio (Keep Wet):** sob evaporação 0 a poça atinge estado estacionário (não cresce pra
  sempre) — guarda o bug de creep da §6b do handoff por física, não por settle-freeze de timeout.

Os algoritmos GPU ficam **livres** para usar a melhor forma (scatter com atomics, contagem de
iterações ímpar, fused passes) desde que satisfaçam os invariantes acima. Os gates `gpu_parity.rs` /
`composite_parity.rs` são **removidos na onda R3** (não em R0 — a math não muda em R1/R2; ver §6).

### 2.2 GPU-compute **requerido**; CPU vira oráculo offline, não twin em tempo-real

- **Aquarela exige compute-GPU.** A promessa de "fallback CPU em tempo-real" do ADR-0049 (§2.3
  device-matrix, §2.8 graceful-degrade-para-wet-mix-matemático-via-CPU-sim, §2.11 det-mode 256²
  como production-adjacent) é **retirada**. Não há sim de fluido na CPU no caminho vivo.
- **Degradação graciosa redefinida (ajusta ADR-0053 tier policy):** device sem compute-GPU capaz →
  **aquarela desabilita** (o brush cai para os modos não-fluidos: wet-mix matemático puramente de
  composite, ou stamp seco), **não** um sim CPU lento. A identidade "molhada" via wet-mix matemático
  de composite (sem solver) permanece como o degrade honesto; o que sai é a tentativa de rodar **o
  solver** na CPU em tempo-real.
- **HR-5 / determinismo cross-OS:** a garantia determinística aplica-se à **GEOMETRIA dos traços
  vetoriais** (`.ph2d-painter`, ADR-0046) — o replay re-roda o sim GPU e produz um resultado
  **visualmente equivalente, não bit-idêntico** cross-device (FP de GPU difere por device; bit-idêntico
  de fluido é fisicamente impossível e ninguém precisa). O que é determinístico e versionado é a
  ENTRADA (stamps, params, ordem), não os pixels do fluido.
- **Oráculo CPU offline (opcional, não-twin):** o que sobrar do `diffusion.rs` (§2.5) pode rodar
  offline para **validar invariantes** (§2.1) e, se barato, o **bake determinístico de reproject** —
  mas **sem espelhamento bit-a-bit** do caminho vivo.

### 2.3 O novo contrato de PERF (substitui os budgets soltos do ADR-0049 §2.10/§2.12)

A arquitetura-alvo Watercolor v2 (handoff §2, invariantes I1–I4), como contrato executável onde der:

1. **Single-submit hot path.** Splat + substeps + composite vão para **um `CommandEncoder`, um
   `queue.submit`** por frame de stroke. Meta: do hot-path de ~5 submits de fluido para **1**.
   *Gate (R5):* contar submits do caminho de fluido no hot path = 1 (instrumentação do
   `pass_profiler`).
2. **Zero cópia full-canvas por frame.** A textura de preview do compositor **é** a textura amostrada
   pelo sprite (`PreviewOverride`), sem `copy_texture_into_individual` do canvas inteiro. Se o store
   exigir posse, copiar **só a dirty-rect**. *Gate (R5):* `assert dirty_rect ⊊ canvas` no hot path
   (região copiada < canvas sempre que a água ativa < canvas).
3. **Zero readback no hot path do traço.** `canvas_rgba` (bake CPU) só se atualiza no **pen-up** (e no
   `flush_pending_bake` síncrono do pointer-down que protege o undo — **FICA**). Mid-stroke o preview
   é 100% GPU-residente. *Gate (R5):* nenhum `map_async`/readback de canvas no caminho de stroke
   mid-gesture.
4. **Sim + composite O(bbox de água ATIVA), não O(grid) nem O(união all-time).** O STEP e o COMPOSITE
   por-frame escopam ao bbox atual de água viva (stats GPU, com pad). A **união monotônica continua
   dimensionando/semeando a TEXTURA persistente** (cobre áreas que secaram e precisam de UM composite
   final — cerca de Chesterton do ADR-0083, **não apagar**); só o TRABALHO por-frame é sparse.

Estes substituem `fluid_perf_budget_under_3p5_ms_4k` (ADR-0049 §2.15) e os budgets soltos do §2.10/2.12
como o contrato de perf vigente. A medição é o `[gpu]` GPU-timestamp profiler (`PH2D_FLUID_PROFILE`),
não wall-span (que o TBDR do Metal infla — ver `feedback_measure_perf_symptom_scale`).

### 2.4 Destino do `diffusion.rs` — **reduzir a oráculo/bake-offline, não deletar**

`diffusion.rs` (2808 LOC, hot-path-morto desde W15.3) **encolhe drasticamente** (handoff §4):
- Sai: tudo que existia só para o espelhamento bit-a-bit do caminho vivo (o twin lane-for-lane).
- Fica (no máximo): um **oráculo de referência** que valida os invariantes da §2.1 offline + (se
  barato) o **bake determinístico de reproject**. Sem sincronia bit-a-bit com a GPU.

Recomendação: **reduzir, não deletar** — preserva o oráculo de invariantes e o bake, ambos baratos. A
decomposição HR-18 do `solver.rs`/`composite.rs` (handoff §4) anda junto, mas não é normativa neste
ADR (é trabalho de R3/R5, não contrato).

### 2.5 O que NÃO muda (reafirmado — §8 do handoff)

- **ABIs de superfície/UI/persist (ADR-0043–0051):** `Stamp = 96B align(16)`, `RenderingMode = 6`,
  `ColorProfile = 8`, `Brush ≤ 168`, `AdjustmentKind ≤ 32`, `PainterUiEdit ≤ 24`, `DeviceTier = 5`,
  history `.ph2d-painter` (ADR-0046). **Congelados.** Os caps `FluidSim ≤ 12` / `FluidParams ≤ 12` /
  `GravitySource ≤ 6` (ADR-0049 §2.13, gate `architecture_painter_contract_surface::fluid`) **ficam** —
  são superfície, não física.
- **Modelo de cor K–M espectral (ADR-0080):** 24 bandas K/S + err + mass + stain, `PIG_CH = 32`, base
  7-curvas em `pigment_mix.rs`. **Fica** — é o melhor que temos. Só perde o espelhamento 0-ULP do
  §2.5 daquele ADR (vira GPU-first; a corretda da mistura subtrativa é guardada por invariante de
  conservação, não por ULP).
- **O look validado (ADR-0081/0082/0084):** blooms, edge-darkening, granulação, franja capilar,
  branching dendrítico, sheen úmido, lift/backdrop-lift. Cada um foi ratificado visualmente pelo Enio.
  A reescrita **preserva o look**, validado por paridade VISUAL (não bit) com o Enio, onda a onda.
- **Residência 4K + união monotônica (ADR-0083):** o teto de storage-buffer e a textura persistente
  dimensionada pela união all-time ficam.

---

## 3. Impacto em contratos congelados

| Contrato | ADR | Impacto |
|---|---|---|
| Paridade bit-a-bit CPU↔GPU (`gpu_parity`/`composite_parity`, 0-ULP) | 0049-am1 / 0080 §2.5 / 0081 / 0082 | **SUPERSEDED** → invariantes físicos (§2.1). Gates removidos em R3. |
| Fallback CPU em tempo-real + det-mode 256² production-adjacent | 0049 §2.3/§2.8/§2.11 | **RETIRADO.** GPU-compute requerido; CPU = oráculo offline (§2.2/§2.4). |
| Device-matrix / tier policy (low-tier = sim CPU) | 0049 §2.12 / 0053 | **AJUSTADO.** Low-tier sem compute = aquarela off, não sim CPU (§2.2). |
| Budgets de perf soltos (`<3.5ms 4k`) | 0049 §2.10/§2.15 | **SUBSTITUÍDOS** pelo contrato de perf da §2.3 (single-submit / no full-copy / no stroke-readback / O(bbox ativo)). |
| Caps de superfície `FluidSim`/`FluidParams`/`GravitySource` | 0049 §2.13 | **INTACTOS** — superfície, não física. |
| K–M espectral 24 bandas, `PIG_CH=32` | 0080/0081 | **INTACTO** (perde só o 0-ULP). |
| ABIs `Stamp`/`RenderingMode`/`ColorProfile`/`Brush`/etc. | 0043–0051 | **INTACTOS** (congelados). |
| Residência 4K + união monotônica | 0083 | **INTACTO** (cerca de Chesterton). |

**Gate de contrato (`architecture_painter_contract_surface.rs`):** a `mod fluid` (caps de superfície)
**não muda** em R0. A remoção dos gates de paridade (`gpu_parity.rs`/`composite_parity.rs`, que vivem
no crate `ph2d-painter-fluid`, não no contract-surface) acontece em **R3** junto com a mudança de math.
Em R0, o header do contract-surface ganha uma nota apontando para este ADR; os novos gates de perf
(§2.3) são adicionados quando o código de R1/R5 os tornar testáveis.

---

## 4. Consequências

### Positivas
- **Destrava a reestruturação do frame** (single-submit, direct-render, sparse) que a §1 exige e que a
  paridade bit-a-bit impedia. É o que entrega o ganho de FPS que o Enio espera (R1).
- **−2808 LOC de twin CPU** (`diffusion.rs` encolhe) + −158 asserts de paridade = menos imposto de
  manutenção, mais chance de caçar os bugs da água (mandato §0.6).
- **Liberdade algorítmica na GPU:** a math pode usar a melhor forma (R3), não a forma-de-CPU.
- **Promessa de device honesta:** GPU-compute-required + degrade explícito (aquarela off em hardware
  sem compute) substitui a promessa de sim-CPU-em-tempo-real que nunca foi real (~5 strokes/s).

### Negativas / Custos
- **Perde-se a verificação cross-OS bit-idêntica do fluido.** Mitigação: bit-idêntico de fluido GPU é
  fisicamente impossível cross-device e ninguém precisa; o determinismo que importa (geometria do
  traço, `.ph2d-painter`) é preservado (§2.2). Os invariantes físicos (§2.1) cobrem corretude.
- **Hardware sem compute-GPU perde aquarela** (degrada para não-fluido). Mitigação: o público-alvo da
  aquarela física já é tier médio/alto (ADR-0049 §2.12 era ≥80% mid-tier); o degrade é honesto.
- **Trabalho de reescrita não-trivial** (R1–R5). Mitigação: faseado por risco, cada onda com smoke
  visual do Enio (handoff §7); R1 isolado entrega o ganho sem mexer na math.

### Neutras
- O `ph2d-painter-fluid` continua crate folha, feature-gated `fluid`. O default build não muda.
- A cerca de Chesterton da união monotônica (ADR-0083) e o `flush_pending_bake` (proteção de undo)
  ficam — são corretos, não dívida.

---

## 5. Alternativas consideradas

### 5.1 Manter a paridade bit-a-bit e otimizar perf por dentro dela
**Rejeitada.** Foi o que um dia inteiro de band-aids tentou (`HANDOFF_painter_fluid_perf_block.md`).
Os fixes (epsilon-clamp, gate Curtis, settle-freeze, decimação idle) **funcionam e ficam** (o idle
recupera ~60fps), mas mascaram; a causa estrutural (submits/cópia/readback) **não é tratável** enquanto
740 gates travam a topologia do solver/compositor numa forma-de-CPU. A paridade é o imposto (§1).

### 5.2 Deletar o `diffusion.rs` inteiro
**Rejeitada (recomendação: reduzir).** O oráculo de invariantes e o bake determinístico de reproject
são baratos e úteis. Deletar joga fora a referência offline. Encolher para oráculo non-twin é o
meio-termo correto (§2.4).

### 5.3 Manter o sim CPU como fallback de tempo-real (não retirar a promessa)
**Rejeitada.** A própria ADR-0049 §2.11 mede ~5 strokes/s a 256² — não é tempo-real. Manter a promessa
é manter uma ficção de device-matrix. O degrade honesto é aquarela-off em hardware sem compute (§2.2).

### 5.4 Trocar K–M espectral por modelo mais barato na reescrita GPU-first
**Rejeitada.** O K–M espectral 24 bandas (ADR-0080) é o que produz azul+amarelo→verde subtrativo real,
ratificado pelo Enio. É o look, não o imposto. **Fica** (§2.5). O que sai é só o 0-ULP, não o modelo.

---

## 6. Verificação e ondas (handoff §7)

R0 (este ADR) é **só contrato** — sem código de runtime. As ondas seguintes:

```sh
# Inner loop (cada onda):
cargo check -p ph2d-painter-fluid --features fluid --tests
cargo check -p ph2d-host-desktop --features fluid          # o bridge

# Medição (smoke visual do Enio, cada onda):
PH2D_FLUID_PROFILE=1 ./play.command
#   R1 alvo: pintando, present-stall colapsa (≈0), passes/frame cai (1 submit de fluido), ~60fps.

# Fim de onda (Metal --ignored):
#   gpu_parity / composite_parity  → válidos ATÉ R3 (a math não muda em R1/R2); removidos em R3.
#   pass_profiler_gpu / layer_compositor_gpu / architecture_painter_contract_surface.
```

- **R0 (este ADR):** Coord-only. ADR-0085 + nota no header do contract-surface. **Smoke: Enio ratifica
  o texto.**
- **R1:** single-submit + direct-render + no-stroke-readback (§2.3 I1–I3). Math inalterada; paridade
  ainda passa. **Smoke: FPS a ~60 pintando.**
- **R2:** sparse tiles (§2.3 #4). **Smoke: wash/canvas grande mantém FPS.**
- **R3:** remove gates de paridade (§2.1); encolhe `diffusion.rs` (§2.4); a math pode mudar; constrói
  os gates de invariante físico (§2.1) e de perf (§2.3). **Smoke: look intacto + poça em equilíbrio.**
- **R4:** borda fininha de deposição (realismo, handoff §6a). **Smoke: anel escuro fino no wash seco.**
- **R5:** fechamento HR-18 + gates de perf executáveis + smoke cross-feature.

---

## 7. Tracking

- Handoff vivo: [`docs/HANDOFF_watercolor_v2_refactor.md`](../../HANDOFF_watercolor_v2_refactor.md).
- Perf-block SUPERSEDED (registro da investigação): [`docs/HANDOFF_painter_fluid_perf_block.md`](../../HANDOFF_painter_fluid_perf_block.md).
- Plano vivo: [`docs/Painter_projeto/15_plano_de_implementacao.md`](../../Painter_projeto/15_plano_de_implementacao.md) (W15).
- Pesquisa estado-da-arte (validar a física antes de mexer): [`docs/Painter_projeto/pesquisa_aquarela_estado_da_arte.md`](../../Painter_projeto/pesquisa_aquarela_estado_da_arte.md).
- Memória: `project_watercolor_v2_gpu_first_refactor`, `feedback_measure_perf_symptom_scale`.

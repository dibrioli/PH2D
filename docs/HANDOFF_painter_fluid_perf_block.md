# HANDOFF — Painter Fluid PERF block (A): the fixed per-frame GPU stall (2026-06-10)

> ⛔ **SUPERSEDED por [`HANDOFF_watercolor_v2_refactor.md`](HANDOFF_watercolor_v2_refactor.md)
> (2026-06-10).** Enio mandou abandonar a estratégia incremental: a causa da pintura-lenta NÃO é
> "custo fixo" nem "envelope grande" (perseguidos aqui) — é a **topologia do frame**
> (submit-bound + cópia full-canvas + readback no hot path), e o conserto limpo exige reescrita
> GPU-first (largar o twin CPU bit-a-bit). Os fixes JÁ LANDADOS deste bloco **ficam e funcionam**
> (crash water/water, undo multi-step, sprite-wide paint, stats pipelinado, epsilon-clamp
> `WATER_EPS`, gate Curtis `CAPILLARY_MIN_SATURATION`, settle-freeze do Keep Wet, decimação idle —
> o idle recupera a ~60fps). Leia este doc só como REGISTRO da investigação (a §1/§4b/§10e têm a
> evidência que fundamenta o novo handoff). O trabalho ativo é o v2 refactor.
>
> **Bloco fundacional dedicado** (ADR-0083 tiling/sparse + `project_painter_fluid_4k_perf_architecture`).
> Executar em **contexto fresco e focado** — toca o caminho vivo do motor de aquarela (4000+ gates de
> paridade). Leia inteiro antes de mexer; a §2 (a pista) é a mais cara.

## §0 — O QUE JÁ LANDOU nesta sessão (commit local, sem push — fast-mode `--no-verify`)

Tudo compila (`-p ph2d-render --tests`, `-p ph2d-painter-fluid --features fluid --tests`,
`-p ph2d-host-desktop` default **e** `--features fluid`). Testes não-GPU verdes (picking, layer_compositor
ABI/naga, undo). Gates GPU `--ignored` (Metal) compilam — **rodar 1× pra confirmar**.

| Fix | O quê | Arquivos | Estado |
|---|---|---|---|
| **CRASH water/water** | `composite_frame` (sync) re-mapeava o `staging` ainda mapeado pelo readback PIPELINED → erro wgpu → processo aborta. Agora drena o `pending` antes de re-mapear (espelha `begin_stroke`). | `composite.rs` | ✅ corrigido na raiz; **Enio re-testar** |
| **stats poll(wait) crescente** | O dry-check `read_field_stats` fazia `poll(wait_indefinitely)` (drena a fila GPU inteira) → crescia 0.5→6ms num runaway. Novo `read_field_stats_pipelined` (async, staging persistente, lê o disparo anterior; dry-check tolera 1 ciclo). Síncrono mantido p/ os gates. | `solver.rs`, `painter_fluid_bridge.rs` | ✅ **PROVADO no log do Enio: 6ms → 0.008ms** |
| **Bug 2: sprite sempre paintável** | Gate `[0,1)` do quad derrubava samples na borda → `break_stroke_segment` → sim estancava. Novo `sprite_world_to_uv_unclamped`; clamp a jusante (envelope + cutoff de splat). | `picking.rs`, `lib.rs`, `painter_input.rs` | ✅ + unit test |
| **Bug 3: undo multi-step** | `canvas_rgba` é assado com atraso (catch-up pós pen-up); o snapshot do próximo traço pegava estado stale → 1 undo apagava vários. `flush_pending_bake` assa o pendente sincronamente no pointer-down, antes do `begin_stroke`. | `painter_fluid_bridge.rs`, `painter_input.rs` | ✅ **visual pendente** |
| **Bug 1: cadeia E5 region-scoped (multi-camada 4K)** | A cadeia E5 refazia 3 ops full-canvas/frame (`Region::full`). Novo `composite_region_into_canvas` (escreve em coords-canvas num `out` persistente) + inject/slot-copy sub-rect + seed-full no 1º frame. Premul fica full (barato). | `compositor.rs`, `mod.rs` (layer_compositor), `layer_composite.wgsl`, `individual.rs`, `renderer.rs`, `painter_gpu_preview.rs`, `painter_fluid_bridge.rs`, `painter_fluid_support.rs` | ✅ compila + gate Metal `composite_region_into_canvas_refreshes_region_and_preserves_outside`; **NÃO validado visual** (o teste do Enio foi 1 camada → E5 nem rodou). Vale p/ o objetivo multi-camada 4K. |
| **Instrumentação `[frame]`** | Profiler de frame (total / cpu-encode-raw / present-acquire-stall / painter-dispatch), gated no `PH2D_FLUID_PROFILE`. É o ponto de partida deste bloco. | `mod.rs` (render_loop) | ✅ ligado |

## §1 — O PROBLEMA (não resolvido, é este bloco)

FPS despenca pintando **aquarela com água**, e **fica baixo mesmo parado depois de pintar**. Medido
(Enio, Metal, Mac **8 GB**, present **Immediate**, canvas **1408×768**):

```
[fluid] step=1.0ms composite=0.16ms stats=0.008ms total≈1.2ms     ← submit de CPU do drive (enganoso)
[frame] total=134ms (~7fps) | cpu-encode(raw)=2.81ms | present/acquire-stall=131.67ms | painter-dispatch=0.01ms
```

**Lido corretamente:** a CPU encoda em **2.8ms**; os **131ms estão na execução da GPU** (o `acquire_frame`
espera a GPU drenar — em Immediate, logo NÃO é vsync; é backpressure de trabalho real de GPU). O profiler
`[fluid]` mede o **submit de CPU** (~1ms), **não** a execução na GPU — por isso o fluido "parecia" inocente.

## §2 — A PISTA QUE MUDA TUDO (Enio, 2026-06-10) — leia com atenção

> **"Uma única pequena mancha de água em Immediate já era suficiente para a queda de FPS."**

Isso **refuta** a hipótese de envelope-crescente / pressão de RAM (que eu persegui). Uma mancha PEQUENA já
dá 131ms → o custo é **FIXO por frame, INDEPENDENTE do tamanho do molhado**, disparado por ter **qualquer
campo vivo**. Não é O(área), não é 4K-scaling, não é memória. É **uma operação de GPU pesada de custo
fixo que roda todo frame quando um campo de fluido está vivo** — bug muito mais específico e tratável que
"tiling pra 4K". **Keep Wet** só piora porque mantém o campo vivo pra sempre (o custo fixo nunca para).

## §3 — DESCARTADOS (com evidência) vs SUSPEITOS (a investigar)

**Descartado:**
- CPU encode (raw=2.8ms), painter-dispatch CPU (0.01ms), o preview CPU.
- present-mode/vsync (Immediate também trava — é backpressure de GPU).
- envelope-crescente / RAM (mancha pequena já trava — §2).
- o `stats` (pipelinado, 0.008ms).
- caminho CPU `on_tick`/`tick_wet_field` (gated por `fluid_hires`; o `[fluid]` imprime ⇒ drive GPU ativo).

**Suspeitos (precisam de GPU-timestamp pra cravar) — algo full-canvas/fixo POR FRAME quando há campo vivo:**
1. **`copy_preview_into_slot`** (`painter_fluid_support.rs`): `copy_texture_into_individual(id, tex, cw, ch)`
   — copia o canvas **inteiro** (cw×ch) GPU→GPU **todo frame**, ignorando a região molhada. Candidato #1.
2. **Shallow-water S3d sempre-ligado** (`WATERCOLOR_VELOCITY=1.3` etc.): ~11 passes/substep
   (add_forces+divergence+2 clear+6 Jacobi+project). Region-scoped, mas verificar se algum pass/`project`
   tem custo fixo full-canvas ou convergência cara. Foi adicionado recente e "perf live nunca medida".
3. **Sheen** (`publish_wet_sheen_between_strokes`): composite EXTRA por frame ocioso enquanto molhado+ShowWet.
4. **`cs_reduce`** (stats) full-canvas com atomics — sporadic (20 frames), provavelmente não, mas medir.
5. **Múltiplos `queue.submit()` por frame** (step, composite, copy, inject, stats) → splits de encoder /
   serialização no Metal. Medir se a soma de submits estoura.
6. **map_async/poll(Poll)** dos readbacks pipelinados (composite + stats) interagindo com o `acquire` do
   present sob Immediate.

## §4 — PLANO

**Passo 1 (medir — NÃO pular): ✅ IMPLEMENTADO (sessão 2026-06-10b).** GPU-timestamp profiler landado:

- **`ph2d-gpu/src/pass_profiler.rs`** (novo): `QuerySet` TIMESTAMP global (512 queries/frame), API
  `compute_writes(label)`/`render_writes(label)` (attach no descriptor do pass), spans p/ copies
  (`copy_span_begin/end` — marker passes vazios no mesmo encoder) e p/ submits estrangeiros
  (`span_begin/end` — Vello). `end_frame` (chamado no `present.rs` após o compositor) resolve + readback
  PIPELINADO (ring de 3, zero poll blocking) e imprime a cada 120 frames AMOSTRADOS:
  `[gpu] avg ...: gpu-busy(span)=Xms passes/frame=N` + tabela `label=ms(×passes)` ordenada desc.
  Inerte sem `PH2D_FLUID_PROFILE=1` (todas as fns viram no-op; `timestamp_writes: None`).
- **Feature `TIMESTAMP_QUERY`** pedida em `context.rs` (interseção com o adapter — não falha nunca).
- **Instrumentado:** solver per-kernel (`fluid.splat/forces/divergence/clear_p/jacobi/project/lift/
  diffuse/advect_v/advect/transfer/evaporate/capillary/combine/stats`), composite por entrada
  (`fluid.comp_sync/comp_pipe/comp_tex/comp_straight/comp_init/comp_buffer/comp_rows`), copies
  (`copy.slot`/`copy.region` em `individual.rs`), os 4 passes principais (`render.sprite/tonemap/
  compositor` + bracket `render.vello`), `render.clip/premul/layer_comp`. Linha `[fluid-ctx]`
  (bridge, a cada 120 frames) dá grid/região/substeps/dabs — distingue custo-fixo de O(área).
- **PROVADO no Metal:** gate `ph2d-gpu/tests/pass_profiler_gpu.rs` (`--ignored`) — kernel ocupado de
  1M threads mede 4-5ms reais, zero validation errors, report imprime. Paridade intacta
  (gpu_parity 13✓ + composite_parity 19✓ + layer_compositor_gpu 25✓¹).
- ⚠️ **Caveat Metal:** spans de marker (copy/vello) podem SUB-medir se o trabalho bracketed não tem
  hazard com os markers (Metal sobrepõe encoders independentes). No app real há hazards (vello escreve
  a intermediate que o compositor lê; o copy escreve o slot que o sprite pass sampleia), então a ordem
  é forçada — mas leia esses dois labels como piso, não teto. Os passes REAIS (timestamps próprios)
  são exatos. Se `gpu-busy(span) >> Σ labels` → o custo está ENTRE passes (overhead de
  scheduling/encoder boundaries — ~40 passes/frame no fluido) ou em submits não-instrumentados.

¹ `gpu_composite_50_layers_dirty_rect_under_5ms` é borderline (mediana 4.81ms vs cap 5ms): falha no run
cheio sob carga da máquina, passa isolado. Pré-existente, não-relacionado (profiler OFF nos testes).

## §4b — VEREDITO DA MEDIÇÃO (Enio, 2026-06-10, Metal, 1408×768, scale=1, Keep Wet ON)

Quatro relatórios `[gpu]`+`[fluid-ctx]` durante/pós strokes de água. Três conclusões:

**1. NÃO é custo fixo — é O(envelope), e o ENVELOPE é um runaway monotônico.** A prova está no
`[fluid-ctx]` entre relatórios: `region` foi `(873,3)-(1407,641)` → `(833,0)` → `(819,0)` → `(810,0)-
(1407,655)` — **crescendo ~10 células/janela MESMO IDLE com `dabs=0`/`stroke_active=false`**, já
encostado nas bordas do canvas (y=0, x=1407), cobrindo ~390k células (~36% do canvas). Mecanismo
(positive feedback): o solver pada a região (`SOLVER_REGION_PAD=6`), diffuse/capillary empurram um
FILME numérico de água pro pad, o `read_field_stats(threshold=1e-4)` captura, `wet_bbox` (união
monotônica) cresce, o pad anda, repete. Keep Wet zera a evaporação ⇒ o filme nunca morre ⇒ cresce até
o canvas inteiro e FICA — exatamente os sintomas do §1/§2 ("mancha pequena já trava"; "baixo mesmo
parado"). O custo parece "fixo" porque o envelope satura na escala do canvas independente da mancha.

**2. Onde o tempo de GPU vai (passes compute, confiáveis):** `fluid.comp_tex≈36-39ms` (composite ss=2
sobre o envelope, TODO frame — incl. idle via sheen/`comp_pipe`) + step somado ≈30ms (`advect_v` 10-11
×6, `capillary` 6.5 ×4, `transfer` 5 ×2, `diffuse` 4 ×2, `combine` 2.2, `lift` 1.3, jacobi/etc <1)
≈ **65-70ms de GPU/frame ≈ o frame de 65-86ms medido (12-15fps)**. Bate com o modelo de BANDWIDTH:
campo 32-canais = 128B/célula, stencil 5-pontos ≈ 768B/célula/pass × 392k células × ~20 passes ≈
~5GB/frame ÷ ~70GB/s ≈ 70ms — **bandwidth-bound**, como o ADR-0083 previa pra 4K. Não há "um pass
bugado": é o produto envelope × canais × passes.

**3. Artefatos de medição (não persiga):** `render.sprite/tonemap/vello/copy.slot` mostrando 36-66ms
são WALL-SPAN inflado (TBDR separa vertex/fragment; encoders sobrepõem) — um tonemap fullscreen não
custa 60ms reais. `gpu-busy(span)` 190-620ms ≫ frame = fila multi-frame profunda (backpressure),
consistente. Os suspeitos #1 (copy full-canvas) e #5/#6 (submits/map_async) estão DESCARTADOS como
causa primária; #2 (shallow-water) é só uma fatia proporcional do O(envelope).

**Passo 2 (corrigir — na ordem):**
- **2a. Matar o runaway do envelope** (o bug específico): (i) **epsilon-clamp da água** — célula com
  `water < EPS` (~1e-3?) vai a 0 no evaporate/capillary para o filme numérico não se propagar (sob
  keep-wet o clamp é o ÚNICO freio); (ii) reavaliar o threshold do bbox (1e-4 captura névoa invisível;
  o comentário diz que é pra fringe THIN — calibrar contra o fringe REAL visível). ⚠️ Paridade: o CPU
  reference (`ph2d-painter-brush::diffusion`) tem que receber o MESMO clamp (gates `*_matches_cpu`),
  e validação visual com o Enio (fringe/blooms intactos) — é mexer no motor validado.
- **2b. Idle ≠ stroke:** com pointer up o sim+composite rodam full-cadência sobre o envelope inteiro
  (sheen recomposita todo frame). Depois do 2a o envelope idle encolhe ao molhado REAL; se ainda
  pesar, baixar a cadência idle (step/composite a cada 2-3 frames) é aceitável-a-validar.
- **2c (= Passo 3, o objetivo maior): frente molhada ativa / tiling esparso** (ADR-0078 S1b /
  ADR-0083): o trabalho POR FRAME escopa ao bbox ATUAL de água (das stats GPU, com pad), não à união
  all-time; a união monotônica continua dimensionando/semeando a TEXTURA persistente (cerca de
  Chesterton das "quinas retangulares" — ela existe pra cobertura do composite de áreas que SECARAM,
  que precisam de UM composite final e depois ficam estáticas). Validação visual estágio-a-estágio
  com o Enio (blooms, edge-darkening, granulação, sheen).

## §5 — COMO RODAR / MEDIR
```bash
PH2D_FLUID_PROFILE=1 ./play.command   # [fluid] (drive CPU) + [frame] (total/raw/stall) + [gpu] (passes GPU) + [fluid-ctx]
# Repro: pincel água, Keep Wet ON, UMA mancha pequena → já cai pra ~7fps (custo fixo, §2).
# O [gpu] imprime a cada 120 frames AMOSTRADOS (sob 7fps ≈ 20-40s; aguarde 2 relatórios).
cargo test -p ph2d-gpu --test pass_profiler_gpu -- --ignored        # round-trip do profiler no Metal
cargo test -p ph2d-painter-fluid --features fluid --test gpu_parity --test composite_parity -- --ignored
cargo test -p ph2d-render --test layer_compositor_gpu -- --ignored   # inclui o novo gate region-into-canvas
```

— sessão 2026-06-10: bugs de input/undo/crash/stats fechados; FPS isolado como GPU-execution-bound de
**custo fixo por frame** (§2).
— sessão 2026-06-10b: **Passo 1 implementado, provado E MEDIDO** (§4b): a hipótese "custo fixo" do §2
caiu — é O(envelope) bandwidth-bound, com o envelope em runaway monotônico (cresce idle, satura no
canvas, permanente sob Keep Wet). Próximo: **Passo 2a** (epsilon-clamp da água + threshold do bbox,
CPU+GPU em paridade, validação visual) → 2b/2c.
— sessão 2026-06-10c: **Passo 2a IMPLEMENTADO** (validação visual pendente — Enio):
  - **Epsilon-clamp** `WATER_EPS = 1e-4` (canônico em `ph2d_painter_brush::diffusion`, espelho
    literal em `fluid.wgsl::cs_evaporate`): pós-evaporação, célula `< EPS` vai a 0 — o trickle
    sub-1e-4 nunca acumula, então a névoa numérica (o runaway) morre; inflow por-step ≥ EPS
    acumula normal. ⚠️ **1e-3 foi testado e REJEITADO**: matava o halo cromatográfico (o teste
    `capillary_water_wicks_ahead_of_pigment` caiu — o halo se constrói de inflows em [1e-4,1e-3)
    que acumulam legitimamente). 1e-4 = o piso do flow-gate existente (`g ≤ 1e-4`).
  - **Threshold do bbox** 1e-4 → `WET_BBOX_WATER_THRESHOLD = 1e-3` (o contorno do fringe REAL
    visível — a constante dos próprios testes de fringe; pigmento ⊆ wet_bbox(1e-3) pelo
    invariante §2.2 + `CAPILLARY_FRINGE_PAD = 8` cobre a margem de ~2 células).
  - **Contrato de conservação re-codificado**: pigmento exato (inalterado); água agora tem leak
    UNIDIRECIONAL limitado (a mist da frente que o clamp destrói, ~0,2%/40 steps — é o freio;
    asserts `leak ∈ (−FMA, 0.5%)` nos 2 testes de conservação).
  - **Verde**: 371 CPU (painter-brush) + 217 tool-painter + Metal `gpu_parity` 19✓ +
    `composite_parity` 13✓ + `contract_surface` 8✓. Arquivos: `diffusion.rs`, `fluid.wgsl`,
    `painter_fluid_bridge.rs`.
  - **Validação (Enio)**: `PH2D_FLUID_PROFILE=1`, pincel água + Keep Wet ON, mancha pequena →
    (a) `[fluid-ctx] region` deve ESTABILIZAR idle (não crescer ~10 células/janela); (b) FPS
    deve recuperar pós-stroke; (c) visual: halo transparente/blooms/edge-darkening/granulação/
    fingers do branching intactos. Se ainda pesar idle → 2b (cadência idle) e o objetivo maior
    2c (frente ativa / tiling esparso, bbox ATUAL vs união all-time).
— sessão 2026-06-10d (medição do Enio sobre 2a: "sem melhoras" — envelope AINDA crescia idle,
  114k→128k células com dabs=0): o clamp 1e-4 freou a névoa numérica mas o crescimento restante
  era **água REAL** — capillary é difusão pura sem terminação; sob Keep Wet (evaporação 0) a
  física espalha pra sempre. Dois fixes (validação visual pendente):
  - **Gate de saturação Curtis δ_s (`CAPILLARY_MIN_SATURATION = 0.005`)**: face capilar só flui
    enquanto o DOADOR (lado mais molhado, `max(wc,wn)`) excede o piso — "o pavio esgota"
    (camada capilar do Curtis 1997 canônico: transferência só acima de saturação mínima).
    Simétrico na face ⇒ fluxo anti-simétrico ⇒ conservação + bit-parity. Equilíbrio LIMITADO:
    a frente para onde a borda dilui < δ_s (envelope ≲ água_total/δ_s). Gate na FONTE — a
    cerca "NOT the wet gate" do doc do `capillary_flow` é sobre o RECEPTOR (wick pra célula
    seca segue aberto). Calibração: 0.02 e 0.01 REJEITADOS (halo cromatográfico caiu a 1-2
    células; teste exige ≥3); 0.005 → halo 3 ✓ todos os capillary_* verdes. CPU `face` closure
    + espelho `capillary.wgsl::face_info`.
  - **2b cadência idle (`IDLE_STEP_EVERY = 3` no bridge)**: pointer up + 0 dabs ⇒ step +
    composite/sheen/readback só a cada 3 frames; frames pulados republicam a textura
    persistente do slot (campo estático, nada a recompor). Segurado enquanto
    `texture_mode_dirty` pendente (latência do bake pós pen-up inalterada); frames pintando
    nunca decimam. Custo idle esperado ≈ ⅓ (comp_tex 12.6ms + comp_pipe 4.9ms + step ~10ms
    eram TODO frame). Dinâmica idle evolui a ⅓ do wall-clock (aceitável-a-validar do plano).
  - **Refactor HR-18**: braço de readback extraído pra `run_readback_lane` em
    `painter_fluid_support.rs` (bridge tinha passado de 600 LOC).
  - **Verde**: 371 painter-brush + Metal gpu_parity 19✓ + composite_parity 13✓ + host-desktop
    88✓ (incl. LOC cap) + check default e `--features fluid`.
  - **Validação (Enio)**: igual à 2a acima; agora o esperado é o `region` PARAR de crescer
    (não só desacelerar) e o stall idle cair ~3×. Se o fringe ficou curto demais no visual,
    δ_s desce (0.003/0.002 — re-rodar `cargo test -p ph2d-painter-brush capillary`); se o
    envelope ainda incomodar em wash grande, é o 2c (frente ativa / tiling esparso).
— sessão 2026-06-10e (medição do Enio sobre 2a-bis+2b: passes fluid em multiplicidade CHEIA
  todo frame idle + creep na MESMA taxa pré-δ_s):
  - **Bug: decimação nunca engatou.** O skip exigia `texture_mode_dirty == None`, mas sob
    backpressure (~16fps) o readback pipelinado retorna bands intercalados com vazios → o
    contador "2 consecutivos" reseta sempre → dirty NUNCA limpa → gate permanentemente
    desligado (o `comp_pipe ×0.1-0.3` no [gpu] foi a prova). Fix: condição removida; em vez
    dela um warmup full-cadence de 6 frames pós pen-up (`IDLE_WARMUP_FRAMES`); a correção do
    bake continua sendo o `flush_pending_bake` no pointer-down (qualquer "espere o catch-up"
    pode estolar pra sempre sob backpressure — lição).
  - **Creep idle ≠ capillary.** A taxa não mudou com o δ_s ⇒ quem espalha no app vivo é a
    **shallow-water** (`WATERCOLOR_VELOCITY=1.3` sempre-on): com evaporação 0 é física de
    poça — espalha pra sempre. O δ_s fica (termina a cauda capilar), mas o freio do Keep Wet
    é semântico: **settle-freeze** (`KEEP_WET_SETTLE_FRAMES = 180`): após ~180 frames idle
    sob Keep Wet o campo CONGELA inteiro (zero step/composite/creep; a textura úmida
    publicada é só republicada). Dab/stroke descongela (`idle_frames = 0`) e pinta no wash
    ainda-molhado como antes. Keep Wet = "tinta trabalhável", não "wash invade o canvas".
    Secagem normal (Keep Wet off) nunca congela.
  - **Esperado na próxima medição**: idle pós-settle → fluid passes ZERO no [gpu], region
    estático, FPS ≈ baseline sem fluido; entre pen-up e settle → passes a ⅓. **Pintando**
    continua ~19fps com envelope grande (comp_tex ~30ms + step ~20ms sobre 280k células TODO
    frame de stroke) — esse é o **2c** (ADR-0083: step+composite escopados à região ATIVA,
    não ao envelope inteiro; contexto fresco recomendado).

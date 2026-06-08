# HANDOFF — W15.3 GPU watercolor composite (DONE, smoke OK 2026-06-07)

> **W15.3 está FECHADO** ([ADR-0049](architecture/decisions/0049-fluid-brushes.md) + amendment-1):
> a aquarela wet-on-wet roda inteira no GPU (solver + composite K–M), sem o stall de readback de
> pigmento, e foi **ratificada visualmente pelo Enio** ("smoke ok. Corrigiu!"). Este doc é o registro
> do que ficou + os **aprendizados** (leia §3 — o mais caro). Tudo commitado local, sem push.

---

## §1 — O QUE FICOU (pipeline final, `--features fluid`)

Por frame, `drive_fluid_gpu` (`shells/desktop/src/render_loop/painter_fluid_bridge.rs`):
1. pigmento **GPU-residente** em `solver.pig_a`; os dabs do frame sobem como **deposit aditivo**
   (`cs_deposit`) + diffuse/advect no GPU (sem evaporate GPU, **sem readback de pigmento**).
2. **água = espelho CPU** (sobe pro gate; a CPU evapora + faz o dry-check → sem readback de água).
3. **composite GPU** (`composite.wgsl`, K–M espectral + **2×2 supersample AA**) lê `pig_a` + backdrop
   sobre o **envelope molhado monotônico**; só a faixa de linhas molhada volta pra `canvas_rgba`.
4. dry-check (drop só **após pen-up**), epoch reset, full-res (`scale=1`) em GPU capaz.

Paridade GPU↔CPU **0 LSB** (testes `composite_parity` + `gpu_parity`, `--ignored`, Metal). CPU path
(`composite_wet_field`) é o fallback bit-near. `play.command` roda com `--features fluid`.

## §2 — BUGS ENCONTRADOS + CORRIGIDOS (a jornada)

| Sintoma (Enio) | Causa-raiz | Fix | Commit |
|---|---|---|---|
| (gate) | — | shader K–M provado bit-exato | `2d0f5d3` |
| (seam) | — | composite lê `pig_a` residente | `5e535d5` |
| (integração) | — | resident drive (deposit + composite + row readback) | `2cbc823` |
| Recortes retangulares no traço curvo | apply escrevia a faixa **full-width** → apagava colunas fora da bbox | apply só as colunas `[px_lo,px_hi)` | `807d30c` |
| Bordas serrilhadas | cobertura amostrada 1×/pixel (borda íngreme em traço opaco) | **2×2 coverage supersampling** (AA), espelhado CPU+WGSL | `dbd49ab` |
| — | half-res era budget de CPU | full-res `scale=1` GPU-condicional | `4e3c2ee` |
| Fluid morria no fim do traço após pausar | pausa seca o campo em ~0.3s → drop no meio do traço | drop só `dry && !stroke_active` | `eb184cf` |
| **"Quinas retangulares"** (mesmo em full-res) | composite usava a **bbox da ÁGUA**, que **recua** na evaporação enquanto o pigmento (conservado) fica espalhado → mancha redonda cortada num retângulo | composite sobre o **envelope monotônico** (união de todas as bboxes molhadas; nunca recua) + pad 1→2 células | `2b1b0a0` |
| **"Baixa resolução nas bordas"** | **o canvas é 64×64** (sprite de demo, `ATLAS_SPRITE_PX`); aquarela "full-res" de 64px é minúscula com zoom de ~12× | **não é o pipeline** — pintar em canvas maior (ver §3) | — |

Auditoria multiagêntica (39 agentes) achou o `2b1b0a0` (envelope). 2 itens low-sev deferidos (§4).

## §3 — APRENDIZADOS (o caro — leia antes de mexer em aquarela/render)

1. **"Baixa resolução" pode ser o tamanho do CANVAS, não o sim/shader.** O painter edita o sprite na
   resolução **nativa** (`read_sprite_source` usa `img.width/height`, sem upscale). Os sprites de demo
   são **64×64** (`ATLAS_SPRITE_PX=64`, `shells/desktop/src/integration.rs`). Render de borda macia
   (aquarela) em 64px, exibido a ~800px (~12× zoom), VIRA borrão — independente de `scale=1`/`2` (64 vs
   32, ambos minúsculos). Brush duro parece nítido (borda seca → blocos com transição seca); aquarela
   macia → mush. **Antes de caçar escala de sim/shader, cheque a resolução real do canvas/source.**
   Pra testar alta-res: **arraste um PNG grande** (importa via `DroppedFile` → `import_image_at_camera`
   na res nativa). Gap em aberto: o painter ("sucessor do Procreate") só edita os sprites pequenos do
   atlas — não há "novo canvas" grande dedicado (§4).
2. **`water_bbox ≠ extensão do pigmento` sob evaporação.** Água só evapora (bbox recua); pigmento é
   conservado e difusão/advecção até empurram pigmento 1 célula PRA FORA do gate. Compositar sobre a
   bbox da água corta a mancha. Use o **envelope molhado all-time** (limite superior real) ou a bbox de
   pigmento. O comentário "wet ⊇ pigment" em `diffusion.rs` era falso (corrigido) — foi a cerca de
   Chesterton que plantou o bug.
3. **Paridade-verde ≠ caminho-real-exercido.** Todos os `composite_parity` passavam `region=
   (0,0,gw-1,gh-1)` → `composite_canvas_region` clampa pro canvas inteiro → o caminho de **bbox
   apertada** (onde mora o clip) NUNCA era testado. "0 LSB" era verdade e irrelevante. Os 2 testes de
   regressão novos (`fluid_gpu_envelope_never_recedes_under_evaporation`,
   `composite_region_must_cover_pigment_or_it_clips`) cobrem o caminho real.
4. **Verifique hipótese plausível, não descarte por raciocínio de poltrona.** Levantei "water vs
   pigment bbox" cedo e **descartei errado** (assumi `water⊇pigment`), custando rounds. Um teste de
   região-apertada teria provado/refutado na hora.

## §4 — PERF: arquitetura real-time 4K + multi-camada (PRÓXIMO BLOCO — fundacional)

> Pedido do Enio: pintar em imagens **até 4K**, **pinturas animadas grandes em múltiplas camadas**,
> **tudo em tempo real** — "o melhor do mundo, melhor que o Procreate". Os fixes de W15.3 (SS=1 full-res,
> buffers persistentes, pre-warm, GPU-clear) deram **~10%**. Isso NÃO escala pra 4K porque o gargalo é
> **estrutural** (custo por-frame `O(grid)` em CPU + transferências CPU↔GPU), não micro-otimização.

### Medido (Metal, M-series, canvas **1408×768**, brush 32px, `--release`)
| etapa por-frame | ~ms @1408 | natureza | @4K (×~16 área) |
|---|---|---|---|
| `fluid_frame_step_inputs` | ~2.0 | CPU `O(grid)`: **alloc** Vec água+depósito + **scan** wet-bbox + evaporate + clear | ~32ms |
| `step_resident` (upload+GPU) | ~2.0 | **upload** água+depósito full-grid (transfer `O(grid)`) + diffuse/advect GPU | ~32ms |
| `composite_frame` | ~1.5 | composite GPU (já SS=1) + **`device.poll(wait)`** readback da faixa | ~6–24ms |

→ a soma em 4K estoura o orçamento de 16ms (60Hz) **só com a água+pigmento de UMA camada**. Multi-camada
multiplica. Root cause: **a água e o pigmento NÃO são GPU-residentes** — todo frame a CPU aloca/varre o
grid inteiro e faz upload full-grid; o composite ainda faz **um readback síncrono** que serializa GPU↔CPU.

### Alvo arquitetural (mata os 3 custos `O(grid)` de uma vez)
1. **Sim GPU-residente (água + pigmento + paper):** já existe `pig_a` residente + `step_resident`; estender
   pra **água residente** (sobe 1× no `begin_stroke`, nunca mais) → **elimina o upload de água por-frame**.
2. **Splat GPU por lista-de-dabs (`cs_splat`):** a tool empurra uma `Vec<DabGpu>{cx,cy,r,water,rgb}`
   (pequena, `O(dabs)` ~dezenas) em vez do depósito full-grid; `cs_splat` soma na água+pigmento residentes
   → **elimina o upload de depósito + o alloc/clear `O(grid)` da CPU**. ⚠️ **paridade de forma**: o
   `cs_splat` tem que reproduzir o splat da CPU (`r.max(0.5)`, cutoff, perfil) **bit-a-bit** ou o traço
   muda de cara — testar headless contra o splat CPU + validar visual.
3. **Evaporate GPU no `step_resident`:** `cs_evaporate` já existe no `fluid.wgsl`; rodar sobre a água
   residente → **elimina o evaporate `O(grid)` da CPU**. ⚠️ **sinal de dry-check** sem o grid de água CPU:
   ou (a) redução GPU max-água → readback **esporádico** (a cada N frames), ou (b) a CPU rastreia uma
   **massa escalar** de água (splat soma, evaporate subtrai `k`) — (b) é `O(1)`, preferível.
4. **Wet-bbox por redução GPU:** o envelope molhado (pra compositar só a região ativa) sai de uma redução
   min/max GPU em vez do **scan `O(grid)` da CPU** → readback de 4 u32 (esporádico, com folga de pad).
5. **Composite → textura de preview, SEM readback por-frame:** o composite escreve direto numa textura que
   o renderer amostra; o `device.poll(wait)` por-frame some (GPU pipeline à frente da CPU). Readback **1×
   no pen-up** pra assar `canvas_rgba` (a camada canônica do Apply/undo). ⚠️ plumbing: o drive
   (`mod.rs:242`) não tem o renderer/slot — o slot vive no `painter_bridge::dispatch`; passar o alvo.
6. **Multi-camada:** com cada camada GPU-residente, o compositor de camadas (já GPU, ADR-0048) encadeia
   sem voltar pra CPU; o custo vira `O(Σ dabs)` + composites GPU, não `O(Σ grids)` em CPU.

### Plano em estágios (cada um valida sozinho — visual + perf na tela; commit local, push só após Enio OK)
- **E1** água residente + evaporate GPU + massa-escalar dry-check → tira upload-de-água + evaporate-CPU.
- **E2** `cs_splat` + lista-de-dabs (paridade headless vs splat CPU) → tira upload-de-depósito + alloc CPU.
- **E3** wet-bbox por redução GPU → tira o scan CPU; readback esporádico de 4 u32.
- **E4** textura de preview sem readback por-frame; readback 1× no pen-up.
- **E5** medir @1408 e @4K; encadear multi-camada.

→ depois de E1–E4 o custo por-frame vira `O(dabs)` + passes GPU; o `O(grid)` da CPU **desaparece** do hot
loop. Esse é o caminho pra 4K/multi-camada em tempo real. **Recomendação:** executar em **contexto fresco
e focado** (reescrita fundacional do hot-path solver/tool/render-loop, validação visual estágio-a-estágio).

### ⏳ EM EXECUÇÃO (2026-06-08) — núcleo GPU-residente landado (E1+E2+E3 fundidos) — SMOKE OK ✓ (Enio), E4 em andamento
> **Descoberta de design:** E1/E2/E3 NÃO são separáveis como o plano sugeria — o espelho de água da CPU
> alimentava 3 consumidores (bbox-scan E3, evaporate E1, dry-check E1) e a entrada de dab alimentava água
> (E1) + pigmento (E2). No instante em que a entrada vira GPU-only (`cs_splat`), a CPU perde a água e
> bbox+evaporate+dry-check quebram **juntos**. Então E1+E2+E3 viram **um** núcleo coerente "sim residente".
>
> **Landado (3 commits locais, sem push):**
> - `693b6f3` **cs_splat** (WGSL + `DabGpu`/`splat_dabs`) — lista-de-dabs → splat direto na água+pig
>   residentes; dispatch único na union-bbox, cada célula varre a lista NA MESMA ORDEM da CPU →
>   forma exata (só FMA ~8e-8). Gate `cs_splat_matches_cpu_splat` (<1e-6, medido 8e-8).
> - `1d31dc5` **step_resident_splat** (splat + diffuse/advect/**evaporate** residentes, sem upload/readback;
>   bate <1e-6 vs CPU splat+step) + **`cs_reduce`/`read_field_stats`** (max-water + wet-bbox por 1 pass
>   atômico + readback de 5 u32, esporádico) substituindo os scans `O(grid)` `max_water`/`water_bbox`.
>   max-water bit-idêntico (bits IEEE monotônicos p/ water≥0); bbox exato. Gates verdes.
> - `8772132` **wiring live**: o drive por-frame troca o caminho-depósito (upload full-grid + alloc/scan/
>   evaporate CPU `O(grid)`) pelo caminho dab-list residente. `queue_pointer` captura dabs numa lista
>   (`FluidDab`) + cresce o envelope monotônico das bboxes-de-dab (superset do water-bbox → nunca corta);
>   `fluid_take_dabs`/`fluid_dry_check_and_drop_gpu`. **Gate em `fluid_hires`** (≠ `gpu_fluid_driven`,
>   que só é setado após o drive do frame) p/ não perder os PRIMEIROS dabs do traço. 214 testes lib verdes
>   + novo `gpu_resident_path_captures_dabs_to_list_not_grid`; ambos os crates `check` com `--features fluid`.
>
> **Mata, por-frame:** alloc do Vec depósito, clone do mirror de água, **upload full-grid água+depósito**,
> evaporate CPU, scans CPU `max_water`+`water_bbox` — toda a linha "fluid_frame_step_inputs ~2ms +
> step_resident upload ~2ms" da tabela §4. **Resta:** o composite + seu readback de faixa por-frame (= **E4**).
>
> **Caminho-depósito MANTIDO** como rede de segurança (`fluid_frame_step_inputs`, solver `step_resident`,
> `cs_deposit`) + seus gates até o Enio validar visualmente; removível depois.
>
> **PENDENTE (Enio):** smoke visual e2e — `./play.command` (release, `--features fluid`), pintar um traço de
> aquarela e confirmar que continua igual (forma/bloom/secagem) e mais fluido. *Unit-verde ≠ vivo no produto.*
>
> **FOLLOW-UP 4K-memory (E5, NÃO feito):** o `DiffusionGrid` da CPU ainda é alocado por-traço só p/
> paper+dims+existência (`O(grid)` **1×/traço**, não por-frame). 4K real quer **paper-gen no GPU** + dropar
> o grid CPU. O custo por-frame já é `O(dabs)`+passes GPU; o `O(grid)`/traço de alloc+paper continua.

### 🎯 NORTE: ADR-0078 — padrão-ouro definitivo (ratificado pelo Enio 2026-06-08)
[ADR-0078](architecture/decisions/0078-watercolor-gold-standard-resident-tiled-shallow-water.md): aquarela
física **Curtis 1997 de 3 camadas** (shallow-water velocidade + deposição/granulação + capilar
backruns/edge-darkening) + Kubelka–Munk multi-pigmento, a **4K, multi-camada, 60–120Hz**, via engine
**GPU-residente tiled-sparse** (`O(frente molhada)`) integrada como **nó do compositor**. Supera Procreate/
Fresco (aproximação) e Rebelle (não é 4K-multicamada-real-time). A difusão de ADR-0049 vira graceful-degrade;
referência CPU vira det-fallback. Estágios S0..S5.

**Medido (Metal `--release`) — bench headless `perf_resident`:**
| canvas | step+composite (traço típico) | nota |
|---|---|---|
| 1408×768 | **1.8ms** | era 3.4ms pré-S1 |
| 2048×2048 | **4.8ms** | era 8.2ms |
| 3840×2160 (4K) | **6.5ms** | era 13.1ms — ~10ms de folga sob 60Hz |
(wash de canvas cheio fica ~21ms — região = grid inteiro, o pior caso real.)

**Estágios:**
- **S0** núcleo GPU-residente (dab-list) — ✅ smoke OK (693b6f3, 1d31dc5, 8772132).
- **S1a** passes do solver **region-scoped** (`O(frente)`, bit-exato dentro da região; invariante
  solver⊇composite) — ✅ commit `0ec2978`, 2× em 4K típico. **PENDENTE: re-validação visual do Enio**
  (muda o caminho vivo — risco classe-§2 apesar do teste bit-exato).
- **S1b** active-tile set + indirect dispatch (regiões disjuntas) + dropar grid CPU + paper-gen GPU.
- **S2** composite como nó do compositor + zero readback por-frame (foundational `ph2d-render`) + bake no pen-up.
- **S3a** ✅ commit `734c30e` — **referência CPU da camada de deposição** (Curtis `TransferPigment`):
  `deposited` layer + `transfer_pigment` (edge-darkening via `deposition_dry·(1−gate)` + granulação via
  `granulation·(1−paper)`), conservativa, **dormante por default** (0 = look atual, 8/8 gates GPU verdes).
  Tuning em `DiffusionParams` (não-capado); `FluidParams` (≤12 frozen) intacto. 4 testes invariantes verdes.
- **S3b** ✅ commit `bbe4446` — GPU `cs_transfer` (espelho da deposição), **parity-exato** (worst |Δ|
  flowing+deposited = 0.000000): buffer `deposited` + pass region-scoped + `set_deposition`/`read_deposited`/
  `deposited_buffer`/`clear_resident_deposited_gpu`; `GpuParams` 64→80B (interno, `FluidParams` intacto).
  Dormante em produção (composite ainda lê só flowing). 9/9 gates GPU verdes.
- **S3c** ✅ commit `d253b8f` — **deposição VISÍVEL**. Decisão de design: NÃO mexer no composite (parity-gated)
  nem no `FluidParams` (≤12 congelado) — um passo `cs_combine` escreve `total = flowing + deposited` e o
  compositor liga `total` (uma ligação, como antes) via `total_buffer()`. Deposição liga por **constantes**
  do solver (`WATERCOLOR_DEPOSITION_BASE/_DRY/_GRANULATION`), não amendment. Bridge: epoch limpa deposited +
  `set_deposition(consts)` + liga `total_buffer`. Deposited assa no `canvas_rgba` (persiste após secar).
  CPU fallback = difusão pura (degrade). 10/10 gates GPU verdes. **PENDENTE: validação visual** (anel escuro
  na borda + granulado no papel). Tuning nas 3 constantes.
- **S2 (perf)** ✅ commits `7dea61f` + `c7d4d9f` — **readback pipelined**. Profiler (Metal, 32px, demo 64²)
  apontou: step 0.27ms / **composite+readback 2.6ms** / stats 0.25ms = 3.2ms (RAW 250→140). O 2.6ms era
  puro sync: `composite_frame`'s `device.poll(wait)` drenava a fila GPU **inteira** (incl. render da UI)
  todo frame; a transferência em si é 0.03ms. Fix: `composite_frame_pipelined` — `poll(Poll)` não-bloqueante
  + lê o band do frame **anterior** (1 frame atrasado, imperceptível); byte-idêntico ao síncrono (gate
  `composite_frame_pipelined_matches_sync`). Pós-fix: composite **2.6→0.14ms**, total **3.2→0.82ms**, RAW
  não cai mais. **Delay clique→traço** (o 1º composite vinha vazio): 1º frame do traço é **primado
  síncrono** (aparece na hora; hitch único de ~2.6ms só no clique). **stats** virou a maior fase (0.41ms,
  o `poll(wait)` do dry-check) → `DRY_CHECK_EVERY` 6→20. **Pendências:** stats async (matar o último hitch
  periódico); textura-alvo S2 puro (zero-readback de banda, escala 4K); investigar warn "dropped sim time".
- **S3d** campo de velocidade shallow-water (MoveWater + pressure relax) → fluxo direcional + blooms fortes.
- **S4** multi-pigmento K–M + multi-camada @4K. **S5** BFECC + supersampling adaptativo + capilar LBM (MoXi) + 120Hz.

## §5 — EM ABERTO (deferidos menores, não-bloqueantes)

- **Canvas de pintura grande** (o gap real pra aquarela brilhar): hoje só edita sprites 64×64 do atlas
  ou imagens importadas. Decisão do Enio: "novo canvas" 1024²/2048², ou subir `ATLAS_SPRITE_PX` (muda o
  atlas — `integration.rs`, regenera fixtures), ou sempre pintar em imagens importadas.
- **gate de perf-headroom** (`fluid_pass_eligible(.., f32::INFINITY)` é no-op) — auditoria low-sev.
- **AA de splat em raio minúsculo** (`r.max(0.5)` + cutoff `d>=1.0` quebra brushes <3px) — low-sev.

— deixado por Claude (sessão brush-overhaul + W15.3 GPU composite — FECHADO, smoke OK, 2026-06-07;
  arquitetura perf 4K em §4, pronta pra executar — 2026-06-08).

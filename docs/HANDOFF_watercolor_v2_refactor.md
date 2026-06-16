> ⚠️ **SUPERSEDED por [ADR-0096](architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md) (Enio 2026-06-14):** toda a simulação de aquarela/fluido/wash foi **REMOVIDA** do código (crate `ph2d-painter-wash` deletada, canvas voltou a CPU-residente). Doc mantido só como histórico. Norte atual = **Brush Engine (mixer-brush)**, ver [`docs/Novo Painter/`](Novo%20Painter/). Backups em `backups/wash_2026-06-14`.

# HANDOFF — Refatoração GPU-first do motor de aquarela (Watercolor v2)

> **Bloco fundacional, contexto fresco e dedicado.** Toca o caminho vivo do motor de aquarela
> (ph2d-painter-fluid + ph2d-painter-brush + o bridge no shell). Leia ESTE doc inteiro antes de
> tocar código. Substitui a estratégia incremental de `HANDOFF_painter_fluid_perf_block.md`
> (que está SUPERSEDED — ver §1). Mandato do Enio em 2026-06-10 após um dia inteiro de
> band-aids de perf sem resultado.

---

## §0 — O MANDATO (Enio, 2026-06-10) — leia primeiro, é a bússola

Palavras do dono/decisor, traduzidas em lei de projeto:

1. **GPU-first, tempo-real-only.** "Aqui tudo será em tempo real, aqui tudo será simulado em
   tempo real." Abandona-se o conceito de **fazer tudo para CPU E GPU**. Se algo não roda em
   tempo real na CPU, **não se implementa na CPU**. Implementa-se onde entrega o melhor e mais
   otimizado resultado — a GPU. O twin CPU bit-a-bit deixa de ser a fonte da verdade.
2. **Padrão extraordinário, sem desculpa de custo.** A ambição é a melhor engine 2D que o mundo
   já viu. "Não existe justificativa lógica para um game engine que pretende ser a melhor do
   mundo não conseguir criar uma poça d'água sem queda de FPS." A poça custa ~1ms de compute; o
   resto é estrutura errada. Conserta-se a estrutura.
3. **Arquitetura melhor, arquivos menores.** Temos arquivos gigantes (`diffusion.rs` 2808,
   `solver.rs` 2165, `tool/tests.rs` 4188, `composite.rs` 1335…). Arquivos menores e bem
   isolados = manutenção mais eficiente e mais chance de caçar os bugs. HR-18 (≤600 LOC) vale.
4. **Revisar TODO parâmetro e seu efeito.** Cada slider/constante do motor deve ter efeito
   documentado, range validado e exposição UI coerente. Hoje há 20 controles + ~15 constantes
   espalhados; muitos nunca foram validados visualmente contra a física que dizem modelar.
5. **Mudar os planos e os contratos** para atingir isso. Contratos congelados estão **na mesa**
   (Enio, dono, autoriza) — via ADR superseding (§3), não renegociando ad-hoc.
6. **Caçar os bugs durante a refatoração.** "A melhor coisa que temos — a água onde o pigmento
   pode ser difundido — está bugada e com queda profunda de FPS." E falta a **borda fininha
   realista de deposição** (edge-darkening). A refatoração é a oportunidade de fechar os dois.

---

## §1 — DIAGNÓSTICO: por que pintar derruba o FPS (a causa REAL, com evidência)

Um dia inteiro perseguiu o sintoma errado. A medição final (Enio, Metal, 8 GB, Immediate,
1408×768) com o `[gpu]` GPU-timestamp profiler crava a causa:

```
[frame] total=52.50ms (~19 fps) | cpu-encode(raw)=2.49ms | present/acquire-stall=50.02ms
[fluid-ctx] region=(...) (3900 cells) substeps=2 dabs=9 stroke_active=true   ← região PEQUENA
[gpu] passes/frame=21
  render.sprite=10ms  render.tonemap=10ms  copy.slot=9.67ms(×0.4)  fluid.comp_tex=8.22ms(×0.4)
  fluid.advect_v=2.2ms  fluid.capillary=1.2ms  ...  (Σ kernels reais < 5ms)
```

**A leitura correta:**

- **Os kernels do sim somam <5ms reais** (timestamps próprios, exatos). O compute NÃO é o
  problema — uma poça é barata.
- **O custo está na ESTRUTURA do frame**, em três pontos concretos:
  1. **Sobre-submissão.** ~12–15 `queue.submit()` por frame pintando (vs 4–6 sem fluido). Cada
     submit é um ponto de sincronização no Metal; o driver não sobrepõe trabalho através deles.
     No hot path: `splat_dabs` (1) + `step_resident_splat` (1) + `composite_frame_to_texture`
     (1) + `copy_texture_into_individual` (1) + readback pipelined (1) + base render (sprite,
     tonemap, vello×N, compositor) — cada um seu encoder+submit. Anatomia completa em
     [HANDOFF mapeado] §2 abaixo.
  2. **Cópia do canvas INTEIRO todo frame.** `copy_preview_into_slot` →
     `copy_texture_into_individual(id, tex, cw, ch)` chama `encoder.copy_texture_to_texture`
     sobre `Extent3d { width: cw, height: ch }` — o canvas **inteiro**, não a dirty-rect
     ([`crates/ph2d-render/src/individual.rs`](../crates/ph2d-render/src/individual.rs#L333),
     [`shells/desktop/src/render_loop/painter_fluid_support.rs:116`](../shells/desktop/src/render_loop/painter_fluid_support.rs#L116)).
     A 1408×768 RGBA16F ≈ 8.6 MB R+W/frame; a 4K ≈ 67 MB+/frame. **Por nada** — só a região
     molhada mudou.
  3. **Readback no hot path do traço.** `composite_frame_pipelined` + `fluid_apply_gpu_composite_rows`
     mantêm `canvas_rgba` (CPU) atualizado TODO frame de stroke
     ([`painter_fluid_bridge.rs`](../shells/desktop/src/render_loop/painter_fluid_bridge.rs)).
     `canvas_rgba` só precisa estar atual no **pen-up** (snapshot de undo, commit, thumbnail) —
     não a cada frame enquanto se pinta.
- **Os `render.sprite/tonemap=10–49ms` são WALL-SPAN inflado** (TBDR do Metal separa
  vertex/fragment; encoders sobrepõem) — um tonemap fullscreen não custa 49ms reais. MAS o
  `present/acquire-stall` de 50ms é REAL: é backpressure: a fila de GPU está profunda de tanto
  submit + a cópia full-canvas, e o `acquire_frame` espera a GPU drenar. Atacar (1)+(2)+(3)
  encurta a fila → o stall colapsa.

**Conclusão que muda o plano:** o sintoma não é "o sim é caro" nem "envelope grande" (perseguidos
ontem). É **frame mal-estruturado**: submits demais, cópia full-canvas, readback no lugar errado.
Isso é arquitetura, não tuning — e não dá pra consertar limpo enquanto 740 gates de paridade
bit-a-bit travam a topologia do solver e do compositor. Por isso a refatoração GPU-first (§0.1)
e o conserto de perf são **o mesmo trabalho**.

> **`HANDOFF_painter_fluid_perf_block.md` está SUPERSEDED por este doc.** Os fixes daquele bloco
> (epsilon-clamp `WATER_EPS`, gate Curtis `CAPILLARY_MIN_SATURATION`, settle-freeze do Keep Wet,
> decimação idle) **funcionam e ficam** (o idle recupera a ~60fps), mas eram band-aids no frame
> errado. A causa de pintura-lenta é a desta §1.

---

## §2 — ARQUITETURA-ALVO (Watercolor v2): GPU-first, single-submit, direct-render

O alvo, em 5 invariantes. Cada um ataca um ponto da §1.

### I1 — UM encoder, UM submit por frame para sim + composite
Splat + (substeps de diffuse/advect/transfer/evaporate/capillary/shallow) + composite vão
para **um único `wgpu::CommandEncoder`**, um `queue.submit`. O wgpu insere os barriers RAW
entre compute passes — não precisa de submit por pass. `step_resident_splat` já batcheia os
substeps num encoder ([solver.rs:1636](../crates/ph2d-painter-fluid/src/solver.rs#L1636)); o
trabalho é **estender esse encoder** para incluir o splat e o composite, em vez de 3 submits
separados. Meta: do hot-path de ~5 submits de fluido para **1**.

### I2 — Renderizar a textura de preview DIRETO como sprite (zero cópia full-canvas)
A saída do compositor (a textura premultiplicada do preview) deve **ser** a textura amostrada
pelo sprite via `PreviewOverride`, sem `copy_texture_into_individual`. Hoje o pipeline é
`composite → copy canvas inteiro → slot → sprite amostra slot`. O alvo é
`composite → override aponta direto pra textura do compositor`. Se o `IndividualTextureStore`
exigir posse da textura, copiar **só a dirty-rect** (a região molhada), nunca o canvas todo.
Elimina o `copy.slot` de 9–35ms do hot path.

### I3 — Nenhum readback no hot path do traço
`canvas_rgba` (o bake CPU) só se atualiza no **pen-up** (e no flush de pointer-down que protege
o undo — `flush_pending_bake`, que FICA). Mid-stroke o preview é 100% GPU-residente; nada volta
pra CPU. O `composite_frame_pipelined` sai do loop de stroke e vira um bake único pós-traço.

### I4 — Sim + composite escopados ao bbox de água ATIVA (sparse), não à união all-time
O trabalho por frame escopa ao bbox ATUAL de água viva (das stats GPU, com pad), não à união
monotônica de toda a sessão. A união continua dimensionando/semeando a TEXTURA persistente
(cobre áreas que secaram e precisam de UM composite final) — isso é a cerca de Chesterton das
"quinas retangulares", **não apague**. Mas o STEP e o COMPOSITE por-frame só tocam tiles ativos.
Esse é o antigo "Passo 2c" / ADR-0083 / ADR-0078-S1b, agora possível porque I1–I3 limparam o
frame. Escala 4K.

### I5 — Física GPU-first; CPU vira oráculo offline opcional, NÃO twin bit-a-bit
A GPU é a fonte da verdade do look vivo. Os ~740 gates de paridade bit-a-bit CPU↔GPU
(`gpu_parity.rs` 99 asserts, `composite_parity.rs` 59 asserts) **são removidos** — eles forçam
o solver a algoritmos com forma-de-CPU (gather kernels, contagem fixa de Jacobi par, lane-for-lane)
e travam a topologia que a §1 precisa reestruturar. No lugar:
- **Gates de INVARIANTE FÍSICO** (não bit-paridade): conservação de massa (pigmento), limites
  `[0,1]` da água, simetria, monotonia da secagem, ausência de NaN/runaway. Tolerâncias
  frouxas, rodam sobre a saída GPU.
- **O `diffusion.rs` (2808 LOC, hoje hot-path-morto desde W15.3 — já é GPU-residente) é
  drasticamente reduzido**: o que sobra é, no máximo, um oráculo de referência para validar
  invariantes offline + (se barato) o bake determinístico de reproject (ver §3/§8). Não é mais
  espelhado bit-a-bit.

> **Decisão recomendada (ADR-0085, §3):** o caminho VIVO e o OFFLINE são ambos GPU. Reproject/
> replay de aquarela re-roda o sim GPU → **visualmente equivalente, não bit-idêntico** cross-OS
> (FP de GPU difere por device — bit-idêntico de fluido é fisicamente impossível e ninguém
> precisa). A garantia determinística cross-OS aplica-se à GEOMETRIA dos traços vetoriais
> (`.ph2d-painter`, ADR-0046), não aos pixels do fluido. Dispositivos sem compute-GPU: aquarela
> **desabilita graciosamente** (cai pra brushes não-fluidos), em vez de um sim CPU lento — isso
> muda a promessa de device-matrix do ADR-0049/0053 e precisa do ADR-0085.

---

## §3 — O QUE MUDA NOS CONTRATOS (ADR-0085 — Coord-only, PRIMEIRO passo)

Mexer em contrato congelado é Coord-only + ADR (CLAUDE.md §2/§6, DIRETRIZ §4). Enio autoriza.
**Tarefa 1 do novo agente:** escrever e ratificar **ADR-0085 — Watercolor v2: GPU-first
real-time architecture (supersedes the CPU↔GPU bit-parity requirement)**. Deve:

1. **Superseder a paridade bit-a-bit** do ADR-0049-amendment-1 / ADR-0080–0082 (a regra
   "GPU mirror of `ph2d_painter_brush::diffusion`, bit-exact lane-for-lane"). Nova regra:
   invariantes físicos, não bit-paridade (§2-I5).
2. **GPU-compute-required** para aquarela; remover a promessa de fallback CPU em-tempo-real do
   ADR-0049 (§2.3 device-matrix) e ajustar ADR-0053 tier policy: low-tier sem compute = aquarela
   off, não sim CPU.
3. **Reafirmar o que NÃO muda** (§8): o ABI do `Stamp=96B`, `RenderingMode=6`, `ColorProfile=8`,
   `Brush≤168`, etc. (ADR-0043–0051) são contratos de SUPERFÍCIE/UI, não de física — ficam
   congelados. O modelo K–M espectral (24 bandas, ADR-0080) fica (é o melhor que temos).
4. **Novo contrato de perf** (substitui os budgets soltos): single-submit hot path; zero cópia
   full-canvas por frame; zero readback no stroke; sim+composite O(bbox ativo). Com um gate
   executável onde der (ex.: contar submits no hot path; assert dirty-rect ⊊ canvas).
5. **Decidir o destino do `diffusion.rs`** (§2-I5): reduzir a oráculo/bake-offline, ou deletar.
   Recomendação: reduzir, não deletar — preserva o bake determinístico de reproject e o oráculo
   de invariantes, baratos. Mas SEM espelhamento bit-a-bit.

Antes de escrever, releia (não tudo, só o que tocar): os ADRs 0049, 0078, 0080, 0083 e o gate
`architecture_painter_contract_surface` em
[`crates/ph2d-painter-contracts/tests/`](../crates/ph2d-painter-contracts/tests/). Atualize o
gate de contrato para refletir o que sai/fica.

---

## §4 — DECOMPOSIÇÃO DE ARQUIVOS (HR-18: arquivos menores e isolados)

Alvos confirmados (>600 LOC) e a decomposição proposta. A refatoração estrutural anda JUNTO da
de perf — não é cosmética, é o que torna a §1 tratável.

| Arquivo | LOC | Decomposição proposta |
|---|---|---|
| `ph2d-painter-brush/src/diffusion.rs` | 2808 | **Encolhe drasticamente** (§2-I5): vira oráculo de invariantes + bake offline. O que for hot-path-morto sai. |
| `ph2d-painter-fluid/src/solver.rs` | 2165 | Quebrar por responsabilidade: `solver/fields.rs` (buffers/bind groups), `solver/passes.rs` (encode dos kernels), `solver/region.rs` (dispatch escopado), `solver/stats.rs` (reduce/bbox). Um encoder, um submit (§2-I1). |
| `ph2d-tool-painter/src/tool/tests.rs` | 4188 | Split por feature (não é foco da refatoração de fluido, mas é dívida HR-18 — split quando tocar). |
| `ph2d-painter-fluid/src/composite.rs` | 1335 | `composite/frame.rs` (composite → out_buf), `composite/preview_texture.rs` (premul/straight → textura, §2-I2), `composite/readback.rs` (bake offline, §2-I3). |
| `tool/lifecycle.rs` 1787, `adjustments/compute.rs` 1633, `tool/layers.rs` 1206 | Dívida HR-18 adjacente; split quando o caminho passar por eles. Não bloquear a refatoração de fluido por isto. |
| Bridge no shell (`painter_fluid_bridge.rs` ~600, `_support.rs`, `_gpu_preview.rs`) | — | Re-arquitetar para o frame single-submit/direct-render (§2-I1/I2/I3). Manter cada arquivo ≤600. |

**Regra:** crate de física = `ph2d-painter-fluid` (GPU). O `ph2d-painter-brush` mantém o motor
de brush/stamp (Stamp ABI, scheduler) — só a parte de *diffusion* encolhe. Não criar acoplamento
novo entre eles além dos contratos.

---

## §5 — REVISÃO DE PARÂMETROS (todos, com efeito documentado)

Hoje: 20 controles UI (`WatercolorParams::CONTROLS`,
[`crates/ph2d-painter-brush/src/watercolor.rs:122`](../crates/ph2d-painter-brush/src/watercolor.rs#L122))
+ ~15 constantes nomeadas espalhadas (`diffusion.rs`, `solver.rs`, `*.wgsl`). Inventário completo
abaixo (use como checklist — cada linha precisa: efeito real verificado visualmente, range
sensato, default justificado, exposição UI coerente).

**Parâmetros de solver (DiffusionParams → GPU UBO):**
`diffusivity` (bloom), `evaporation` (secagem), `downhill` (β canal do papel), `flow_outward`
(λ wet→dry, edge-darkening), `w_lo`/`w_hi` (banda do gate de umidade), `perm_valley`/`perm_crest`
(permeabilidade vale/crista), `deposition`/`deposition_dry`/`granulation` (camada depositada —
§6), `velocity`/`viscosity`/`drag`/`pressure` (shallow-water), `capillary`/`capillary_mobility`
(franja capilar), `sharpness` (MacCormack), `lift` (re-mobilização), `capillary_branching`
(franja dendrítica).

**Presets WATERCOLOR_\* (solver.rs):** `WATERCOLOR_VELOCITY=1.3`, `_VISCOSITY=0.18`, `_DRAG=0.1`,
`_PRESSURE=0.3`, `_DEPOSITION_BASE=0.012`, `_DEPOSITION_DRY=0.10`, `_GRANULATION=1.4`. **Suspeito
nº1 do bug + da queda de FPS:** o `velocity=1.3` (shallow-water sempre-on) é provavelmente o que
espalha a poça pra sempre sob Keep Wet (o creep idle medido NÃO mudou com o gate capilar →
é a shallow-water, não o capilar — ver perf-block §10e). Re-derivar cada preset da física e
validar visualmente com o Enio.

**Constantes de estabilidade/numéricas:** `WATER_EPS=1e-4`, `WET_BBOX_WATER_THRESHOLD=1e-3`,
`CAPILLARY_MIN_SATURATION=0.005`, `RELAX_ITERS=6` (pode deixar de ser par — a paridade que exigia
isso morre em §2-I5), `SOLVER_REGION_PAD=6`, `CAPILLARY_FRINGE_PAD=8`, `LIFT_BLEED_KEEP=0.25`,
`BRANCH_GATE_LO/HI=0.40/0.60`, `MAX_DABS_PER_DISPATCH=4096`, `HEIGHT_FREQ=0.13`.

**Entregável:** um **registro único de parâmetros** (uma fonte da verdade — struct + tabela
doc-comentada) em vez de constantes espalhadas, com efeito/range/default/exposição por linha.
Parâmetros hardcoded que mereçam controle do artista sobem pra UI; os puramente numéricos ficam
documentados no registro.

---

## §6 — REALISMO: a borda de deposição + os bugs da água

### 6a — A borda fininha realista (edge-darkening) — falta realismo
O mecanismo existe: `transfer_pigment` com `rate = deposition + deposition_dry·(1−gate)`
([`diffusion.rs:983`](../crates/ph2d-painter-brush/src/diffusion.rs#L983)) — ao secar, a célula
congela pigmento; na borda rimosa (pouca água) congela mais → anel escuro. Mas o Enio diz que
"ainda não temos aquela borda fininha realista". Hipóteses a investigar (a refatoração é a
chance):
- A água precisa **recuar deixando o pigmento concentrado numa LINHA FINA** na fronteira que
  seca — hoje o `flow_outward` empurra água+pigmento juntos; o realismo do edge-darkening do
  Curtis vem do pigmento ser **deixado pra trás** quando a água recua/evapora na borda
  (chromatographic deposition). Revisar a ordem evaporate→transfer e a co-advecção
  `capillary_mobility` para que a borda fique uma linha fina concentrada, não um gradiente largo.
- O canvas de demo é 64×64 nativo → render macio borra a borda fina por causa da res baixa do
  source, não do sim (ver `project_painter_canvas_res_64_not_sim_scale`). Validar a borda numa
  res de canvas decente antes de culpar a física.
- Alvo visual: definir um smoke específico ("pinte um wash redondo, deixe secar → anel escuro
  fino e nítido na borda, mais escuro que o interior") e iterar `deposition_dry`/`flow_outward`/
  ordem-de-passes contra ele com o Enio.

### 6b — A água bugada + queda de FPS
A água (campo onde o pigmento difunde) é "a melhor coisa que temos" e está bugada. Bugs
conhecidos/suspeitos a fechar na refatoração:
- **Creep/runaway sob Keep Wet** = shallow-water espalhando a poça pra sempre (evaporação 0).
  O settle-freeze (perf-block) mascara; a refatoração deve resolver na física: a poça deve
  ATINGIR EQUILÍBRIO (a água para de espalhar quando a tensão/altura equilibra), não congelar
  por timeout. Revisar `velocity`/`pressure`/`drag` (§5).
- **Queda de FPS pintando** = §1 (estrutural). I1–I4 resolvem.
- **Crash water/water** já corrigido (perf-block §0) — manter o fix (drenar `pending` antes de
  re-mapear staging) na reescrita.
- Re-validar undo multi-step, sprite-wide paint (os fixes do perf-block §0) sobrevivem à
  reescrita do bridge.

---

## §7 — PLANO FASEADO (ondas R0–R5) — ordem de risco, cada onda com smoke do Enio

Cada onda fecha com `cargo check -p` verde + smoke visual do Enio (visual-first, DIRETRIZ).
Commits locais fast-mode (`--no-verify`), sem push até o Enio mandar (CLAUDE.md §3).

- **R0 — Contrato (Coord-only).** ADR-0085 (§3) + atualizar o gate de contrato. Sem código de
  runtime ainda. **Smoke:** Enio ratifica o ADR.
- **R1 — O conserto urgente de perf (single-submit + direct-render).** I1+I2+I3 (§2). NÃO exige
  ainda deletar o twin CPU nem mexer na MATEMÁTICA — só reestrutura a topologia do frame (merge
  de encoders; override aponta pra textura do compositor; bake só no pen-up). Os gates de
  paridade existentes devem continuar passando (a math não muda). **Smoke:** pinte água, Keep
  Wet ON, mancha — FPS deve ir a ~60 pintando (o `present-stall` colapsa; `[gpu]` mostra
  passes/frame despencar). **Esta onda entrega o ganho que o Enio espera.**
- **R2 — Sparse tiles.** I4 (§2). Step+composite escopados ao bbox ativo. **Smoke:** wash grande
  / canvas grande mantém FPS; `[fluid-ctx] region` reflete só a água viva.
- **R3 — GPU-first cleanup + revisão de parâmetros + caça aos bugs.** Deletar os gates de
  paridade bit-a-bit; encolher `diffusion.rs` (§2-I5); decompor `solver.rs`/`composite.rs` (§4);
  construir o registro de parâmetros (§5); resolver o equilíbrio da poça (§6b). Aqui a MATEMÁTICA
  pode mudar (melhor algoritmo GPU). **Smoke:** look validado intacto (blooms, granulação,
  franja, sheen) + a poça atinge equilíbrio sem settle-freeze.
- **R4 — Realismo da deposição.** A borda fininha (§6a) contra o smoke dedicado. **Smoke:** anel
  escuro fino e nítido no wash seco.
- **R5 — Fechamento.** Decompor o que sobrou >600 LOC; reconstruir a suíte de testes como
  invariantes físicos (frouxos), não bit-paridade; gate de perf executável; smoke completo
  cross-feature (layers + fluido + adjustments). Atualizar docs.

**≤3 cargos simultâneos (RAM 8 GB).** Inner loop = `cargo check -p ph2d-painter-fluid --features fluid`
/ `-p ph2d-painter-brush` / `-p ph2d-host-desktop --features fluid`. Gates GPU `--ignored`
(Metal) 1× no fim de cada onda. Slot warm por CoW (`scripts/slot-seed.sh`).

---

## §8 — O QUE PRESERVAR (não jogar o bebê fora com a água)

- **O modelo de cor K–M espectral (ADR-0080):** 24 bandas K/S + err + mass + stain, 8 vec4/célula
  (`PIG_CH=32`), base 7-curvas em `pigment_mix.rs`. É o melhor que temos — a difusão multi-pigmento
  correta. **Fica.** Só perde o espelhamento bit-a-bit (vira GPU-first).
- **O look validado:** blooms, edge-darkening, granulação, franja capilar, branching dendrítico,
  sheen úmido, lift/backdrop-lift (ADR-0081/0082/0084). Cada um foi ratificado visualmente pelo
  Enio. A reescrita deve **preservar o look**, não reinventá-lo — re-derivar a IMPLEMENTAÇÃO
  GPU-first, validar paridade VISUAL (não bit) com o Enio onda a onda.
- **Os ABIs de superfície (ADR-0043–0051):** `Stamp=96B`, `RenderingMode=6`, `ColorProfile=8`,
  `Brush≤168`, `AdjustmentKind≤32`, history `.ph2d-painter` (ADR-0046). Contratos de UI/persist,
  não de física. **Congelados** — não tocar.
- **Os fixes do perf-block §0** (crash water/water, undo multi-step, sprite-wide paint, stats
  pipelinado): sobrevivem à reescrita do bridge.
- **A cerca de Chesterton da união monotônica** (§2-I4): a textura persistente é dimensionada/
  semeada pela união all-time DE PROPÓSITO (cobre áreas secas que precisam de um composite final).
  O sparse escopa o TRABALHO por-frame, não a textura.

---

## §9 — COMO MEDIR / VALIDAR

```bash
PH2D_FLUID_PROFILE=1 ./play.command
# [frame] total/raw/present-stall  +  [fluid] submit-CPU  +  [gpu] passes/frame + tabela de kernels  +  [fluid-ctx] região/dabs
# R1 alvo: pintando, present-stall colapsa (≈0), passes/frame cai (1 submit de fluido), ~60fps.
# Repro do bug original: água + Keep Wet ON + mancha pequena → tem que ficar a ~60fps pintando E parado.

cargo check -p ph2d-painter-fluid --features fluid --tests     # inner loop
cargo check -p ph2d-host-desktop --features fluid               # o bridge
# Fim de onda (Metal --ignored): gpu_parity (até R3), composite_parity (até R3),
#   pass_profiler_gpu, layer_compositor_gpu, contract_surface.
```

O `[gpu]` GPU-timestamp profiler (`crates/ph2d-gpu/src/pass_profiler.rs`, gated em
`PH2D_FLUID_PROFILE`) é a ferramenta de medição — kernels reais são exatos; spans de
marker (copy/vello) são piso, não teto (caveat Metal no perf-block §4). **Bench-verde ≠ vivo:**
toda onda fecha com smoke visual do Enio, não só teste verde.

---

## §10 — LIÇÕES E ARMADILHAS (deste dia — não repetir)

- **Meça a ESCALA e a CLASSE do sintoma antes da causa.** Um dia caçou "envelope grande" e
  "custo fixo por frame"; a causa era topologia de submit. O GPU-timestamp profiler é que crava
  — wall-span engana (TBDR infla; `render.sprite=49ms` não é real). Veja
  `feedback_measure_perf_symptom_scale`.
- **Não confie em "espere o catch-up" sob backpressure.** A decimação idle gated em
  `texture_mode_dirty==None` NUNCA engatou porque os bands pipelinados intercalam vazios sob
  carga e o "2 consecutivos" nunca fecha. Correção de bake = `flush_pending_bake` no pointer-down
  (síncrono, determinístico), não um contador que pode estolar pra sempre.
- **Keep Wet semântico, não físico.** "Tinta trabalhável", não "wash invade o canvas pra sempre".
  Sob evaporação 0 a poça tem que atingir EQUILÍBRIO por física (§6b), não crescer.
- **O twin CPU bit-a-bit é o imposto.** 740 gates + 2808 LOC + design com forma-de-CPU. Foi o que
  impediu reestruturar o frame ontem. É o que esta refatoração remove (§2-I5).
- **Canvas de demo é 64×64** — render macio borra por res baixa do source, não pelo sim
  (`project_painter_canvas_res_64_not_sim_scale`). Cheque a res antes de culpar a física (§6a).
- **Isolamento (CLAUDE.md §2):** edite só a área de fluido/brush/bridge. Precisou de
  foundational/contrato/outra crate → ADR-0085 (Coord) ou PARE e reporte. `git add -- <seus paths>`,
  nunca `-A`.

---

## §11 — ÍNDICE DE ANCORAGEM (arquivos-chave, para não re-descobrir)

- **GPU solver:** [`crates/ph2d-painter-fluid/src/solver.rs`](../crates/ph2d-painter-fluid/src/solver.rs)
  (`step_resident_splat` L1616, `SOLVER_REGION_PAD` L253, presets `WATERCOLOR_*` L64–81).
- **GPU composite/preview-texture:** [`crates/ph2d-painter-fluid/src/composite.rs`](../crates/ph2d-painter-fluid/src/composite.rs)
  (`composite_frame_to_texture` L776, `composite_frame_pipelined` L671).
- **Shaders:** `crates/ph2d-painter-fluid/src/shader/` (`fluid.wgsl` diffuse/advect/evaporate,
  `capillary.wgsl`, `shallow.wgsl`, `transfer.wgsl`, `combine.wgsl`, `composite.wgsl`, `reduce.wgsl`, `splat.wgsl`).
- **CPU referência (a encolher):** [`crates/ph2d-painter-brush/src/diffusion.rs`](../crates/ph2d-painter-brush/src/diffusion.rs)
  (`step` L552, `transfer_pigment` L983, `capillary_flow` L1097, params L76, consts L239–307).
- **Modelo K–M:** [`crates/ph2d-painter-brush/src/pigment_mix.rs`](../crates/ph2d-painter-brush/src/pigment_mix.rs).
- **Params UI:** [`crates/ph2d-painter-brush/src/watercolor.rs`](../crates/ph2d-painter-brush/src/watercolor.rs) (`CONTROLS` L122).
- **Bridge no shell:** [`shells/desktop/src/render_loop/painter_fluid_bridge.rs`](../shells/desktop/src/render_loop/painter_fluid_bridge.rs),
  [`painter_fluid_support.rs`](../shells/desktop/src/render_loop/painter_fluid_support.rs) (`copy_preview_into_slot` L98),
  [`painter_gpu_preview.rs`](../shells/desktop/src/render_loop/painter_gpu_preview.rs).
- **Render integration:** [`crates/ph2d-render/src/individual.rs`](../crates/ph2d-render/src/individual.rs) (`copy_from_texture` L333),
  `sim_extract.rs` (`PreviewOverride` L241), `present.rs` (ordem dos passes), `pass_profiler.rs`.
- **Gates de paridade (a remover em R3):** `crates/ph2d-painter-fluid/tests/gpu_parity.rs` (99 asserts),
  `composite_parity.rs` (59 asserts).
- **Gate de contrato:** `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs`.
- **Profiler/medição:** `crates/ph2d-gpu/src/pass_profiler.rs`.
- **Plano vivo:** [`docs/Painter_projeto/15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md) (W15).
- **Look-alvo (preservar):** [`docs/HANDOFF_painter_fluid_continuation.md`](HANDOFF_painter_fluid_continuation.md),
  [`docs/HANDOFF_painter_fluid_gpu_composite.md`](HANDOFF_painter_fluid_gpu_composite.md) (W15.3).
- **Pesquisa estado-da-arte (validação do modelo, ler antes de mexer na física):**
  [`docs/Painter_projeto/pesquisa_aquarela_estado_da_arte.md`](Painter_projeto/pesquisa_aquarela_estado_da_arte.md)
  (motor = Curtis 1997 canônico + extensão capilar; K–M é frontier) +
  [`docs/Painter_projeto/avaliacao_e_melhorias.md`](Painter_projeto/avaliacao_e_melhorias.md) (crítica de engenharia).
  Memória: `reference_watercolor_state_of_art`.
- **Perf-block SUPERSEDED (registro da investigação):** [`docs/HANDOFF_painter_fluid_perf_block.md`](HANDOFF_painter_fluid_perf_block.md).

---

— Handoff aberto 2026-06-10 (Enio mandou abandonar a estratégia incremental e refatorar
GPU-first). Primeira ação do novo agente: **§3 ADR-0085 (Coord-only)** → **§7 R1** (o ganho de
perf). Visual-first, padrão-ouro, sem adiamentos.

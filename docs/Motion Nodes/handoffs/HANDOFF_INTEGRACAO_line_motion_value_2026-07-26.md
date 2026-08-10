# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (reabertura, 2026-07-26)

> Para o **agente integrador**, munido deste handoff (DIRETRIZ §1.5.3–1.5.9). A
> linha está **FECHADA**. Integração e ship **só por ordem explícita do Enio** — o
> implementador NÃO integra nem pusha (CLAUDE.md §0.7). Este documento é o mapa.

## TL;DR

- **38 commits** à frente do `main` (base = `0afc6bb28`). Todos os smokes aprovados
  pelo Enio.
- **Cinco clusters de feature** (não é só nós de valor): motion.delay/path + canal
  `external` · dock da timeline no Motion · FX de passe (glow) · auto-arrange de
  grafo · **17 nós-crate do domínio de VALOR, todos GPU-resident**.
- **Contrato congelado INTACTO** — `NodeOp=2`/`OpResolver=1`/`NodeManifest=8`, o gate
  `architecture_contract_surface` passa. **Zero canal novo** no resolver: os nós de
  valor usam os canais de kernel/reduce EXISTENTES (ADR-0126), acrescentados por
  linhas já integradas. **Nenhum ADR novo** (os docs são notas em `docs/Motion
  Nodes/`, não `docs/architecture/decisions/`) ⇒ sem disputa de número.
- **Sem bump de schema** — nós de grafo não têm `PROJECT_SCHEMA`; o grafo de Motion é
  texto-serializado no formato existente. `physics-ecs-c9` **não é tocado**.
- **registry-init: 113 crates** (node-sync regenerou).

## O que a linha entrega (5 clusters)

1. **`motion.delay` + `motion.path` + o canal `external`** (docs 63, 65). O
   `motion.path` é *"como qualquer coisa que o APP possui entra no grafo"* — um canal
   de **entrada externa** ao cook. Módulo **NOVO** `ph2d-nodegraph/src/external.rs` +
   integração no `cook.rs` + `cook_external_tests.rs`.
2. **Dock da timeline no Motion** (doc 64, W4.T4) — a timeline doca no slot do Motion.
   Toca o **hero screen** (`ph2d-editor-core`: `hero.rs`, `paint.rs`, `layout.rs`).
3. **FX de passe: o GLOW** (docs 66–67, "Opção B"). Nó `fx.glow` + RT HDR próprio,
   aditivo, byte-idêntico no neutro; mip bloom (COD/Jimenez), não Kawase. Módulo
   **NOVO** `ph2d-render/src/motion_fx.rs` + `shaders/bloom.wgsl` + integração no
   renderer.
4. **Auto-arrange ciente de subgrupos** — módulos **NOVOS**
   `ph2d-nodegraph/src/layout.rs` + `ph2d-motion-doc/src/layout.rs`; os smokes o usam
   (`smoke_layout::arrange_and_mark`).
5. **17 nós-crate do domínio de VALOR, GPU-resident bit-comparável** (docs 68–84):
   - **Produtores:** `value.noise` · `value.pattern` · `value.time`
   - **Shapers:** `value.curve` · `value.gain` · `value.quantize` · `value.step` ·
     `value.unary` · `value.wrap` · `value.wave`
   - **Combinadores:** `value.mix`
   - **Redutores:** `value.normalize` · `value.reduce`
   - **Filtros:** `value.smooth` (passa-baixa) · `value.slope` (derivada) ·
     `value.median` (não-linear) · `value.percentile` (morfológico/rank)
   - (`value.lfo`/`value.math`/`value.switch`/`value.instance_field`/`value.attribute`
     e `value.map_range` já estavam no `main` da integração anterior.)

## Superfície foundational / compartilhada tocada (o mapa de risco de merge)

Estes são os arquivos FORA dos crates-nó novos. Cada um roda pelo gate da árvore
combinada (`scripts/foundational-integrate.sh`); onde outra linha tocou o mesmo
arquivo, o Mergiraf funde o resíduo textual (ADR-0107).

| Crate / arquivo | Novo? | Cluster | Risco de conflito |
|---|---|---|---|
| `ph2d-nodegraph/src/external.rs` | **NOVO** | canal external | baixo (módulo irmão) |
| `ph2d-nodegraph/src/layout.rs` | **NOVO** | auto-arrange | baixo (módulo irmão) |
| `ph2d-nodegraph/src/cook.rs` | mod | external + LOC split | **médio** (substrato quente) |
| `ph2d-nodegraph/src/cook_fingerprint.rs` | mod | split de LOC (fingerprint saiu do cook.rs) | médio |
| `ph2d-nodegraph/src/lib.rs` | mod | declara os módulos novos | médio (declarações append) |
| `ph2d-motion-doc/src/{layout.rs,lib.rs}` | NOVO+mod | auto-arrange | baixo |
| `ph2d-render/src/motion_fx.rs` | **NOVO** | glow | baixo (módulo irmão) |
| `ph2d-render/src/shaders/bloom.wgsl` | NOVO | glow | baixo |
| `ph2d-render/src/{renderer.rs,renderer_draw.rs,lib.rs,sprite_collect.rs}` | mod | integra o passe FX | **ALTO** — o renderer é foundational MUITO compartilhado |
| `ph2d-editor-core/src/screens/{hero.rs,paint.rs,layout.rs}` | mod | dock timeline | **ALTO** — o hero screen é compartilhado |
| `ph2d-gpu-cook/Cargo.toml` | mod | 17 dev-deps de valor | baixo (append) |
| `ph2d-gpu-cook/tests/gpu_cpu_parity.rs` | mod | +17 gates RTX + cross-gate | baixo (append) |
| `ph2d-node-registry-init/{Cargo.toml,src/lib.rs}` | mod | node-sync (17 crates) | **regenerável** — se conflitar, rode `cargo run -p ph2d-node-sync` DEPOIS do rebase |
| `shells/desktop/src/{main.rs,render_loop/*,init.rs,app_state.rs,motion_state.rs}` | mod | wiring dos smokes + FX + dock | médio |
| `project-memory/feedback_a_correct_number_can_carry_a_false_story.md` | NOVO | memória | baixo (⚠️ o índice `MEMORY.md` NÃO foi atualizado — adicione a linha de ponteiro na integração) |

⚠️ **Os dois ALTO (renderer + hero screen) são a preocupação real.** Se outra linha
da jornada tocou `ph2d-render/src/renderer*.rs` ou `ph2d-editor-core/src/screens/
hero*.rs`, o rebase pode conflitar FORA dos meus arquivos (colisão de mesmo-símbolo,
DIRETRIZ §1.5.5) — nesse caso **PARE e reporte ao Enio** (é o caso que exige decisão).

⚠️ **`registry-init` é o conflito mais provável e o mais fácil:** ele lista TODOS os
crates-nó em regiões geradas. Se outra linha adicionou nós, as duas listas conflitam.
**Não resolva à mão** — resolva pegando qualquer lado e rode `cargo run -p
ph2d-node-sync` (ele re-gera a lista canônica da árvore combinada). Depois confira que
`EXPECTED_TYPED`/o gate de registry passa.

## Contrato congelado: INTACTO (a garantia)

- `NodeOp=2` · `OpResolver=1` · `NodeManifest=8` — gate `architecture_contract_surface`
  (roda em `ph2d-nodegraph`), verde durante toda a linha.
- **Zero canal novo** no `KernelResolver`: os 17 nós de valor chamam
  `register_gpu_kernel` / `register_reduces` (o canal de reduce já existe, ADR-0126) —
  side-metadata, nunca contrato.
- **`value.pattern`/`value.expression`** não bumparam o `NodeManifest`: params não-f32
  vivem no `Graph` (text param), o padrão canônico já estabelecido.
- **Nenhum ADR** em `docs/architecture/decisions/` foi tocado ou criado ⇒ o gate
  `architecture_adr_numbers_are_unique` não corre risco desta linha.

## Como integrar (a ordem)

1. **Reabrir a worktree e rebasear:** a branch e a worktree já existem.
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value
   git fetch && git rebase main
   ```
   Rota "linha reaberta" do MODELO_ABERTURA_LINHA. Varra marcadores de conflito em
   CADA commit (DIRETRIZ §1.5.6); resolva pelos ESTÁGIOS do índice, não pelos markers.
2. **Se conflitar em `registry-init`:** pegue um lado, `cargo run -p ph2d-node-sync`,
   re-stage.
3. **Se conflitar FORA dos meus arquivos** (renderer/hero de outra linha, mesmo
   símbolo): **PARE e reporte ao Enio** (§1.5.5).
4. **Gate da árvore combinada:** `bash scripts/foundational-integrate.sh` (o gate
   testado do ADR-0107) — Mergiraf funde o resíduo textual do foundational.
5. **`--ff-only`** para o `main` só depois de tudo verde.

## Gates que o INTEGRADOR roda (não rodam sozinhos no fechamento)

- **⚠️ Os ~17 gates de paridade RTX são `#[ignore]`** — precisam de adapter. Rode na
  workstation (LG/RTX):
  ```
  cargo test -p ph2d-gpu-cook --test gpu_cpu_parity --release -- --ignored --nocapture
  ```
  Cada `value_<nó>_kernel_matches_the_cpu_on_the_device` prova CPU↔GPU no dispositivo
  real. Os de SELEÇÃO (`median`/`percentile`) exigem `max|d| == 0` (bit-exato); os
  aritméticos `< 1e-4`. **Sem adapter eles fazem skip gracioso — que NÃO é verde.**
- **`time_into_wave_equals_the_lfo`** roda em CI (NÃO é `#[ignore]`) — o gate do dual
  `value.wave`↔`value.lfo`, byte-a-byte; já passou no fechamento.
- **`./scripts/ship.sh`** — paridade EXATA com o CI (fmt · clippy `--all-targets` ·
  machete · deny · audit · nextest `--cargo-profile ci-test` · typos). Corrija TODO
  `✗`. **Cuidado:** o `ship.sh` roda `--release`/`ci-test`; alguns nós (ex.: os que
  colhem janela) e o `ph2d-flip-colorize` de outras áreas têm diferença debug×release,
  mas os value-nodes daqui passam nos dois perfis (medido).
- **Gate de LOC da shell** (`shells/desktop/tests/file_loc_caps.rs`, 600) e o
  `architecture_workspace_file_loc_cap` (crates, 700): rode isolados — o `cargo test
  -p` filtrado NÃO os alcança (a armadilha do `file_loc_caps`). `cook.rs` já foi
  split (fingerprint saiu) para caber nos 700.

## Smokes (todos aprovados pelo Enio; env var + `--release`)

Rodar: `env <VAR>=1 cargo run -p ph2d-host-desktop --release`.

| Cluster | Env var |
|---|---|
| Delay | `PH2D_MOTION_DELAY_SMOKE` |
| motion.path | `PH2D_MOTION_PATH_SMOKE` (ver `motion_node_path_smoke.rs`) |
| FX glow | `PH2D_MOTION_FX_SMOKE` |
| value.curve/noise/mix/quantize | `PH2D_VALUE_CURVE_SMOKE` · `..._NOISE_..` · `..._MIX_..` · `..._QUANTIZE_..` |
| value.gain/step/normalize/unary/reduce/smooth | `PH2D_VALUE_GAIN_SMOKE` · `_STEP_` · `_NORMALIZE_` · `_UNARY_` · `_REDUCE_` · `_SMOOTH_` |
| value.pattern/wrap/time/slope/median/percentile/wave | `PH2D_VALUE_PATTERN_SMOKE` · `_WRAP_` · `_TIME_` · `_SLOPE_` · `_MEDIAN_` · `_PERCENTILE_` · `_WAVE_` |

⚠️ `PH2D_VALUE_TIME_SMOKE` e `PH2D_MOTION_FX_SMOKE` exigem **PLAY** (são temporais);
os demais nós de valor são estáticos.

## Riscos / armadilhas conhecidas

- **Renderer e hero screen (ALTO):** ver acima — o único caminho para "PARE e
  reporte" é uma colisão de mesmo-símbolo nesses dois.
- **Os gates RTX não rodam sozinhos** — se você fechar sem rodá-los na RTX, a
  paridade de dispositivo fica NÃO-verificada (skip ≠ pass).
- **`MEMORY.md` não indexa a memória nova** (`feedback_a_correct_number...`) —
  adicione a linha de ponteiro.
- **Após integrar: adicione a entrada em CLAUDE.md §5** (union contra a main de HOJE,
  só ADICIONE — DIRETRIZ) documentando os 5 clusters: motion.delay/path + canal
  external · dock da timeline · FX glow · auto-arrange · os 17 nós de valor
  GPU-resident. A §5 atual só menciona a integração de 2026-07-12; esta reabertura
  não está lá.

## Estado final

- registry-init: **113 crates**.
- Contrato: `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` — **intacto**.
- Schema: **sem bump** (`PROJECT_SCHEMA` inalterado; grafo de Motion é texto).
- Docs: `docs/Motion Nodes/63..84` (21 notas).
- **Domínio de valor: COMPLETO para esta linha de fan-out.** O que resta genuinamente
  novo (`accumulate`/scan, `sort`/rank, `gather`) é **foundational** (canal de scan
  f32 / bitonic sort / array no device) e **NÃO deve ser enxertado nesta linha** — é
  uma linha foundational GPU dedicada (CLAUDE.md §5), aberta só por ordem do Enio.

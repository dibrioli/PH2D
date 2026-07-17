# BRIEFING — continuação de `line/gpu-nodes` (GPU/M5 **Fase 3**): a **simulação** na GPU

> Para o **novo agente implementador** que assume a linha Motion/GPU. A Fase 2 + a F1.2 landaram e
> estão fechadas; o **desenho da sua fatia já está escrito** ([ADR-0127](architecture/decisions/0127-gpu-simulation-pre-is-arc-pingpong-plan-becomes-a-dag.md))
> — você **executa** o ADR, não o re-inventa. **Autor:** o agente da Fase 2/F1.2, 2026-07-16, a pedido
> do Enio. Leia inteiro antes de tocar em código — é curto de propósito.

---

## §0 — ABERTURA (faça ANTES de qualquer coisa)

1. **Leia, nesta ordem:**
   - [`HANDOFF_line_gpu_nodes_fase2_2026-07-15.md`](HANDOFF_line_gpu_nodes_fase2_2026-07-15.md) — o
     que você herda (10 kernels, o híbrido, os gates, os gotchas).
   - **[ADR-0127](architecture/decisions/0127-gpu-simulation-pre-is-arc-pingpong-plan-becomes-a-dag.md)
     — É O SEU DOCUMENTO CENTRAL.** As 5 decisões (D1..D5) e as 5 fatias já estão desenhadas, com o
     porquê. Não reabra D4 (determinismo) nem D5 (scrub).
   - [ADR-0126](architecture/decisions/0126-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md)
     — o kernel é metadata LATERAL; o contrato de nós fica **8/2/1**.
   - [`docs/plans/2026-07-gpu-resident-node-pipeline.md`](plans/2026-07-gpu-resident-node-pipeline.md)
     — o roadmap (você faz a Fase 3; a 4 é journey futuro).
   - `CLAUDE.md` §0 (os 7 inegociáveis) + §6 (contratos congelados).
   - [`HANDOFF_line_gpu_nodes_fase1_2026-07-15.md`](HANDOFF_line_gpu_nodes_fase1_2026-07-15.md) —
     histórico do motor (as peças). Consulte, não precisa decorar.

2. **A linha JÁ ESTÁ ABERTA — não crie worktree novo:**
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && git log --oneline -1
   ```
   Branch `line/gpu-nodes`, HEAD **`4d176f9d`**, árvore limpa. **Empilhe seus commits aqui.**
   (O briefing anterior mandava ramificar um worktree novo por fatia; eu **não** fiz — um worktree
   virgem força rebuild frio do workspace inteiro ([[project_modo_l_speed_hole_worktree_targets_slow_path]])
   e o integrador landa em ordem pela **fronteira de commit** de qualquer jeito. Faça o mesmo.)

3. **Regras permanentes (Modo L):** trabalhe SÓ neste worktree (**SEMPRE comece o comando com
   `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes &&`** — o cwd escorrega pro repo
   primário; aconteceu nas 3 jornadas anteriores) · foundational é editável aqui (ADR-0107) ·
   `git commit --no-verify` · inner loop = `cargo check -p` · gates 1× no fechamento · **NÃO integre,
   NÃO pushe, NÃO rode `ship.sh`** — feche, escreva o handoff, PARE (§0.7). · **Se sentir vontade de
   bumpar o `NodeManifest`: PARE e releia o ADR-0126** — a resposta é sempre o canal lateral.

---

## §1 — A missão: **as 5 fatias do ADR-0127, nesta ordem**

O ADR tem o desenho; aqui está o porquê da ORDEM. **Feche cada fatia antes da próxima.**

1. **Fatia 1 — o motor DAG.** `plan()` hoje é uma **cadeia linear com UMA fronteira** e o `cook()`
   costura os stages numa variável `stream` única. Vira: walk de **N inputs** → `stages` em ordem
   **topológica** · `boundaries: Vec<(NodeId, port)>` (uma por input não-elegível) · o `cook()`
   threading por **mapa `NodeId → GpuStream`**. **A aresta `delayed` deixa de ser recusa e vira
   PARADA** (aquele input vem do `prev`, fatia 2). **Invariante que trava a fatia: a F1.1 e a Fase 2
   têm de ficar BYTE-IGUAIS** — a cadeia linear é um DAG de um caminho só; os 9 gates de paridade são
   o seu oráculo. Isto sozinho já destrava **`look_at`/`combine`** (multi-input sem estado).
2. **Fatia 2 — o `prev` ping-pong.** `GpuCook.prev: BTreeMap<NodeId, GpuStream>`, populado no fim do
   frame pela **MESMA regra** do `Cook::advance_tick_scoped` (nós que são fonte de aresta `delayed`).
   Sai quase de graça (D1): segurar o `Arc` É o estado; o `BufferPool` recicla por refcount.
3. **Fatia 3 — os kernels.** `motion.integrate` (pareamento **posicional** só — D3) + as **5 forças**
   (`force.wind`/`drag`/`attractor`/`vortex`/`curl` — single-input, `Pure`, escrevem `accel`; são o
   padrão da Fase 2, receita §3) + `motion.spring`.
4. **Fatia 4 — os gates.** Paridade de **UM passo** (D4 — nunca uma trajetória), o laço **DISPARA**,
   e a forma-do-stream **recua** (D3: stream com `id` → CPU).
5. **Fatia 5 — o ring de GPU** (D5): o scrub sem readback, capeado por **bytes**.

**Por que as forças NÃO vêm primeiro** (contra-intuitivo, e me custou a leitura pra descobrir): elas
são single-input e portáveis hoje — **mas só rodam DENTRO do laço**, que é CPU. Portadas antes das
fatias 1–2 seriam **kernels que nunca disparam**, e você não conseguiria escrever o gate que prova
que dispararam ([[feedback_an_optimization_needs_a_gate_that_proves_it_fires]]). **A unidade de valor
é o laço inteiro**, não o nó.

**Não-metas desta linha:** o gather por `id` (emitter/partículas com nascimento e morte — fatia
seguinte) · JFA/voronoi e spatial-hash/boids (Fase 3 do plano, journey próprio) · renderer consumindo
colunas cruas sem lowering (Fase 4) · mudar o default do `PH2D_GPU_COOK`.

---

## §2 — Onde a Fase 2 / F1.2 te deixou (o que você herda, JÁ no branch)

- **O motor (`crates/ph2d-gpu-cook/`):** `plan()` (sufixo-GPU/prefixo-CPU, **linear** — é o que você
  vai generalizar) · `GpuCook::cook()` (single submit; pipelines cacheadas por `(tipo,
  presença-de-colunas)` — params/playhead são uniforms, NUNCA recompilam) · `GpuStream`/`BufferPool`
  (colunas imutáveis `Arc<wgpu::Buffer>`, write = buffer novo do pool, **reclaim por refcount** — é
  o que torna a D1 de graça) · `codegen.rs` (`plan_bindings` é a MESMA função que monta o bind group)
  · `lower.rs` (compute → layout `RenderInstance`, 46 words) · `read_instances()` (readback dos gates).
- **10 kernels registrados:** `grid` (gerador, `source_count`) · `oscillator` (X/Y, waveform HR-5 +
  `applicable`) · `move` (`ReadWriteExisting`) · `output` (`PASSTHROUGH`) · **+ Fase 2:** `transform`
  · `rotate` · `scale` · `falloff` (enums por round-half-away) · `tint` (**Solid only**, `applicable`)
  · `wiggle` (**noise integer-hash bit-exata**).
- **O híbrido (F1.2):** `MotionCookPump::advance_or_scrub_to_node_scoped` + `boundary_stream()` —
  cozinha o nó de fronteira no `Cook` **persistente** (memo + `pre` + marcha de ticks) e entrega o
  stream cru. Sink e boundary compartilham UM `CookTarget` privado (não divergem). A rota
  (`Cpu | FullyGpu | Hybrid`) é função **pura** em `shells/desktop/src/render_loop/motion_bridge_gpu.rs`,
  unit-testada headless.
- **Perf medida (RTX, `--release`):** 500k = **1,0 ms/frame** · **2M = 4,0 ms/frame** (o demo
  `DEMO=1`). Sonda: `gpu_cook_millions_timing` (`#[ignore]`).
- **Gates verdes:** 9 de paridade ε (`--ignored`, RTX) + naga (todo subconjunto de colunas) +
  `plan_analysis` + contrato **8/2/1** + `cook_determinism`/`transform_determinism` + 566 do shell.

### Smoke (rodar e clicar na tool Motion)

```
# Full-GPU: 1250x1600 = 2.000.000 instancias, 100% GPU, ~4 ms/frame. Zoom out.
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=1 cargo run --release -p ph2d-host-desktop

# Hibrido (F1.2): ondula em Y (GPU) E gira (CPU), nos dois lados da costura.
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=2 cargo run --release -p ph2d-host-desktop
```
Gate de GPU (headless, precisa de adapter):
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && cargo test -p ph2d-gpu-cook --test gpu_cpu_parity --release -- --ignored --nocapture
```

---

## §3 — A RECEITA de port por nó (fatia 3 — repita por nó)

Para cada nó, nesta ordem — **não pule o passo 1**:

1. **Leia a `eval` da CPU inteira** e classifique: *map puro por-elemento?* → portável direto ·
   *tem REDUÇÃO?* (`twist`/`bend` têm — `r_max`/`x_extent`) → não cabe num kernel por-elemento ·
   *lê vizinho / `pre`?* → é a fatia 2, não um port.
2. **Escreva o `GpuKernel` const** no crate do nó (o kernel mora com o dono): corpo em `wgsl`,
   helpers em `wgsl_lib` (prefixo `<nó>_`), bindings com o `ColumnAccess` que **ESPELHA a semântica de
   ausência da CPU** (`ReadWrite` materializa da identidade · `ReadWriteExisting` dropa o write —
   **leia o braço do match, não chute**), `identity` = o fallback da CPU, `applicable` se a cobertura
   for parcial. **HR-5:** porte as aproximações polinomiais do próprio nó; não troque por `sin`/`cos`
   do WGSL.
3. **`reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL)`** no `register()`.
4. **Gates (por nó, antes do próximo):** adicione o crate ao `registry()` de
   `tests/generated_wgsl_validates.rs` **e** de `tests/gpu_cpu_parity.rs` (+ dev-dep no `Cargo.toml`
   do `ph2d-gpu-cook`) → o naga valida TODO subconjunto de colunas de graça. Paridade ε: chain real,
   ≥25,6k instâncias, params **NÃO-default e NÃO-inteiros-bonitos**
   ([[feedback_test_with_product_numbers_not_convenient_ones]]), **assert `is_fully_gpu` +
   `dispatching_stages` ANTES de comparar** (o gate tem que provar que DISPARA).
5. `cargo check -p <crate> -p ph2d-gpu-cook` no inner loop; o resto 1× no fechamento.

**Para um nó SEQUENCIAL a receita muda no gate:** D4 — paridade de **UM passo** a partir de um estado
semeado. Nunca compare uma trajetória e afrouxe o ε até passar.

---

## §4 — Determinismo (DECIDIDO — não reabra)

- **CPU = caminho CANÔNICO** (`cook_determinism` + `transform_determinism` + `c9_replay` intocáveis).
  GPU = performance/preview, reconciliada por **ε**. Por-device a GPU é determinística (o re-cook
  byte-idêntico da F1.1 prova); cross-vendor **nunca** foi prometido.
- **D4 — um nó sequencial ACUMULA o erro.** `x_{n+1} = f(x_n)` realimenta o ε: depois de N ticks a
  GPU e a CPU são **animações diferentes**, e **isso não é bug**. O gate mede **UM passo**.
- **D5 — o scrub não se vende por escala.** O ring de checkpoint vai pro **device**
  (`copy_buffer_to_buffer`, sem readback), **capeado por BYTES** (a lição do ADR-0117: contagem é
  multiplicador, não teto). Estourou → o sim recua pra CPU, que é onde o scrub bit-exato já mora.

---

## §5 — Armadilhas nomeadas (custaram iteração; você VAI esbarrar)

**Do domínio:**
- **`round` CPU↔GPU diverge no meio-ponto** (Rust half-away, WGSL half-even) — todo param-enum cai
  nisso. Use o helper `*_round` (half-away) ou thresholds `< 0.5`/`< 1.5` (concordam com o round no
  domínio admitido). [[feedback_cpu_gpu_rounding_conventions_diverge]].
- **noise/hash inteiro na GPU:** `bitcast<u32>(i32)` (== Rust `as u32`; **`u32(x)` é value-cast e
  diverge em negativos**); u32 do WGSL wrappa mod 2³² como `wrapping_*`; constantes com sufixo `u`.
  Divergiu 1 célula → o valor pula pra outro pseudo-aleatório (O(amplitude), não ε).
- **identidade constante NÃO expressa fallback posicional:** `ColumnBinding.identity` é uma
  CONSTANTE, não `f32(i)` — foi por isso que o Gradient do `tint` ficou de fora. **O `integrate` vai
  bater nisto** (o pareamento posicional precisa de `i`, que o corpo TEM; o por-`id` precisa de gather).
- **`applicable` só vê PARAMS, não colunas** (D3) — a condição "o stream tem `id`?" não cabe nele
  hoje. É a decisão pequena da sua fatia 3; **o default é recuar**, nunca responder errado.
- **`array<vec3<f32>>` tem stride 16** — coluna Vec3 nova: `element_stride` é a única porta.
- **`source_count` = `param_as_count` EXATO** (floor + clamp + produto saturado); o corpo WGSL repete
  o mesmo floor/clamp pros derivados.
- **Não segure o `&mut` do `entry().or_insert_with()`** através de `pool.acquire`/`uniform_slot` —
  insira, solte, re-busque (precedentes em `lib.rs`).
- **Semântica de ausência de coluna:** teste também com a coluna AUSENTE. O grid não emite
  `falloff`/`size`/`rot`/`tint`, então os gates já rodam com o alvo ausente — mantenha isso.
- **Duas portas divergem:** o pump de sink e o de fronteira compartilham UM `CookTarget` de
  propósito. 3º consumidor? **estenda o enum, não copie a marcha.**
  [[feedback_two_doors_to_the_same_question_diverge]].

**Do ferramental (me custaram tempo real nesta jornada):**
- **Crase em mensagem de commit = substituição de comando.** `` `as u32` `` fez o fish rodar o
  assembler e **apagou o trecho da mensagem**. Use `git commit -F <arquivo>`.
  [[feedback_backticks_in_commit_message_are_command_substitution]].
- **Pipe mascara o exit code.** `typos … | tail && echo OK` checa o `tail`, não o `typos`.
  [[feedback_pipe_masks_script_exit_code]].
- **`docs/**/*.md` está EXCLUÍDO do typos project-wide** (`.typos.toml` `[files]`). Um `typos
  <doc.md>` avulso vai acusar pt-BR ("Fases") que o CI **nunca** vê. Rode `typos` sem argumento (a
  invocação do ship) e ignore o resto.
- **LOC cap = 600, e SPLIT (nunca allowlist)** ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).
  **Perto do teto agora:** `motion_state_tests.rs` **572** · `motion_bridge.rs` **547** ·
  `gpu_cpu_parity.rs` — a próxima adição provavelmente pede split.
- **`NodeId` é tuple struct** `NodeId(pub u32)` — não existe `NodeId::new`.
- **"Audit = compilar" é falso** ([[feedback_painter_inefficiency_4_causes]]) — verde de naga não
  prova número; **o gate de paridade É o audit**.
- **Meça em `--release`** e na RTX ([[reference_display_topology_workstation]]).

---

## §6 — Gate de fechamento + protocolo

`cargo check --workspace --all-targets` · `architecture_contract_surface` **verde (8/2/1)** ·
`cook_determinism` + `transform_determinism` · **os 9 gates de paridade da Fase 2 ainda verdes**
(é o oráculo de que o DAG não regrediu o linear) + a paridade nova + naga · nextest nos crates
tocados + shell · clippy `--all-targets` + `typos` (sem argumento) + `file_loc_caps` + `cargo machete`
· smoke command pronto no handoff (caminho ABSOLUTO + `-p ph2d-host-desktop`).

Depois: **feche a linha, escreva o handoff da próxima fatia, e PARE** — não integre, não pushe, não
rode ship. **Só por ordem EXPLÍCITA do Enio**, via integrador dedicado.

### Para o integrador (quando o Enio mandar)

- **Base:** `line/cook-parallel` (Fase 0) → F1.1 → Fase 2/F1.2 → a sua, tudo em `line/gpu-nodes` em
  ordem de commit. Fast-forward natural. Marcos: `74a19784` (Fase 0) · `74aa2b00`..`e7605cfd` (F1.1)
  · `6325a3a8`/`86a2fe35` (Fase 2) · `f877b8a0`/`88326d00`/`8c018447` (F1.2) · `72301921` (2M) ·
  `4d176f9d` (ADR-0127).
- **Foundational tocado até aqui:** `ph2d-nodegraph` (+`gpu.rs`) · `ph2d-node-registry` · `ph2d-render`
  · `ph2d-eval-motion` (pump) · 10 node crates · shell · crate nova `ph2d-gpu-cook`.
- **Conflitos esperados:** `Cargo.lock` (dev-deps novas) · o número do ADR-0126/0127 se outra linha
  reivindicou (renumerar; os stamps vão junto). Contrato **8/2/1** intocado — nenhum escape §1.5.5.

# BRIEFING — continuação de `line/gpu-nodes` (GPU/M5 **F1.2 + Fase 2**): portar os nós pro motor

> Para o **novo agente implementador**. A F1.1 landou e foi **aprovada no smoke pelo Enio**
> (2026-07-15): o motor GPU-resident existe, o chain `grid → oscillator → move → output`
> cozinha inteiro na GPU e o renderer lê o buffer sem readback. Você constrói em cima.
> **Autor:** o agente da F1.1 (`line/gpu-nodes`), 2026-07-15, a pedido do Enio.
> Leia inteiro antes de tocar em código — é curto de propósito.

---

## §0 — ABERTURA (faça ANTES de qualquer coisa)

1. **Leia, nesta ordem:**
   - [`HANDOFF_line_gpu_nodes_fase1_2026-07-15.md`](HANDOFF_line_gpu_nodes_fase1_2026-07-15.md) —
     o que você herda (as peças, os gates, os gotchas §"Gotchas"). **É o documento central.**
   - [`docs/architecture/decisions/0122-…`](architecture/decisions/0122-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md) —
     a decisão de contrato (kernel = metadata LATERAL; agora com stamp de implementação).
   - [`docs/plans/2026-07-gpu-resident-node-pipeline.md`](plans/2026-07-gpu-resident-node-pipeline.md) —
     o roadmap (você faz a Fase 2; Fases 3–4 são journeys futuros).
   - `CLAUDE.md` §0 (os 7 inegociáveis) + §6 (contratos congelados).
2. **Abra a linha:**
   `git worktree add Worktrees/line-gpu-nodes-2 -b line/gpu-nodes-2 line/gpu-nodes`
   — ramifique de **`line/gpu-nodes`** (HEAD `41e7a461`+), **não** de `main` nem de
   `line/cook-parallel`: você precisa do motor. O integrador do Enio landa as fases em ordem
   (Fase 0 → F1.1 → a sua).
3. **As regras permanentes (Modo L):** trabalhe SÓ no seu worktree (**SEMPRE prefixe
   `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes-2 &&`** — o cwd escorrega
   pro repo primário, aconteceu nas duas jornadas anteriores) · foundational é editável aqui
   (ADR-0107) · `git commit --no-verify` · inner loop = `cargo check -p` · gates 1× no
   fechamento · **NÃO integre, NÃO pushe, NÃO rode `ship.sh`** — feche, escreva o handoff,
   PARE (§0.7). · **Se sentir vontade de bumpar o `NodeManifest`: PARE e releia o ADR-0122** —
   a resposta é sempre o canal lateral (`register_gpu_kernel`).

---

## §1 — A missão (em ordem; feche cada fatia antes da próxima)

1. **F1.2 — o híbrido no shell.** O MOTOR já cozinha prefixo-CPU/sufixo-GPU (o gate
   `the_hybrid_boundary_chain_matches_the_cpu_within_epsilon` está verde), mas o
   `motion_bridge` só liga o caminho GPU quando o plano é **fully-GPU**. Ligue o caso
   boundary: cozinhe o nó de fronteira pela via CPU que JÁ existe e entregue o stream ao
   `GpuCook::cook(…, Some(&stream), …)`.
2. **Fase 2 — portar os nós hot, um `register_gpu_kernel` por vez**, cada um com seu gate
   (§4). Ordem por impacto (do plano §3): deformers O(N) puros primeiro
   (`transform`/`rotate`/`scale` → `twist`/`bend`/`look_at`) · `wiggle`/`tint`/`falloff` ·
   os canais Rotation/Size do `oscillator` · forças (`force.wind`/`drag`/`attractor`/
   `vortex`/`curl`) — **as forças param antes do `integrate`** (estado entre frames é a
   fatia 3).
3. **(Se sobrar) o estado na GPU:** `motion.integrate`/`spring` — o `pre` vira ping-pong de
   buffer ENTRE frames (plano §3 Fase 1 "Estado"). É uma extensão do MOTOR (o plano hoje
   refuta `pre`), não um port de kernel. Desenhe antes de codar; pode virar o journey seguinte.

**Não-metas desta linha:** JFA/voronoi e spatial-hash/boids (Fase 3) · renderer consumindo
colunas cruas sem lowering (Fase 4) · mudar o default do `PH2D_GPU_COOK` (decisão da Fase 4,
depois que os readouts do painel lerem a GPU).

---

## §2 — Onde a F1.1 te deixou (o que você herda, JÁ no seu branch)

- **O motor (`crates/ph2d-gpu-cook/`):** `plan()` (sufixo-GPU/prefixo-CPU, puro, por-frame) ·
  `GpuCook::cook()` (single submit; pipelines cacheadas por `(tipo, presença-de-colunas)` —
  params/playhead são uniforms, NUNCA recompilam) · `GpuStream`/`BufferPool` (colunas
  imutáveis `Arc<wgpu::Buffer>`, write = buffer novo do pool, steady state zero-alloc de
  buffer) · `codegen.rs` (gera o módulo WGSL; `plan_bindings` é a MESMA função que monta o
  bind group) · `lower.rs` (compute → layout `RenderInstance`, 46 words) ·
  `read_instances()` (o readback deliberado dos gates).
- **O contrato do kernel (`ph2d_nodegraph::gpu`):** o corpo vê `i`, `params.*`,
  `read_<col>(i)`/`write_<col>(i, v)`; helpers module-level vão em `wgsl_lib`;
  `ColumnAccess::{Read, Write, ReadWrite (materializa da identidade), ReadWriteExisting
  (dropa o write se a coluna não existe)}`; `source_count` (geradores) e `applicable`
  (cobertura parcial de params) são avaliados no plan, na CPU.
- **4 kernels de referência** (copie o formato): `motion.grid` (gerador),
  `motion.oscillator` (X/Y + waveform HR-5 no `wgsl_lib` + `applicable`), `motion.move`
  (`ReadWriteExisting`), `motion.output` (`PASSTHROUGH`).
- **Perf baseline:** 500k instâncias = **1,0 ms/frame** na RTX (CPU Fase 0: 4,93 ms). Sonda:
  `gpu_cook_500k_timing` (`#[ignore]`).
- **Smoke aprovado:** `PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=1 cargo run --release -p
  ph2d-host-desktop` + tool Motion (262k instâncias, onda viajante).

---

## §3 — A RECEITA de port por nó (Fase 2 — repita isto N vezes)

Para cada nó, nesta ordem — **não pule o passo 1**:

1. **Leia a `eval` da CPU inteira** (`crates/ph2d-node-motion-<x>/src/lib.rs`). Classifique:
   - *map puro por-elemento?* → portável direto.
   - *tem REDUÇÃO sobre todas as instâncias?* (`twist`/`bend` têm — o fold `r_max`/`x_extent`
     que a Fase 0 deixou serial de propósito) → **a redução NÃO cabe num kernel por-elemento**.
     Opções honestas: (a) um passe de redução no motor (extensão; desenhe antes), (b) deixar o
     nó fora e o plano recuar (correto, só não acelerado). NÃO compute a redução na CPU lendo
     coluna da GPU — isso é readback no hot path, o anti-padrão.
   - *lê vizinho / estado pareado / `pre`?* → fora da Fase 2 (fatia 3 ou Fase 3).
2. **Escreva o `GpuKernel` const** no crate do nó (drop-crate isolation — o kernel mora com o
   dono): corpo em `wgsl`, helpers em `wgsl_lib` (prefixe `<nó>_` — ex.: `osc_wave`), bindings
   com o `ColumnAccess` que ESPELHA a semântica de ausência da CPU (materializa vs. ignora —
   confira no código, não chute), `identity` = o fallback que a CPU usa (falloff→1,
   size→`SIZE_IDENTITY`, P/rot→0), `applicable` se a cobertura for parcial, `source_count`
   pra gerador (espelhe `param_as_count` + caps EXATOS).
   - **HR-5:** porte as aproximações polinomiais do próprio nó (cada nó tem seu `trig.rs`/
     helpers — copie a matemática, não chame `sin`/`cos` do WGSL no lugar de uma aproximação
     da CPU; a exceção é o lowering, que é PresentWorld).
3. **`reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL)`** no `register()` do crate.
4. **Gates (por nó, antes de ir pro próximo):**
   - Adicione o crate ao `registry()` de `crates/ph2d-gpu-cook/tests/generated_wgsl_validates.rs`
     **e** de `gpu_cpu_parity.rs` (+ dev-dep no `Cargo.toml` do ph2d-gpu-cook) — o naga então
     valida TODO subconjunto de colunas do seu kernel de graça.
   - **Paridade ε**: um teste no padrão dos existentes — chain real com o nó, ≥25,6k
     instâncias (acima do `PAR_THRESHOLD`), params NÃO-default e NÃO-inteiros-bonitos
     ([[feedback_test_with_product_numbers_not_convenient_ones]]), assert `is_fully_gpu` +
     `dispatching_stages` ANTES de comparar (o gate tem que provar que DISPARA), campos
     dentro de ε com budget DERIVADO (veja o cabeçalho de `gpu_cpu_parity.rs` — copie o
     raciocínio, não o número).
   - `cargo check -p <crate> -p ph2d-gpu-cook` no inner loop; o resto 1× no fechamento.

---

## §4 — F1.2 (o híbrido no shell) — o desenho

Hoje em `motion_bridge.rs`: `if plan.is_fully_gpu() { gpu_cook.cook(…, None, …) } else
{ pump CPU inteiro }`. O que falta: o meio-termo `plan.boundary == Some((node, port))`.

- **Quem cozinha a fronteira é o `pump.cook`** (o `Cook` persistente do `MotionCookPump`) —
  NUNCA um `Cook` novo por frame: o memo e o `pre` feedback moram nele, e um nó sequencial no
  prefixo (emitter/integrate acima da fronteira) PRECISA da marcha de ticks do pump
  (`ticks_owed`/`advance_or_scrub_scoped`). O caminho honesto: marche os ticks como hoje, mas
  com o COOK dirigido ao nó de fronteira em vez do sink, e sem o `lower_to_instances_onto`
  (o lowering é da GPU agora). Isso provavelmente pede um método novo no pump (ex.:
  `pump_to_node(...) -> &Stream`) — **`ph2d-eval-motion` é foundational, você PODE** (Modo L);
  desenhe-o pra servir os dois chamadores (o CPU-only de hoje reusa? cheque; duas portas pra
  mesma pergunta divergem — [[feedback_two_doors_to_the_same_question_diverge]]).
- O stream de fronteira sobe por `GpuCook::cook(…, Some(&stream), …)` — 1 upload/frame, a
  fronteira explícita. NÃO otimize o upload antes de medir.
- **Gate no seam do shell:** os testes do bridge são headless e não têm device — teste a
  DECISÃO (que plano/qual caminho o bridge escolhe, `gpu_live` vs pump) com um `GpuContext`
  opcional/mock na função de decisão extraída, não o dispatch inteiro. A paridade do híbrido
  em si já está gateada no motor.
- Depois do wire: atualize o demo (`PH2D_GPU_COOK_DEMO`) ou crie um 2º chain com um nó
  sem kernel no topo, pro smoke do Enio VER o híbrido funcionando.

---

## §5 — Determinismo (decidido — NÃO reabra)

Igual à F1.1, sem mudanças: **CPU = caminho canônico** (`cook_determinism` +
`transform_determinism` + `c9_replay` intocáveis) · GPU = performance/preview, reconciliada
por **ε** (nunca gate bit-a-bit cross-OS/vendor; byte-igual só re-cook no MESMO device) ·
HR-5 portado kernel a kernel.

---

## §6 — Armadilhas nomeadas (custaram iteração na F1.1 — você VAI esbarrar)

- **`round` ≠ `round`:** Rust half-away, WGSL half-even — enum-por-param diverge de RAMO no
  meio-ponto. Use `osc_round`-style (half-away explícito) ou reformule pra comparação que
  concorda no domínio admitido. Memória:
  [[feedback_cpu_gpu_rounding_conventions_diverge]]. **Todo nó com param-enum cai nisso.**
- **`array<vec3<f32>>` tem stride 16** — colunas Vec3 novas na GPU: `element_stride` é a
  única porta; upload apertado desindexa.
- **`source_count` = `param_as_count` EXATO** (floor + clamp + produto saturado) — dispatch
  ≠ count da CPU mata a paridade na largada; o corpo WGSL repete o mesmo floor/clamp pros
  derivados (cx/cy do grid é o exemplo).
- **Não segure o `&mut` do `entry().or_insert_with()`** através de `pool.acquire`/
  `uniform_slot` — insira, solte, re-busque (2 precedentes em `lib.rs`).
- **"Audit = compilar" é falso** ([[feedback_painter_inefficiency_4_causes]]) — verde de
  naga não prova número; o gate de paridade É o audit, e ele afirma que o plano DISPARA
  ([[feedback_an_optimization_needs_a_gate_that_proves_it_fires]]).
- **Meça em `--release`** e na RTX ([[reference_display_topology_workstation]]); GPU em
  debug mente menos que CPU, mas mente.
- **Semântica de ausência de coluna:** cada nó da CPU decide sozinho se materializa
  (identidade) ou ignora a coluna ausente — LEIA o braço do match antes de escolher
  `ReadWrite` vs `ReadWriteExisting`; errar isso é paridade verde num fixture cheio e
  divergência silenciosa num stream magro ([[feedback_a_gate_only_proves_what_its_fixture_contains]]
  — teste também com a coluna AUSENTE).

---

## §7 — Gate de fechamento + protocolo (repita o que a F1.1 fez)

`cargo check --workspace --all-targets` · `architecture_contract_surface` **verde (8/2/1)** ·
`cook_determinism` + `transform_determinism` · paridade ε de TODO nó portado
(`--ignored`, na RTX) + naga (roda em qualquer lane) · nextest nos crates tocados + shell ·
clippy `--all-targets` + typos · smoke command pronto no handoff (caminho ABSOLUTO +
`-p ph2d-host-desktop`). Depois: **feche a linha, escreva o handoff da próxima fatia, e
PARE** — não integre, não pushe. O integrador do Enio reconcilia (conflitos esperados:
`Cargo.lock`; o resto da F1.1 é aditivo).

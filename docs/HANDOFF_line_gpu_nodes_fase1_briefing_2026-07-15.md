# BRIEFING — `line/gpu-nodes` (GPU/M5 **Fase 1**): o motor de nós GPU-resident

> Para o **novo agente implementador**. Isto te passa a próxima fase e a linha. Leia inteiro antes de
> tocar em código — é curto de propósito. A Fase 0 já landou; você constrói em cima dela.
> **Autor:** o agente da Fase 0 (`line/cook-parallel`), 2026-07-15, a pedido do Enio.

---

## §0 — ABERTURA (faça ANTES de qualquer coisa)

1. **Leia, nesta ordem:**
   - [`docs/plans/2026-07-gpu-resident-node-pipeline.md`](plans/2026-07-gpu-resident-node-pipeline.md) — o roadmap das 4 etapas.
   - [`docs/architecture/decisions/0122-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md`](architecture/decisions/0122-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md) — **ACEITO pelo Enio**; a decisão de contrato que você EXECUTA (§3 abaixo).
   - [`docs/HANDOFF_line_cook_parallel_2026-07-15.md`](HANDOFF_line_cook_parallel_2026-07-15.md) — o que a Fase 0 entregou (você herda).
   - `CLAUDE.md` §0 (os 7 inegociáveis) + §6 (contratos congelados).
2. **Abra a linha:** `git worktree add Worktrees/line-gpu-nodes -b line/gpu-nodes line/cook-parallel`
   — ramifique de **`line/cook-parallel`** (HEAD `74a19784`), **não** de `main`: a Fase 1 usa o cook da
   CPU (paralelizado na Fase 0) como caminho canônico + fallback, e o golden `cook_determinism`. O
   integrador do Enio landa a Fase 0 primeiro, depois esta.
3. **As regras permanentes (Modo L — valem até o fim):**
   - **Você trabalha no worktree DESTA linha.** Foundational é editável aqui (Modo L / ADR-0107) — inclusive
     o `NodeRegistry` e o `ph2d-render`. **NUNCA edite o `main` direto.** O Enio disse: *"liberdade de
     trabalhar na fundação **dessa linha**, não na do main"* — e que **o integrador reconcilia os conflitos
     no fim.** Então não trave por medo de conflito; trabalhe limpo e deixe o merge pro integrador.
   - **NÃO integre, NÃO pushe, NÃO rode `ship.sh`.** Você **fecha a linha, escreve o handoff, e PARA** (§0.7
     do CLAUDE.md). Integração e ship são só por ordem explícita do Enio, via integrador dedicado.
   - **`git commit --no-verify`** no worktree; `cargo check -p <crate>` no inner loop; gate 1× no fechamento.
   - **Determinismo é lei** (§6 abaixo). A CPU é o oráculo; a GPU nunca vira canônica.

---

## §1 — A missão

Levar o cook de **CPU-paralelo** (onde a Fase 0 o deixou) para **GPU-resident**: as colunas do `Stream`
(`P`/`vel`/`tint`/`size`/…) vivem em **buffers de GPU**; cada nó com kernel vira um **compute pass WGSL**; o
grafo é sequenciado por topologia num **único submit**; a saída é lida **direto pelo renderer, sem readback**.
Alvo final (Fase 4): **1M+ instâncias animadas a 60fps**. Você faz a **Fase 1** (a fundação do motor) e,
se sobrar, começa a **Fase 2** (portar os primeiros nós). As etapas 3–4 são journeys futuros.

---

## §2 — Onde a Fase 0 te deixou (o que você herda, JÁ no seu branch)

- **`ph2d_nodegraph::attr::par_build(n, f)`** + `PAR_THRESHOLD=8192` — o cook da CPU já usa todos os cores
  para os maps por-instância (16 nós retrofitados). **Este é o teu caminho CANÔNICO e o fallback.**
- **Golden `crates/ph2d-eval-motion/tests/cook_determinism.rs`** — reprodutibilidade + FNV pinado de um
  chain a 25.6k instâncias. **Todo kernel GPU que você portar tem que casar com a `eval` CPU dentro de ε; o
  golden protege a CPU.**
- **Perf medida:** cook de 500k = 4,93 ms na CPU (32 threads). É o teu baseline — a GPU precisa bater isso
  com folga (o alvo é milhões, não 500k).
- **O renderer JÁ desenha na GPU** (`ph2d-render`, instancing wgpu). O que falta é o **cook** virar
  GPU-resident e o renderer **ler os buffers do cook direto** em vez do marshalling CPU.

---

## §3 — O contrato: kernel GPU é metadata LATERAL (ADR-0122, ACEITO)

**NÃO toque o `NodeManifest`.** O kernel entra por um canal novo no `NodeRegistry`, exatamente como a UI:

- `reg.register_gpu_kernel(id, GpuKernel)` (espelho de `register_ui`) — **opt-in**: um nó sem kernel não
  chama nada, não muda em nada. `NodeManifest=8` / `NodeOp=2` / `OpResolver=1` ficam **intactos** (gate
  `architecture_contract_surface` **verde** — confira que continua verde).
- `GpuKernel { wgsl: &'static str, bindings: &'static [ColumnBinding], params: &'static [&'static str] }`,
  tipo novo em `ph2d-nodegraph`. `ColumnBinding` = nome da coluna + modo (In / Out / InOut→ping-pong).
- **Por quê:** um campo no `NodeManifest` obrigaria `gpu_kernel: None` em 92 literais **e** descongelaria o
  §6. O canal lateral não tem nem um nem outro — e é o padrão que manteve o contrato intacto a linha Motion
  inteira (params no `Graph`, UI no registry). **Se você sentir vontade de bumpar o `NodeManifest`, PARE —
  releia o ADR-0122; a resposta é o canal lateral.**

---

## §4 — O 1º corte concreto (não tente o motor inteiro de uma vez)

**Fatia F1.1 — um chain simples 100% na GPU, alimentando o renderer sem readback.**
Escolha o chain mais simples: `grid → oscillator → output` (ou `grid → move → output`). Entregue:

1. **`GpuStream`** (crate nova, ex. `ph2d-gpu-cook`, ou módulo em `ph2d-render`/`ph2d-eval-motion`): colunas
   como `wgpu::Buffer` + len/dtype + um pool. **Isolada de propósito** (foundational append-only).
2. **`register_gpu_kernel` no `NodeRegistry`** + o tipo `GpuKernel` no `ph2d-nodegraph` (§3).
3. **Kernels WGSL** para os 2-3 nós do chain (`grid` gera; `oscillator`/`move` transformam). Reuse as
   aproximações HR-5 (sin parabólico) nos kernels WGSL — **paridade com a CPU dentro de ε** (não bit-a-bit).
4. **O sequenciador:** topologia → compute passes num submit; ping-pong onde um nó escreve o que lê; o
   `pre`/estado é ping-pong entre frames. Nó sem kernel → readback + `eval` CPU (a fronteira explícita).
5. **O renderer lê os buffers direto** (pular o `lower_to_instances_onto` para esse chain) — o `game_rt`
   já existe; o instance buffer passa a ser a coluna GPU. Caminho híbrido para cenas CPU-only.
6. **Gate de paridade** (o padrão [[project_painter_w4_spatial_gpu_bloom_sh]]): um dev-dep/teste headless
   que cozinha o chain na CPU E na GPU e compara **dentro de ε** (não bit-a-bit). Headless GPU roda
   (`--features gpu -- --ignored`, [[reference_gpu_tests_run_headless_metal]]; na workstation Linux mede-se
   no LG/RTX — [[reference_display_topology_workstation]]).

**Done da F1.1:** o chain simples roda 100% na GPU, o renderer lê sem readback, e a paridade GPU-vs-CPU
passa. Aí sim a Fase 2 porta os outros nós, um `register_gpu_kernel` de cada vez, cada um com seu gate.

---

## §5 — Determinismo (o problema difícil, decidido — NÃO reabra)

- **A CPU é o caminho CANÔNICO.** `transform_determinism` (ECS), `c9_replay`, e o `cook_determinism` (Fase 0)
  rodam sobre a `eval` da CPU e **não podem quebrar**. Se o replay-hash canônico precisar de um valor, ele
  vem da CPU (readback), nunca da GPU.
- **A GPU é performance/preview**, reconciliada por **tolerância (ε)**, porque float na GPU não é
  bit-reproduzível cross-vendor/driver. **Nunca** faça um gate GPU bit-a-bit cross-OS — ele vai piscar.
- HR-5 na GPU: WGSL não tem as garantias transcendental-free do Rust; **porte as aproximações polinomiais**
  (sin/cos parabólico dos `trig.rs` dos nós, o atan2 do look-at) para os kernels, pra manter a paridade ε.

---

## §6 — Os seams no código (onde tudo está)

- **O cook (CPU, canônico):** `crates/ph2d-nodegraph/src/cook.rs` — `Cook::cook_node` (recursão memoizada).
  O `LoweringKind` já tem `Wgsl` desenhado: `crates/ph2d-nodegraph/src/node.rs` (~:91) — nunca usado; é o
  gancho que o design PREVIU.
- **O pump + marshalling:** `crates/ph2d-eval-motion/src/lib.rs` — `MotionCookPump::pump` → `cook_sinks_into`
  → `lower_to_instances_onto` (o que a GPU-residência vai pular). A Fase 0 já paralelizou os dois.
- **O registry (onde o canal lateral entra):** `crates/ph2d-node-registry/src/lib.rs` — `register_ui`,
  `register_param_ui` são o molde exato do `register_gpu_kernel`. `NodeRegistry` impl `OpResolver`.
- **O renderer GPU:** `crates/ph2d-render/` — `SpriteRenderer`, o instance buffer, o `game_rt` HDR. É onde a
  coluna GPU vira instance buffer sem readback.
- **Prior art GPU-resident (não comece do zero):** [[project_wash_gpu_resident_reimpl]] ·
  [[project_watercolor_v2_gpu_first_refactor]] (single-submit, sparse, sem fallback CPU no hot path) ·
  [[project_painter_composite_perf_2026_06_03]] (WGSL compute, Metal 1,7ms vs 55ms CPU) ·
  [[project_painter_fluid_4k_perf_architecture]] (o anti-padrão: hot loop CPU + readback por-frame).

---

## §7 — Gate de fechamento + protocolo (repita o que a Fase 0 fez)

- `cargo check --workspace` (o `foundational-integrate.sh` força — você toca `ph2d-nodegraph`/`-registry`/
  `-render`/`-eval-motion`, todos foundational).
- `architecture_contract_surface` **verde (8/2/1)** — prova que você NÃO tocou o contrato (o canal lateral).
- `cook_determinism` + `transform_determinism` verdes (a CPU não regrediu).
- O **gate de paridade GPU-vs-CPU** (ε) do teu chain.
- clippy `--all-targets` + typos, 1× no fechamento.
- **Feche a linha, escreva o handoff (o próximo — F1.2 ou Fase 2), e PARE.** Não integre, não pushe. O
  integrador do Enio reconcilia com o `main` (ADR número, Cargo.lock, registry-init são os conflitos
  esperados; ele resolve).

---

## §8 — Armadilhas nomeadas (custaram caro em outras linhas)

- **"Audit = compilar" é falso** ([[feedback_painter_inefficiency_4_causes]]) — verde-de-compilação não
  prova a GPU. Meça o pixel/o buffer. O gate de paridade É o audit.
- **Meça em `--release`** ([[project_painter_composite_perf_2026_06_03]]) — debug mente sobre GPU.
- **O harness reproduz o mecanismo, não o contexto** ([[feedback_harness_reproduces_mechanism_not_context]]) —
  um teste GPU headless que passa não prova o app; smoke no fim.
- **Readback por-frame é o anti-padrão** ([[project_painter_fluid_4k_perf_architecture]]) — a fronteira
  CPU↔GPU tem que ser rara e explícita, não um copy por nó.
- **cwd escorrega para o primário** — SEMPRE coloque `cd Worktrees/line-gpu-nodes &&` na frente dos comandos cargo/git,
  senão você builda/comita no repo errado (aconteceu comigo nesta jornada).

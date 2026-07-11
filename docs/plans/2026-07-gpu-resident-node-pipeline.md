# Plano — Motor de nós GPU-resident (o "animar milhões de cópias")

**Status:** PROPOSTA / fila. Aberto por pedido do Enio (2026-07-11): *"coloque na fila tudo que for
necessário para ter a implementação mais poderosa da Galáxia."* **NÃO iniciado.** Isto é o roadmap; a
execução é uma **linha foundational dedicada + ADRs** (ver §7) — **não** deve ser enxertada na linha de fan-out
de Motion nodes (mataria a integração limpa dela).

> Escopo: levar a avaliação do grafo de nós (Motion e além) de **CPU single-thread** para um pipeline
> **GPU-resident** capaz de **milhões de instâncias animadas a 60fps**, com o renderer lendo os buffers da GPU
> **sem readback**. Mantém os inegociáveis (ECS-decoupling, determinismo, HR-5 onde aplicável).

---

## 1. Estado atual (VERIFICADO — 2026-07-11, não é chute)

| Camada | Onde roda | Arquivo | Escala |
|---|---|---|---|
| **Render das instâncias** | **GPU** (instancing wgpu, buffer por-frame, agrupado por textura) | `ph2d-render/src/sprite/instance.rs` | ✅ ~100k+ @ 60Hz ([[project_m5_perf_validated]]) |
| **Cook dos nós** | **CPU, 1 thread** — walk topológico chamando `op.eval` em série | `ph2d-nodegraph/src/cook.rs:471` | ❌ 1 de 32 cores |
| **Lowering GPU** | `LoweringKind { Cpu, Wgsl, Luau }` — **`Wgsl` desenhado, NUNCA usado** | `ph2d-nodegraph/src/node.rs:91` | ❌ zero nós |
| **Pump → instâncias** | CPU: `Stream` (colunas `Vec<f32>`) → `Vec<RenderInstance>` → upload | `ph2d-eval-motion/src/lib.rs` | marshalling CPU por-frame |

**Diagnóstico:** a GPU **desenha** milhões; quem **calcula** as posições é a CPU num core só, e depois faz
marshalling CPU→GPU por-frame. Nós O(N) (grid/oscillator/move/transform) processam N instâncias em série; nós
O(N²) (voronoi Lloyd, boids) são patológicos. `LoweringKind::Wgsl` prova que o design PREVIU a GPU — nunca foi
construído.

**Prova de conceito já feita (2026-07-11):** `motion.voronoi` paralelizou o hot loop com rayon
(bit-idêntico ao serial, replay-hash intacto) → count 180 de ~20ms → 3ms em debug. É um aperitivo da Fase 0;
não muda o `O(count²)` nem toca o contrato.

## 2. O norte (target architecture)

Um **pipeline GPU-resident**: as colunas do `Stream` (`P`/`vel`/`tint`/`size`/…) vivem em **buffers de GPU**;
cada nó vira um **compute pass WGSL** que transforma esses buffers (ping-pong quando escreve o que lê);
o grafo é sequenciado por topologia num **command encoder**; o `pre`/estado sequencial é ping-pong de buffer
entre frames; a saída final é **lida direto pelo renderer** (mesmos buffers, zero readback). Readback CPU só
quando um consumidor CPU (ou o replay-hash canônico) precisa.

Regra-mãe (lição [[project_wash_gpu_resident_reimpl]] / [[project_watercolor_v2_gpu_first_refactor]]):
**GPU-resident, single-submit, sem fallback CPU no hot path** — a lentidão anterior (wash) era submit/copy-bound.

## 3. Roadmap por fases (do barato+seguro ao ambicioso)

### Fase 0 — Paralelizar o cook na CPU (rayon) · **sem tocar contrato · pode ser a 1ª linha**
- **Graph-level:** sub-árvores independentes cozinham em paralelo (as 2 cenas do demo, p.ex.). No `Cook`.
- **Intra-node:** helper de `par_map` sobre instâncias no substrato; retrofit dos nós hot O(N)
  (grid/oscillator/transform/tint/forces/integrate). Cada nó opta-in.
- **Determinismo:** redução em ordem fixa (como o voronoi já faz) → replay-hash intacto.
- **Ganho:** ~10–20× na workstation; destrava ~100k–1M em nós O(N) na CPU. **Toca:** `ph2d-nodegraph::cook` +
  `ph2d-eval-motion` (foundational — Modo L pode, com cuidado). **NÃO** toca `NodeOp`/`NodeManifest`.
- **Done:** grid de 500k @ 60fps num chain O(N); nextest verde; replay-hash igual.

### Fase 1 — Buffers GPU-resident + runtime de lowering WGSL · **EXIGE ADR (descongela contrato)**
- **Representação:** `GpuStream` = colunas como `wgpu::Buffer` + metadados (len, dtype). Alocação/pool.
- **Contrato de kernel:** um nó com `LoweringKind::Wgsl` fornece (a) o corpo WGSL, (b) o layout de bindings
  (quais colunas lê/escreve), (c) uniforms de params. **Isso ESTENDE `NodeManifest`/`NodeOp`** — o contrato
  está CONGELADO (§6 do CLAUDE.md: `NodeOp=2`/`NodeManifest=8`) → **ADR obrigatório** (Coord/Enio).
- **Sequenciador:** topologia → sequência de compute passes num submit; ping-pong pra nós que escrevem o que
  leem; barreiras entre dependências.
- **Estado (`pre`):** o feedback sequencial vira ping-pong de buffer entre frames (o `sim_t`/`dt` já é dado).
- **Fallback híbrido:** nós sem kernel WGSL (ou o replay canônico) fazem readback e rodam na CPU; a fronteira
  CPU↔GPU é explícita (minimizar cruzamentos).
- **Done:** um chain simples (grid→oscillator→output) roda 100% na GPU, alimentando o renderer sem readback.

### Fase 2 — Portar os nós hot pra kernels WGSL
- Ordem por impacto: `grid`/`lattice`/`fibonacci` (source) · `transform`/`move`/`rotate`/`scale`/`twist`/
  `bend`/`look_at` (deformers O(N)) · `oscillator`/`wiggle`/`tint` · forças + `integrate`/`spring`
  (sim O(N), ping-pong de estado). Cada um: kernel WGSL + paridade bit-a-bit reconciliada com a versão CPU
  (padrão [[project_painter_w4_spatial_gpu_bloom_sh]] — dev-dep que compara GPU vs CPU).
- HR-5 na GPU: WGSL não tem as mesmas garantias transcendental-free; reusar as aproximações polinomiais
  (parabolic sin/cos, Rajan atan2) nos kernels pra manter paridade.

### Fase 3 — Estruturas espaciais na GPU (os O(N²) que rayon não salva)
- **voronoi:** **Jump Flooding (JFA)** na GPU (Rong & Tan 2006) — Voronoi/CVT em O(passes·pixels),
  independente do count → milhões de sementes. Substitui o grid-Lloyd CPU no hot path.
- **boids:** **spatial hash / uniform grid** na GPU (bucket por célula) → vizinhança O(N) → milhões de agentes.
- **soft_body/verlet:** XPBD/Verlet em compute (constraints em paralelo, Jacobi/coloring).

### Fase 4 — Renderer consome os buffers GPU direto
- Cena GPU-resident: pular o marshalling `Stream → Vec<RenderInstance>` na CPU; o renderer lê as colunas GPU
  como instance buffers (`P`/`rot`/`size`/`tint`/`uv`). Caminho híbrido pra cenas CPU-only.
- **Done:** 1M+ instâncias animadas por um chain GPU, 60fps, zero readback no hot path.

## 4. Os problemas difíceis (decidir cedo)

- **Determinismo cross-vendor na GPU:** float na GPU não é bit-reproduzível entre fornecedores/drivers → o
  **replay-hash** (gate de CI) quebra se o caminho canônico for GPU. **Decisão:** a **CPU continua o caminho
  CANÔNICO** do replay-hash (determinístico, HR-5); a GPU é o caminho de **performance/preview**, reconciliado
  por tolerância nos gates de paridade (não bit-a-bit). Mesma filosofia da lição "scrub não pode divergir do
  playback" ([[project_wash_undo_event_driven_rebuild]] e o survey de determinismo).
- **Descongelar o contrato de nós (§6):** a Fase 1 estende `NodeManifest`/`NodeOp` → ADR que amende
  [ADR-0039](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md). Projetar a extensão como
  **append-only** (campo `gpu_kernel: Option<…>`) pra não quebrar os 52 nós existentes.
- **Escopo/risco:** é um motor novo. Fasear com gates de paridade a cada nó portado; a CPU nunca sai (é o
  oráculo + o fallback).

## 5. Prior art no repo (não começar do zero)

- [[project_wash_gpu_resident_reimpl]] · [[project_watercolor_v2_gpu_first_refactor]] — GPU-resident,
  single-submit, sparse; portar física B1–B9 sem fallback CPU.
- [[project_painter_composite_perf_2026_06_03]] — compositor WGSL, Metal 1.7ms vs 55ms CPU.
- [[project_painter_fluid_4k_perf_architecture]] — hot loop O(grid) CPU + readback é o anti-padrão; alvo
  GPU-resident + `cs_splat`.
- [[project_painter_w4_spatial_gpu_bloom_sh]] — padrão de reconciliação bit/tolerância GPU-vs-CPU via dev-dep.
- [[reference_gpu_tests_run_headless_metal]] — testes GPU rodam headless (`--features gpu -- --ignored`).

## 6. O que JÁ está pronto / próximo passo

- **Pronto (aperitivo P0):** `motion.voronoi` paralelizado (rayon, determinístico). Confirma que dá pra usar
  os cores sem quebrar o replay-hash.
- **Próximo passo REAL:** abrir uma **linha foundational dedicada** (worktree próprio, Modo L) pra Fase 0, e um
  **ADR** pra Fase 1 antes de tocar o contrato. **É ordem do Enio** — não parte de uma linha de fan-out.

## 7. Como executar (respeitando o protocolo)

1. **Fase 0** = linha foundational própria (`line/cook-parallel`), toca `ph2d-nodegraph::cook` +
   `ph2d-eval-motion`, sem contrato → integra pelo `foundational-integrate.sh`.
2. **Fase 1+** = ADR (descongela o contrato) + linha própria `line/gpu-nodes`. Coord/Enio-level.
3. A linha de **Motion nodes** (esta) segue **aditiva e limpa** — os nós que ela cria herdam a GPU de graça
   quando a Fase 2 portar cada tipo. **Nada aqui bloqueia o motor GPU; e o motor GPU não bloqueia os nós.**

> Resumo: o engine JÁ renderiza na GPU; falta o **cook GPU-resident**. Fase 0 (rayon) é ganho imediato sem
> contrato; Fase 1 (WGSL lowering) é o motor real e exige ADR; Fases 3–4 levam voronoi/boids a milhões via
> JFA/spatial-hash. A CPU permanece o oráculo canônico (determinismo/replay-hash). Aguardando "go" do Enio pra
> abrir a linha foundational.

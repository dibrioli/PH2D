# ADR-0122 — O kernel GPU de um nó é metadata LATERAL (o contrato congelado NÃO é tocado)

- **Status:** **ACEITO** (Enio, 2026-07-15: *"temos a liberdade para trabalhar na fundação **dessa linha**
  e não na fundação do main"* + seguir para a Fase 1). O trabalho da Fase 1 é feito **no worktree da linha
  `line/gpu-nodes`** (Modo L / ADR-0107: foundational é editável pela linha), **nunca editando o `main`
  direto**; o **integrador reconcilia com o `main` no fim** (o Enio disse que resolve os conflitos lá).
- **NÃO amende** [ADR-0039](0039-nodegraph-contract-freeze-w2t4.md) — ao contrário, **reafirma o freeze**:
  o kernel GPU entra por um canal lateral no registry, então `NodeOp=2` / `OpResolver=1` / `NodeManifest=8`
  ficam **intactos** (gate `architecture_contract_surface` verde).
- **Número tentativo:** linhas paralelas podem reivindicar 0122 antes; renumerar na integração se colidir.
- **IMPLEMENTADO (F1.1, `line/gpu-nodes`, 2026-07-15):** `ph2d_nodegraph::gpu` (tipos) +
  `NodeRegistry::register_gpu_kernel` + crate **`ph2d-gpu-cook`** (plano sufixo-GPU/prefixo-CPU,
  sequenciador single-submit, lowering compute → layout `RenderInstance`, zero readback) +
  `SpriteRenderer::render_with_streams` + shell `PH2D_GPU_COOK=1`. Gate de paridade ε verde
  (full 4,4e-4 · híbrido bit-exato); contrato **8/2/1 intacto**; 500k instâncias = 1,0 ms/frame
  na RTX (CPU Fase 0: 4,93 ms). Handoff: `docs/HANDOFF_line_gpu_nodes_fase1_2026-07-15.md`.

## Contexto

A GPU **desenha** milhões de instâncias, mas quem **calcula** era a CPU num core só. A **Fase 0**
([plano](../../plans/2026-07-gpu-resident-node-pipeline.md), linha `line/cook-parallel`) já paralelizou o
cook na CPU com rayon — ganho imediato, sem tocar o contrato. A **Fase 1** é o motor real: as colunas do
`Stream` viram **buffers de GPU** e cada nó vira um **compute pass WGSL**, com o renderer lendo os buffers
**sem readback**. A pergunta de contrato: **como um nó fornece seu kernel WGSL** (corpo + bindings + params)
sem quebrar o congelamento §6?

## Decisão — o kernel é registrado DO LADO, como a UI

**Não tocar o `NodeManifest`.** O módulo já tem o padrão canônico de dar a um nó dados extras sem mexer no
manifesto congelado, e o usou a linha Motion inteira: **os params vivem no `Graph`** (não no manifesto), **o
text-param idem**, **a UI e os param-hints são registrados do lado** (`reg.register_ui(id, …)` /
`reg.register_param_ui(id, …)`). O kernel GPU segue **exatamente** essa via:

1. **`reg.register_gpu_kernel(id, GpuKernel)`** — um canal novo no `NodeRegistry` (como `register_ui`),
   keyado por `NodeTypeId`. **Opt-in:** um nó sem kernel não chama nada e não muda em NADA. Zero churn nos
   92 nós; zero campo novo em `NodeManifest`; o gate `architecture_contract_surface` fica em **8/2/1**.
2. **`GpuKernel` (tipo novo, `'static`):** `{ wgsl: &'static str (corpo do compute), bindings: &'static
   [ColumnBinding] (quais colunas lê/escreve + o modo — in/out/inout→ping-pong), params: &'static [&'static
   str] (uniforms) }`. Tudo `&'static`.
3. **`NodeOp`/`NodeManifest`/`OpResolver` inalterados.** A `eval` da CPU **permanece o caminho CANÔNICO**; o
   sequenciador consulta `registry.gpu_kernel(id)` e, quando é `None` (ou o replay canônico pede), **cai na
   `eval` da CPU**. O kernel é um caminho de performance **opcional**, nunca a definição do nó.

**Por que lateral e não no manifesto:** um campo novo em `NodeManifest` obrigaria `gpu_kernel: None` em
**92 literais** de manifesto (struct literal exige todos os campos) **e** um descongelamento do §6 (ADR).
O canal lateral não tem nem um nem outro — é append-only de verdade, e é o precedente do módulo.

## O invariante que protege o determinismo (o problema difícil, decidido cedo)

Float na GPU **não é bit-reproduzível cross-vendor/driver**. O **replay-hash**, o `transform_determinism`
(ECS) e o golden `cook_determinism` (Fase 0) são a lei. Então:

- **A CPU é o caminho CANÔNICO** — os goldens rodam sobre a `eval` da CPU, determinística (HR-5, libm
  pinado). A GPU **nunca** vira o oráculo.
- **A GPU é performance/preview**, reconciliada por **tolerância** nos gates de paridade (não bit-a-bit) — o
  padrão do [[project_painter_w4_spatial_gpu_bloom_sh]] (dev-dep compara GPU vs CPU dentro de ε).
- Mesma filosofia de "scrub não pode divergir do playback": onde o determinismo é lei, a CPU responde.

## Esboço do motor (Fase 1 — para dimensionar, não é o contrato)

- **`GpuStream`:** cada coluna vira um `wgpu::Buffer` (+ len/dtype), com pool/alocador. O sequenciador
  percorre a topologia → uma sequência de compute passes num **único submit**; ping-pong de buffer para nós
  que escrevem o que leem; barreiras entre dependências.
- **Estado (`pre`):** o feedback sequencial vira ping-pong de buffer **entre frames**.
- **Fronteira CPU↔GPU explícita:** nós sem kernel (ou o replay canônico) fazem readback e rodam na CPU;
  minimizar cruzamentos.
- **Renderer lê direto:** pular o marshalling `Stream → Vec<RenderInstance>` — o renderer consome as colunas
  GPU como instance buffers. Caminho híbrido para cenas CPU-only.

## Consequências

- **+** Milhões de instâncias animadas a 60fps, zero readback no hot path.
- **+** **Contrato congelado intacto (8/2/1)** — nenhum ADR de descongelamento, nenhum churn nos 92 nós. A
  Fase 2 porta cada tipo para WGSL incrementalmente (um `register_gpu_kernel` por nó), cada um com um gate de
  paridade (tolerância) contra a `eval` da CPU. Um nó sem kernel simplesmente roda na CPU.
- **−** Complexidade de um motor GPU novo — faseado, com a CPU sempre como oráculo + fallback.
- **−** `NodeRegistry` ganha um método (`register_gpu_kernel`/`gpu_kernel`) — foundational, mas fora do §6.

## Alternativas rejeitadas

- **Campo `gpu_kernel: Option<GpuKernel>` no `NodeManifest` (8→9):** descongela o §6 (ADR) **e** força
  `gpu_kernel: None` em 92 literais. O canal lateral entrega o mesmo sem tocar o contrato — rejeitado.
- **Bumpar `NodeOp` com `gpu_eval`:** o kernel é DADO (`&'static`), não comportamento; um método é
  desnecessário. `NodeOp` fica em 2.
- **GPU como caminho canônico:** quebra o replay-hash cross-vendor. A CPU é o oráculo.
- **Reescrever tudo de uma vez:** risco. Faseado por impacto (grid/deformers → forças/sim → espacial JFA).

# ADR-0122 — O motor de nós GPU-resident ESTENDE (append-only) o contrato congelado

- **Status:** **PROPOSTO** — aguarda o "aprova" do Enio para abrir a linha `line/gpu-nodes`.
  Nada aqui foi implementado; a linha `line/cook-parallel` (Fase 0) **não** toca o contrato.
- **Amende:** [ADR-0039](0039-nodegraph-contract-freeze-w2t4.md) (o congelamento `NodeOp=2` /
  `OpResolver=1` / `NodeManifest=8`).
- **Número tentativo:** linhas paralelas podem reivindicar 0122 antes; renumerar na integração se colidir.

## Contexto

A GPU **desenha** milhões de instâncias, mas quem **calcula** era a CPU num core só. A **Fase 0**
([plano](../../plans/2026-07-gpu-resident-node-pipeline.md), linha `line/cook-parallel`) já paralelizou
o cook na CPU com rayon (`attr::par_build`, marshalling, golden `cook_determinism`) — ganho imediato,
**sem tocar o contrato**. A **Fase 1** é o motor real: as colunas do `Stream` viram **buffers de GPU** e
cada nó vira um **compute pass WGSL**, com o renderer lendo os buffers **sem readback**. O `LoweringKind::Wgsl`
já existe no contrato (desenhado, nunca usado) — mas para um nó **fornecer um kernel** ele precisa carregar
o corpo WGSL + o layout de bindings + os uniforms de params, e isso **estende o `NodeManifest`**, que está
**CONGELADO** (§6 do CLAUDE.md, gate `architecture_contract_surface`).

## Decisão

**Estender o contrato de forma APPEND-ONLY**, para que os 92 nós existentes fiquem byte-idênticos.

1. **`NodeManifest` ganha um campo, apendado por último:** `gpu_kernel: Option<GpuKernel>` (contagem de
   campos **8 → 9**; o gate `architecture_contract_surface` sobe para 9 — **este ADR autoriza o bump**). Um
   nó que não fornece kernel deixa `None` → serialização/comportamento idênticos.
2. **`GpuKernel` (tipo novo, `'static`):** `{ wgsl: &'static str (corpo do compute), bindings: &'static [ColumnBinding]
   (quais colunas lê/escreve, e o modo — in/out/inout→ping-pong), params: &'static [&'static str] (uniforms) }`.
   Tudo `&'static` como o resto do manifesto (porta dinâmica é impossível — a mesma restrição que forçou os
   subgrafos a serem dobra-de-vista).
3. **`NodeOp` NÃO ganha método novo** (fica em 2). A `eval` da CPU **permanece o caminho CANÔNICO**; o kernel
   é um caminho **paralelo opcional** que o sequenciador escolhe quando toda a sub-árvore tem kernel.

## O invariante que protege o determinismo (o problema difícil, decidido cedo)

Float na GPU **não é bit-reproduzível cross-vendor/driver**. O **replay-hash** e o golden
`cook_determinism` (Fase 0) são a lei. Então:

- **A CPU é o caminho CANÔNICO** — o replay-hash, o `transform_determinism`, o `cook_determinism` rodam
  sobre a `eval` da CPU, determinística (HR-5, libm pinado). A GPU **nunca** vira o oráculo.
- **A GPU é performance/preview**, reconciliada por **tolerância** nos gates de paridade (não bit-a-bit) —
  o padrão do [[project_painter_w4_spatial_gpu_bloom_sh]] (dev-dep compara GPU vs CPU dentro de ε).
- Mesma filosofia de "scrub não pode divergir do playback": onde o determinismo é lei, a CPU responde.

## Esboço do motor (Fase 1, para dimensionar — não é o contrato)

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
- **+** Append-only → os 92 nós não mudam; a Fase 2 porta cada tipo para WGSL incrementalmente, cada um com
  um gate de paridade (tolerância) contra a `eval` da CPU. Um nó sem kernel simplesmente roda na CPU.
- **−** O gate `architecture_contract_surface` sobe 8→9 (autorizado aqui). Mexer no contrato é **Coord/Enio-level**.
- **−** Complexidade de um motor GPU novo — faseado, com a CPU sempre como oráculo + fallback.

## Alternativas rejeitadas

- **Bumpar `NodeOp` com um método `gpu_eval`:** desnecessário — o kernel é dado (`&'static`), não
  comportamento; cabe no `NodeManifest`. Manter `NodeOp=2` preserva a superfície mínima.
- **GPU como caminho canônico:** quebra o replay-hash cross-vendor. Rejeitado — a CPU é o oráculo.
- **Reescrever tudo de uma vez:** risco. Faseado por impacto (grid/deformers → forças/sim → espacial JFA).

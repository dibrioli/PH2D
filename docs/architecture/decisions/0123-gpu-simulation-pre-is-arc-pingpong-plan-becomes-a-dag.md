# ADR-0123 — A simulação na GPU: o `pre` é ping-pong de `Arc`, o plano vira **DAG**, e o scrub fica no device

- **Status:** PROPOSTA (fatia 3 do briefing da Fase 2 — *"é uma extensão do MOTOR, não um port de
  kernel. Desenhe antes de codar; pode virar o journey seguinte"*). Escrito ANTES do código.
- **Linha:** `line/gpu-nodes` (Modo L, ADR-0107 — foundational é editável pela linha).
- **Não amende** [ADR-0122](0122-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md): o kernel
  segue metadata lateral e o contrato de nós fica **8/2/1**. O que muda é o **motor**
  (`ph2d-gpu-cook`), não a superfície congelada.
- **Número tentativo:** 0123 (0122 é o último desta linha); renumerar na integração se colidir.

## Contexto

A Fase 2 fechou os nós **stateless** (10 kernels; 2M instâncias a ~4 ms/frame). O que falta para
"milhões de partículas **simuladas**" é o **laço de simulação**: `motion.integrate` / `motion.spring`
+ as 5 forças (`force.wind`/`drag`/`attractor`/`vortex`/`curl`).

**As forças já estariam prontas** — são single-input, `Pure`, mapas por-elemento que acumulam na
coluna transiente `accel`, exatamente o padrão que a Fase 2 portou 6 vezes. Mas elas **só rodam
dentro do laço**, e o laço é refutado. Portá-las sozinhas seria um kernel que **nunca dispara**
([[feedback_an_optimization_needs_a_gate_that_proves_it_fires]]). **A unidade de valor não é "mais
um nó" — é o laço inteiro.**

Duas regras do `eligible()` refutam o laço hoje:

```rust
if manifest.inputs.len() > 1 { return false; }              // integrate = rest + forces
if …input_edge(node, 0) is delayed { return false; }         // a aresta `pre`
```

E a topologia é um **ciclo**:

```text
rest ──fwd──> integrate.rest
integrate.out ──pre──> force₁ ─…─ forceₙ ──fwd──> integrate.forces
```

## Decisões

### D1 — O `pre` é ping-pong de `Arc`, e sai quase de graça

`motion.integrate` **já foi desenhado para isto**: o doc do módulo diz que o estado vive como
*"visible stream columns instead of hidden per-node maps (**replayable, GPU-lowerable ping-pong**)"*.
O estado É `vel` / `sim_d` / `sim_t` — colunas. E um `GpuStream` já é *coluna = `Arc<wgpu::Buffer>`*.

Então "a saída do tick anterior" = **segurar o `Arc`**. O `BufferPool` recicla por **refcount**, logo
um buffer segurado simplesmente não é reciclado. **Zero readback, zero cópia, zero barreira manual.**

`GpuCook` ganha `prev: BTreeMap<NodeId, GpuStream>` — o espelho exato do `Cook::prev_outputs`
(CPU), populado no fim do frame para os nós que são fonte de uma aresta `delayed` (a MESMA regra do
`advance_tick_scoped`).

### D2 — O plano vira um **DAG** (a mudança estrutural real)

Hoje `GpuPlan { boundary: Option<(NodeId, usize)>, stages: Vec<GpuStage> }` é uma **cadeia linear com
UMA fronteira**, e `cook()` costura os stages numa variável `stream` única. Um laço de sim precisa de:

- **walk dos N inputs** (não só o port 0) → `stages` em **ordem topológica**;
- **N fronteiras** (`boundaries: Vec<(NodeId, port)>`), uma por input não-elegível;
- o `cook()` threading por **mapa `NodeId → GpuStream`**, não uma variável linear.

Isto é o que o plano ([§2 do roadmap](../../plans/2026-07-gpu-resident-node-pipeline.md)) sempre
descreveu: *"o grafo é sequenciado por topologia num command encoder"*. A F1.1 entregou o caso
linear; esta fatia entrega o geral.

**A aresta `delayed` deixa de ser uma recusa e vira uma PARADA:** ao subir e encontrar um `pre`, o
plano **para ali** — aquele input não vem de um stage, vem do `prev` (D1). É a regra que fecha o
ciclo sem recursão infinita.

### D3 — O pareamento por `id` é um GATHER → fora do 1º corte

`integrate::pairing` casa o estado com os elementos por **`id`** (um `BTreeMap`) quando o stream tem
identidade (o emitter carimba `id`), e **posicionalmente** quando não tem (grid/cloner, `sn == n`).

- **Posicional** = `state[i]` ↔ `rest[i]` → trivial na GPU.
- **Por `id`** = um gather (hash/sort na GPU) → **1º corte NÃO cobre**; um stream com `id` recua pra CPU.

**Gap conhecido do contrato:** `applicable` só enxerga **params**, e isto é uma condição de
**coluna**. Ou o `eligible()` ganha um teste de forma-do-stream no plan-time, ou o kernel declara
"recuso a coluna X". É a decisão pequena que esta fatia tem de tomar; o **default é recuar**, nunca
responder errado.

### D4 — Determinismo: um nó sequencial **acumula** o erro (não é ε por frame)

Esta é a diferença dura entre a Fase 2 e esta fatia, e ela **muda o gate**:

- Um kernel **stateless** é `f(params, playhead)` — o ε é **limitado por frame**. É por isso que os
  10 gates da Fase 2 comparam trajetória nenhuma: cada frame é independente.
- Um **sim** é `x_{n+1} = f(x_n)` — o ε **realimenta**. Depois de N ticks a GPU e a CPU são
  **animações diferentes**, e isso **não é bug**: é float não-reproduzível dentro de um laço.

Portanto: **o gate de paridade de um nó sequencial afirma UM passo a partir de um estado semeado**,
nunca uma trajetória longa. Comparar 1000 ticks e afrouxar o ε até passar seria um oráculo que
modela o filtro, não a verdade ([[reference_topic_oracle_discipline]]).

A política do ADR-0122 **não muda e é o que autoriza isto**: a **CPU é o caminho canônico**
(replay-hash, `cook_determinism`); a GPU é **performance/preview**. Por-device a GPU segue
determinística — o re-cook byte-idêntico da F1.1 já prova.

### D5 — O scrub (M2.N2) fica **no device** — não se troca uma feature por escala

O `Cook::checkpoint()` clona `prev_outputs` (streams de CPU) e o `CheckpointRing` do pump guarda um
por tick: é o scrub bit-exato do M2.N2. Se o estado mora na GPU, o caminho ingênuo (readback por
tick) **mataria exatamente o que a fatia existe pra ganhar**.

**Decisão:** o ring de checkpoints do sim GPU é **de buffers de GPU** (`copy_buffer_to_buffer`, sem
readback), **esparso** (1 a cada K ticks, como o ring da CPU já faz por `should_record`). Restaurar =
copiar de volta + re-simular pra frente — o MESMO GGPO save/load/advance, sem sair do device.

Custo = VRAM: `estado × profundidade`. A 100k partículas (o regime real de um sistema de partículas)
são ~20 B/elemento × 64 = **128 MB**; a 2M seria 2,5 GB → o ring é **capeado por bytes**, não por
contagem de ticks (a lição do [ADR-0117](0117-audio-editor-memory-is-measured-not-declared.md): o cap
é em BYTES porque a contagem é um multiplicador, não um teto). Estourou o cap → o sim recua pra CPU,
que é onde o scrub bit-exato mora de qualquer jeito.

**Alternativa rejeitada:** "GPU = forward-only; scrub re-semeia". Trocaria o M2.N2 (um milestone
inteiro) por escala, sem precisar — D5 entrega os dois.

## Consequências

- **+** O laço de sim inteiro na GPU: `integrate` + `spring` + as 5 forças passam a rodar, e as
  forças deixam de ser inventário inerte.
- **+** O plano DAG é o alvo que o roadmap §2 sempre descreveu — destrava também **multi-input**
  (`look_at`, `combine`) e é pré-requisito das Fases 3–4.
- **+** Nenhum contrato descongelado (8/2/1 intacto); o kernel segue lateral (ADR-0122).
- **−** É um **rewrite do modelo de plano** (linear → DAG) + o threading do `cook()`. A F1.1/Fase 2
  continuam verdes por construção (a cadeia linear é um DAG de um caminho só).
- **−** O 1º corte não cobre pareamento por `id` (emitter/partículas com nascimento e morte) — o
  regime mais interessante fica pra fatia seguinte (o gather).
- **−** O ring de GPU consome VRAM; capeado por bytes, com recuo pra CPU.

## Fatias (nesta ordem)

1. **Motor DAG** — `plan()` walk de N inputs + `boundaries: Vec<…>` + stages topológicos + `cook()`
   por mapa `NodeId → GpuStream`. A `delayed` vira parada. **A F1.1/Fase 2 têm de ficar byte-iguais.**
2. **`prev` ping-pong** — `GpuCook.prev`, populado pela regra do `advance_tick_scoped`.
3. **Kernels** — `integrate` (posicional) + as 5 forças + `spring`.
4. **Gates** — paridade de **UM passo** (D4) · o laço **dispara** · a forma-do-stream recua (D3).
5. **Ring de GPU** (D5) — o scrub sem readback, capeado por bytes.

> Resumo: o estado já era coluna, então o `pre` é ping-pong de `Arc` e sai quase de graça; o preço
> real é o plano virar DAG; o determinismo de um sim acumula, então o gate mede UM passo; e o scrub
> não se vende por escala — o ring vai pro device.

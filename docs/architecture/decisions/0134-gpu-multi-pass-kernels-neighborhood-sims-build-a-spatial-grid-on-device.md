# ADR-0134 — Kernels GPU multi-passe: a simulação de VIZINHANÇA (boids/collide/SPH) constrói uma grade espacial no dispositivo

- **Status:** PROPOSTA. Sucede a linha `line/gpu-nodes` (Fases 0–2 + o emitter/id-gather do [ADR-0130](0130-gpu-emitter-the-id-gather-is-arithmetic-because-the-window-is-dense.md) integradas). É o Fase 3 do [plano mestre](../../plans/2026-07-gpu-resident-node-pipeline.md) — *"estruturas espaciais na GPU (os O(N²) que rayon não salva)"*.
- **NÃO toca o contrato congelado** ([ADR-0126](0126-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md)): `NodeOp=2`/`NodeManifest=8`/`OpResolver=1` intactos. O programa de passes é **metadado lateral** do `GpuKernel`, **append-only** — um kernel single-passe (todos os 32 de hoje) é o caso vazio e fica **byte-idêntico**.
- **Método:** o **censo medido** (item #1 do handoff de continuação, `shells/desktop/src/motion_gpu_coverage.rs`) + o **deep-dive da máquina do sequenciador**. Os dois fatos decisivos foram verificados no código ANTES de qualquer desenho ([[feedback_a_frontier_is_not_a_census]]): (a) qual documento real está na CPU e por quê; (b) exatamente o que o sequenciador NÃO sabe fazer.

---

## Contexto

O ADR-0126→0130 levou a engine a **32 kernels, GPU no default**, e a sim de partículas por FORÇAS (`emitter → wind → drag → integrate`) roda a **4,19 M em 3,6 ms** — milhões de elementos **INDEPENDENTES**, cada um lido/escrito uma vez por passe.

O que falta é a classe **interativa**: um elemento cujo passo depende dos **VIZINHOS** dele. O censo de cobertura confirmou (medido, não chute):

- O único documento **de artista** que existe — a neve de boot (`sim.zone`) — roda **quase toda na CPU** (2 stages despacham na GPU; 16 tipos de nó cozinham atrás da costura). As 6 demos `PH2D_GPU_COOK_DEMO` já são 100% GPU (andaimes de smoke, várias moldadas de propósito).
- **Não existe "próximo kernel de cobertura" que destrave um documento real.** A moagem incremental acabou. O que resta é a POTÊNCIA: os nós O(N²) que a §0.0 do CLAUDE.md aponta como o extraordinário.

O `motion.boids` é o representante canônico (Reynolds 1987): cada agente soma três forças (separação/alinhamento/coesão) sobre os vizinhos dentro de um `radius`. Hoje é um scan **all-pairs O(N²)** exato (`crates/ph2d-node-motion-boids/src/lib.rs`, fn `step`), capado pelo custo da CPU — o próprio doc do nó diz: *"A spatial hash (the standard scale trick) is a pure-perf follow-up; it would not change a single emitted position."* `motion.voronoi` (JFA) e `motion.soft_body`/`verlet_rope` (XPBD) são a mesma classe.

**O gap medido no sequenciador** (`ph2d-gpu-cook`, deep-dive):

| capacidade | estado | evidência |
|---|---|---|
| laço `pre`/estado entre ticks | **EXISTE** | `lib.rs` `prev` ping-pong + o ring de scrub |
| leitura de linha ARBITRÁRIA `read_P(j)` (vizinho) | **EXISTE** | acessores são parametrizados por índice; o gather já lê linha 0 e linhas computadas |
| barreira entre passes | **EXISTE (implícita)** | encoder único + buffers frescos imutáveis ⇒ wgpu auto-rastreia |
| um nó emitir **VÁRIOS passes** | **NOVO** | `plan.rs` `GpuStage` = 1 nó → 1 pass; `lib.rs`/`encode.rs` 1 dispatch por stage |
| **buffers auxiliares** (contagem por célula, prefix-sum, índices ordenados) | **NOVO** | todo buffer é coluna de stream de tamanho `count` |
| **`atomic<u32>`** no codegen | **NOVO** | codegen só emite `array<f32>`/`array<vecN>` |
| **tamanho de dispatch por-passe** (num_cells / scan de 1 workgroup) | **NOVO** | dispatch = o único `count` do stage |

Ou seja: os DOIS pré-requisitos que já assustavam (o estado que sobrevive ao tick, e ler o vizinho) **já existem**. O que falta é o **modelo de execução multi-passe**.

---

## Decisões

### D1 — O alvo é a classe de VIZINHANÇA a milhões; boids primeiro, collide/SPH herdam a grade

A meta não é "a neve na GPU" (otimizar uma cena que já roda na CPU a contagem modesta) — é a **capacidade que não existe em lugar nenhum**: interação a milhões. `motion.boids` é o primeiro payload porque é o mais icônico e o mais simples de verificar (3 regras locais + seek). `sim.collide` (colisão por vizinhança — **e ele está no prefixo CPU da neve**) e uma futura SPH reusam a MESMA grade. Voronoi (JFA) é outra estrutura e vem depois.

### D2 — Um `GpuKernel` pode declarar um PROGRAMA DE PASSES, append-only

Estende o metadado lateral (não o contrato congelado — ADR-0126): um kernel opcionalmente declara uma **sequência de passes**, **buffers auxiliares** (com regra de comprimento e tipo, incluindo `atomic<u32>`) e um **tamanho de dispatch por passe**. Um kernel sem programa de passes é o caso de hoje (byte-idêntico; gate de impressão digital). **Cada nó autora o próprio programa na sua drop-crate** — a filosofia leaf-level do ADR-0126; um serviço de "grade" fixo no sequenciador seria uma 2ª resposta para cada algoritmo novo (JFA, XPBD, radix), então **NÃO** é serviço, é contrato.

### D3 — A aceleração é um SPATIAL HASH (não grade limitada) — sem redução de bounds

`num_buckets = f(count)` (pow2 ~2·count), `cell = radius`, `bucket = hash(⌊p/cell⌋) % num_buckets`. Isso **evita a redução de min/max** das posições — que exigiria ou um passe de redução + **readback** (o anti-padrão medido-negativo da §0.0) ou um 2º sistema. O passo query varre as **9 células vizinhas** (3×3) e filtra por distância real ⇒ o CONJUNTO de vizinhos dentro do `radius` é **idêntico** ao all-pairs da CPU (colisões de hash só adicionam candidatos, que o teste de distância descarta). A construção é um **counting-sort exato** (limpa → conta → **scan** → espalha): sem overflow, sem vizinho perdido.

### D4 — A CPU continua CANÔNICA; tick 0 é bit-exato, os passos são ε

O replay-hash nunca roda na GPU (ADR-0126). O **seed do tick 0** é `hash3` **INTEIRO** (splitmix) ⇒ **bit-exato** entre CPU e GPU. Os passos seguintes diferem só na **ordem de soma** das forças de vizinhança (float não-associativo) ⇒ **paridade ε**, reconciliada por tolerância (padrão [[project_painter_w4_spatial_gpu_bloom_sh]]). HR-5: o kernel reusa as MESMAS aproximações polinomiais da CPU (aqui só há `sqrt` IEEE, que casa).

### D5 — O teto se MEDE (§0.0), não se herda do fallback

O slider do boids para em 500 e o `param_as_count` capa em `1<<24` — o 500 é **hint de UI** para o custo O(N²) da CPU. O caminho GPU não herda esse teto: o count real é **medido na GPU** e o número escrito com a tabela ao lado (o caso `feedback_the_ceiling_is_the_hardwares_never_the_fallbacks` — o emitter capado em 16 k com 4 M medidos). A CPU segue sendo o oráculo de paridade numa contagem que ela aguente; a GPU voa até onde a memória/banda deixar.

---

## Fases (cada uma com gate de paridade; a CPU nunca sai)

1. **1a — O scan (prefix-sum) reusável na GPU.** O sub-componente mais difícil, ISOLADO de propósito: um utilitário de exclusive-scan multi-nível, testado contra um oráculo CPU em wgpu real (RTX, `--ignored`). Sem ele o counting-sort não fecha.
2. **1b — O contrato multi-passe + o sequenciador.** `GpuKernel` ganha `passes`/`aux`; `encode`/`codegen` ganham buffers auxiliares, `atomic<u32>` e dispatch por-passe. Provado por um nó-oráculo **counting-sort** (ordenar por chave), GPU == CPU.
3. **2 — A grade espacial (spatial hash).** limpa→conta→scan→espalha ⇒ `(bucket_start, sorted_indices)`. Gate: a grade da GPU casa com uma grade construída na CPU (mesmos buckets).
4. **3 — O passo do boids.** As 3 regras de Reynolds lendo a grade (9 células) + o branch de seed do tick 0. Paridade ε vs o all-pairs da CPU. `motion.boids` vira GPU-claimable.
5. **4 — Milhões + cap medido + demo.** Mede boids na GPU, escreve o teto com a tabela, sobe o slider para o caminho GPU, e uma cena `PH2D_GPU_COOK_DEMO` de milhões de agentes coalescendo.
6. **5 (herança) — `sim.collide`** reusa a grade (rumo à neve e à família `sim.*`).

---

## Consequências

- **É um motor novo** (um 2º modelo de execução de kernel). O risco é gateado por fase; a CPU é o oráculo E o fallback (um kernel multi-passe que recuar por qualquer razão cai na CPU, exatamente como hoje).
- **O scan é o pedaço frágil** — por isso é a Fase 1a isolada, com oráculo trivial, antes de qualquer lógica de boids.
- **A superfície do `GpuKernel` cresce** (append-only). O `output_shape`/`eligible` do plano não muda: um kernel multi-passe ainda declara as colunas do stream que a ÚLTIMA passe escreve; os buffers auxiliares são invisíveis ao plano (não são colunas).
- **Número de ADR provisório:** escolhido nesta linha; o integrador reconcilia se colidir (como 0130→0131 na física — [[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

## Alternativas rejeitadas

- **Grade como SERVIÇO no sequenciador** (o sequenciador sabe construir grade para quem pedir). Mais simples, mas menos geral: JFA (voronoi) e XPBD (soft-body) são OUTROS algoritmos multi-passe; cada um precisaria do próprio serviço. O contrato multi-passe autora-no-nó serve os três. (ADR-0126: leaf-level, autorável na drop-crate.)
- **Bucket de capacidade fixa com overflow descartado** (sem scan). Evita o scan, mas PERDE vizinhos ⇒ quebra a paridade exata com o all-pairs da CPU. Rejeitado: a paridade é o oráculo.
- **Grade limitada por bounding-box** (redução de min/max). Exige um readback dos bounds (anti-padrão da §0.0) ou um passe de redução extra. O spatial hash não precisa de bounds.
- **Não fazer nada (ficar na cobertura incremental).** O censo provou que não há mais cobertura incremental que destrave um documento real. Seria parar a linha no platô.

# ADR-0136 — GPU: nós que MUDAM CONTAGEM — compaction ordem-preservante, contagem no HOST, boundary estático é híbrido legal

- **Status:** aceito (implementado nesta linha, `line/gpu-nodes`)
- **Data:** 2026-07-20
- **Contexto:** continuação de ADR-0126/0127/0130/0134/0135; fila §E do relatório da
  auditoria (`docs/HANDOFF_line_gpu_nodes_auditoria_RESULTADO_2026-07-20.md`), item 1.

## Contexto

A fundação do laço de simulação na GPU landou (ADR-0135): `sim.zone`/`sim.step`/
`sim.collide` cozinham no dispositivo e o plano recua quando não pode reclamar o laço
INTEIRO. O que falta para o documento de ARTISTA (a neve de boot,
`motion_demo_strobe::build_sim_zone`) é a classe que a linha adiou três vezes: os nós
cuja **contagem de saída não é a contagem de entrada** — `sim.spawn` (nascimento),
`sim.lifetime` (morte por idade), `motion.cull` (morte por predicado), `motion.combine`
(concatenação) — mais dois órfãos de kernel no mesmo prefixo (`value.attribute`, o `.t`
do `motion.color_ramp`) e **um sétimo que a fila não enumerava**: o template do spawn é
`motion.distribute_poisson`, cujo algoritmo (Bridson dart-throwing) é **sequencial por
natureza** — um "port" seria outro algoritmo, com outros pontos, e a paridade CPU↔GPU
deixaria de ser uma pergunta respondível.

O problema estrutural: o despacho é dimensionado no HOST (`CountLawCtx` é explícito:
uma lei de contagem pode perguntar a LARGURA dos inputs, nunca o conteúdo), mas a
contagem de `lifetime`/`cull` é função dos DADOS (quem sobreviveu), que vivem na GPU.

## Decisões

### 1. A compaction é ORDEM-PRESERVANTE (flags → scan exclusivo → scatter de rows → gather)

A CPU mantém os sobreviventes na ordem original **de propósito** (`sim.lifetime`:
*"reshuffling the set every tick would make every downstream index-based node
flicker"*). Um append atômico embaralharia a ordem por frame — flicker em alpha
sobreposto e paridade morta. O motor reusa o `Scan::exclusive` que o counting-sort da
grade de vizinhança já construiu (ADR-0134): *predicado escreve flag por elemento →
scan exclusivo das flags → scatter `rows[scan[i]] = i` dos sobreviventes → gather de
TODAS as colunas por `rows`*.

### 2. A contagem VOLTA AO HOST na costura da compaction (readback limitado de 8 bytes)

Depois do scatter de rows, o cook **fecha o submit corrente e lê de volta**
`scan[n-1] + flag[n-1]` (o total). Todo o resto do tick é encodado com contagens
exatas conhecidas no host — janelas, pareamento de broadcast, uniforms, ring, tap:
**nada da maquinaria existente muda**.

- Isto respeita a regra medida do módulo (`debug_read.rs`): *"nothing on a frame path
  may read back an UNBOUNDED amount"* — 8 bytes é o mais limitado possível; o custo é
  o SYNC (split de submit), não os bytes, e ele é **constante em N** (o teto de
  partículas do hardware não se move).
- **O caminho de upgrade fica nomeado, não construído:** contagens GPU-residentes +
  `dispatch_workgroups_indirect` removeriam o sync — ao preço de reescrever o guard de
  TODO kernel gerado e o pareamento de janelas para ler contagem de buffer. Só se
  paga quando a constante MORDER (medição no gate de escala da zona).

### 3. UM canal novo no `KernelResolver`: `stream_op` (padrão grid/state_select, append-only)

As quatro operações ESTRUTURAIS de stream viram side-metadata
(`ph2d_nodegraph::gpu::StreamOp`), ao lado do kernel de MAP por-elemento:

| variante | quem | o que a maquinaria faz |
|---|---|---|
| `Compact` | `sim.lifetime`, `motion.cull` | predicado WGSL (codegen com os MESMOS accessors/params dos kernels) → scan → scatter → readback → gather; o kernel normal do nó (se não-passthrough) roda DEPOIS, na stream compactada — o `life` do lifetime é um kernel comum |
| `SourceRows` | `sim.spawn` | kernel de rows (aritmética pura: `slot(first+j)`) que TAMBÉM escreve as colunas próprias (`id`); depois o mesmo gather genérico copia todas as colunas do template. Contagem = count law (host), **sem readback** |
| `Concat` | `motion.combine` | união de colunas com zero-fill por `copy_buffer_to_buffer` + `clear_buffer` — **zero shader**; contagem = soma (host) |
| `Project` | `value.attribute` | o nome da coluna é TEXT PARAM (dinâmico — inexprimível numa `ColumnBinding` estática): o cook resolve o nome contra o mapa da stream em runtime; scalar = cópia, length = kernel fixo, ausente = zero-fill |

O gather genérico (u32 rows → toda coluna, stride pela `element_stride` — a mesma
porta que uploader e binder usam) é UMA pipeline compartilhada por `Compact` e
`SourceRows`; nenhum codegen por coluna.

### 4. `sim.spawn`: a lei de contagem ganha `dt`, e o id ENVELOPA (C3 executado)

- `CountLawCtx` ganha `dt: f64` — **a mesma expressão** do `EvalCtx::dt` da CPU
  (`prev_playhead.map_or(0.0, |p| playhead - p)`); o `GpuCook` guarda `last_playhead`
  e o zera no `forget_state`, espelhando o seed (dt=0 ⇒ nada nasce no 1º tick — o
  comportamento documentado do nó).
- A janela nasce do padrão do emitter (ADR-0130): `born_in` em f64, `first` envelopado
  em `ID_WRAP` ANTES de virar u32.
- **C3 (auditoria) é executado aqui:** a CPU escrevia `id = k as f32` SEM wrap
  enquanto o contrato envelopa — passado 2²⁴ nascimentos, dois ids colapsariam num só
  em silêncio. A CPU agora escreve `(k % ID_WRAP) as f32`; o wrap é invisível a todo
  consumidor porque identidade só é lida como DIFERENÇA dentro de uma janela
  (`SourceWindow` docs), e agora os dois lados escrevem o MESMO número.

### 5. Um boundary ESTÁTICO não derruba o laço (o poisson fica na CPU, e isso é certo)

O retreat do plano (ADR-0135) recuava com **qualquer** boundary coexistindo com um
`pre`-source reclamado — o guarda contra "duas simulações de um estado". Mas um
boundary cujo chain transitivo é todo `Effect::Pure`, sem edge `delayed` e sem escopo
de tempo é uma **CONSTANTE**: o pump re-cozinhá-lo não é uma segunda simulação de coisa
nenhuma. O retreat agora só dispara para boundary TEMPORAL; a ponte
(`motion_bridge_gpu`) marcha os ticks do laço entregando as MESMAS streams estáticas a
cada tick. Sem isso, **todo** documento de artista com template estático (grid,
poisson, scatter — a forma comum) ficaria fora da GPU para sempre por causa do nó mais
barato do grafo.

### 6. `motion.color_ramp.t` + o bug de CPU que o port expôs

O `RefuseIfPresent` do `t` vira `ReadBroadcast` (a maquinaria de broadcast já existe e
a auditoria acabou de cercá-la com a recusa de comprimento misto). O port expôs um bug
CPU: `colorize` lê `t_field.get(i).unwrap_or(0.0)` — um campo de comprimento 1 (a
convenção de broadcast de TODO consumidor de valor; `motion.look_at::target_at` é o
cânone `0/1/n`) colorizava só o elemento 0 e pintava o resto com `t=0`. A CPU é
consertada para a convenção canônica; gate red-first.

## Consequências

- A neve de boot cozinha 100% no dispositivo (prefixo estático na CPU por decisão,
  não por buraco), com paridade ε por nó e e2e gateada contra a CPU canônica.
- **Custo do sync de compaction, MEDIDO** (`gpu_stream_ops::the_compaction_seam_cost_probe`,
  RTX, laço de zona a 65 536 elementos): **0,225 ms por seam** (0,025 ms/tick sem
  compaction → 0,251 ms/tick com uma) — constante em N, como o desenho prevê (8
  bytes; o custo é o split de submit, não os dados). A neve tem 2 seams
  (lifetime + cull) ≈ 0,45 ms/tick = 2,7% de um frame de 60 fps. Burst de replay
  paga 2N syncs; a reforma do ring (C1, fatia 2 da fila §E) é quem reduz N. O
  upgrade nomeado (contagens GPU-residentes + dispatch indireto) só se paga
  quando esta constante morder.
- `motion.trail` (o composto: compact do state + epílogo + concat com o head) fica
  para a fatia seguinte — ele compõe EXATAMENTE destas primitivas, e entrar aqui
  incharia a fatia sem mover a neve (o trail não está no grafo dela).
- `keeps_dense_window` continua default-false para toda a família — quem filtra,
  duplica ou reescreve `id` quebra a janela densa e o plano recua do id-gather
  aritmético, como desenhado (ADR-0130).

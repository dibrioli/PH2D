# HANDOFF (briefing de continuação) — `line/gpu-nodes` · pós-integração 2026-07-18

> **Você é o agente que continua esta linha em contexto fresco.** A linha foi
> **INTEGRADA à `main`** (2026-07-18) e o worktree já está sincronizado por cima
> dela. Este doc é o ponto de partida; os anteriores viraram histórico (§5).

---

## §0 — Inegociáveis (memorize antes de tocar em nada)

0. ⚠️ **O ALVO É O EXTRAORDINÁRIO** (CLAUDE.md **§0.0**,
   [[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]). Antes de
   escrever qualquer limite — cap, teto, `MAX_*`, faixa de slider — **MEÇA**, e
   escreva o número que a medição deu, com a tabela ao lado. **Nunca deixe o
   caminho de referência (CPU) definir o teto do dispositivo.** Esta linha já
   errou isso duas vezes; a segunda custou uma correção do Enio.
1. **Trabalhe SÓ em `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes`.
   SEMPRE prefixe todo comando com `cd <esse path> &&`.** A cwd escorrega pro
   repo primário — aconteceu **6×** em jornadas anteriores, e uma delas eu **li a
   `main` achando que era a linha**. Se um `git branch --show-current` disser
   `main`, você escorregou.
2. **NÃO integre, NÃO pushe, NÃO rode `ship.sh`.** Feche o trabalho, atualize
   este handoff, e PARE (CLAUDE.md §0.7).
3. **Contrato congelado 8/2/1 intocado** (`NodeManifest`/`NodeOp`/`OpResolver`,
   [ADR-0126](architecture/decisions/0126-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md)).
   Tudo que você mexe é **metadado lateral**: `GpuKernel`, `SourceWindowFn`,
   `KernelResolver`, o `output_shape` do plano, os kernels.
4. **O gate É o audit.** Verde-de-compilação vale ZERO. Kernel novo = paridade ε
   contra a CPU (canônica) **+ mutação** (mate o código, exija vermelho, restaure
   com `cp` — **NUNCA** `git checkout`). `git commit --no-verify`; crase em msg de
   commit = execução → `git commit -F <arquivo>`; um pipe mascara o exit code;
   `typos` sem argumento.
5. **Inner loop = `cargo check -p <crate>`.** Gates 1× no fechamento, em
   `--release` na RTX (os de GPU são `#[ignore]` ⇒ `-- --include-ignored`).
6. **LOC cap: SPLIT, nunca allowlist.** O de workspace (`crates/*/src`, 700) NÃO
   roda com `cargo test -p` (mora na `ph2d-editor-core`); o do shell (600) roda.

---

## §1 — Onde estamos (tudo abaixo está na `main`)

`emitter → [forças] → integrate/spring → tint → output` cozinha **100% na GPU** e
casa a CPU dentro do ε. Medido na RTX:

| janela | GPU ms/tick | CPU ms/tick |
|---:|---:|---:|
| 262.144 | 0,277 | 13,060 |
| 1.048.576 | 0,984 | 52,608 |
| **4.194.304** | **3,636** | 227,800 |

**O que existe** (ADR-0126 · 0127 · **0130 + emenda 1**):

- **Fase 1** buffers GPU-resident + runtime de lowering WGSL; **fully-GPU** e
  **híbrido** (1 fronteira).
- **Fase 2** 20 nós portados.
- **Fase 3** o laço de simulação (`pre` feedback, scrub por ring esparso, 6
  forças + integrate + spring).
- **ADR-0130** o **emitter**: janela densa provável (`dense_window`), recusa
  condicional (`ColumnAccess::GatherKey`), **gather aritmético**
  (`prev_row = id − prev_first`) com bounds-check por-elemento distinto do global.
- **Emenda 1** a **identidade é exata em qualquer rate**: `SourceWindowFn`
  devolve `{count, first, age_first}`, lei de contagem **única** em `f64`, id com
  **wrap em `ID_WRAP`**, `age` sem cancelamento, `MAX_ALIVE` = **4.194.304**
  (orçamento de MEMÓRIA).
- **Edit ao vivo** (D7): só `rate` re-numera ⇒ `reseed_from_next_tick` (O(1));
  `life`/`max` são live e **exatos** (encolher é bit-idêntico).
- **Limites soft/hard de param** (`ParamHardMax`).

**Rodar tudo verde hoje:**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && \
  cargo test -p ph2d-gpu-cook --release -- --include-ignored
```
⇒ **16** lib · **2** WGSL · **15** paridade · **17** sim · **5**+**12** plano ·
**4** invalidação. Mais: emitter **16** · panel-motion-params **14** ·
shell `--bins` **776** · nodegraph **84**.

**Smoke:** `PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=5 cargo run -p ph2d-host-desktop --release`
→ 1,2 MILHÃO de partículas coloridas por idade, ~1 ms/tick.

---

## §2 — O PRÓXIMO GARGALO, medido (leia antes de escolher trabalho)

Dois números que enquadram tudo:

| fato | número |
|---|---|
| nós com kernel GPU | **20 / 72** (28%) |
| o caminho GPU é… | **opt-in** (`PH2D_GPU_COOK=1`); o pump da CPU é o **default** |

⇒ **Os 4,19 M partículas em 3,6 ms existem e nenhum usuário os alcança.** Estão
atrás de uma env var, e só para grafos montados a partir dos 20 nós cobertos.
Esse é o gap que o §0.0 cobra agora — não é mais "a GPU é rápida?", é "**quem
consegue chegar nela?**".

Há **três alavancas**, e elas têm uma ordem:

### (B) N fronteiras no shell — **a recomendação**

Hoje: *"Several boundaries recuse: the motor plans a DAG with N seams (ADR-0127
D2), but the pump hands over ONE cooked node per tick"*
(`render_loop/motion_bridge_gpu.rs`). **O motor já planeja N costuras; o SHELL só
entrega uma.** Com 52 nós descobertos, qualquer grafo real tem várias ⇒ cai
**inteiro** na CPU.

**É o multiplicador**: sem isso, o kernel nº 21 só ajuda grafos feitos exatamente
do conjunto coberto. Com isso, **a parte coberta de qualquer grafo roda na GPU**.
E o trabalho é **shell-side** (o motor está pronto), então é a alavanca mais
barata por unidade de alcance.

⚠️ O problema real a resolver: o pump marcha **um** nó cozido por tick, e marchar
duas vezes avançaria o relógio dele duas vezes. Desenhe isso antes de codar.

### (C) Mais kernels (os 52 restantes)

Moagem linear, cada um independente e barato. **Só compõe depois de (B)** — vide
acima. Candidatos naturais pro domínio de partículas: `motion.noise`,
`motion.trail`, `motion.orbit`, `motion.wave`, `motion.stagger`. Multi-input
(`look_at`/`combine`) **o motor já suporta** — falta só o kernel de cada.

### (A) GPU por default

O objetivo final, e **bloqueado**: os readouts/probe do painel de grafo leem o
memo da CPU, que o caminho fully-GPU não alimenta (**Fase 4**). Ligar por default
hoje quebraria o painel. ⇒ **Fase 4 antes.**

**Ordem recomendada: (B) → (C) em lotes → Fase 4 → (A).**

Outros abertos (menores): reduções na GPU (destrava `twist`/`bend` — desenhe
antes) · os 2 bloqueios de motor do `t` do `color_ramp`/`value.*` (nome de coluna
dinâmico via text param; e os `value.*` descartam a base) · Fases 3–4 do plano
(JFA voronoi, spatial-hash boids, renderer consumindo colunas cruas).

---

## §3 — Gotchas desta linha (não re-descubra)

1. **A densidade é do emitter NU.** `sort`/`cull`/`combine`/`clone`/`mirror`/
   `trail` quebram a janela densa. A recusa condicional depende de
   `output_dense_window == Some(true)` — nunca reivindique o gather sem ela.
2. **`age` é re-derivado, nunca acumulado.** ⚠️ O **`sim.step`** (o OUTRO
   integrador) **acumula** `age`; os dois não se misturam num stream.
3. **A identidade tem WRAP.** Todo consumidor lê identidade como **diferença
   dentro de uma janela**. Se você escrever código que compara ids em valor
   absoluto, está errado.
4. **Casts de VALOR, nunca `bitcast`.** Ids são armazenados por valor
   (`f32(em_id)`); `u32(max(id, 0.0))` casa o `f.max(0.0) as u32` da CPU.
5. **Uma lei de contagem só.** `emit` e o `source_window` chamam a MESMA
   `window()`. Se sentir vontade de recomputar a janela em qualquer lugar: não.
6. **Defesa em camadas ⇒ gate POR camada.** O bounds-check global (`HAS_*`,
   "existe QUALQUER estado?") e o por-elemento (`gather_paired(i)`, "ESTE
   elemento tem linha?") são perguntas diferentes e cada uma tem gate próprio.

**Armadilhas de fixture que já custaram tempo aqui** — todas pegas por
pré-condição, nunca por asserção:

- `FIXED_DT` é **0,05**, não 1/60. Uma `life` curta demais dá **um tick** de vida
  e o gather nunca é exercitado.
- Rate alta com janela capada = **1 ms de história**: nada se moveu, e a paridade
  compara dois campos de zeros.
- Rate alta demais faz a janela **virar N× por tick**: nada **sobrevive**, então
  não há estado a carregar. Sobreviventes exigem `life > FIXED_DT`.
- Um gate de teto tem de **superá-lo em NASCIMENTOS**, não em pedido (`emitter_graph`
  fixa `rate 40`: pedir um bilhão e receber 401 **passa**).
- `grep` mente: um `Stream::new(...).with("P")` que você achou pode ser o
  **fixture de teste do próprio crate**, não o `eval`
  ([[feedback_a_negative_search_needs_a_positive_control]]).
- `str.replace(old, new, 1)` com `assert old in s` acerta a **primeira**
  ocorrência — que pode ser outra demo. Assert de presença ≠ de unicidade.

---

## §4 — Ao fechar a fatia (o protocolo)

1. Gates 1× (paridade `--ignored` na RTX + plano sem device + WGSL). Todas as
   mutações VERMELHAS → restauradas com `cp`.
2. `cargo fmt` · `cargo clippy --all-targets` · `typos` (sem arg) · **os 2** LOC
   caps · `cargo machete` se mexeu em deps.
3. Atualize ESTE handoff (hashes + números medidos).
4. Lição durável ⇒ escreva em `project-memory/` **e** indexe no `MEMORY.md`
   (só ADICIONE — remover linha é operação de integração).
5. **PARE.** O handoff de integração só se escreve por ordem do Enio.

---

## §5 — Histórico (não leia salvo arqueologia)

| doc | o que era |
|---|---|
| [`HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-18.md`](HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-18.md) | o briefing do integrador desta rodada — **integrado** |
| [`HANDOFF_line_gpu_nodes_emitter_ADR0130_2026-07-17.md`](HANDOFF_line_gpu_nodes_emitter_ADR0130_2026-07-17.md) | as fatias 1-5 do ADR-0130 + a emenda 1, passo a passo |
| [`HANDOFF_line_gpu_nodes_fase3_2026-07-16.md`](HANDOFF_line_gpu_nodes_fase3_2026-07-16.md) | o laço de simulação (§9 tem o Aberto que alimenta o §2 acima) |
| [`HANDOFF_line_gpu_nodes_fase2_2026-07-15.md`](HANDOFF_line_gpu_nodes_fase2_2026-07-15.md) · [`fase1`](HANDOFF_line_gpu_nodes_fase1_2026-07-15.md) | os nós portados · o motor |

# HANDOFF (briefing de continuação) — `line/gpu-nodes` · pós-integração 2026-07-18

> História verbatim: [arquivo](../../archive/motion-nodes-2026-10-12/handoffs/HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| Ligar o `t` do `motion.color_ramp` ao dispositivo | Recusa o nó inteiro (o bloqueio do `t` já documentado) | vivo §2 (B) |
| `motion.integrate` com `forces` plano+descoberto | O nó recusa (D3 shape): o `pre` vira `GpuSource::Prev`, não boundary | vivo §2 (B) |
| Limite «por segurança» sem medição | O §0.0 recusa o palpite; a medição que o justificaria é a que se faz | vivo §2 |
| Duas escritas da MESMA coluna num módulo WGSL | O naga recusou (`redefinition of out_v`): uma só | [arquivo](../../archive/motion-nodes-2026-10-12/handoffs/HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md) §4.5.1 |
| Parâmetro `target` num kernel | O naga rejeitou o módulo inteiro | [arquivo](../../archive/motion-nodes-2026-10-12/handoffs/HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md) §4.5.1 |
| Uma costura dentro de um laço `pre` | Faz o `plan` recusar a simulação inteira | [arquivo](../../archive/motion-nodes-2026-10-12/handoffs/HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md) §4.5.1 |

**Status:** FECHADO 2026-07-18 · no `main` em `1d8d5294c` (o commit que trouxe este arquivo).

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
   [ADR-0126](../../architecture/decisions/0126-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md)).
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
⇒ **16** lib · **2** WGSL · **15** paridade · **18** sim · **5**+**12** plano ·
**4** invalidação · **2** `boundary_arity` (novos, 2026-07-18). Mais: emitter
**16** · panel-motion-params **14** · shell `--bins` **776** · nodegraph **84**.

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

Há **três alavancas**, e elas têm uma ordem — **corrigida por medição em
2026-07-18 (`3428c1fa`); a ordem abaixo NÃO é mais a que este doc recomendava.**

> ### ⚠️ A recomendação anterior (B primeiro) foi MEDIDA e REPROVADA
>
> O §2 abaixo recomendava **(B) N fronteiras no shell** como *"o multiplicador"*,
> sobre a frase *"com 52 nós descobertos, qualquer grafo real tem várias
> [fronteiras]"*. **A frase confunde MUITOS NÓS DESCOBERTOS com MUITAS COSTURAS**,
> e as duas coisas não são a mesma.
>
> A região reivindicada cresce **PARA CIMA** a partir do sink, então as
> `boundaries` são a **FRONTEIRA** dela — e uma **CADEIA** de nós descobertos
> apresenta exatamente **um** nó de fronteira: o walk para no primeiro e nunca vê
> o resto. Para a fronteira **ramificar**, um nó **ESTAGIADO** precisa de 2+
> entradas cujas fontes estejam ambas descobertas. Hoje só **três** nós com kernel
> têm 2 portas, e as três declinam essa forma:
>
> | nó | 2ª porta | por que não ramifica |
> |---|---|---|
> | `motion.integrate` | `forces` | o `pre` vira `GpuSource::Prev`, não boundary; ligado plano+descoberto o nó **recusa** (D3 shape) |
> | `motion.spring` | `state` | idem (o `out --pre--> state` auto-wired) |
> | `motion.color_ramp` | `t` (VALUE) | ligar o `t` **recusa o nó inteiro** (o bloqueio do `t` já documentado) |
>
> **Medido em 5 formas: TODAS dão exatamente 1 boundary.** ⇒
> **`plan.boundaries.len() > 1` é INALCANÇÁVEL hoje**, e o arco
> `_ => GpuRoute::Cpu` do shell é **código morto**. (B) seria maquinaria para um
> estado que não pode ocorrer.
>
> **E o que DE FATO morde é bem pior que N costuras:** a costura só entrega o
> **SUFIXO** ao dispositivo, então **UM** nó descoberto no caminho do stream
> derruba a sim inteira de 4,19 M partículas na CPU — a região reivindicada
> encolhe para o `output` pass-through e despacha **ZERO** (medido:
> `dispatching 3 → 0`). Não é problema de **costura**; é de **COBERTURA**.
>
> ⇒ **A ordem inverte: (C) é o multiplicador, e (B) é CONSEQUÊNCIA de (C)** — o
> dia em que um kernel multi-input pousar. Isso não é uma nota que apodrece: o
> gate `boundary_arity.rs` é um **TRIPWIRE** que fica vermelho exatamente nesse
> dia, dizendo *"agora a fatia B é real — construa"*.
>
> **⚠️ ATUALIZAÇÃO (mesmo dia): o tripwire DISPAROU.** `motion.look_at` pousou; 2
> costuras são medidas e reais. A ordem volta a ser **(B) agora** — ver (B) logo
> abaixo. O resto desta caixa continua valendo como o registro de por que (B) não
> era trabalho antes.

### (B) N fronteiras no shell — ✅ **CONSTRUÍDA (2026-07-18)**

O tripwire disparou (`motion.look_at` pousou com kernel) e a fatia foi construída
no mesmo dia. **O pump é plural:** `CookTarget::Boundary(NodeId)` →
`Boundaries(&[NodeId])`, `boundary_stream: Option<Stream>` →
`boundary_streams: Vec<(NodeId, Stream)>`, e
`advance_or_scrub_to_nodes_scoped` recebe **o conjunto inteiro**.

**Uma marcha, N consumos.** A frase que manteve isto singular — *"marchar duas
vezes avançaria o relógio duas vezes"* — descrevia o **CHAMADOR**: a marcha e o
`pre` são por CHAMADA, e só o consume difere por target. O laço foi para dentro do
consume; a marcha continua uma.

**O que era LEITURA e agora é MEDIÇÃO.** O mapa afirmava, por leitura do `cook.rs`,
que dois `cook_scoped` no mesmo playhead batem no memo — e admitia que a medição
era impossível porque não existia grafo de 2 fronteiras. Existe agora, e o gate
**CONTA os evals** (nunca cronometra: uma barra de tempo passa numa máquina rápida
com o prefixo re-simulado). Prefixo compartilhado = **1 eval**; prefixos
**disjuntos** = **2** (nem 1, que seria chave de memo colapsada, nem 4).

As duas perguntas que o mapa marcou como não-verificadas foram **gateadas**:
**(b)** prefixos disjuntos (cada fronteira leva a resposta da PRÓPRIA cadeia) ·
**(c)** scrub para trás com N fronteiras (a fonte carimba o playhead no `P.x`, e o
gate exige o valor do tick rebobinado — sem isso ele só checaria que dois streams
voltaram, o que um scrub devolvendo o futuro também satisfaz).

**Deduplicação, como o mapa avisou:** `plan.boundaries` traz o mesmo nó duas vezes
quando ele alimenta duas portas estagiadas (`push` por porta); o pump entrega uma
vez só.

**Uma fronteira que falha não cala as outras** — as saudáveis entregam, o conjunto
sai curto, e o sequenciador **recusa** por `BoundaryMismatch` (a checagem mora num
lugar só; duplicá-la no shell seria uma 2ª opinião sobre o que é uma entrega
completa). Derrubar todas por causa de um nó ruim transformaria um erro de edição
num frame preto.

**Shell:** `GpuRoute::Hybrid(NodeId)` → `Hybrid` **sem carregar os nós** — o
`plan.boundaries` já os nomeia, e copiá-los para a rota seria uma 2ª lista para
manter em dia. A regra de `dispatching_stages >= 1` continua e é **independente**
do número de costuras.

**Gate e2e no caminho do PRODUTO:** `two_cpu_seams_hand_over_in_one_march_and_
match_the_cpu` — `grid → look_at` com dois `value.instance_field` (sem kernel) nas
portas de target, dirigindo o `MotionCookPump` REAL. Um gate que cozinhasse as duas
fronteiras à mão passaria com o pump ainda singular.

**7 mutações, 7 mortas** (pump singular · sem dedupe · rota recusando N · nunca
scrubar · e as 3 do broadcast).

⚠️ **Split de LOC:** `ph2d-eval-motion/src/lib.rs` foi a 730 ⇒ o **lowering** saiu
para `lower.rs` (a costura já estava lá: `lower.rs` responde *"como um stream fica
na tela"* e não sabe de tick/memo/`pre`; o `lib.rs` roda o RELÓGIO). 555 + 194.

**✅ MEDIDO (mesmo dia) — e a leitura ingênua estava errada.** Sonda
`two_seam_hybrid_timing`:

| elementos | CPU pura | híbrido+sync | híbrido CPU-side | prefixo |
|---:|---:|---:|---:|---:|
| 1.024 | 0,006 | 0,058 | 0,017 | 0,001 |
| 4.096 | 0,022 | 0,060 | **0,020** | 0,002 |
| 16.384 | 0,082 | 0,089 | 0,034 | 0,008 |
| 65.536 | 0,268 | 0,205 | 0,089 | 0,038 |
| 524.176 | 3,727 | 1,387 | 0,544 | 0,266 |
| 2.002.225 | **22,205** | 6,972 | **3,609** | 2,527 |

A coluna `+sync` espera o dispositivo todo frame, e lida assim o híbrido parece
**3× MAIS LENTO abaixo de 16k** — o que pediria um piso de tamanho na rota. **É a
coluna errada: o produto nunca espera.** O shell submete e segue, então quem
disputa o orçamento do frame é o `CPU-side`, e por essa medida o híbrido é mais
barato **a partir de ~4k** (empate ali, 1,5× em 8k, **5,9× em 2 M**) e custa no
máximo **0,012 ms** a mais abaixo disso — 0,07% de um frame de 60 fps.

⇒ **Nenhum limiar foi adicionado.** Um limite escrito para desviar de 0,012 ms é
o palpite "por segurança" que o §0.0 recusa; a medição que o justificaria é a
coluna que é artefato da sonda.

⇒ **O que ela ARGUMENTA é COBERTURA:** em 2 M, **70% do custo CPU-side do híbrido
é o PREFIXO** (2,527 de 3,609 ms) — imposto pago para cozinhar os nós sem kernel
que alimentam as portas. Cada kernel encurta esse prefixo; kernels suficientes e o
plano não tem costura para pagar.

### (A) GPU por default

O objetivo final, e **bloqueado**: os readouts/probe do painel de grafo leem o
memo da CPU, que o caminho fully-GPU não alimenta (**Fase 4**). Ligar por default
hoje quebraria o painel. ⇒ **Fase 4 antes.**

~~**Ordem recomendada: (B) → (C) em lotes → Fase 4 → (A).**~~
**Ordem MEDIDA (2026-07-18): (C) em lotes → Fase 4 → (A)** — e (B) quando o
tripwire `no_plan_can_leave_more_than_one_seam_today` abrir.

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

## §4.5 — O que esta sessão fez (2026-07-18, pós-integração)

A sessão começou **verificando a recomendação deste doc antes de construir sobre
ela** — que era o que o §2 mandava fazer (*"verifique antes de confiar"*) — e a
verificação a **derrubou**. O resto foi gasto no que a medição apontou:
**COBERTURA**.

**Cobertura: 20 → 32 kernels, e 6 deles de ⅖/½ para INTEIROS.** (O `spring` e o
`stagger` já contavam entre os 30 — tinham kernel, cobrindo X/Y; o lote final os
completa e acrescenta dois nós novos.)

| commit | o que |
|---|---|
| `3428c1fa` | **medição:** um plano não deixa mais de UMA costura hoje ⇒ (B) é inalcançável. 2 gates, mutação-testados, um deles **tripwire** |
| `258d80fb` | **doc:** o §2 corrigido — a ordem inverte, (C) é o multiplicador |
| `8362d806` | **medição:** a costura-ESPELHO (sim na GPU + readback + cauda na CPU) é **negativa** — 268 ms de readback contra 227 ms da CPU inteira. A alternativa arquitetural está morta, com números |
| `c2eee051` | shortlist ranqueada + **2 bloqueios de motor** (palavra reservada do WGSL · os `value.*` bloqueiam 6 nós de uma vez) |
| `49499df2` | **fix:** o probe não cozinha 1,2 M na CPU ao lado de uma GPU que já está desenhando — e o readout fica **honesto** (`—`, nunca o número velho) |
| `6b1ff654` | o painel volta a saber a massa dos fios num frame GPU-resident (`CookShape`: nomes e contagens, **nunca** buffers) |
| `75ece09e` | **motor:** param com nome de palavra reservada do WGSL (`wgsl_field`) ⇒ `motion.noise` (21º) |
| `55a846b4` | **motor:** emitir stream de outra ESPÉCIE (`rides_base` derivado do manifesto) ⇒ destrava a família `value.*` |
| `0dea633d` | `motion.luminance` + `value.map_range` (22-23º) — e a família `value.*` ganha prova **NUMÉRICA** (`read_column`) |
| `779d4f30` | `motion.orbit` (25º); e `sim.step` **sai da fila com motivo medido** (`dt ≡ 0` fora da `sim.zone`) |
| `d51055a4` | `motion.pin_constraint` (26º); um param pode se chamar como um uniform do motor |
| `02be152a` | `motion.stagger` (27º), com o easing inteiro (8 famílias × 3 direções) |
| `d6ac6725` | **broadcast tentado e REVERTIDO** — a corrente tem uma etapa antes dela; bug latente medido |
| `8a7ce80d` | **a LEI DE CONTAGEM** (`count_law`/`CountLawCtx`/`count.rs`) + o bug latente do `value.lfo` fechado e gateado + split `encode.rs` |
| `97c156a9` | **o BROADCAST** (`ColumnAccess::ReadBroadcast` + `bcast_one`) e **`motion.look_at`, o 28º kernel** — e o **tripwire DISPAROU**: 2 costuras são medidas, a fatia B está desbloqueada com o grafo vermelho-primeiro escrito |
| `c88f1e9a` | **A FATIA B CONSTRUÍDA** — o pump é plural (uma marcha, N entregas), o memo virou MEDIÇÃO (conta evals), (b) e (c) do mapa gateados, split `lower.rs` |
| `a2226787` | **a fatia B MEDIDA** (o híbrido paga a partir de ~4k; sem limiar, com a tabela) + **`value.instance_field` na GPU, o 29º kernel** — o nó que a própria medição apontou |
| `571740f1` | **`motion.drive` na GPU, o 30º** — o domínio de VALOR agora chega à tela sem CPU nenhuma (X/Y; Rot/Size/Opacity recuam por `applicable`, e o motivo é ESTRUTURAL) |
| `48aee001` | **VARIANTES POR-PARAM no motor** (`GpuKernel::variant_by_param`) — e com ela `drive`/`oscillator`/`noise`/`wiggle` passam a cobrir **TODOS** os canais |
| (este) | **a família de canais FECHA** (`spring`/`stagger` saem de X/Y para inteiros) + **`value.math` e `value.switch`** (31º e 32º) com a 3ª lei de contagem — e `applicable` fica sem sujeito vivo (kernel SINTÉTICO) |

**Estado:** tudo verde — **53 gates de GPU** (34 paridade + 19 sim) na RTX, shell
inteiro (14 binários, zero falhas), `fmt`/`clippy`/`machete`/`typos`/os **2** LOC
caps limpos. **Nada integrado, nada pushado** (§0.2).

## §4.9 — SMOKE APROVADO e a linha FECHADA (2026-07-19)

O Enio smokou a cena `PH2D_GPU_COOK_DEMO=6` com a GPU no default e **aprovou**.
A linha está fechada; o handoff de integração é
[`HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-19.md`](HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-19.md).

Estado no fechamento: **28 commits**, a `main` **não andou** desde o fork (a
integração é `--ff-only` limpa), **7.667 testes do workspace verdes**, 55 gates de
GPU na RTX + 2 de seam no shell, contrato congelado **intacto** (`cook.rs` e
`node.rs` com zero diff), `fmt`/`clippy`/`machete`/`typos`/os 2 tetos de LOC
limpos. **Nada integrado, nada pushado.**

## §5 — Histórico (não leia salvo arqueologia)

| doc | o que era |
|---|---|
| [`HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-18.md`](HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-18.md) | o briefing do integrador desta rodada — **integrado** |
| [`HANDOFF_line_gpu_nodes_emitter_ADR0130_2026-07-17.md`](HANDOFF_line_gpu_nodes_emitter_ADR0130_2026-07-17.md) | as fatias 1-5 do ADR-0130 + a emenda 1, passo a passo |
| [`HANDOFF_line_gpu_nodes_fase3_2026-07-16.md`](HANDOFF_line_gpu_nodes_fase3_2026-07-16.md) | o laço de simulação (§9 tem o Aberto que alimenta o §2 acima) |
| [`HANDOFF_line_gpu_nodes_fase2_2026-07-15.md`](HANDOFF_line_gpu_nodes_fase2_2026-07-15.md) · [`fase1`](HANDOFF_line_gpu_nodes_fase1_2026-07-15.md) | os nós portados · o motor |

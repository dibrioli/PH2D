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

### (C) Mais kernels (os 52 restantes) — **O MULTIPLICADOR (corrigido)**

Moagem linear, cada um independente e barato. ~~**Só compõe depois de (B)**~~ —
**o contrário: (B) só existe depois que (C) trouxer um kernel multi-input.** E o
payoff de (C) é **não-linear**, não incremental: pela medição acima, um único nó
descoberto no caminho do stream **forfeita a sim inteira**, então cada kernel
novo não adiciona uma fatia de velocidade — ele pode **destravar o caminho GPU
por completo** para a classe de grafos que só esbarrava naquele nó.
Candidatos naturais pro domínio de partículas: `motion.noise`,
`motion.trail`, `motion.orbit`, `motion.wave`, `motion.stagger`. Multi-input
(`look_at`/`combine`) **o motor já suporta** — falta só o kernel de cada.

#### Shortlist RANQUEADA (levantada 2026-07-18 — comece por aqui)

Classificação por *valor* (aparece a jusante numa cadeia real de partículas) ÷
*custo* (classe de viabilidade). O gabarito de tamanho: `motion.rotate`
(`crates/ph2d-node-motion-rotate/src/lib.rs:62`) = **24 linhas**; `motion.tint`
≈ 70; `motion.spring` ≈ 130 (o teto, com `gather_paired`).

| # | nó | classe | por quê |
|---|---|---|---|
| 1 | `motion.noise` | PER_ELEMENT | o *"faça parecer vivo"* de toda cadeia; `force.curl` **já traz fBm em WGSL** (`ph2d-node-force-curl/src/lib.rs:131-176`) — mas ⚠️ ver o bloqueio do `type` abaixo, e é noise de **GRADIENTE**, não o de VALOR do curl |
| 2 | `sim.step` | PER_ELEMENT | **o** nó a jusante de toda `sim.zone`; todo binding que precisa (`accel` Consume, `sim_t`, `inv_mass`, `age`) **já existe** no kernel do `integrate` — é transcrição, não desenho |
| 3 | `sim.collide` | PER_ELEMENT | logo depois do `sim.step`; o mundo vem **todo de params**, então lê só `P`/`vel` do próprio `i` — corpo do tamanho de uma força, zero superfície nova de motor |
| 4 | `motion.look_at` | PER_ELEMENT (3 in) | orientação a jusante (setas/peixes/pétalas); `atan2_approx` já é polinômio Rajan **transcendental-free** → porta literal, paridade fácil |
| 5 | `motion.orbit` | PER_ELEMENT | o corpo mais barato que resta (forma do `rotate` + `cos_sin_cycles`) |
| 6 | `motion.pin_constraint` | PER_ELEMENT | corpo trivial, mas **estratégico**: fica a MONTANTE da sim e `inv_mass` já é coluna ligada no `integrate`/`spring` ⇒ tira uma costura do meio de uma cadeia senão fully-GPU |

**Excluídos de propósito:** `motion.trail` (o handoff o nomeia, mas é
`CHANGES_COUNT` **e** feedback — dois eixos não-suportados de uma vez) ·
`motion.wave` (vizinhança estruturada serviria bem à GPU, mas é nó `pre` com
stencil Laplaciano) · `motion.twist`/`bend`/`spherize`/`four_point_warp`/
`spline_wrap` (**NEEDS_REDUCTION** — todos fazem um `fold` de max/centroide/bbox
antes do passe por-elemento; é a primitiva de redução já escopada como item
separado) · `motion.collide`/`boids` (all-pairs).

#### ✅ Feito nesta jornada (cobertura 20 → 26)

`motion.noise` · `value.lfo` · `motion.luminance` · `value.map_range` ·
`motion.orbit` · `motion.pin_constraint`. Cada um com gate de paridade
mutação-testado. **Três bloqueios de MOTOR removidos, e nenhum era sobre kernel:**

1. **Palavra reservada do WGSL** (`motion.noise` tem param `type`) — o codegen
   emitia o nome cru e o naga recusava.
2. **Nome de uniform do motor** (`motion.pin_constraint` tem param `count`) — o
   `plan::eligible` recusava o kernel explicitamente.
   → os dois pela mesma porta: **`codegen::field_name::wgsl_field`** (sufixo `_`).
3. **O modelo de stream só sabia "base + escritas"** — os `value.*` emitem um
   stream de outra ESPÉCIE. Derivado do manifesto (tipo da porta 0 in vs out),
   sem campo novo no `GpuKernel`, no-op para todos os kernels já existentes.

**Saíram da fila com motivo medido** (não são "kernel que falta"):
- **`sim.step` / `sim.collide`**: não têm `pre` próprio — o relógio vem da
  `sim.zone`. Fora do zone `dt ≡ 0`, então o kernel **nunca rodaria**. Voltam se a
  `sim.zone` (um escopo/laço, não um map) for portada.
- **`motion.look_at` / `value.math` / `value.switch` / `motion.drive`**: todos
  fazem **BROADCAST** — uma stream de VALOR de comprimento 1 vale para todo
  elemento. O dispatch é dimensionado pela porta 0, e ler `port_v[i]` num buffer de
  comprimento 1 é fora do intervalo: o naga torna seguro, mas **diverge** do
  broadcast da CPU. O motor não tem **comprimento por-porta** (o `gather_prev_n` é
  específico do gather).

#### ✅ FECHADO — a LEI DE CONTAGEM (o motor, e o bug latente que a exigiu)

**Landou.** `GpuKernel.source_window` (gerador, função só dos params) virou
**`GpuKernel.count_law`** — UMA porta para *"quantos elementos este nó emite?"*,
com `None` = **"tão largo quanto a porta 0"** (o default, zero boilerplate, certo
para os ~25 transformadores). O `Some` agora recebe um **`CountLawCtx`** com os
três — e só os três — fatos de que uma contagem já dependeu neste grafo, cada um
com o nó que o exigiu: `param` (`motion.grid`: rows × cols) · `playhead`
(`motion.emitter`: a janela viva `n(t)`, ADR-0130) · **`inputs`** (os `count` de
cada porta, em ordem — o que faltava).

⚠️ **`inputs` são os COMPRIMENTOS, nunca os streams.** Uma lei pode perguntar
quão LARGA é a entrada, jamais o que há dentro dela — isso seria um readback, e o
readback é medido-**negativo** (§2 A).

Motor: **`ph2d-gpu-cook/src/count.rs`** (módulo novo, porta única `stage_window`).
A contagem e a janela de identidade viraram **UMA resposta** (`SourceWindow`
carregado do walk até o encode): perguntar duas vezes deixaria o stage despachar
`n` elementos e contar ao kernel uma janela de outro tamanho, e **nenhum gate
notaria — os dois números são plausíveis separadamente**.

E a pergunta *"este módulo declara os uniforms de janela?"* virou
**`codegen::declares_window(port_names)`**, perguntada pelo codegen que os DECLARA
e pelo sequenciador que os EMPACOTA. Antes eram duas expressões (`source_window.
is_some()` nos dois lados); com a lei generalizada elas deixariam de coincidir —
um transformador com lei declarada não tem janela de spawn. **Um gerador (sem
portas de entrada) é o único nó que tem janela**: ele CUNHA seus elementos, então
identidade e idade são fatos que só o host sabe; um transformador HERDA os dele.

**O bug latente que motivou tudo — CONSERTADO e GATEADO.** `value.lfo`
desconectado: CPU `count = 1`, GPU `count = 0`. O `eval`
faz `n = ctx.input(0).count().max(1)` (desligado ⇒ **um** valor global), e o motor
dimensiona pela porta 0 — entrada vazia ⇒ `count = 0` ⇒ o stage é **pulado**
(`if count == 0 { … continue }`) e o stream sai vazio. **Medido**, não deduzido.

Era **inalcançável** (nenhum consumidor de VALUE tem kernel: o `t` do
`color_ramp` recusa), e foi por isso que o gate do `value.lfo` não pegou — ele
liga o lfo a um grid, então **nunca visita o caso desligado**. O nó agora declara
`count_law: Some(lfo_count)` = `max(port0, 1)`, a MESMA expressão do `eval`.

**Gate novo: `an_unconnected_value_node_is_one_global_value_not_zero_of_them`**
(irmão do que existia — o fixture NÃO conecta nada). Contagem **e** número, via
`read_column`: forma sozinha ficaria verde com o valor errado. E os params foram
escolhidos para a resposta ficar **longe de zero** (medido: −1,278), porque um
stage que nunca rodou deixa buffer zerado e um fixture cujo certo é `0.0` não
distingue os dois. Paridade `|dv| = 4,8e-7`.

**3 mutações, 3 mortas:** tirar o `.max(1)` ⇒ RED só neste gate (era o bug) ·
ignorar toda lei declarada ⇒ **19 de 22** gates RED (a lei sustenta quase todo
fixture, que passa por um `motion.grid`) · `declares_window → false` ⇒ RED
exatamente no emitter, *"invalid field accessor `window_first`"*, provando que a
janela é carga viva. ⚠️ `declares_window → true` **sobrevive por projeto**: as
duas metades agora perguntam à MESMA função, então o modo de falha que ela existe
para impedir (elas discordarem) ficou **irrepresentável** — é o padrão
`feedback_layered_defenses_need_per_layer_gates` visto do outro lado.

**Split de LOC junto:** `lib.rs` chegou a **700/700** exatos com a lei. O walk
(`cook` — quais stages, em que ordem, com que largura) ficou; **tudo que acontece
DENTRO de um stage saiu para `encode.rs`** (`encode_kernel_stage`: pipeline,
uniform, bind group, dispatch — 189 LOC). São perguntas diferentes e só a do walk
é sobre o GRAFO. **535/700** agora — a folga que o broadcast vai gastar.

#### ✅ FECHADO — o BROADCAST (e `motion.look_at`, o 28º kernel)

**Landou.** `ColumnAccess::ReadBroadcast`: uma porta de comprimento 1 **difunde**
(elemento `i` lê a linha 0), que é o `1 => vals[0]` do `target_at` da CPU,
declarado. Absent segue lendo a identidade (o arm `0 =>`); só o caso de
comprimento 1 é novo. `column_present` ganhou a 3ª regra de comprimento — sem ela
uma porta de comprimento 1 era julgada **AUSENTE** e lia a identidade, e a
diferença é um bando inteiro virado para a origem em vez do ponto que o artista
animou.

**Um bitmask, não N campos** (`bcast_one`, bit `p` = "a porta `p` tem exatamente
um elemento"). A alternativa — um `n_<nome da porta>` por porta — daria ao motor
uma fatia **dinâmica** do namespace do struct, feita de nomes autorados para o
ARTISTA, e alargaria o uniform a cada porta nova. Um bit responde a única
pergunta que o leitor tem, para qualquer aridade, em 4 bytes, sob um nome que o
`wgsl_field` guarda como os outros campos do motor.

`UNIFORM_BYTES` 64 → **128**: 64 comportava 14 params e mais nada, então um nó com
muitos params **e** um campo condicional escreveria um param por cima do campo
seguinte e leria como lixo plausível. É um slot, não alocação por elemento.

**🔴 BUG REAL que o gate pegou (não teoria):** `plan_bindings` **ENUMERAVA** os
acessos que não escrevem (`Read`, `Consume`, `RefuseIfPresent`, `GatherKey`) e
mandava todo o resto para `_ => WriteBuffer` — então a variante nova, read-only,
virou **escritora** em silêncio. Com duas portas de broadcast as duas reivindicaram
escrever a MESMA coluna e o naga recusou (*"redefinition of `out_v`"*). Com UMA só
porta teria sido mudo. Havia duas portas para *"isto escreve?"*
(`ColumnAccess::writes` e essa lista), e elas discordaram assim que a lista ficou
velha — [[feedback_a_condition_that_enumerates_its_readers_rots]]. Agora o
`plan_bindings` **pergunta** ao `writes()`.

**Gate: `look_at_broadcasts_a_single_target_and_reads_a_field_per_element`** —
roda os DOIS comprimentos a partir do mesmo grafo, com um fio movido. Os dois
comprimentos saem do próprio `value.lfo` (desligado = 1 pela lei de contagem,
ligado ao grid = N), então o gate falha se o broadcast **ou** a lei regredir.
Medido: broadcast `|drot| 1,5e-5°` · per-element `7,6e-5°`, 144 elementos.
3 mutações, 3 mortas (leitor ignora o bitmask ⇒ 327° de erro · comprimento-1
julgado ausente ⇒ o mesmo 327° · re-enumerar os não-escritores ⇒ o naga de volta).

⚠️ **Todo leitor é qualificado por porta num nó multi-input** — `read_in_P`, não
`read_P`. A regra é por NÓ, não por coluna, e custou uma rodada.

Ainda destrava, com a plumbing já pronta: `value.math`, `value.switch`,
`motion.drive`.

⚠️ **`value.math` vai precisar da 3ª lei** (`max(a.len, b.len)`), e ela **só entra
junto com o kernel que a consome** — motor sem consumidor foi exatamente o que se
reverteu aqui.

⚠️ **`motion.look_at` é MULTI-INPUT**: quando ele pousar, um grafo com 2 entradas
descobertas passa a deixar **2 fronteiras**, e o tripwire
`no_plan_can_leave_more_than_one_seam_today` fica vermelho de propósito — aí a
fatia B (§2 B) vira trabalho real e obrigatório. Acrescente o fixture ao tripwire
no MESMO commit do kernel, senão o gate fica incompleto em silêncio.

#### ⚠️ Dois bloqueios de motor que o levantamento expôs

1. **Um param com nome de palavra RESERVADA do WGSL não compila** — e o nº 1 da
   lista tem um. `codegen.rs:210` emite o nome do param **verbatim** no struct
   (`src.push_str(&format!("    {p}: f32,\n"))`), e `motion.noise` tem o param
   **`type`**. Medido com o naga: `type` → *"name `type` is a reserved keyword"*;
   `type_` → OK. **Nenhum dos 20 kernels de hoje usa palavra reservada**, então um
   sanitizador (reservada → sufixo `_`) é **no-op para todos eles** — é a correção
   mínima e geral. A alternativa barata é gatear por `applicable` só o `Fbm`
   (padrão) e deixar Turbulence/Ridged na CPU, no precedente do `motion.oscillator`
   — mas isso entrega ⅓ do nó.
2. **Os `value.*` descartam a base** (já conhecido, e agora dimensionado): esse
   ÚNICO gap bloqueia **6 nós PER_ELEMENT de uma vez** — `value.lfo`,
   `value.instance_field`, `value.map_range`, `value.math`, `value.switch` e
   `motion.luminance` (todos emitem um `Stream::new(n)` fresco em vez de estender a
   base). É provavelmente o **melhor retorno por unidade de trabalho da lista
   inteira** — um conserto de motor que destrava 6 kernels triviais. Considere-o
   ANTES de moer kernel a kernel. (`value.attribute` e `motion.expression` ficam de
   fora mesmo assim: a coluna/fórmula deles é **text param**, e `ColumnBinding.column`
   é `&'static str`.)

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

### §4.5.1 — O lote de fechamento da cobertura (este commit)

**`motion.spring` e `motion.stagger` fecham a família de canais.** Os dois eram
os últimos presos no limite que o `variant_by_param` removeu, e o do spring era o
caro: dentro de um laço `pre` **uma** costura faz o `plan` recusar a **simulação
inteira**, então um spring em Rotation arrastava todas as forças do grafo de volta
para a CPU com ele. Os kernels saíram para `kernel.rs` em cada crate (LOC), e o
`wgsl_lib` é **uma constante lida três vezes** em vez de três cópias — a tabela de
easing do stagger tem 90 linhas e precisaria concordar consigo mesma em três
lugares que nenhum gate compara entre si.

⚠️ **`target` e `out` são palavras RESERVADAS do WGSL.** O `sp_solve` nasceu com
um parâmetro `target` e o naga rejeitou o módulo inteiro. Custou um minuto porque
o `generated_wgsl_validates` parseia todo kernel registrado em `cargo test`, sem
device — sem ele seria uma falha de runtime confusa numa máquina só.

**`value.math` e `value.switch` trazem a 3ª lei de contagem**, e ela é **uma**, não
duas: *"tão largo quanto a entrada mais larga"* (`max` sobre TODAS as portas)
serve os dois. O `switch` inclui a porta `select` no `max` de propósito — um
seletor animado de comprimento N sobre fontes de comprimento 1 é exatamente o mux
por-elemento que o nó anuncia, e a saída tem de ser tão longa quanto a seleção.
A lei pousou **junto do kernel que a consome** (motor sem consumidor já foi
revertido uma vez nesta linha).

**Fixtures que nasceram cegas e o que as abriu.** Quatro gates novos só passaram a
medir alguma coisa depois de uma mutação sobreviver ou de uma medição:

- **`sg_round`/`vm_round`/`vs_round` (meio-par vs meio-ímpar) só é observável em
  `.5`.** A varredura de `ease_curve`/`op`/`select` era toda de inteiros, onde as
  duas convenções concordam — trocar `sg_round` pelo `round` do WGSL
  **SOBREVIVEU**. Com `ease_curve = 6.5` uma via escolhe Bounce e a outra Back;
  com `op = 0.5`, Subtract e Add; com `select = 0.5`, `in1` e `in0`. São entradas
  DIFERENTES, não números próximos.
- **A identidade do canal Size** (`[1,1]`, nunca `[0,0]`) é lida **só quando a
  coluna está ausente**, e o oscilador do fixture do spring a materializa ⇒ um
  kernel declarando zero seria byte-perfeito contra a CPU. Gate irmão:
  `a_spring_on_size_reads_unit_scale_from_an_absent_column`, atrás de uma grade
  pelada. (No stagger o `deformer_chain` já não emite `size`, então lá a
  identidade sempre foi exercida.)
- **O ε de `size` no arquivo de sim era 1e-5 e não era um orçamento.** Medido:
  **todos** os 25 gates que não dirigem size leem `max |dsize| = 0e0` — é uma
  checagem de identidade-bit sobre a constante `DEFAULT_SIZE`. O spring em Size
  mede `3,58e-5`, contra os `2,17e-4` que o **mesmo solver** produz em posição sob
  o `2e-3` que o arquivo já declara. Então o valor apertado FICA onde é merecido
  (`EPS_SIZE_UNDRIVEN`) e o campo dirigido recebe `EPS_POS` — em vez de afrouxar
  um número para todo mundo.
- **A varredura de canais foi RED pelo motivo errado.** `motion.spring` não estava
  registrado no `registry()` daquele arquivo, então o nó era irresolvível e
  *qualquer* plano costuraria ali — a mensagem acusava o canal, não a omissão. A
  varredura agora **valida** o grafo e o erro nomeia a crate faltando.

**`applicable` ficou sem sujeito vivo, e o gate agora tem um SINTÉTICO.** Com
spring e stagger cobertos, `applicable: Some` não existe em nenhuma crate do
repo. O `an_uncovered_param_space_puts_the_boundary_at_that_node` já tinha sido
rebaseado do oscilador para o spring pelo mesmo motivo — duas vezes o fixture se
dissolveu porque o trabalho de cobertura DEU CERTO. Um terceiro nó emprestado
compraria o mesmo tanto de tempo, então o sujeito é o `HalfCovered` de
`plan_analysis.rs`: `mode >= 0.5` é recusado por construção, o que não é um item
de backlog que alguém possa fechar. Ganhou também o **irmão de PRESENÇA** que
nunca teve (sem ele a asserção vale igual para um `plan` que recusa tudo). No
arquivo de sim o controle equivalente é o `motion.sort` — permutação global
contra um contrato por-elemento, incobrível por ESTRUTURA.

**Mutações: 12, todas matam** — 11 nos kernels (variante de canal esquecida ·
identidade Size · Size escrevendo só X · as 3 convenções de arredondamento · as 2
leis de contagem · a guarda do divisor · o clamp do switch) e 1 no motor
(`plan.rs` nunca consultando `applicable`, que só morre por causa do kernel
sintético). A M5 **sobreviveu na primeira rodada** e é a que está descrita acima.

**Onde o próximo agente começa.** As três alavancas do §2 estão em estados
diferentes agora:

1. **(C) COBERTURA — o multiplicador, e o caminho mais curto.**
   A shortlist do `c2eee051` **acabou**: `value.math`, `value.switch`,
   `motion.spring` e `motion.stagger` pousaram, e com eles a família de canais
   fecha. **Escolher o próximo nó agora quer uma medição nova**, não esta lista —
   o §2 manda perguntar quais nós de fato aparecem no prefixo CPU dos documentos
   que existem, e a resposta de ontem foi `value.instance_field`/`motion.drive`
   justamente porque foi medida.

   ⚠️ **Lacuna conhecida e NÃO fechada:** a mistura de comprimentos que não é
   `1→N` (ex.: um campo de 3 contra um de 5). A CPU faz `debug_assert` e depois
   degrada lendo elemento-a-elemento com `0.0` além do fim; a GPU chama a porta
   ausente e lê a identidade `0.0` em TODO índice. As duas degradações são
   diferentes nos índices `[0, min_len)`. É propriedade do `ReadBroadcast`
   (herdada de `drive`/`look_at`, não introduzida aqui) e não há hoje mecanismo de
   recusa com essa granularidade — `applicable` só vê params, nunca comprimentos.
2. **MEDIR a fatia B.** A paridade de 2 costuras está gateada; o **ganho** não. Um
   documento de 2 costuras na GPU contra a CPU pura é a medição que falta, e o §0.0
   cobra número.
3. **(A) GPU por default** — segue bloqueada pela Fase 4 (readouts/preview do
   painel), que é a última coisa entre os 4,19 M partículas e um usuário.

Nada aqui está bloqueado por outra coisa: escolha por retorno.

## §5 — Histórico (não leia salvo arqueologia)

| doc | o que era |
|---|---|
| [`HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-18.md`](HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-18.md) | o briefing do integrador desta rodada — **integrado** |
| [`HANDOFF_line_gpu_nodes_emitter_ADR0130_2026-07-17.md`](HANDOFF_line_gpu_nodes_emitter_ADR0130_2026-07-17.md) | as fatias 1-5 do ADR-0130 + a emenda 1, passo a passo |
| [`HANDOFF_line_gpu_nodes_fase3_2026-07-16.md`](HANDOFF_line_gpu_nodes_fase3_2026-07-16.md) | o laço de simulação (§9 tem o Aberto que alimenta o §2 acima) |
| [`HANDOFF_line_gpu_nodes_fase2_2026-07-15.md`](HANDOFF_line_gpu_nodes_fase2_2026-07-15.md) · [`fase1`](HANDOFF_line_gpu_nodes_fase1_2026-07-15.md) | os nós portados · o motor |

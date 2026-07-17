# HANDOFF — linha `line/gpu-nodes` (GPU/M5 **Fase 3**: a simulação na GPU), 2026-07-16

> **Status:** FECHADA, pronta para integração. **NÃO integrei, NÃO pushei, NÃO rodei `ship.sh`**
> (§0.7 — só por ordem explícita do Enio, via integrador dedicado). Construída em cima da Fase 2/F1.2
> (`994c1aa2`), no MESMO branch `line/gpu-nodes` (stack de commits).
> **Autor:** o agente da Fase 3, a pedido do Enio ("linha Motion, doc na pasta da linha").
> Briefing executado: [`HANDOFF_line_gpu_nodes_fase3_briefing_2026-07-16.md`](HANDOFF_line_gpu_nodes_fase3_briefing_2026-07-16.md).
> Documento central: [ADR-0123](architecture/decisions/0123-gpu-simulation-pre-is-arc-pingpong-plan-becomes-a-dag.md).
>
> **As 5 fatias do ADR estão fechadas, e o escopo dele também** — o `motion.spring` (o último item que
> faltava) e o `force.buoyancy` (a 6ª força) entraram depois, a mando do Enio ("siga implementando").
> O que ficou aberto está em §Aberto; nada ali é do escopo do ADR-0123.

---

## O que landou

**O laço de simulação inteiro cozinha na GPU.** As 5 forças existiam desde o M2 e nunca tinham rodado
um frame na GPU: elas só são avaliadas DENTRO do `pre` loop, e o laço era exatamente o que o plano
recusava. A unidade de valor era o laço, não o nó — o briefing estava certo em não deixar portá-las
primeiro.

| Commit | Fatia | O que |
|---|---|---|
| `2d66217e` | 1 + 2 | o plano vira **DAG** · o `pre` vira **ping-pong de `Arc`** |
| `05632829` | 3 + 4 | `motion.integrate` + as 5 forças + os gates |
| `b7977f2d` | 5 | o **scrub no device** (ring de checkpoints) |
| `b82678f9` | — | cena de smoke `PH2D_GPU_COOK_DEMO=3` + gate de plano |
| `b044742e` | — | **ondas de cor**: `motion.color_ramp` na GPU (a pedido do Enio) |
| `20c04bf5` | — | `cargo fmt --all` + `Cargo.lock` (paridade com o ship) |
| `f4c576a7` | — | splits de LOC (o meu **e** o que a F1.1 tinha estourado) |
| `ced76b73` | 3 | **`force.buoyancy`** — a 6ª e última força sem cobertura |
| `1c09f865` | 3 | **`motion.spring`** — o escopo do ADR-0123 FECHOU |
| `cc4f4345` | — | cena **o MAR** (`PH2D_GPU_COOK_DEMO=4`) + gate + split de LOC |

**O escopo do ADR-0123 está fechado.** As 6 forças + `motion.integrate` + `motion.spring` têm kernel;
`motion.color_ramp` entrou de brinde. **16 dos 90 nós** têm kernel — e o número que importa não é
esse: cobertura de força **não é aditiva, é penhasco**. Um nó sem kernel DENTRO do laço deixa
fronteira, e a fronteira faz o `plan` recusar a simulação inteira (§3). Cinco forças na GPU valiam
zero no grafo que jogasse uma boia n'água; agora não há força que derrube o laço.

### Perf medida (RTX, `--release`)

- **2M instâncias stateless: 4,02 ms/frame** — inalterado (`gpu_cook_millions_timing`).
- Sim (490k, 4 forças): rode o smoke `DEMO=3`. **Não medi com sonda** — ver §Aberto 3.

---

## §1 — As peças novas do contrato lateral (ADR-0122: **8/2/1 intocado**)

`ph2d_nodegraph::gpu` ganhou 3 coisas. Nenhuma toca `NodeOp`/`OpResolver`/`NodeManifest`.

- **`ColumnBinding.port`** — de qual input a coluna é lida (a escrita sempre vai pro output único).
  `0` em todo nó single-input, e é escrito em cada binding de propósito: **a porta não pode ser
  propriedade do nome da coluna** — o `integrate` lê `vel` das DUAS (o seed vem do `rest`, o passo vem
  do `forces`). **Porta 0 é a BASE**: o output nasce dela, e toda coluna que o kernel não menciona
  passa por ali (é o "copie o input primário, reescreva meu canal" que todo nó da CPU faz).
- **`ColumnAccess::Consume`** — lido e **não emitido** (o `accel` transiente). Sem isso um `accel`
  velho passaria pela base e o `add_accel` do próximo tick somaria em cima — divergência, não ε.
- **`ColumnAccess::RefuseIfPresent`** — recusa de **plan-time** (D3). Não virou campo novo em
  `GpuKernel` porque 10 kernels alheios teriam de carregar `refuses: &[]` para o benefício de um só;
  e como binding ela sai **por-porta** de graça, que é a precisão que o D3 precisa (recuso `id` no
  *rest*, não no *forces*).

E o **codegen**: num nó multi-input todo leitor é nomeado por porta (`read_rest_P` /
`read_forces_vel` — `read_vel` nu resolveria em silêncio pra uma das duas), mais um
**`const HAS_<col>: bool`** por leitor. O `HAS_` é o único jeito de expressar um comportamento da CPU
que **ramifica** na ausência em vez de substituir um valor: o `read_*` responde a identidade dos dois
jeitos. O seed do `integrate` é exatamente isso.

## §2 — D3, resolvido: a recusa é **provável** ou não é

O ADR deixou a escolha em aberto ("ou o `eligible()` ganha um teste de forma-do-stream, ou o kernel
declara 'recuso a coluna X'"). **É plan-time, e são as duas coisas:** o kernel declara, e o plano
**deriva** a coluna-set de uma sub-cadeia 100% GPU (`plan::output_shape`, a MESMA regra que o `cook`
costura: base = porta 0, + escritas, − consumidas). Fronteira CPU ⇒ desconhecido ⇒ **recuo**.

A alternativa (recusar no `cook()`) foi descartada: gastaria um upload por frame numa cadeia de
emitter e me obrigaria a **duas portas pra mesma pergunta**.

## §3 — A armadilha que o ADR não nomeia: **duas simulações do mesmo estado**

Pior que o gather. Se o `integrate` roda na GPU e **um nó do laço de forças cai na CPU**, o plano
deixa uma fronteira ali — e o pump, pra cozinhar esse nó, **re-cozinha o `integrate` com o `prev`
DELE**. Viram dois sims do mesmo estado, e a GPU integra um `accel` calculado sobre uma trajetória
que não é a dela, todo tick, pra sempre.

`plan()` agora recusa a reivindicação **inteira** quando um nó staged é fonte de `pre` e sobrou
fronteira. **O laço tem de ser reivindicado INTEIRO.** (Gate: `a_cpu_node_inside_the_loop_refuses_the_whole_claim`.)

Corolário: o shell recusa **>1 fronteira** (o pump entrega UM nó cozido por tick; marchar duas vezes
adiantaria o relógio dele duas vezes). O motor modela N (o ADR pediu), o shell suporta 1.

## §4 — Bug latente da Fase 1 que esta fatia expôs (o mais importante deste doc)

**`presence_signature` colidia** — 1 bit por binding = "lê buffer OU escreve buffer". Num `ReadWrite`
esse bit é **1 nos dois casos** (ausente ainda escreve), então **presente e ausente compartilhavam a
chave do cache de pipeline**, e os módulos diferem por um binding de leitura inteiro. wgpu rejeita o
bind group contra o layout errado: **não é número errado, é crash.**

Invisível na Fase 2 porque **nenhuma cadeia dela muda a presença de uma coluna**. A sim muda no 1º
tick (estado vazio → cheio). Mas a causa é geral, então o gate é da Fase 2:
`grid → scale → scale → output` (o mesmo TIPO com presenças diferentes; a chave é por tipo) —
**isso crashava no código que a Fase 2 entregou**. A assinatura agora sai do `here`, a única coisa
que determina o texto do módulo.

## §5 — Gates verdes no fechamento (rodados 2026-07-16, este worktree)

- `cargo check --workspace --all-targets` ✓ · **contrato `8/2/1`** ✓ · `cook_determinism` +
  `transform_determinism` ✓ (a CPU segue canônica)
- **Paridade ε na RTX** (`--release --ignored`): **11 + 11 = 22**
  - os 9 da Fase 2 **com os ε idênticos** (4,386902e-4 full chain · 1,9e-6 falloff/wiggle · 3,8e-6
    transform · 0 nos bit-exatos) — o oráculo de que o DAG não regrediu o linear
  - o 10º é novo: a regressão do §4 · o 11º é o `color_ramp` (5 presets × 2 interps)
  - os 11 do sim: `seed` **Δ=0** · `seed velocity` 3,8e-6 · `wind+drag` 9,5e-7 ·
    `attractor+vortex` 9,5e-7 · `curl` 1,6e-5 · **`buoyancy` 9,5e-7** · **o mar clampado** 7,0e-4 ·
    **`spring`** 3,6e-5 (tension 8) / 2,2e-4 (tension 60) · **`spring` em Rotation recua** (com o
    irmão de presença) · o pool · o **scrub** 9,5e-7
- **naga**: todo subconjunto de colunas de todo kernel (250 variantes) ✓
- `plan_simulation` (6, sem device) · `plan_analysis` (5) · shell **569 + 10 gates de plano** ✓
- `clippy --all-targets` **zero** · `typos` **zero** · `cargo fmt --all --check` **limpo** ·
  `cargo machete` limpo · **LOC: workspace + shell** ✓

### D4 honrado

O gate do sim mede **UM PASSO a partir de um estado semeado**, no **MESMO** orçamento de ε da Fase 2
(2e-3). Nada afrouxado, nenhuma trajetória longa.

**O falso-verde contra o qual esses gates foram construídos:** um passo de um seed fresco move
`a·dt²` — a 1/60 isso é ~1e-3, que **é o próprio ε**. A comparação seria "dois seeds concordam" e
ficaria **verde com o integrador deletado**. Todo gate lá primeiro afirma, **só na CPU**, que o passo
MOVEU o campo 50× acima do ε (medido: 0,39 a 1,22 — 200 a 600×).

## §6 — Smoke (rodar e clicar na tool Motion)

```
# Fase 3: 490.000 particulas SIMULADAS 100% na GPU (vortex+attractor+drag+curl).
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=3 cargo run --release -p ph2d-host-desktop

# Full-GPU stateless (F1.1): 2.000.000 instancias, ~4 ms/frame. Zoom out.
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=1 cargo run --release -p ph2d-host-desktop

# O MAR: 490.000 particulas caindo numa onda viajante (gravidade + buoyancy).
PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=4 cargo run --release -p ph2d-host-desktop

# Hibrido (F1.2): ondula em Y (GPU) E gira (CPU), nos dois lados da costura.
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=2 cargo run --release -p ph2d-host-desktop
```

**O que olhar no `DEMO=3`:** a nuvem entra em órbita (vortex + attractor no mesmo centro + drag — sem
o drag um vortex puro espirala pra fora por deriva centrífuga) e o curl noise faz ela respirar em vez
de assentar num anel limpo. As **ondas de cor** são advecção de corante: o campo nasce em bandas
horizontais de arco-íris (o ramp atravessado no índice do grid) e o vortex as enrola em espirais que
apertam — a onda é o escoamento ficando visível, não uma animação de cor por cima.
**Arraste o playhead pra trás**: a pose tem de VOLTAR (é o ring, §D5) — sem ele o campo ficaria
congelado na pose do futuro, calado.

**O que olhar no `DEMO=4` (o mar):** metade do campo nasce seca e CAI, metade nasce submersa e SOBE —
as duas na tela ao mesmo tempo. Elas se encontram na superfície e **ficam boiando**: cada uma passa do
ponto e o `drag` tira a energia devagar. Repare que elas também derivam **de lado, pros vales** — o
empuxo é normal à superfície, não pra cima, então um flanco empurra ladeira abaixo. A gravidade é um
`force.wind` apontado pra baixo (o nó de empuxo não tem param de gravidade de propósito: força
direcional constante já existe), e é a **disputa** 12-contra-4 que assenta o campo em vez de lançá-lo.

Gates de GPU (headless, precisa de adapter):
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && cargo test -p ph2d-gpu-cook --release -- --ignored --nocapture
```

## §7 — Gotchas (custaram iteração; a próxima fatia VAI esbarrar)

**Do domínio (herdados, todos ainda valem):** `round` CPU↔GPU diverge no meio-ponto (use `*_round`,
half-away) · noise integer-hash: `bitcast<u32>`, **nunca `u32(x)`** (value-cast, diverge em negativos)
· `array<vec3<f32>>` tem stride 16 · `source_count` = `param_as_count` EXATO · não segure o `&mut` do
`entry().or_insert_with()` através de `pool.acquire` · teste **com a coluna AUSENTE**.

**Novos:**

- **`max_storage_buffers_per_shader_stage` era 8** (o piso do WebGPU, que nenhum adapter de desktop
  respeita) e o `integrate` quer **11** com tudo presente. Subido ao máximo do adapter em
  `ph2d-gpu/src/context.rs` — o MESMO argumento já documentado ali pros outros caps. Um kernel acima
  do limite agora **recua** (`GpuCookError::TooManyBindings`) em vez de crashar: o plano não vê device
  nem stream, então a checagem mora no `cook`. **Um kernel novo com muitos bindings vai bater nisto.**
- **O `dt` sai de `sim_t[0]`**, um broadcast do elemento 0 da coluna do estado. Se a coluna sumir, a
  identidade **não pode ser uma constante** (a CPU preenche com `playhead`) — daí o `HAS_forces_sim_t`
  + `select(params.playhead, ...)`.
- **`is_finite()` não existe em WGSL.** É `abs(x) <= 3.4028235e38` **por lane** (toda comparação com
  NaN é falsa; ±inf estoura o max). Não use `max()`: o comportamento dele com NaN é
  implementation-defined.
- **Um `pre` só é aceito se fechar o laço num nó que este plano também stage** — um ancestral já
  reivindicado **ou o próprio nó** (o auto-laço `out --pre--> forces` que o editor auto-liga é a
  topologia DEFAULT, e o nó ainda não está em `claimed` quando o `eligible` roda: a elegibilidade é
  decidida ANTES do walk se comprometer). Esqueci o `src != node` e recusei o documento mais comum que
  existe.
- **Dois gates com nomes parecidos e escopos disjuntos:** `file_loc_caps` (shell, cap 600) e
  `architecture_workspace_file_loc_cap` (`crates/*/src/`, cap **700**, allowlist própria). A Fase 2
  rodou o primeiro e o segundo ficou vermelho desde a F1.1 (`renderer.rs` 1000→1051). **Rode os dois.**
- **A árvore estava fmt-suja no HEAD** (`lower.rs`, `ph2d-eval-motion`, `attr.rs`, o `eval` do
  `rotate`) — o `ship.sh` roda `cargo fmt --all -- --check` e reprovaria. Absorvido; rode
  `cargo fmt --all`, não `-p`.

## §8 — Lições de gate desta jornada (todas me pegaram)

1. **[[feedback_layered_defenses_need_per_layer_gates]] — no arquivo onde eu tinha citado a lição.**
   O gate "forma inderivável ⇒ recuo" passava pela razão ERRADA: quem segurava o documento
   `emitter → integrate` era a recusa das duas simulações (§3), não a da forma. As duas camadas
   apontam pro mesmo lado no produto, então o fixture compartilhado não provava nenhuma. A regra é de
   **motor**, então o gate virou de motor (`test.refuser`: nomeia uma coluna, sem laço `pre`, onde só
   a camada 3 pode decidir). Mutação confirma.
2. **[[feedback_a_gate_only_proves_what_its_fixture_contains]].** "O seed lê `vel` do FORCES em vez do
   REST" **sobreviveu à suíte inteira**: o grid não emite `vel`, então os dois leitores respondem a
   identidade e as duas portas são indistinguíveis — e é a única coisa que um leitor-por-porta existe
   pra impedir. Não removi o binding (seria uma mina pro dia do emitter na GPU, que é a próxima
   fatia): tornei o caso **alcançável** com `test.velgen`, a forma que um emitter na GPU vai ter.

3. **Um fixture pode cair num regime CAÓTICO, e aí o ε não é o problema — o fixture é.** O clamp
   `wave_length.max(1e-3)` do `buoyancy` só é observável **onde o domínio dele é vazio**
   ([[feedback_a_threshold_must_live_where_the_domain_is_empty]]), então o gate dele usa
   `wave_length = 0`. Mas o valor que o clamp ENTREGA (`1e-3`) é ele próprio um mar patológico: com
   `amplitude = 0.6`, `slope = amp·2π/λ ≈ 3770·cos`, a normal deita quase horizontal e a **direção**
   dela vira com o **sinal do cosseno**. Magnitude limitada (`|a| ≤ density`, a normal é unitária) com
   sinal virando **é** uma divergência de 2·density, e 1 ulp de fase decide: medido, os dois caminhos
   se separaram por **0,2022 ≈ 2·40·dt²**. Isso é o **D4 do ADR**, não bug de porte. O conserto é
   `amplitude = 1e-4` — o mesmo mar clampado com `slope ≈ 0,63`, bem-condicionado, e o clamp segue
   igualmente observável (quem NaN-a um kernel sem clamp é a **`phase`**, que a amplitude não toca).
   **Nunca um ε afrouxado até o caos caber embaixo** — esse oráculo modelaria o filtro, não a verdade.
4. **Um sobrevivente pode ser inobservável por CONSTRUÇÃO, e vale dizer isso em vez de fingir.** Tirar
   o clamp de `depth` sobreviveu e **fica sem gate, deliberadamente**: o `clamp(sub, 0, 1)` a jusante
   já doma o ±inf, então os dois caminhos só diferem numa instância exatamente **na** linha d'água —
   onde o CPU lê `0/1e-3 = 0` (seca, não move) e o GPU sem clamp lê `0/0 = NaN`, que o guard de
   finitude do `motion.integrate` rejeita (**também** não move). O guard converge os dois. O clamp fica
   porque espelha o `eval`, que é o contrato — não porque um gate vermelho o segura, e o doc do kernel
   diz exatamente isso.

**8 rodadas de mutação no total** (plano ×4, kernel ×6, ring ×2); 3 sobreviventes → 3 gates novos
(1 sobrevivente **aceito e documentado**, o `depth`).

## §9 — Aberto (a próxima fatia / journeys futuros)

1. ~~**`motion.spring`**~~ — **LANDOU** (`1c09f865`), junto com o `force.buoyancy` (`ced76b73`). A
   receita desta seção estava certa e foi seguida à risca; 3 coisas que ela não previa e a próxima
   fatia vai reencontrar:
   - **O `pairing` não precisou de tradução.** O CPU pareia posicionalmente exatamente quando o state
     tem `spring_value` **e** `state.count() == n`; a regra de presença do sequenciador é *"a porta
     tem a coluna E o count bate"* — o **mesmo predicado**. Então `HAS_state_spring_value` **é** o
     `pairing().is_some()`, e o ramo seed/step não é uma re-derivação da condição do CPU.
   - **O idioma `MUST_MOVE` desta suíte não protege a mola.** Em todo outro gate o mover sob teste é
     a única coisa mexendo o campo, então *"moveu?"* e *"o kernel disparou?"* são a mesma pergunta.
     Na mola elas se separam: quem move é o **oscilador**, e uma mola compilada como pass-through
     passaria por `MUST_MOVE` **e** por paridade (o pass-through do CPU e o do GPU concordam
     perfeitamente). O oráculo é a mesma cadeia **sem** a mola: o trabalho dela é *não ter chegado
     ainda*, e é esse atraso que se afirma.
   - **O sub-passo adaptativo não estava sendo exercitado.** No `tension = 8` default, `steps =
     ceil(dt/sqrt(STABLE/tension)) = 1` — o laço roda uma vez e um kernel com `steps = 1` fixo ficava
     verde. O gate roda os dois regimes (8 → 1 passo, 60 → 2); a mutação `steps = 1u` só derruba
     depois disso.
2. **O gather por `id`** (emitter/partículas com nascimento e morte) — o regime mais interessante, e a
   fatia que o D3 nomeia. O `test.idgen`/`test.velgen` já existem como a forma que um emitter na GPU
   vai ter, e o `RefuseIfPresent` já está gateado pro dia em que o gather chegar.
3. **Medir o sim com sonda.** Só tenho o número stateless (2M @ 4,02 ms). Um `gpu_sim_millions_timing`
   ao lado do existente fecha isso; o `DEMO=3` (490k × 4 forças) é o fixture.
4. **Reduções na GPU** (destrava twist/bend: `r_max`/`x_extent`) — extensão do motor; desenhe antes.
5. **Multi-input real** (`look_at`/`combine`): o motor **já suporta** (o DAG anda em N inputs) — falta
   só o kernel de cada um. Era item 3 do Aberto da Fase 2 e **saiu de graça** nesta.
6. **Gradient do `motion.tint`** — o item 4 do Aberto da Fase 2, agora **trivial**: era "precisa de
   identidade posicional (`identity` é CONSTANTE, não `f32(i)`)", e o `const HAS_<col>` resolve isso
   (o `motion.color_ramp` já foi portado assim — copie o padrão dele).
7. **`t` do `color_ramp`** (e todo nó `value.*`): dois bloqueios de MOTOR, nomeados —
   (a) `value.attribute` lê a coluna por **text param**, e `ColumnBinding.column` é `&'static str`:
   um nome dinâmico não vira binding estático; (b) os nós `value.*` **descartam a base** (`Stream::new(n).with("v")`),
   e o output deste motor nasce sempre da porta 0 — não há como expressar "base: nenhuma".
   Destravar os dois daria colorir-por-velocidade (`vel → v → t`), que é a onda que o campo
   *calcula* em vez de carregar.
8. **N fronteiras no shell** (o motor planeja; o pump entrega uma por tick) ·
   **readouts/probe no modo GPU** (Fase 4) · **Fases 3–4 do plano** (JFA voronoi, spatial-hash boids,
   renderer consumindo colunas cruas).

### Desvio deliberado do ADR (D5) — leia antes de "consertar"

O D5 dizia: *"estourou o cap → o sim recua pra CPU"*. **Não recua.** Um fallback pra CPU no meio da
sessão **não pode** funcionar: o pump não estava marchando e o relógio dele está velho, então ele
responderia com uma simulação **diferente**, não com um rewind. Alvo fora da janela **ancora no seed do
tick 0 e re-simula pra frente** (qualquer tick passado é alcançável) — que é exatamente a política que
o próprio ring da CPU já usa pra mesma pergunta. Preço: um scrub muito distante re-simula muitos ticks
num frame (a CPU tem o mesmo penhasco).

E o ring é **esparso** (`RING_STRIDE = 8`) onde o da CPU é **denso** — de propósito. A CPU decidiu
"denso a menos que o custo da CÓPIA domine"; aqui a cópia é de graça (refcount) e o custo mudou de
lugar: **residência**. Um buffer em checkpoint é um que o pool não recicla, então um ring denso faz a
sim alocar o estado inteiro TODO TICK. Medido: mesma janela, 21 checkpoints/5,9 MB → **3/590 KB**.

## §10 — Integração (briefing pro integrador do Enio)

> **Como ler:** esta seção é autossuficiente. O QUE a linha entrega está no topo + §Aberto; o PORQUÊ de
> cada peça está em §1–§4; os gates de fechamento em §5. Você **funde e faz o ship** — a linha só fecha.

### O que é

Todo o arco **GPU/M5** numa branch linear: **Fase 0** (cook paralelo na CPU, rayon, bit-idêntico) →
**F1.1** (motor de cook GPU-resident, `ph2d-gpu-cook`) → **Fase 2 / F1.2** (10 kernels stateless + o
cook híbrido CPU-prefixo/GPU-sufixo no shell) → **Fase 3** (o laço de simulação na GPU) →
**`force.buoyancy` + `motion.spring` + a cena do mar**. O escopo do ADR-0123 está fechado.

### Base e forma do merge

- **Fork:** `12ccaecd` — **o HEAD atual de `main`**. `main..line/gpu-nodes` é **fast-forward puro,
  31 commits, zero merge, zero divergência.** Se `main` não andou desde então, é `git merge --ff-only`.
- **Se `main` andou** (outra linha integrou antes): os conflitos prováveis estão listados abaixo. O
  gate combinado é `scripts/foundational-integrate.sh` (ADR-0107) + Mergiraf no resíduo textual.
- Marcos (todos em ordem de commit, do fork ao HEAD `e99a19bd`): `74a19784` Fase 0 · `74aa2b00` F1.1 ·
  `6325a3a8`/`86a2fe35` kernels Fase 2 · `f877b8a0`/`88326d00`/`8c018447` F1.2 · `72301921` 2M ·
  `4d176f9d` ADR-0123 · `2d66217e` (DAG+ping-pong) · `05632829` (integrate + 5 forças) · `b7977f2d`
  (scrub no device) · `b82678f9` (`DEMO=3`) · `b044742e` (color_ramp/ondas de cor) · `f4c576a7` (splits
  de LOC) · **`ced76b73` (buoyancy) · `1c09f865` (spring) · `cc4f4345` (o mar `DEMO=4`)**.

### O que foi tocado (e o que NÃO foi)

- **`ph2d-nodegraph` — foundational, mas o CONTRATO CONGELADO está intocado.** Só `gpu.rs` mudou (o
  **canal lateral** do ADR-0122: `ColumnBinding.port` + `ColumnAccess::{Consume, RefuseIfPresent}` —
  §1). `node.rs` (`NodeManifest=8`/`NodeOp=2`/`OpResolver=1`) **não mudou** — verificado por diff, e o
  gate `architecture_contract_surface` **passa** (§5). `gpu.rs` é **append-only por design** (§0.2:
  foundation projetada pra isolamento), então uma linha irmã que só ADICIONE ali funde sem colidir.
- **`ph2d-gpu`** (`context.rs`): **1 limite** subido (`max_storage_buffers_per_shader_stage`, o
  `integrate` precisa de 11; o default WebGPU é 8), seguindo o precedente documentado do próprio arquivo.
- **`ph2d-gpu-cook`**: o motor. Módulos novos `plan.rs`/`ring.rs`/`instances.rs`/`codegen.rs`/`stream.rs`.
  Praticamente todo dele é desta linha.
- **`ph2d-render`** (`renderer.rs`): **só um split** — as 3 fns `render*` saíram pro
  `renderer_draw.rs` (o débito de LOC que a F1.1 tinha deixado). Comportamento idêntico.
- **23 node-crates tocadas** (diff vs `main`): o arco GPU/M5 inteiro registra kernel em **19** delas
  (`ph2d-node-registry` ganhou `register_gpu_kernel`; `motion.output` é passthrough). O grosso é da
  Fase 2/F1.1 (grid/oscillator/move/transform/rotate/scale/falloff/tint/wiggle); **esta fatia (Fase 3)
  adicionou 9**: `motion.integrate` · as 6 forças (wind/drag/attractor/vortex/curl/**buoyancy**) ·
  **`motion.spring`** · `motion.color_ramp`. As demais só ganharam o campo mecânico `port: 0` no binding.
- **shell** (`shells/desktop`): `render_loop/motion_bridge_gpu.rs` (a rota GPU) + `motion_state.rs`
  (opt-in via env) + os irmãos novos `motion_state_gpu_demos.rs` / `motion_state_gpu_tests.rs`.

### Conflitos prováveis (se `main` andou)

- **`Cargo.lock`** — 34 linhas de dep novas (as node-crates viraram dev-deps do gate de paridade).
  **Regenerar** com `cargo build`/`check`, não fundir à mão.
- **Número do ADR (0122/0123)** — se uma linha irmã reivindicou 0122/0123 antes, **renumerar os dois
  arquivos + os stamps** (`grep -rn "0122\|0123"`). São doc; não há gate sobre o número.
- **`renderer.rs` · `motion_state.rs` · `motion_state_tests.rs` ENCOLHERAM por split.** Se outra linha
  os editou, o Mergiraf pode precisar de mão — mas os blocos movidos estão **intactos** em
  `renderer_draw.rs` / `motion_state_gpu_demos.rs` / `motion_state_gpu_tests.rs`. Resolva pelos estágios
  do índice, não pelos marcadores ([[feedback_resolve_conflicts_from_index_stages_not_markers]]).

### Ship — o que o `ship.sh` NÃO cobre sozinho

**Os 22 gates de paridade GPU são `#[ignore]`** (precisam de adaptador). O `nextest` do `ship.sh` **não
os roda** — eles são o **audit** desta linha (§5), então rode-os você, **na RTX**, como parte do
fechamento:

```
cd <worktree-ou-arvore-integrada> && cargo test -p ph2d-gpu-cook --release -- --ignored
```

Fora isso, `ship.sh` é a paridade normal (fmt/clippy/machete/deny/audit/nextest/typos) — no fechamento
desta linha estava **tudo verde** (§5). O ship do integrador drena latentes de 2–4 iterações
([[project_integrator_ship_catches_latents_budget_iterations]]); o gate per-linha não basta.

### Sem escapes

**Nenhum caso §1.5.5** (nenhuma colisão de mesmo-símbolo fora dos meus arquivos; nenhum contrato
congelado tocado). **Nenhum ADR novo** — o 0122 cobre o canal lateral e o 0123 já estava escrito; esta
linha o **executou**. Vale emendar o D5 do 0123 com o desvio deliberado documentado acima (§9, "Desvio
deliberado do ADR").

### Nota ao Enio (fora do escopo do integrador)

As 2 lições de gate desta jornada (o **fixture em regime caótico** e o **4º desfecho de um sobrevivente**)
foram escritas na memória versionada (`project-memory/`, via symlink → repo **primário**, `main`), e estão
**sem commit** lá, junto dos ~24 arquivos de memória que já estavam `??` desde o começo da sessão. Não são
desta linha (repo/branch diferente) — ficam pra você decidir quando commitar a memória.

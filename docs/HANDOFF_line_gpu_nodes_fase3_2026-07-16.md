# HANDOFF — linha `line/gpu-nodes` (GPU/M5 **Fase 3**: a simulação na GPU), 2026-07-16

> **Status:** FECHADA, pronta para integração. **NÃO integrei, NÃO pushei, NÃO rodei `ship.sh`**
> (§0.7 — só por ordem explícita do Enio, via integrador dedicado). Construída em cima da Fase 2/F1.2
> (`994c1aa2`), no MESMO branch `line/gpu-nodes` (stack de commits).
> **Autor:** o agente da Fase 3, a pedido do Enio ("linha Motion, doc na pasta da linha").
> Briefing executado: [`HANDOFF_line_gpu_nodes_fase3_briefing_2026-07-16.md`](HANDOFF_line_gpu_nodes_fase3_briefing_2026-07-16.md).
> Documento central: [ADR-0123](architecture/decisions/0123-gpu-simulation-pre-is-arc-pingpong-plan-becomes-a-dag.md).
>
> **As 5 fatias do ADR estão fechadas.** O que ficou aberto está em §Aberto — e o item 1 é o único
> pedaço do escopo do ADR que NÃO entregou (`motion.spring`), com receita pronta.

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
- **Paridade ε na RTX** (`--release --ignored`): **11 + 7 = 18**
  - os 9 da Fase 2 **com os ε idênticos** (4,386902e-4 full chain · 1,9e-6 falloff/wiggle · 3,8e-6
    transform · 0 nos bit-exatos) — o oráculo de que o DAG não regrediu o linear
  - o 10º é novo: a regressão do §4 · o 11º é o `color_ramp` (5 presets × 2 interps)
  - os 7 do sim: `seed` **Δ=0** · `seed velocity` 3,8e-6 · `wind+drag` 9,5e-7 ·
    `attractor+vortex` 9,5e-7 · `curl` 1,6e-5 · o pool · o **scrub** 9,5e-7
- **naga**: todo subconjunto de colunas de todo kernel (250 variantes) ✓
- `plan_simulation` (6, sem device) · `plan_analysis` (5) · shell **578** (era 566) ✓
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

## §8 — Duas lições de gate desta jornada (as duas me pegaram)

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

**5 rodadas de mutação no total** (plano ×4, kernel ×3, ring ×2); 2 sobreviventes → 2 gates novos.

## §9 — Aberto (a próxima fatia / journeys futuros)

1. **`motion.spring` — o único item do escopo do ADR que não entregou.** Deferido conscientemente:
   a fatia 5 (o scrub) é **correção** (sem ela o scrub mente calado) e o spring é **mais um nó** sobre
   um padrão que já está provado. **Receita** (é o `integrate` de novo): 2 inputs (`in`/`state`, o
   mesmo `pre`) · bindings `P` ReadWrite porta 0 + `falloff`/`inv_mass` Read porta 0 +
   `spring_value`/`spring_vel`/`sim_t` ReadWrite porta 1 + `id` **RefuseIfPresent** porta 0 ·
   `applicable` = `channel ∈ {0,1}` (X/Y — o precedente exato do `wiggle`/`oscillator`; Rotation/Size
   recuam) · o sub-passo adaptativo (`steps = ceil(dt/ideal)`, `ideal = sqrt(STABLE/tension)`) é um
   laço WGSL, como o do `curl` · guard de NaN reseta pro TARGET (não pro seed) · o blend final é
   `falloff × inv_mass`. Gate: `sim_chain` já aceita a topologia; copie
   `one_step_of_wind_and_drag_matches_the_cpu` e **faça o campo MOVER** acima do `MUST_MOVE`.
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

## §10 — Integração (pro integrador do Enio)

- **Base:** `line/cook-parallel` (Fase 0) → F1.1 → Fase 2/F1.2 → **esta**, tudo em `line/gpu-nodes` em
  ordem de commit. Fast-forward natural. Marcos: `74a19784` (Fase 0) · `74aa2b00`..`e7605cfd` (F1.1) ·
  `6325a3a8`/`86a2fe35` (Fase 2) · `f877b8a0`/`88326d00`/`8c018447` (F1.2) · `72301921` (2M) ·
  `4d176f9d` (ADR-0123) · **`2d66217e`..`f4c576a7` (Fase 3)**.
- **Foundational tocado nesta fatia:** `ph2d-nodegraph` (`gpu.rs` — 3 adições ao canal lateral;
  contrato **8/2/1 intocado**) · `ph2d-gpu` (`context.rs`: 1 limite) · `ph2d-gpu-cook` (motor;
  `plan.rs`/`ring.rs`/`instances.rs` novos) · `ph2d-render` (só o split `renderer_draw.rs`) · 16 node
  crates (9 × `port: 0` mecânico + **7 kernels novos**) · shell (`motion_bridge_gpu` + `motion_state`
  demo + `motion_state_gpu_tests.rs` novo).
- **Conflitos esperados:** `Cargo.lock` (7 dev-deps novas — regenerar) · o número do ADR-0122/0123 se
  outra linha reivindicou (renumerar; os stamps vão junto) · `renderer.rs`/`motion_state_tests.rs`
  **encolheram por split** — se outra linha os editou, o Mergiraf pode precisar de ajuda (os blocos
  movidos estão intactos em `renderer_draw.rs` / `motion_state_gpu_tests.rs`).
- **Nenhum escape §1.5.5.** Nenhum ADR novo (o 0122 cobre o canal lateral; o 0123 já estava escrito e
  esta linha o EXECUTOU — vale emendar o D5 com o §9 acima).

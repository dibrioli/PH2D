# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, grupos I..P + a auditoria (2026-08-16)

> **Estado:** linha FECHADA, gate completo VERDE, **aguardando ordem de integração do Enio.**
> A linha não integra e não pusha (CLAUDE.md §0.7). Este documento é o que o **agente integrador** precisa.
>
> **Tip da linha:** `6515abcf4` · **53 commits** contra o `main`.

## §1 — O que entra

Oito grupos da segunda volta da [conferência dos nós](../89_plano_conferencia_dos_nos.md),
cada um com a sua cena de smoke (ordem do Enio: *"a cada grupo uma cena de smoke"*),
mais a **auditoria multiagêntica** que o Enio pediu no fecho.

| grupo | o que fecha | cena | smoke |
|---|---|---|---|
| **I** | a VIZINHANÇA vira um número (`motion.proximity`) — e *Scale*/*Hide* saem por COMPOSIÇÃO | `=49` | ✅ |
| **J** | o PINO alcança as três simulações (`inv_mass` pela cadeia de estado) | `=50` | ✅ |
| **J′** | o report do smoke da `=50`: a prescrição do corpo mole + o espaço pessoal do bando | `=51` | ✅ ⚠️ ver §1.1 |
| **K** | o peso por partícula (`soft_body`) + os SUB-PASSOS (`verlet_rope`) | `=52` | ✅ |
| **L** | o TETO DA TAXA (`motion.delay`: `max_step` + `max_accel`) | `=53` | ✅ |
| **M** | a CONTAGEM da conferência deixa de ser escrita à mão (sem código de produto) | — | — |
| **N** | o `motion.wiggle` ganha as OITAVAS, o multiplicador e o LAÇO | `=54` | ✅ |
| **O** | o `motion.oscillator` ganha o PULSE WIDTH e o `motion.stagger` o OFFSET | `=55` | ✅ |
| **P** | o `motion.drive` escreve uma COLUNA NOMEADA — a **§10.0 do plano FECHA** | `=56` | ✅ |

✅ **As OITO cenas (`=49`..`=56`) foram smokadas e aprovadas pelo Enio**, cada uma à medida
que o grupo dela fechou.

O mecanismo de cada grupo está na **§5 do `CLAUDE.md`**, escrito no commit de cada um —
não foi copiado para cá.

### §1.1 — A `=51` MUDOU depois de aprovada

⚠️ A auditoria (§6) achou que o par da cena `=51` diferia em **DUAS** coisas: a banda tratada
carregava o alcance novo **e** um peso de separação 3,75× maior — e um peso desses abre um
bando **sozinho**.

**A banda TRATADA ficou byte-idêntica à aprovada** (mediana 1,6137, 0/40 sobrepostos — os
números que a mensagem do smoke cita). Só o **CONTROLE** se moveu (0,803/34 → 1,182/11),
porque era ele o confundidor.

**Re-smoke é barato e o Enio decide se o quer.** A leitura que a cena promete
(*"o alcance abre o bando"*) continua verdadeira e agora é a única coisa que o par mede.

## §2 — Superfície de colisão (MEDIDA contra o `main` de HOJE)

| eixo | valor | como foi medido |
|---|---|---|
| `PROJECT_SCHEMA` | **84 INTOCADO** | `git diff main...HEAD -- 'shells/desktop/src/project*.rs'` → **vazio** |
| contrato congelado | **INTOCADO** | `git diff` vazio em `ph2d-nodegraph/src/node.rs` e `ph2d-core/src/tool.rs` |
| ADR | **NENHUM novo** | `git diff main...HEAD -- docs/architecture/decisions/` → vazio ⇒ **fora de toda disputa de número** |
| registro do `ph2d-ecs` | **INTOCADO** ⇒ os **três** espelhos também | `git diff` vazio em `registry.rs`, `ph2d-render/`, `ph2d-script/` |
| `ph2d-i18n` | **INTOCADO** | ⇒ a cadeia `vector::tr(k).or_else(sculpt3d::tr)` fica de pé |
| crates novas | **1** — `ph2d-node-motion-proximity` (folha drop-in, glob member) | |
| pacotes externos novos | **ZERO** | o único `+name` do `Cargo.lock` é a própria crate |
| `Cargo.toml` | **4** | a nova · o glob do registry · `[dev-dependencies]` do `ph2d-gpu-cook` · `ph2d-fbm` no wiggle |
| cenas de smoke | **1..56 contínuas, sem duplicata** — ⚠️ **próxima livre: 57** | contado no `match` do roteador, não numa nota |

⚠️ **A crate-nó entra em `[dev-dependencies]` da `ph2d-gpu-cook`**, nunca em `[dependencies]`:
ela só existe ali para o gate de paridade CPU×GPU e o `src/` não a usa ⇒ **machete-safe**
(o precedente das cinco crates-nó da `line/gpu-nodes`).

⚠️ **`PROJECT_SCHEMA` intocado quer dizer que o corte do `project.rs` NÃO alcança esta linha.**
A `line/physics` partiu aquele arquivo em 15/08 (a escada e a constante mudaram-se para o irmão
`project_schema.rs`) e o modo de falha é **silencioso** — um degrau escrito no arquivo antigo
funde limpo e evapora. Esta linha **não escreve degrau nenhum**, então não há o que conferir
nos três sítios.

## §3 — O REBASE (medido no dia, não estimado)

⚠️ **A caixa que envelhece entre o fechamento e a ordem é esta.** O `main` **andou 28 commits**
desde o fork — ele recebeu a **F4b da `line/Vector`** (o corpo de seção que dobra) mais o gate
dos 11 GB.

| pergunta | resposta medida |
|---|---|
| commits do `main` desde o fork | **28** |
| arquivos que a LINHA toca | **98** |
| arquivos que o `main` moveu | **72** |
| **interseção** | **1** — `CLAUDE.md` |
| simulação do merge (`git merge-tree --write-tree`) | **SEM CONFLITO** |

**O rebase é limpo.** A única interseção é o `CLAUDE.md`, e as duas edições caem em regiões
diferentes do arquivo (o `main` mexeu na lista de abertos da §5 do **Vector**; esta linha
escreve na §5 do **Motion Nodes**).

⚠️ **Conferir mesmo assim, e a razão está escrita neste repo:** o `merge-tree` responde sobre
TEXTO, e a família do `project_tokens::install` (04/08) e a do `paint_brush.rs` (15/08) mostram
que **um merge limpo pode estar semanticamente quebrado** — uma edição pode fundir para o lado
errado de um corte que outra linha fez. Aqui o risco é baixo (a interseção é um doc), mas a
varredura de marcadores tem de incluir **`|||||||`** (diff3), não só `<<<`/`>>>`/`===`.

## §4 — Ponto de merge sensível: UM

`shells/desktop/src/motion_state_demo_router.rs` cruzou 600 LOC ⇒ as **nove** cenas de grupo
(`=41..=49`) saíram para o irmão `motion_state_demo_conferencia.rs`, **uma função por cena**.

⚠️ **O irmão NÃO tem `match` nenhum, de propósito.** O roteador continua a ser a ÚNICA lista de
níveis, porque dois `match` em dois arquivos deixariam um nível reivindicado duas vezes passar
**em silêncio** (o compilador só vê `unreachable pattern` dentro de um mesmo `match`).
**Uma linha que acrescente uma cena tem de escrevê-la no roteador.**

## §5 — Mudanças de comportamento (nomeadas)

1. **`motion.verlet_rope` / `soft_body` / `boids` honram `inv_mass`** (grupo J) — um grafo com
   `motion.pin_constraint` no laço passa a segurar o que antes ignorava.
2. **Um `soft_body` de massa infinita SEGUE a prescrição** (`anchor + rest[i]`) em vez de
   congelar no lugar — mover `spacing`/`rows`/`cols`/âncora com a sim rodando agora move o pino.
3. **O `boids` ganhou `separation_radius`** (default `0,0` = byte-idêntico) e a GPU **RECUSA o
   device** quando ele passa a percepção (a grade é construída com `cell_param: "radius"`).
4. **`motion.delay` ganhou dois tetos** (default `0,0` ⇒ o passe nem corre; byte-idêntico
   **no valor E no objeto cozido**).
5. ⚠️ **UMA MUDANÇA NO DEVICE, e ela é um CONSERTO:** até o grupo O, os variants de WGSL do
   `motion.oscillator` para `rot` e `size` **ignoravam `time_mode`/`bpm`** (só o de `P` os lia).
   Um grafo a dirigir a **rotação ou o tamanho em BPM** corria a uma taxa na CPU e a outra na
   GPU — sem erro e sem aviso. Agora as três rotas concordam, e o gate
   `the_bpm_ruler_reaches_every_oscillator_channel` (que nasceu VERMELHO) as pina.

## §6 — A AUDITORIA MULTIAGÊNTICA (ordem do Enio, 2026-08-16)

Oito lentes de LEITURA em paralelo (disciplina de gate · correção de kernel · paridade
CPU×GPU · neutro/byte-identidade · superfície de colisão · doc-contra-código · cenas de
smoke · costura de UI) → barreira de dedup → **catorze céticos** com `refuted: true` por
omissão → síntese.

⚠️ **A divisão que fez a auditoria valer é *o que se LÊ* contra *o que só um build SERIAL
mede*.** O fan-out achou 36 candidatos; as duas coisas que **nenhuma leitura podia achar**
saíram da medição:

- **o `--ignored` CANCELA na primeira falha**, então as duas suítes de GPU **novas** da
  jornada (`gpu_proximity` / `gpu_boids`) **nunca tinham corrido**. Com
  `--no-fail-fast --test <nome>`: **38/38**.
- o `value_slope` vermelho é **pré-existente**, e agora está confirmado **a todos os
  dígitos** (`1.05023384e-4`, o número que o `CLAUDE.md` já registra como reprovando no
  `main`).

### As QUATRO reais (corrigidas, cada uma com mutação provada)

| # | o defeito | o mecanismo |
|---|---|---|
| 1 | o WGSL do `motion.boids` **elevava ao quadrado um param COM SINAL** que a CPU sanitiza | *o quadrado apaga o sinal*, logo a sanidade tem de vir **ANTES** dele — e o `applicable` que existia para impedir a divergência era cego a `sep_r` negativo |
| 2 | o teto de taxa do `motion.delay` era **INERTE** numa posição alcançável do slider | ele só corria dentro do ramo do LAG; com `ticks = 0` o caminho transparente saltava o `rate_limit` inteiro |
| 3 | o `motion.drive(Custom…)` podia **sobrescrever uma coluna de escrituração** | `is_bookkeeping_column`, porta nova na `ph2d-nodegraph` ao lado do `VALUE_COLUMN` — ⚠️ **NÃO** a lista `INTERNAL` do picker, que responde outra pergunta (`falloff` é lida **e** escrita) |
| 4 | o gate de sub-passos da corda era **TAUTOLÓGICO** | comparava `Params { substeps: 1 }` consigo mesmo; agora passa pela porta de **LEITURA** (param AUSENTE contra `1` explícito), com dois controles |

Mais **DOIS gates de higiene** que a varredura produziu:

- **o sweep do naga DERIVA o registry** (`register_all_nodes`) em vez de o enumerar. A lista
  à mão tinha 49 registros e **exatamente UMA crate com `gpu.rs` estava fora** — a mais nova,
  `motion.proximity`, cuja paridade é `#[ignore]` ⇒ *o WGSL dela só encontrava um compilador
  na máquina de quem tem placa*. Preço medido: **0,27 → 0,28 s**; `cargo machete` limpo.
- a cena **`=51`** deixou de ter um par confundido (§1.1).

### As DUAS que DISSOLVERAM na verificação

*É o resultado honesto de uma varredura, e fica escrito para ninguém as re-abrir:*

- a sanitização do `inv_mass` no device é o **espelho exacto** da CPU (`w <= 0.0` → early-out
  nos dois, negativo incluído). **Não há vão.**
- as duas *"afirmações que contradizem o próprio gate"* eram (a) uma correção **já narrada**
  no teste do `motion.stagger` — o nó em si está certo — e (b) uma *mola de compressão* que
  **não existe** no fonte do `motion.boids`.

### As DUAS MEDIDAS e NÃO curadas (a cura mudaria um smoke aprovado)

- **os sub-passos compõem com o `damping`** da `motion.verlet_rope`. A cena `=52` **não
  escreve `damping`**, então o smoke aprovado já continha a composição — a tabela medida e as
  duas curas candidatas estão no doc-comment do param, com um ⛔ explícito de *não conserte
  sem ordem*.
- **a `=51`**: a banda tratada é byte-idêntica à aprovada; só o controle se moveu (§1.1).

## §7 — Gate de fechamento (rodado na worktree, tip `6515abcf4`)

| gate | resultado |
|---|---|
| `cargo fmt --all -- --check` | **EXIT 0** |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | **EXIT 0, zero warnings** |
| `cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast` | **16.205 correram, 16.204 passaram, 1.587 skipped** |
| gates de GPU (`--run-ignored all`, RTX, `--test-threads=1`) | **264 de 265** |
| censo | **125 nós · 545 params · 526 com hint · 158 com unidade** |

⚠️ **`--no-fail-fast` NÃO é opcional aqui, e a auditoria é a prova:** a primeira corrida
cancelou **3418 testes** na primeira falha, e a falha era uma flake de carga.

⚠️ **As flakes de relógio foram exoneradas por TRÊS testemunhas cada, nunca por opinião**
(crate com `git diff` **VAZIO** contra o `main` · passa **isolada** · `load average` acima do
limiar): `the_cost_of_depth_is_linear_not_explosive` (`ph2d-timeline`) e
`the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` (`flip_smooth`) — ⚠️ e **elas
trocaram de lugar entre duas corridas do mesmo binário**, que é a assinatura da carga.

⚠️ **E os gates de GPU só significam alguma coisa em SÉRIE:** rodados em paralelo, **três**
morreram com `wgpu error: Out of Memory` (cada binário abre o próprio device) e um gate de
razão disparou; com `--test-threads=1` e a máquina em `load 2,2` **os quatro passam**. O
único vermelho constante entre as corridas é o `value_slope` pré-existente.

⚠️ **O que este gate NÃO alcança, e o integrador tem de rodar:** a árvore **combinada**
depois do rebase. *Skip gracioso não é verde.*

## §8 — Smokes

```
env PH2D_GPU_COOK_DEMO=49 cargo run -p ph2d-host-desktop --release   # a vizinhança
env PH2D_GPU_COOK_DEMO=50 cargo run -p ph2d-host-desktop --release   # o pino nas simulações
env PH2D_GPU_COOK_DEMO=51 cargo run -p ph2d-host-desktop --release   # a prescrição + o espaço pessoal
env PH2D_GPU_COOK_DEMO=52 cargo run -p ph2d-host-desktop --release   # o peso + os sub-passos
env PH2D_GPU_COOK_DEMO=53 cargo run -p ph2d-host-desktop --release   # o teto da taxa
env PH2D_GPU_COOK_DEMO=54 cargo run -p ph2d-host-desktop --release   # o tremor com textura
env PH2D_GPU_COOK_DEMO=55 cargo run -p ph2d-host-desktop --release   # a forma da onda
env PH2D_GPU_COOK_DEMO=56 cargo run -p ph2d-host-desktop --release   # a coluna nomeada
```

⚠️ **Toda cena imprime as bandas nomeadas — se a lista não aparecer, PARE.**
As `=50`/`=52`/`=53`/`=54` exigem **PLAY** (as três famílias são `Effect::Temporal`, e uma
foto de um instante não distingue *segurou* de *ainda não caiu*); as `=48`/`=51`/`=55`/`=56`
julgam-se **PARADAS**.

As cenas `=41..=48` **têm de continuar iguais**.

## §9 — Aberto, com o preço ao lado

- ⚠️ **Um gate `#[ignore]` NOVO, com o número e o mecanismo escritos:**
  `the_ceiling_is_honoured_on_every_tick_including_the_turn` (cena `=53`) — o teto vale **ao
  dígito na rampa** (`0,0800`) e sobe a **`0,1678` no tique 70**, a inversão do vaivém. A lei do
  kernel não pode produzir isso (ela clampa `|out − prev|` por construção, e cinco gates de
  unidade sangram sob mutação) ⇒ a diferença mora **entre o kernel e o que a cena monta**, com o
  `prev_out` do gather como candidato nomeado. **Não afrouxe a barra** — o precedente é o par
  `watercolor_app_params_incremental` do Painter.
- **A composição sub-passos × `damping`** (§6) — ⛔ nomeada e **não curada**, com o ⛔ no
  doc-comment do param.
- A folha 03 tem **6 P1**; a folha 07 tem **3**.
- ⚠️ **A primeira coisa de toda wave desta conferência é MEDIR se a composição já exprime o item** —
  **seis** células envelheceram antes de alguém voltar a elas nesta jornada (o `max_force`, o
  *wander*, a idade normalizada, o `motion.lag` inteiro, a posição como variável do
  `motion.expression`, e a colisão da corda). *O que se perde ao não reconferir não é tempo, é
  construir o que já existe.*

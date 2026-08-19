# HANDOFF — troca de agente na `line/physics` · missão NOVA: **o PLAYER DE PLATAFORMA**

**Status:** INDETERMINADO — ver o corpo (diz "NÃO integrado") · no `main` desde 2026-08-03 (`c250da781`).

> Escrito em 2026-08-03 pelo agente que fechou as seis waves da jornada (HEAD `6b80645ac`).
> O destinatário é o **próximo agente da MESMA linha**, em janela de contexto nova.
> Rota: [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).

---

## §0 — O bloco de abertura (execute ANTES de ler qualquer código)

```
═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha:     line/physics
Sua branch:    line/physics
Sua worktree:  Worktrees/line-physics/   (ela JÁ EXISTE — não crie)

⛔ Você está começando na RAIZ do repo, que está em `main`. Os MESMOS
   paths relativos existem aqui e na sua worktree: abrir `crates/...`
   daqui edita a ÁRVORE ERRADA, e isso compila e commita sem um único
   erro. Ninguém descobre até a integração.

FASE 0 — ONDE VOCÊ ESTÁ (execute já, sem pedir confirmação):
1. cd Worktrees/line-physics && pwd && git branch --show-current
      → pwd TEM de terminar em /Worktrees/line-physics
      → a branch TEM de ser line/physics
2. git log --oneline -5 && git status -sb

FASE 1 — RETOMADA:
3. git rebase main        (obrigatório no início de CADA jornada)
4. cargo check -p ph2d-physics-ecs

FASE 2 — ESTADO (leia, nesta ordem, DENTRO da worktree):
5. ESTE arquivo, inteiro.
6. docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md — inteira, e
   RELEIA a cada passo, como ela manda.
7. As REGRAS PERMANENTES DA SESSÃO (A–H) do
   docs/IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md.
═══════════════════════════════════════════════════════════════════
```

⚠️ **A cwd do Bash VOLTA para a árvore primária entre chamadas.** Prefixe **todo** comando
com `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-physics &&`. Não é zelo: é o
modo de falha documentado desta bancada, e ele já mandou trabalho meu para o `main` nesta
mesma linha.

---

## §1 — O que você está assumindo

**10 commits não integrados**, seis waves da jornada de 03/08 (`d32efe289..6b80645ac`).
A linha está **FECHADA e aguardando ordem de integração do Enio**. O handoff de integração
é [`HANDOFF_INTEGRACAO_line_physics_2026-08-03.md`](HANDOFF_INTEGRACAO_line_physics_2026-08-03.md)
e **cobre as seis** — se a sua missão landar waves novas antes de a integração acontecer,
**acrescente uma seção àquele arquivo**, não escreva um segundo.

⛔ **Você NÃO integra e NÃO pusha.** Fecha, escreve o handoff, e PARA (CLAUDE.md §0.7).

**Estado verificado no fechamento:** `nextest-impacted` 7541/7541 · shell release 1781
passed · clippy limpo · contratos congelados 4/4 + 3/3 · `physics_ecs_c9`
`8c7ba62442f1d577…`, 101 corpos, debug ≡ release.

**O tracker do módulo** é [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) (~9200 linhas).
**Não o leia inteiro** — ele é índice: vá pela tabela do topo e pelo `§<nome-da-wave>` que
te interessa. O mapa de waves é [`00_plano_waves.md`](../00_plano_waves.md) (80 linhas de wave).

---

## §2 — A MISSÃO (palavras do Enio, 2026-08-03)

> *"Vamos fazer uma pesquisa profunda sobre mais uma feature importante da física: **o player
> Plataforma**. Vamos criar uma **seção no inspector para Comportamentos** e o primeiro
> comportamento será o de Plataforma. Como temos uma engine moderna com uma física bastante
> funcional, vamos tentar criar **a partir de um Dynamic e não de um Kinematic**. Sei que em
> Rust há gente trabalhando nisso: um player de plataforma com **perfeita compatibilidade com
> a física, podendo interagir com todo o sistema, inclusive Joints**, e manter comportamento
> coerente e correto. É isso que vamos buscar. **Todas as features que fazem dos games de
> plataforma tão precisos, mas sem perder a interação com os objetos físicos.**"*

**Três fases, nesta ordem, com o Enio no laço entre elas:**

| # | fase | entrega |
|---|---|---|
| 1 | **Pesquisa profunda** | `docs/Physics/05_pesquisa_player_plataforma.md` + síntese reportada |
| 2 | **Planejamento muito detalhado** | `docs/Physics/06_plano_player_plataforma.md` + reporte |
| 3 | **Implementação** | waves, gates, mutações, cena de smoke com números MEDIDOS |

Os números `05`/`06` estão livres (há dois `03_` no diretório, de propósito; não renumere).

⚠️ **Reportar entre fases não é pedir permissão** — a regra permanente é *decida, não
pergunte* ([[feedback_decide_dont_ask_gold_standard]]). O Enio estagiou isto explicitamente,
então a fronteira de fase é um **checkpoint real**: entregue a fase, reporte o veredito com
os números, e siga quando ele mandar.

---

## §3 — A matéria-prima que JÁ EXISTE (não reconstrua nada disto)

Esta linha tem **80 waves**. Quase tudo que um platformer precisa **já está construído** —
a missão é o CONTROLADOR, não o mundo em volta dele.

| o que o platformer precisa | já existe? | onde |
|---|---|---|
| **cápsula** (o collider de personagem) | ✅ `=13` | `world/shape.rs`, `ColliderShape::Capsule` |
| **travar rotação** (não tombar na rampa) | ✅ `=16` | marcador `LockRotation` |
| **offset do collider** (foot-box) | ✅ `=17` | `Collider.offset`, `scale.rs` |
| **plataforma one-way** (jump-through) | ✅ `=23` | `world/oneway.rs` — hook `MODIFY_SOLVER_CONTACTS`, direção derivada do frame do collider |
| **sensores / triggers** | ✅ `=10`, `=71` | `world/sensors.rs`, `bridge/triggers.rs` — **e o sensor por PEÇA** (o `isGrounded` clássico) |
| **eventos de contato Begin/End** + **pico de impacto** | ✅ `=29`, `=30`, `=31` | `world/contacts.rs`, `bridge/contacts.rs` — união dos sub-passos, não fim-de-passo |
| **sinal** (colisão dispara evento desacoplado) | ✅ `=73` + W-SignalLeave | `bridge/signals.rs`, `SignalOnHit`/`SignalOnLeave` |
| **camadas de colisão** (matriz global) | ✅ `=5` | `world/layers.rs` |
| **gravity scale por corpo** | ✅ `=12` | `GravityScale` |
| **damping por corpo** (Combine/Replace) | ✅ `=22` | `DampingOverride` |
| **massa manual / dominance** | ✅ `=19`, `=20` | `MassOverride`, `Dominance` |
| **material combine** (Bounce/Friction Max…) | ✅ `=21` | `MaterialCombine` |
| **CCD** (não atravessar parede fina) | ✅ `=15` | marcador `Ccd` |
| **plataforma MÓVEL** (kinematic dirigido por curva) | ✅ | `world/kinematic.rs` + `bridge/kinematic.rs` — ⚠️ a mira é do fim do TICK e o `step` a **fatia** entre sub-passos |
| **zonas** (vento, água, empuxo, arrasto, torque, falloff) | ✅ `=24`..`=27`, `=32`..`=36` | `world/effector.rs`, `buoyancy.rs`, `drag.rs`, `form_drag.rs` |
| **joints** (9 tipos, incl. Custom por eixo) | ✅ | `world/joints.rs`, `joint_custom.rs` |
| **corpo composto** (várias formas num corpo) | ✅ `=69` | `world/parts.rs` |
| **a MÃO** (pegar um Dynamic no play por MOLA, nunca teleporte) | ✅ `=52`, `=53` | `world/grab.rs` + `bridge/grab.rs` — **leia isto primeiro** |
| **IK / FK** (a corda arrastada, a árvore de pose) | ✅ `=74` | `world/ik.rs`, `bridge/fk.rs` |
| **bake para a timeline** | ✅ `=7` | `bake.rs` |
| **params de joint keyframáveis** | ✅ `=78` | `ph2d-timeline` feature `physics` |

⚠️ **`world/grab.rs` é o precedente mais próximo da sua missão** e a leitura obrigatória
antes de qualquer desenho: ele é *"como mover um corpo Dynamic com precisão sem quebrar a
física"*, já resolvido uma vez nesta linha, com os três modos de segurar e as medições ao
lado. Um controlador de player é a mesma pergunta com mais features.

---

## §4 — As TRÊS perguntas que decidem o desenho

Leia estas antes de abrir o navegador. Elas são as que, se forem descobertas depois da
implementação, custam a wave inteira.

### 4.1 — Um corpo Dynamic é impreciso por natureza. O que o torna preciso?

Todo engine ship um character controller **Kinematic** por um motivo: um Dynamic é
resolvido por um solver de contatos que o artista não controla. A literatura tem **três
famílias** de resposta, e a pesquisa tem de escolher uma **com medição**, não com gosto:

- **(a) escrever a velocidade** do corpo Dynamic a cada tick (simples; perde a troca de
  momento fiel — o corpo empurra o mundo mas o mundo mal o empurra);
- **(b) força/impulso com controlador PD** — a *cápsula flutuante*: o corpo não encosta no
  chão, ele paira sobre um raio com mola-amortecedor, e a mola **é** a perna. Resolve
  degrau, rampa e plataforma móvel **sem caso especial**, e o corpo continua sendo um corpo;
- **(c) híbrido** — resolução kinematic-like do movimento próprio + corpo Dynamic para a
  reação do mundo.

⚠️ **O critério de escolha é o pedido do Enio**, não a elegância: *"perfeita compatibilidade
com a física, inclusive Joints"*. Um personagem **pendurado numa corda** só faz sentido em
(b) ou (c) — em (a) o joint puxa e o controlador sobrescreve a velocidade no tick seguinte,
e o cabo vira decoração. **Meça isso**: a cena de smoke tem de conter um personagem
pendurado, um sobre gangorra, e um empurrando caixote.

### 4.2 — ⚠️ O estado do comportamento é POR-TICK, e esta linha tem SCRUB

**Esta é a que pode custar a wave.** A arquitetura do módulo (ADR-0131) é:

- o mundo rapier **não é persistido** — ele é **derivado** dos componentes a cada frame;
- `rewind_to` **reconstrói do `BodyDesc` e re-simula** (rapier não rebobina);
- o W1.5 pôs um **ring de checkpoints** clonando 8 tipos cross-frame do rapier, com
  `STRIDE = 10` **medido** (um checkpoint custa ~um step).

Um controlador de plataforma carrega **estado interno entre ticks**: coyote timer, jump
buffer, `is_grounded`, jump-held, dash-cooldown, wall-cling. Esse estado **não está no
rapier** e **não está no ring** ⇒ um scrub o perde, e a re-simulação diverge — **exatamente**
o defeito que a wave da POLIA pagou (*"um rewind replayava SEM AS CORDAS"*, e ficou **calado**
porque `target == 0` replaya zero passos).

As saídas conhecidas (a pesquisa escolhe e o plano justifica): o estado entra no
**checkpoint** · o estado é **derivável** de `(tick, entrada)` e a entrada é gravada · ou o
comportamento **declara-se não-rebobinável** e o scrub o re-baseliza em silêncio, como o
`discard_contact_history` faz. **Nenhuma é grátis — escolha com o preço ao lado.**

### 4.3 — A entrada é do JOGADOR ⇒ a sim deixa de ser `f(tick, repouso)`

O invariante que o módulo inteiro se apoia — *o mundo é função do tick, dado o repouso
autorado (e as curvas)* — foi **restabelecido** pela auditoria do W4b com o `SceneAtTick`,
depois que o `Kinematic` o quebrou. **Um player o quebra outra vez, e por um motivo que
nenhuma curva tem:** teclado não é reproduzível.

Perguntas que o plano TEM de responder, com o mecanismo:

- **o que "scrub" significa** numa cena com player? (gravar a entrada e replayá-la é o que
  todo netcode determinístico faz — e o ring GGPO desta linha já é meio caminho);
- o `physics_ecs_c9` (o hash determinístico cross-OS de 3 SOs) **inclui** o comportamento?
  Se sim, a entrada tem de entrar no hash de forma reproduzível; se não, diga por quê;
- o **bake** de um player faz sentido? (assar transforma sim em animação — e o `Kinematic`
  do bake **não é movido por joint**, o que a W-BakeJoint já mediu como contradição).

---

## §5 — A pesquisa: o que procurar

**Pistas a CONFERIR, não fatos a repetir.** Meu conhecimento tem corte e os projetos
abaixo se movem; trate cada um como *"existe? o que ele resolve? como? qual o preço?"*.

**Prior art em Rust (o que o Enio quis dizer com "há gente trabalhando nisso"):**

- **`bevy_tnua`** — o mais direto ao alvo: um controlador de personagem para **corpo
  dinâmico**, com backend rapier, explicitamente desenhado para não brigar com a física.
  Confira a arquitetura dele (o "basis"/"action" e como ele trata o chão).
- **`bevy_mod_wanderlust`** — controlador **cápsula flutuante** (mola + raio).
- **`avian`** (ex-`bevy_xpbd`) — exemplo de character controller dinâmico; a comparação
  interessa mesmo com solver diferente do nosso.
- **`KinematicCharacterController` do próprio rapier** — é o que **não** vamos usar, mas a
  lógica dele (slope máximo, auto-step, snap-to-ground, `apply_impulses_to_dynamic_bodies`)
  é o **catálogo do que um personagem precisa resolver**. Nós já dependemos de `rapier2d
  0.28` — leia o código dele na fonte, não a documentação de terceiro.

**Prior art de "feel" (o que faz o platformer ser preciso):**

- **Celeste / TowerFall** (posts de Maddy Thorson) — a referência canônica de precisão;
  ⚠️ e é um controlador **inteiro-em-pixels, sem física**, o que é exatamente a tensão que
  o Enio pediu para resolver. Nomeie o que dele é portável e o que não é.
- a técnica da **cápsula flutuante** (a palestra do *Very Very Valet*, Toyful Games) — é a
  base de (b) acima.
- tratamento de **rampa** em Sonic/Mario; **corner correction** / *ledge assist*.

**O catálogo de features (a pesquisa deve fechar a lista, com prioridade e preço):**

coyote time · jump buffer · altura de pulo variável (soltar o botão corta) · *apex hang*
(gravidade menor no topo) · fast fall · aceleração/atrito de chão separados dos do ar ·
controle aéreo · **corner correction** (bater a quina da cabeça e escorregar em vez de
parar) · limite de rampa + escorregar acima dele · **auto-step** (subir degrau sem pular) ·
**snap-to-ground** (não decolar na quebra de rampa descendo) · carona em plataforma móvel ·
descer de plataforma one-way (a nossa `oneway.rs` já é o mecanismo) · wall slide / wall jump ·
dash · agachar · empurrar caixote · ser empurrado.

**E a lista de INTERAÇÃO, que é o diferencial pedido:** o personagem **pendurado num joint**
· **sobre uma gangorra** (a massa dele inclina) · **dentro de uma zona de água** (o empuxo
do `buoyancy.rs` age nele) · **sob vento** (`effector.rs`) · **sobre plataforma kinematic
dirigida pela timeline** · **agarrado pela MÃO** enquanto anda · **atingido por um impacto**
(o `contact_peaks` já mede a força).

---

## §6 — A §14: a seção de **Comportamentos**

**O ponto de fiação é único e já existe:** `crates/ph2d-panel-inspector/src/paint_frame.rs::paint_physics_sections`,
que hoje pinta **§11 Physics Body · §12 Physics Joint · §13 Pulley Wheel**, cada uma com o
seu **slot de nota** (9, 10, 11). Uma quarta entra ali, com slot 12.

⚠️ **O número da seção se CONTA contra o `main` do dia da integração, nunca se escolhe** —
como `PROJECT_SCHEMA`, ids de gizmo e números de ADR. Outra linha pode ter acrescentado uma
seção na mesma janela ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

**A tensão de desenho a resolver no plano:** o Enio pediu uma seção de *Comportamentos* cujo
**primeiro** comportamento é Plataforma. Um seletor com uma opção só é um **controle morto**
(a lei que esta linha aplica há 80 waves). O desenho tem de servir os dois: **entregar
Plataforma inteiro**, e deixar o ponto de extensão **nomeado e barato** — sem shipar um
framework vazio esperando um segundo inquilino que talvez não venha.

**As quatro condições de fechamento de UI** (política do módulo desde a W-ContactEvents,
plano 02): o componente **existe** (gate `every_physics_component_is_authorable`) · é
**pintado e registrado** (`architecture_panel_wiring_parity`) · o **clique chega ao
barramento** (varredura de seam com `click_real`, não `WidgetEvent` sintético) · e a
**SEQUÊNCIA leva a algum lugar** (`inspector_physics_gesture_tests`). A quarta é categoria
NOVA e **não é implicada pelas outras três**.

⚠️ Widgets registrados em **LAÇO** são ponto cego do `architecture_panel_wiring_parity` (ele
coleta `.register(ids::LITERAL` direto) — o buraco das 36 células do W2c. Se a seção pintar
uma lista, o seam é a única cobertura.

**Norte arquitetural:** feature nova = **drop-crate** (ADR-0075). Um comportamento novo tende
a ser crate própria; o componente ECS vive na `ph2d-physics-ecs` e entra no **registro**
(que está em **27** nesta linha — e **conta** contra o main na integração).

---

## §7 — As leis desta linha que você herda

Ponteiros, não cópias (duas cópias da mesma regra divergem):

- **Determinismo:** nunca ligue `parallel`/`simd-*` no rapier · transcendental por `libm`
  (`std::atan2` **não** é pinado cross-OS) · `BTreeMap`, nunca `HashMap` · toda soma de `f32`
  em ordem fixa · **1 ulp é bug cross-OS**. O `physics_ecs_c9` roda nos 3 SOs do CI e é o
  guardião: se a sua wave o move, **diga por quê** e confira `debug ≡ release`.
- **Componente de física é CONFIG, nunca estado vivo de solver** — o `canonicalize` do undo
  ordena por BYTES do componente, então estado vivo ali faz **cada frame virar um passo de
  undo**. ⚠️ **O estado do seu comportamento (coyote, buffer) é EXATAMENTE o tipo de coisa
  que não pode virar campo de componente autorado** (§4.2).
- **`MEÇA antes de escrever qualquer teto**` (CLAUDE.md §0) — `MAX_*`, faixa de slider, "por
  ora". Escreva o número que a medição deu, com a tabela ao lado.
- **Marcador vs campo:** presença = booleano (idioma de `Ccd`/`LockRotation`) e **não bumpa
  schema**; campo apendado a componente existente é postcard **posicional** ⇒ **bumpa**, e
  um bump **recusa todo projeto já salvo**.
- **LOC:** crates ≤700 · shell ≤600 · arquivos de painel ≤600 · fn de painel ≤200. `fmt`
  **antes** de medir; split por **ASSUNTO** em módulo irmão, nunca allowlist. ⚠️ O
  `shells/desktop/tests/file_loc_caps.rs` e o `arch_safe_clamp_only` **não rodam** num
  `cargo test -p` filtrado — entram no gate de fechamento.
- **Rode a suíte em DEBUG e RELEASE.** Um gate desta família já reprovou só em debug.
- **Git:** nunca `git add -A`/`.`/`stash`/`--force`; `git commit --no-verify`; mensagem com
  crase é substituição de comando (use `git commit -F`). Desfaça mutação com `cp` + `touch`,
  **nunca** `git checkout`.
- **Mutação:** todo gate novo prova RED sobre um VERDE visto. Sobrevivente = **gate
  faltando** ou **oráculo errado** — e já aconteceu de a mutação acusar o meu **doc**, não um
  buraco.
- **Fixture só prova o que contém.** Nesta linha o **controle foi atropelado pelo próprio
  experimento QUATRO vezes**.
- **Cenas de smoke:** a próxima livre é **`PH2D_PHYSICS_SMOKE=80`**. Toda cena **imprime o
  que montou** com números MEDIDOS (`eprintln!` ASCII — `→` dispara o `no_tofu_glyphs`), e o
  prólogo abre a timeline. `--release`, sempre.
- **Toda wave fecha com:** entrada no tracker `HANDOFF_line_physics.md` **e** linha no
  `00_plano_waves.md`, **na mesma sessão**.

---

## §8 — Os números da linha (o que CONTA na integração)

| grandeza | na linha | ⚠️ |
|---|---|---|
| `PROJECT_SCHEMA` | **51** | conta contra o `main` do DIA; **a colisão pode passar MUDA** — se outra linha escrever o mesmo literal, `project.rs` não conflita e um bump evapora com a suíte verde. O sinal é o conflito em `project_schema_tests.rs`. |
| tripla do pin | `(51, 13, 14)` | |
| registro `ph2d-physics-ecs` | **27** | um componente novo o leva a 28 — **conte** |
| ids de gizmo | último **973**, próximo livre **974** | |
| cena de smoke | última **79**, próxima livre **80** | |
| ADR | nenhum novo na jornada | um ADR escolhido numa linha paralela é **PROVISÓRIO** (já renumerou 7 vezes no repo) |
| `physics_ecs_c9` | `8c7ba62442f1d577…`, 101 corpos | debug ≡ release |

---

## §9 — O que sobra do módulo além desta missão

Para você **não** confundir com dívida: o horizonte do plano 02 §8 está **fechado**, e a
lista ordenada §8.1 fechou **6 de 6**. O que resta está registrado no §6 do handoff de
integração e é, em cada caso, **uma destas quatro coisas**:

1. **condicional não satisfeito** — *rows de readout tingidas*, "se o §12 ganhar readouts
   vivos" (o readout de carga vive no OVERLAY, não numa row);
2. **decisão de produto/arquitetura, não engenharia** — o consumidor de gameplay do sinal
   (o `AppGfx.script` é um `Option<ScriptHost>` **nunca tickado**, e a mesma outbox recebe os
   sinais da timeline ⇒ o consumidor é cross-cutting) · **um Ctrl+Z para as duas metades do
   bake** (unir as filas é redesenho do roteador de undo);
3. **mecanismo nomeado com preço** — eixos acoplados no `GenericJoint` · um Custom não pode
   ser elo de árvore IK/FK (`FkDof` modela UM grau de liberdade) · o glifo do Custom não
   desenha direções de eixo linear (`JointView` não carrega o frame do joint);
4. **fora de TODAS as waves (D9)** — soft-body XPBD, fluidos FLIP/PIC, collider-gen vetorial.

⚠️ **O item 2 toca a sua missão de raspão:** um player que dispara um sinal ao encostar
num espinho é o mesmo consumidor ausente. Nomeie a fronteira no plano; **não** a atravesse
sem ordem.

---

## §10 — O primeiro reporte

Depois da FASE 0/1/2, reporte:

> *"Assumi `line/physics` em `Worktrees/line-physics` (HEAD `<sha>`). 10 commits não
> integrados, seis waves, aguardando ordem. Entrando na FASE 1 da missão do player de
> plataforma: pesquisa."*

E **comece pela pesquisa** — não abra código de implementação antes de o `05_pesquisa_*`
existir. A ordem do Enio é explícita, e a razão dela é a mesma de sempre nesta bancada:
**verde-de-compilação é velocidade; no audit vale ZERO.**

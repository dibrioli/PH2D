# 00 · Plano de waves — o motor de física global (`line/physics`)

> Normativo. Companheiro da [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)
> (decide o *quê* e o *porquê*); este plano decide o *como*, wave a wave. Visão:
> [`01_visao.md`](01_visao.md). Estado vivo: [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md).
>
> **Plano VIVO:** waves seguintes o refinam. Cada wave fecha com o **gate batched** (nextest-impacted +
> clippy `--all-targets` + auditoria ≥2 lentes) e um **handoff de tracker**. Sequenciais.
>
> **Regra-mãe (DIRETIVA §3–§5):** verde-de-compilação é velocidade; no audit vale **ZERO**. Todo gate
> nasce **VERMELHO** sobre o bug real, com os **números do PRODUTO**, e morre por uma razão nomeável.
> Toda costura é **exercitada** (que clica, que dá o tick, que olha), não só compilada. Toda defesa em
> camadas ganha **gate POR camada** ([[feedback_layered_defenses_need_per_layer_gates]]).

## Mapa das waves

| Wave | Título | Entrega | Bloqueia |
|---|---|---|---|
| **W1** | Ponte ECS + tick no Playhead + hash no replay gate | o alicerce: sprite cai e assenta, determinístico | tudo |
| **W1.5** | Scrub bit-exato (checkpoint ring) | scrub pra trás sem re-sim O(t) | — (opcional; pode ir depois de W2) |
| **W2a** | Inspector body | a autoria do artista | joints, bake |
| **W2b** | Painel global de mundo | gravidade/solver/arrasto/sono | — |
| **W2c** | Camadas de colisão | a matriz + a camada por-corpo | — |  ✅
| **W3** | Joints | pino/mola/motor/distância; pêndulo, corrente, ragdoll | bake de joints |  ✅
| **W4** | Bake-to-timeline | runtime-truth vira animação editável | — |  ✅
| **W5** | Corpos FILHOS na hierarquia | o collider volta para debaixo do sprite | — |  ✅

### ⚠️ O mapa acima FECHOU — as waves seguintes vivem no tracker

**W1..W5 estão todas ✅, e o módulo continuou muito além delas.** As waves posteriores nasceram do uso (do
smoke do Enio, de uma falta encontrada, de um bug medido) em vez de um plano escrito de antemão, e por isso
são **normativas no [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md)**, cada uma com sua própria seção:

| Wave | Entrega | Smoke |
|---|---|---|
| **W6** | a escala do `Transform` alcança o collider (ball não-uniforme → elipse) | gates |
| **W7** | sensores / triggers | `=10` |
| **Weld** | o 5º joint (`FixedJoint`) | `=11` |
| **BakeChannels** | assar um subconjunto dos canais | — |
| **W8** | gravity scale por corpo | `=12` |
| **Capsule** | o collider de personagem | `=13` |
| **W9** | velocidade inicial por corpo | `=14` |
| **W-CCD** | detecção contínua por corpo | `=15` |
| **W-LockRot** | freeze rotation | `=16` |
| **W-Offset** | offset do collider | `=17` |
| **W-LockPos** | freeze position X/Y | `=18` |
| **W-Mass** | massa manual | `=19` |
| **W-Dominance** | prioridade de colisão | `=20` |
| **W-Material** | regras de combine (bounce/friction) | `=21` |
| **W-Damping** | drag por corpo + modo Combine/Replace | `=22` |
| **W-OneWay** | plataforma jump-through | `=23` |
| **W-Area** | campo de força (área que empurra) | `=24` |
| **W-Contacts** | quem toca quem, onde, sob que carga | `=25` |
| **W-AreaDrag** | a área RESISTE (vento vs água) | `=26` |
| **W-Buoyancy** | Arquimedes: a área sabe quanto do corpo está dentro | `=27` |
| **W-FormDrag** | o arrasto que sabe para onde o corpo aponta | `=28` |
| **W-ContactEvents** | *começou a tocar* / *parou de tocar* — e um scrub não é colisão | `=29` |
| **W-ImpactForce** | *quão forte foi o toque* — o pico entre os sub-passos, não a carga | `=30` |
| **W-TickContacts** | o toque RÁPIDO vira evento — diff por TICK sobre a união dos sub-passos | `=31` |
| **W-AreaTorque** | a MESA GIRATÓRIA — uma área que GIRA o que está dentro (a metade rotacional do campo de força) | `=32` |
| **W-AreaFrame** | o FRAME da zona — girar o sensor gira o vento (toggle `Force Axes: Zone \| World`) | `=34` |
| **W-AreaFalloff** | o FALLOFF da zona — a força e o torque desvanecem do centro até ZERO na borda; a régua é a silhueta da própria zona | `=35` |
| **W-AreaMirror** | o ESPELHO da zona — virar o sprite vira a correia; a força REFLETE (vetor) e o torque INVERTE (pseudoescalar) | `=36` |
| **W-BakeRange** | o INÍCIO do loop é honrado — um loop `[2s, 5s]` assa exatamente `[2s, 5s]`, simulando o front e descartando-o (a metade do bake que estava aberta desde o W4) | `=37` |
| **W-JointAnchor** | a âncora de um joint ganha um DOT âmbar agarrável no canvas (handle de PONTO — os 3 publicadores de `GizmoView` são caixas), arrasta = `Translate` da seleção | `=38` |
| **W-BakeJoint** | assar UM corpo de um rig articulado puxa o componente conexo DINÂMICO inteiro (`jointed_group`) — não há bake parcial coerente de um rig acoplado | `=39` |
| **W-JointAuthoring** | §12 redesenhada: linha por corpo (Body A/B + nome vigente + eyedropper que ARMA um canvas-pick) + smoke de autoria do zero; a criação já existia desde o W3, faltava descobribilidade | `=40` |
| **W-AnchorFollow** (padrão-ouro W1) | a âncora vira **body-local por corpo** (`PhysicsJoint.local_a/b`, rep nativa do rapier) e SEGUE o corpo — mover um corpo não desliza mais o pino (`PROJECT_SCHEMA` 30). A coluna do padrão-ouro; as ex-"waves 2-5" foram ABSORVIDAS pelo plano 02 (ver linha abaixo) | `=41` |
| **W-JointParams** (P0 — correção) | tunar um parâmetro de joint AO VIVO. Report do Enio (*"os parâmetros de Spring não mudam em nada"* + *"Rope a mesma coisa, inconsistente, às vezes funciona"*). **DUAS causas:** (1) a PONTE gateava o re-describe em `at_rest` — gate do W3 (proteger a âncora mid-swing) que o W-AnchorFollow tornou obsoleto (âncora agora body-local, semeada do REPOUSO); fix de 1 linha, cobre todos os params. (2) a COSTURA da UI (§12) enfileirava o `SetComponent` sem dar FLUSH — "às vezes funciona" = só landava quando outro edit drenava a fila; fix = flush por-edit no loop de joint. c9 byte-idêntico. **Smoke OK 2026-07-25** | `=42` |
| **W-J1** (plano 02) | **o joint DESENHA o que ele é.** Havia UMA figura para os 4 tipos; agora glifo por kind (anel · quadrado · zigue-zague · fio), linha de posse **A sólida / B tracejada** (a paleta está cheia ⇒ a diferença é de FORMA), arco de limite com paredes + agulha VIVA, glifo de motor (o mesmo da zona de torque — mesma pergunta), anel de comprimento em MUNDO (dá zoom, cresce) e o vermelho de *restrição não imposta*. Porta única: o desenho lê o `JointView` da ponte (o `desc` que o solver recebeu), nunca o componente. ⚠️ **Medido:** joint do rapier é RÍGIDO — 500× de massa e martelo de 400× abrem **0,00000 m**; quem abre o vão é o corpo **KINEMATIC** curva-dirigido (**1,50 m**), o estado de um rig ASSADO. c9 byte-idêntico | `=43` |
| **W-J2** (plano 02) | **a âncora tem DUAS alças, e um ímã.** Só a ponta A era autorável; a de body B era o que a política de semeadura produzisse (mesmo ponto num Pin/Weld, centro do corpo numa Spring/Rope) e **nenhum gesto do editor a movia**. Agora: 2ª alça (`GIZMO_JOINT_ANCHOR_B`, id 965) desenhada em **anel vazado no MESMO âmbar** — a gramática *sólido = A, vazado = B* das linhas de posse da W-J1; par COINCIDENTE (o caso normal de um Pin) fica concêntrico, com A no quadrado interno e B na faixa de fora; **snap por CTRL** aos 9 pontos do collider (centro/quinas/meios), 14 px, os MESMOS que a alça de pivô já oferece, com CRUZ marcando o capturado. Porta única `bridge/anchors.rs` (`joint_anchor_world` / `set_joint_anchor_world` / `joint_snap_targets`) — o `sync_joint_pivots` passa a ler dela. ⚠️ **O `anchored` MORREU como mecanismo de reposição:** ele é do JOINT INTEIRO, então arrastar A re-derivava B da política e jogaria fora a âncora recém-posta no outro corpo, **em silêncio**; um reposicionamento conhece o lado e escreve o local direto (o sentinela sobrevive só onde re-derivar AMBOS é a intenção: create, troca de kind, re-pick). ⚠️ **As alças agora são REST-ONLY** — o doc do `sync_joint_pivots` já afirmava isso desde a W-AnchorFollow e era **falso**. c9 byte-idêntico | `=44` |
| **W-J2b** (plano 02) | **as alças ficam MAIORES, aparecem sozinhas e ganham o pixel.** Os três pedidos do smoke da W-J2 são a MESMA coisa: **uma joint não tem sprite**, então o `pick_sprites_at_world` não a alcança e a SELEÇÃO era o único jeito de trazer as alças à tela — a rota até uma alça de canvas passava pela **Hierarquia**. Agora `PointGizmoView` carrega uma **LISTA** e toda joint em repouso publica as suas (mesma porta `joint_anchor_world`); vários registram a mesma alça ⇒ o id é **keyed por bits** (`point_handle_id` + `point_hit_map`, o padrão do `keyed_handle_id` dos extras — multiplicadores ímpares e distintos por lado, porque um scrambler LINEAR cancela na comparação e faz ids consecutivos colidirem); **pegar a alça SELECIONA a joint** (a §12 abre no que você pegou); disco 6→9 px e anel 10→15 **com os hit rects seguindo o VISUAL** (marca maior que o retângulo = clique que não faz nada, o modo de falha exato de "deixe maior"); e o z é **ordem de registro** — as alças pintam por ÚLTIMO entre os gizmos, então a âncora sobre a quina de uma sprite é pega como âncora. `joint_entities()` publica o `joints_seen` do reconcile (mais largo que `self.joints`: a joint **dormente** é vista sem ser construída e a ponta A dela segue autorável). ⚠️ **1 mutação sobreviveu e o defeito era do GATE** (pinava uma grafia da seleção, e o bloco a menciona legitimamente porque a ESCREVE) ⇒ o gate passou a afirmar a **lista de argumentos do `open_drag`**. c9 byte-idêntico | `=44` |
| **W-J3** (plano 02) | **pose, não digite.** O canvas MOSTRAVA o alcance de uma dobradiça e o comprimento de uma mola, e mudá-los era voltar ao §12 e digitar — olhar o efeito num lugar e escrever a causa noutro. Agora as **duas paredes do arco** e o **anel de comprimento** têm grip. ⚠️ **Estas duas e não o motor, e a razão é o que uma grandeza É:** um limite é um ÂNGULO e um comprimento é uma DISTÂNCIA — cada um já tem lugar, e arrastar até ele não converte nada; velocidade é uma **TAXA**, nenhum lugar da tela é 120 °/s, e a row do §12 é `num_row` livre **sem faixa** de onde tirar a constante px-por-°/s (as duas leis sem constante falham sozinhas: o arco SATURA em 270°, e uma volta = 360 °/s **DÁ A VOLTA**) ⇒ nomeado, não construído. Quatro espinhas: a geometria que se ARRASTA é a que se DESENHA (`limit_end_screen` é a função que o `limit_arc` usa para a marca radial — a que discordasse seria a INVISÍVEL, o hit rect) · o arrasto escreve pelo MESMO funil do número (`joint_with_edit`, a metade pura do `apply_joint_edit`, agora com dois consumidores) · ⚠️ **uma parede PARA na irmã** (`clamped()` TROCA limites invertidos — certo para quem digita, errado para um gesto: a troca entrega a OUTRA parede à mão no meio do arrasto) · e o **FANTASMA** de B na pose que a parede permite (o *L* do RUBE **sem modo**), que **desenha e nada mais** — o ângulo vem do COMPONENTE, já passado pelo muro, senão seria promessa que o solver quebra. c9 byte-idêntico | `=45` |
| **W-J4** (plano 02) | **criar onde se olha.** Um joint nascia de uma SELEÇÃO, e o preço só aparecia no gesto seguinte: **as âncoras nascem onde a política de semeadura decide**, nunca onde o artista apontava — amarrar uma corda na PONTA de uma prancha era criar, selecionar a joint e arrastar o dot. Agora **aperte o corpo A, arraste, solte no corpo B** e as âncoras nascem NOS dois pontos, com uma corda/mola ganhando de brinde o **comprimento que o arrasto mediu** (medido: a mesma prancha assenta em **rot 104,2°** pendurada pela ponta contra **0,0°** nivelada pela rota do botão — a diferença entre as duas rotas num número). ⚠️ **Uma porta com os pontos OPCIONAIS** (`create_joint_at(.., Option<(wa, wb)>)`; o `None` é a rota antiga byte-idêntica, e o `Some` marca `anchored` — sem isso o reconcile faria o seed e jogaria os dois pontos no lixo com o joint parecendo funcionar); kind que compartilha ponto usa a PRESSÃO nas duas pontas. **E a rota por seleção sobrevive porque ela virou a CORRENTE:** 3+ corpos marcados ⇒ **N−1 joints em UM passo de undo** e o botão passa a CONTAR (`Chain 4 Selected Bodies`) — ⚠️ `join_count: u8` substituiu o `can_join: bool`, porque um bool ao lado de uma contagem discordou dela no dia em que a corrente chegou. Banda âmbar TRACEJADA durante o arrasto, desenhada **FORA do gate `show`** do overlay (contorno é preferência de vista; um gesto em andamento não pode ser invisível por causa dela); release no vazio ou no MESMO corpo = toast e o gesto **segue armado**. ⚠️ **A M1 SOBREVIVEU primeiro e nomeou o buraco:** todos os gates chamavam a porta de criação DIRETO, então descartar os dois pontos no *release* deixava 8 verdes. ⚠️ **E o split de LOC expôs um gate por PROXY:** o `architecture_panel_wiring_parity` enumerava o NOME `populate.rs` ⇒ um code move puro o deixou VERMELHO acusando *"dead on click"* — passou a casar `populate*.rs` por PREFIXO, com mutação provando que ainda sangra. c9 byte-idêntico | `=46` |
| **PLANEJADAS — [`02_plano_joints_ui_authoring.md`](02_plano_joints_ui_authoring.md)** (2026-07-25, pós-pesquisa Unity/Unreal/Godot/Fyrox/RUBE/Algodoo/Newton + rapier source; 44 screenshots em `~/Documentos/Recursos/UI_Reference/`) | **W-J1** o joint se DESENHA (glifo/posse/limites/rest/violação) · **W-J2** duas alças + snap · **W-J3** pose-não-digite (arco/anéis/seta arrastáveis) · **W-J4** criar onde se olha (press-A-drag-B; corrente por seleção ordenada) · **W-J5** Slider/prismatic · **W-J6** servo + guincho · **W-J7** break force · **W-J8** Active/Collide/Swap/nome "A : B" · **W-JG** grupo carrega o rig. Absorve as ex-"waves 2-5" do padrão-ouro | — |

⚠️ **Esta tabela estava faltando até 2026-07-21**, e um plano *normativo* que não menciona metade do módulo é
pior que um plano velho: ele faz a próxima LLM concluir que a linha parou no W5. A regra: **wave nova fora do
mapa entra AQUI na mesma sessão**, com uma linha; o detalhe fica no tracker.

## ⚠️ Toda wave chega à UI — a política, não a boa intenção

Pergunta do Enio no fim da jornada de 2026-07-21 (*"tudo isso está exposto na UI e é possível criar essas cenas
todas usando apenas os parâmetros expostos?"*). A resposta foi **sim**, mas a pergunta expôs que isso era um
hábito e não uma regra. Agora é regra, e vale para **toda wave futura desta linha**.

### O que "chegar à UI" significa, em quatro condições

Uma wave só fecha quando o que ela construiu é alcançável por um artista **sem escrever código**:

1. **Existe** — todo componente registrado tem um caminho de escrita a partir do Inspector.
   Gate: `shells/desktop/tests/every_physics_component_is_authorable.rs` (estrutural, sobre o fonte).
2. **É pintado e registrado** — o controle aparece e é focável.
   Gate: `architecture_panel_wiring_parity`.
3. **O clique chega ao barramento** — cada row/chip despachado, com a recusa no `event`, nunca no laço de pintura
   (*dim não é recusa*). Gate: a **varredura** de seam do painel, que clica **todos** os controles da seção.
4. **A SEQUÊNCIA leva a algum lugar** — o gesto composto produz uma coisa que funciona.
   Gate: `inspector_physics_gesture_tests`.

⚠️ **A (4) é a categoria que esta jornada descobriu, e ela não é implicada pelas outras três.** Todo edit pode ter
gate e o gesto ainda não levar a lugar nenhum: uma row que só aparece depois de outra, um default que atrapalha,
um passo que exige um número que o artista não tem como saber. Foi ela que pegou o passo *"converta para
Capsule"* que eu quase ensinei ao Enio — geometricamente correto, e destrói o tronco.

### A metade VISÍVEL conta como UI

Um controle autorável cujo efeito é invisível está meio construído. A precedência é do **W7** (*um sensor com
nada lendo suas sobreposições é um flag morto — torne-o VISÍVEL primeiro*) e ela se repetiu quatro vezes:

- força de área → **seta laranja** (*para que lado sopra?* não é inferível);
- contatos → **cruz branca**, do tamanho da carga;
- empuxo → **linha d'água** (o único número que o modelo calculava e a tela escondia — achado pelo Enio);
- arrasto → **nada, e é decisão**: um arrasto não tem direção para desenhar, ele se vê nos corpos desacelerando.

A pergunta a fazer no fim de cada wave é *"o que esta wave calcula que a tela não mostra?"* — e a resposta
**pode** ser "nada, de propósito", desde que seja escrita.

### E toda wave ganha uma CENA

`PH2D_PHYSICS_SMOKE=<n>`, com os números **medidos** (a sonda headless roda a cena e reporta; a mensagem
`eprintln!` cita os valores). Uma wave gateada e não-smokável é meia wave — foi o estado do W-FormDrag por uma
hora, e a cena `=28` nasceu para fechar isso.

⚠️ **A cena é uma FIXTURE e adoece como fixture.** Nesta jornada: dois controles foram *atropelados pelo próprio
experimento* (W-Area, W-Buoyancy), um V nasceu de cabeça para baixo (W-Contacts), o `=28` nasceu contaminado
**duas vezes** por geometria que eu não controlava, e uma mensagem afirmava *"fica a meia-água"* sobre uma caixa
que a medição mostrou ir ao **fundo**. **Rode a sonda antes de escrever a mensagem.**

### O que fica FORA da UI, e por quê

Um número que o artista não tem como calibrar não vira knob. Da jornada: o `EDGE_SAMPLES` do arrasto de forma,
o `LOAD_FULL_NS` da cruz de contato, o `ALLOWED_ANGLE` do one-way, o `STRIDE` do ring — todos são **régua de
implementação**, medidos e documentados no código, não superfície. A pergunta é *"o artista sabe o que este
número significa na arte dele?"*.

---

**Fora de TODAS as waves (D9):** soft-body XPBD (`ph2d-physics-soft`, M13+), fluidos FLIP/PIC
(`ph2d-fluids`, M13+), collider-gen vetorial + fratura (ADR-0063, aposentada com a 0108).

---

## W1 — Ponte ECS + tick no Playhead + hash no replay gate · *o alicerce*

**Objetivo:** um sprite com `RigidBody{Dynamic}` cai e assenta sobre um `Collider{Static}` no **ECS
REAL** ao dar play — e o mundo é **determinístico cross-OS**.

### Entregáveis
- **Crate-ponte nova `ph2d-physics-ecs`** (ou módulo no editor-core; a crate isola melhor — regra B'):
  - Components `RigidBody`/`Collider` + enums `BodyKind`/`ColliderShape` (append-only, defaults
    byte-neutros — ADR D3).
  - `register_physics_components(reg: &mut ComponentRegistry)` — a crate possui, o boot agrega em
    `shells/desktop/src/init.rs` ao lado de `register_render_components` (mantém a contagem-32 de
    `ph2d-ecs` intocada). **Registro no MESMO commit que cria os components.**
  - **Sem porta de escala:** o `Transform` já é METROS = rapier metros (1:1, sem conversão nem sinal
    trocado — os dois são Y-up + radianos CCW). A única conversão px→m é a que JÁ existe,
    `ProjectSettings.pixels_per_meter` no import — do projeto, não da física (ADR D4 corrigido no W1).
- **System de sync (o hot path `physics_step`):** components → `PhysicsWorld` (spawn/update do
  handle-map `Entity ↔ RigidBodyHandle`) → `step()` no tick do `Playhead`
  (`ticks_owed(last_stepped, target)`: play = `last+1..=target` sequencial, scrub/paused =
  `target..=target`; `target = round(playhead.time()/fixed_dt)`) → **readback** dos transforms para o
  `SimWorld`. O `PhysicsWorld` + handle-map vivem shell-side (precedente `MotionCookPump`), **NÃO** no
  `WorldSnapshot` (o mundo é rebuild das components — ADR D2). Gancho `should_record`/`record` do ring já
  no laço (W1.5 o usa).
- **Persistência mínima:** as components viajam no `WorldSnapshot` (já registradas) → bump
  `PROJECT_SCHEMA` (**15 → 16**, valor real; +a tripla-pin em `project_tests`). O `PhysicsWorld` é
  reconstruído no load (`rebuild()`; reconcile self-heal é o backstop).
- **Gate de determinismo estendido:** um bin/harness gêmeo `physics-ecs-c9` que exercita **a ponte + o
  caminho do tick** (não o wrapper cru): monta uma `SimWorld` com N entidades carregando
  `RigidBody`/`Collider`, roda sync + `ticks_owed` por 120 ticks, imprime `physics-ecs-c9 hash: <hex>`.
  Plugar em `.github/workflows/spike.yml`: etapa de matriz (ubuntu/macos/windows) + artifact
  `physics-ecs-c9-hash-${os}` + comparação `sort -u | wc -l == 1` no job `determinism-compare`.

### Gates (red-first, mutation-tested)
1. **e2e no app REAL** — sprite com `RigidBody{Dynamic}` sobre `Collider{Static}` **cai e assenta no
   chão** dirigindo o `SimWorld` + a ponte + N ticks do `Playhead` (NÃO um unit do wrapper —
   [[feedback_tool_unit_green_integration_dead]]). Nasce vermelho (sem ponte, o sprite não cai). Assenta
   a `y ≈ chão + raio` em pixels (converte via a porta, prova a escala de ponta a ponta).
2. **hash cross-OS estável do mundo ECS-bridged** — `physics-ecs-c9` byte-idêntico nos 3 OSes. **Mutação:
   trocar a ordem de iteração da ponte (map em vez de sorted) sangra** (o hash muda). Prova o código NOSSO
   no caminho determinístico.
3. **zero-alloc no `physics_step`** — dhat **por capacidade**, não contador global
   ([[feedback_zero_alloc_gate_capacity_not_global_counter]]). Mutação: um `Vec::push` que realoca no laço
   sangra.
4. **tick único** — play anda N steps, scrub anda 1. **Gate de emenda com advance FRACIONÁRIO** (taxa 1:1
   nunca lê o 2º frame — [[feedback_seam_gates_need_fractional_advance]]): um `wall_dt` que deve 2 ticks
   tem que simular os 2; um scrub tem que rodar `anchor..=target` uma vez. Mutação: `last+1..=target` →
   `target..=target` (perde ticks no play) sangra; e vice-versa.
5. **snapshot é ponto fixo** — parado (sem input), **nenhum passo de undo espúrio por frame**
   ([[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]]). A captura é DEPOIS de a ponte convergir.
   Mutação: capturar antes do readback sangra (pose muda entre captura e convergência).

### Smoke
`PH2D_PHYSICS_SMOKE=1` — cena **auto-play** que dropa 1 sprite sobre um chão (exemplo pronto pra smoke,
auto-play — [[feedback_ready_to_smoke_example]]). Comando com o `cd <worktree> &&` junto
([[feedback_run_command_include_cd]]).

### Fora de W1
Painel, joints, bake, scrub-back (o ring é W1.5; W1 deixa só o gancho).

---

## W1.5 — Scrub bit-exato (checkpoint ring) · *o relógio pra trás*

**Objetivo:** arrastar o playhead pra trás re-simula bit-exato **sem** custo O(t) (ADR D5). Pode vir
depois de W2 se o Enio priorizar a autoria; é listada aqui por ser a metade que falta do "relógio único".

### Entregáveis
- **`PhysicsCheckpoint`** = estado cross-frame completo do `PhysicsWorld` (os campos que `step()` muta:
  `bodies`/`colliders`/`impulse_joints`/`multibody_joints`/`ccd_solver`/`islands`/`broad_phase`/
  `narrow_phase` + `step_count`).
- **`PhysicsCheckpointRing`** à imagem do `ph2d-eval-motion::CheckpointRing`: `record(tick, cp)`,
  `anchor_at_or_before(target) → (tick, cp)` (newest ≤ target, senão seed do tick-0), `clear` no rebuild.
  **Cadência ESPARSA** (cada K ticks — o estado é maior que outputs de nó), K tunado contra o budget 20 MB.
- **`advance_or_scrub`** no laço do tick: play = `record` esparso + step forward; scrub-back = `restore` da
  âncora + re-sim ≤ K steps até `target`.

### ✅ kill-check RESOLVIDO (2026-07-18) — passou de primeira
Os 8 tipos cross-frame do rapier **são `Clone`** ⇒ sem `serde-serialize`, sem bincode. O `PhysicsPipeline`
não é `Clone` porque é *workspace*, não estado (confirmado pelo gate de bit-exatidão, não por prosa).
**Cadência decidida por MEDIÇÃO** (`measure_checkpoint.rs`): um checkpoint custa ~**um step** (11,2 µs vs
7,3 µs a 50 corpos), então denso dobraria o custo do play e comeria 17,4 dos 20 MB ⇒ **`STRIDE = 10`**
(1,74 MB de janela, pior caso 10 steps). **Cap em BYTES** (8 MB), nunca em contagem — contagem é
multiplicador (ADR-0117). Detalhe e números: tracker §W1.5.

### Gates (red-first, mutation-tested)
1. **scrub-back é bit-exato** — `restore(anchor) + re-sim até T` produz o **mesmo hash** que `re-sim from
   t=0 até T` (a definição de correção do ring). Mutação: `anchor_at_or_before` devolvendo a âncora errada
   (> target) sangra.
2. **memória do ring medida** — dhat/`size_of`, `tests/measure_physics_checkpoint.rs`, dentro dos 20 MB
   (HR-13, quem declara MEDE — [[feedback_a_rule_that_never_observes_cannot_fire]]). Mutação: cadência densa
   (K=1) estoura o teto.
3. **scrub é O(K), não O(t)** — ratio: scrub num t grande custa o mesmo que num t pequeno (bar é RATIO, não
   wall-clock — `ci-test` compila `opt-level=1`).

### Smoke
**`PH2D_PHYSICS_SMOKE=2`** (cena própria, não a 1 — a do W1 foi aprovada pelo Enio e não se mexe no que já
foi validado): 12 corpos caem numa **pilha**, o playhead se arrasta pra trás e pra frente, a pilha
reconstrói bit-exata sem trava. As cenas seguintes deslocam em 1: **W2 = 3 · W3 = 4 · W4 = 5.**

### Fora
Painel, joints, bake.

---

## W2 — Painel global + Inspector body · *a autoria*

**Objetivo:** o artista liga/desliga a física, seta gravidade/escala no painel de mundo, e edita
massa/restituição/atrito/tipo num sprite selecionado.

### Entregáveis
- **`ph2d-panel-physics` docado (categoria MUNDO — ADR D8):** gravidade (vetor), substeps/iterações do
  solver, damping global, sleep thresholds, **matriz de camadas de colisão** (a escala do mundo é
  `ProjectSettings.pixels_per_meter`, setting do projeto — o painel exibe, não duplica). Tokens +
  i18n (zero hex/`f32`/string hardcoded; inglês). Registrado nos **5 sites** (precedente
  `ph2d-panel-vector`):
  1. `impl Panel` — `ID="physics"`, `NODE_ID=ids::PHYSICS_PANEL` (próximo IconId/panel-node livre, anotar),
     `DEFAULT_VISIBLE=false`, `populate`/`paint`/`apply_event`.
  2. push no `ph2d-panel-registry-init` (GERADO por `cargo run -p ph2d-panel-sync`) + a const
     `EXPECTED_TYPED` à mão.
  3. feature Cargo `panel-physics = ["dep:ph2d-panel-physics"]`.
  4. **a lista de fallback de z-order em `hero/paint.rs`** — sem a entrada, o painel registrado+visível
     **NUNCA é pintado** (a armadilha "never painted").
  5. visibilidade dirigida pela ponte (`hero.panel_visibility.insert("physics", ...)` no `render_loop`).
- **Seção "Physics Body" no Inspector (por-seleção):** type (dynamic/static/kinematic), massa/densidade,
  restituição, atrito, collider-shape. NÃO no painel global. NumberInput com range/clamp const
  ([[reference_topic_panel_registration]]).

### Gates (red-first, mutation-tested)
1. **painel pintado E populado E clicado** — um teste do `ph2d-ui-testkit` que **DIRIGE o clique** em cada
   row e afirma o efeito ([[feedback_widget_is_done_when_a_test_clicks_it]] +
   [[feedback_painted_is_not_populated_paint_gate]]). Nasce vermelho (sem `populate`, o WidgetStore está
   vazio e não há Click).
2. **toda row de setting muda o mundo** — seam que CLICA: mexer na gravidade **muda a aceleração dos
   corpos**; mexer nos substeps/damping **muda a simulação**. Mutação: um arm que não chama
   `apply_ui_edit` (fio órfão) sangra. Varre **cada** row (não "o card mais cheio" —
   [[feedback_the_fullest_card_premise_rots]]).
3. **sem string hardcoded** — gate i18n; todo label resolve via chave `panel.physics.*`.
4. **botão dimmed recusa no `event.rs`** — dim é cosmético ([[feedback_disabled_button_still_dispatches]]);
   editar body sem seleção é no-op explícito (`debug_assert`/`warn`), não corpo vazio.

### Smoke
`PH2D_PHYSICS_SMOKE=3` (W2a, autoria no Inspector) · **`=4` (W2b, o painel de mundo)**.

### Fora
Joints, bake.

---

## W2c — Camadas de colisão · *quem colide com quem*

**Por que NÃO entrou no W2b:** a matriz é metade de uma feature. A outra metade é
**a camada de cada corpo**, que é campo de component (bump de `PROJECT_SCHEMA` +
`ComponentRegistry`) e UI do **Inspector** — a superfície do W2a, já fechada e smokada.
Uma matriz sem atribuição por-corpo é uma matriz 1×1: todo corpo na camada 0, uma única
célula viva, e as outras 255 são chrome que não faz nada. *"Botão que não faz nada é pior
que botão que falta."*

Duas fricções reais que a wave tem de resolver de propósito, não por acidente:
- o gate `architecture_panel_wiring_parity` **não enxerga registro dentro de laço**, e uma
  matriz É um laço — os ids são dinâmicos (`hash` por par de camadas). Precisa do gate
  irmão que o Painter/Flip já têm para ids dinâmicos (`*_dynamic_ids_dont_collide_*`).
- mudar a matriz muda o `InteractionGroups` de **todo collider vivo**, então ela entra no
  mesmo choke point das outras settings (`set_settings` → aplica + limpa o ring), e o
  `BodyDesc` ganha `memberships`/`filter`.

### Entregáveis
- `Collider.layer: u8` (append-only) + a linha no Inspector · `PhysicsSettings.layer_matrix`
  ([u16; 16], triangular na UI como Unity) · `BodyDesc.memberships/filter` → `ColliderBuilder::collision_groups`.
- Bump `PROJECT_SCHEMA` + a tripla-pin.

### Smoke
`PH2D_PHYSICS_SMOKE=5` — dois grupos, duas camadas, um chão.

### Gates
1. **dois corpos em camadas que não colidem se ATRAVESSAM** (oráculo de aparência: o de
   cima chega ao chão). Mutação: a matriz ignorada → colidem → vermelho.
2. **mudar a matriz alcança colliders que já existem** (o irmão do
   `the_defaults_reach_bodies_that_already_exist`).
3. **a matriz é simétrica** — A colide com B ⟺ B colide com A. Uma matriz que pode ficar
   assimétrica tem dois donos para um fato.

## W3 — Joints · *as articulações*

**Objetivo:** pino/mola/motor/distância entre corpos; pêndulo, corrente, ragdoll simples.

### Entregáveis
- Components de joint (registrados no `ComponentRegistry` — append-only), autoria no Inspector/canvas
  (gizmo de ancoragem), mapeamento para `ImpulseJointSet`/`MultibodyJointSet` do rapier (acesso cru via
  `bodies_mut`/`colliders_mut` do wrapper). Determinismo preservado (mesma proibição de simd/parallel).
- ~~Bump `PROJECT_SCHEMA` (**21 → 22**)~~ — **NÃO acontece, e a contagem é que decide** (*"o valor se CONTA, não se escolhe"*). O blob de um componente no snapshot é chaveado por `stable_type_id = blake3(nome_canônico)[..8]`, derivado do **NOME** e não de uma posição no registry: registrar `ph2d::physics::PhysicsJoint` cunha um id novo e **não move nada**. É o oposto do W2c, que apendou `layer` DENTRO do `Collider`, onde postcard é posicional e o bump era obrigatório.
  Bumpar assim mesmo não é neutro: um schema divergente **recusa o arquivo inteiro** (`project.rs`), então jogaria fora todo projeto já salvo — para melhorar a mensagem de erro na única direção que não funciona de qualquer jeito (um build ANTIGO lendo um arquivo com joints). O raciocínio está falsificável em `crates/ph2d-physics-ecs/tests/joint_persistence.rs`: se uma mudança futura de fato mover o layout, o 1º gate fica vermelho e o bump passa a ser devido. **`PROJECT_SCHEMA` segue em 21.**

### Gates (red-first, mutation-tested)
1. **pêndulo de 2 corpos determinístico** — hash estável cross-OS (estende `physics-ecs-c9` com uma cena de
   joint, ou um segundo hash). Mutação: trocar a ordem de inserção dos joints sangra.
2. **joint sobrevive save/load** — schema bump provado por round-trip (grava, carrega, re-simula, mesmo
   hash). Mutação: joint não-registrado no `ComponentRegistry` some do snapshot (a armadilha D3) — o
   round-trip sangra.
3. **mutação de um parâmetro de joint sangra o gate de repro** — mudar stiffness/rest-length muda a
   trajetória; o oráculo de aparência pega.

### Smoke
`PH2D_PHYSICS_SMOKE=6` — pêndulo/corrente auto-play.

### Fora
Bake.

---

## W4 — Bake-to-timeline · *runtime-truth vira animação*

**Objetivo:** o botão "Bake" amostra a sim sobre um range e escreve keys editáveis nas tracks da entidade
— a metade motion-graphics do framing (ADR D11).

### Entregáveis
- Amostragem determinística da pose por frame → **`ph2d-anim::fit_fcurve`/Schneider** (colunas alinhadas,
  pré-filtro passa-baixa se preciso), **1 passo de undo**, via a ponte da timeline/anim. A costura:
  `sim → amostra por tick → fit_fcurve_at → Track::simplify_range → 1 undo step`. Reusa a máquina do record
  da timeline (W5), não reinventa.
- Botão "Bake" no painel/Inspector, range = seleção da timeline.

### Gates (red-first, mutation-tested)
1. **curva assada reproduz a sim dentro da tolerância** — **oráculo de APARÊNCIA** (posição no tempo certo,
   não uma fórmula — [[reference_topic_oracle_discipline]]). Nasce vermelho sobre uma curva que não segue a
   trajetória. Mutação: amostrar no relógio errado (playhead cru vs tick) sangra.
2. **bake é determinístico** — mesma sim → mesma curva (D7). Mutação: um transcendental sem convenção única
   no fit sangra o hash da curva.
3. **1 undo step (não 1 por frame)** — [[feedback_capture_stroke_session_before_pen_up]] análogo: a sessão
   de bake é UM passo. Mutação: 1 key/frame sem simplify vira 1 undo/frame — o gate conta os passos.

### Smoke
`PH2D_PHYSICS_SMOKE=7` — rampa + bola que rola + duas caixas, relógio PAUSADO: seleciona, assa, e dá play.

⚠️ **O bake não "desliga a física": ele ENTREGA a pose.** O apply da timeline escreve o `Transform` e o
readback da física escreve **depois**, então um corpo dinâmico recém-assado é sobrescrito pelo solver todo
frame e o artista veria o botão Bake não fazer nada. Por isso o bake vira `BodyKind::Kinematic` — o corpo
continua no mundo e continua empurrando, mas o movimento vem da curva. É o que *runtime-truth vira
animação* quer dizer. Ver §W4 do tracker.

⚠️ **CORREÇÃO (2026-07-18, W4b):** este parágrafo dizia também *"o desligamento manual seria o desenho
errado de qualquer jeito"* e **isso passou do ponto**. Ele respondia *"o Bake deve desligar a física no
corpo assado?"* (não — ele entrega a pose, pelo motivo acima) e enunciou a resposta como verdade sobre
**qualquer** interruptor. São duas perguntas. A outra — *"o Play tem de dirigir o solver?"* — é do
TRANSPORTE, não do corpo, e a resposta é **sim, o artista escolhe**: o Enio reportou o conflito (*"os
controles de simulação e de animação parecem ser os mesmos … a simulação roda junto com a animação"*) e a
wave **W4b** pôs o toggle **Physics** na barra da timeline, **desmarcado por padrão**. Registro completo
(incluindo por que esta nota enganava) em [`BUGS_physics.md`](BUGS_physics.md) #1. As duas decisões
convivem sem se tocar: o toggle diz se o solver **roda**, o `Kinematic` diz quem **escreve a pose** quando
ele roda. Ver §W4b do tracker.

### Fora
Soft-body, fluidos, collider-gen vetorial, fratura (M13+ / linhas próprias).

---

## Convenções do módulo (valem em todas as waves)

- **Inner loop:** `cargo check -p ph2d-physics-ecs` (ou `ph2d-physics`). Teste/clippy/auditoria **1× no
  fechamento** da wave, sobre o diff acumulado. Workstation voa — rust-analyzer full como oráculo.
- **LOC cap (HR-18):** shells/foundational = 600 LOC/arquivo; campo/mod novo que estoura → **split em módulo
  irmão**, não allowlist. `cargo fmt` re-expande → fmt ANTES de medir.
- **Determinismo:** NUNCA ligar `parallel`/`simd-*` no rapier. Todo transcendental no código NOSSO com
  convenção única; 1 ulp já é bug.
- **Ids/consts/variants novos:** próximo livre, **anotados no tracker** (`HANDOFF_line_physics.md`) para o
  integrador grepar mesmo-símbolo (§1.5.9).
- **Fechamento de wave = gate batched verde + handoff de tracker atualizado. Então PARE.** Integração e ship
  só por ordem EXPLÍCITA do Enio, via agente integrador dedicado (regra E/F).

# Plano — o PLAYER CINEMÁTICO (o 2º modo)

> **O que está VIVO aqui (2026-08-18):** as decisões (§1-§5), a única wave que **sobra**
> (`W-KinTune`, reservada para o que o smoke pedir), a tabela dos **números medidos** (§7), as
> **⛔ recusas medidas** (abaixo), o que **NÃO entra** (§8) e o que a integração cobra (§9).
>
> **O que saiu:** as **waves FECHADAS** (`W-Water` · `W-ClingPull` · `W-Landing` · `W-KinMove` ·
> `W-KinWeight` · `W-KinPush` · `W-KinPure` · `W-KinCarry` · `W-FloatFloor`) e os post-mortems
> **§8.1-§8.4**, movidos **verbatim** para
> [`docs/archive/docs-2026-08-18/Physics/07_plano_player_kinematico.md`](../archive/docs-2026-08-18/Physics/07_plano_player_kinematico.md).
> Vá lá para responder *"por que isto ficou assim?"*. ⛔ Nada foi resumido — as duas metades
> remontam o original byte-a-byte (sha256).

> Normativo. Companheiro do [`06_plano_player_plataforma.md`](06_plano_player_plataforma.md),
> que decidiu o player DINÂMICO e fechou nomeando este item: *"Player Kinematic — o Enio
> disse que virá um dia. Este plano não o proíbe: a lei pura da `ph2d-platformer` é
> agnóstica de como o motor é aplicado, e é exatamente onde um segundo consumidor
> entraria."* Ordem do Enio, 2026-08-07: **vai**.
>
> ⚠️ **E este plano carrega TRÊS waves que não são do modo novo** — os três reports que o
> Enio deu na mesma janela (2026-08-07). São defeitos VIVOS do player **DINÂMICO**, vêm
> **primeiro**, e cada um foi medido antes de ter cura escrita:
>
> | wave | o report | o que a medição achou |
> |---|---|---|
> | **`W-Water`** | *"não interage corretamente com a água, como a jangada faz"* | **não é sobre água**: o sensor de chão trata um SENSOR como matéria sólida, e ele fica **de pé sobre a poça** |
> | **`W-ClingPull`** | *"ao primeiro toque a jangada é ATRAÍDA para o player"* | a metade de baixo da mola **puxa**, e a 3ª lei transmite a puxada: a jangada **sobe 96,9 mm** |
> | **`W-Landing`** | *"a desaceleração ao encostar no chão é muito lenta e artificial"* | **0,500 s** para assentar, e a causa é o `1 − k·dt²`: com a rigidez certa o mesmo pouso custa **0,133 s** |
>
> ⚠️ **A ORDEM entre elas é load-bearing:** a cura do pouso é uma mola mais RÍGIDA, e uma
> mola mais rígida **amplifica a puxada** do `W-ClingPull`. Consertar o pouso primeiro
> pioraria a jangada no mesmo commit.
>
> ⚠️ **A frase acima é meia-verdade, e a metade que falha decide o tamanho da wave.**
> Ela vale para andar/pular/arrancar/parede/agachar/quina. Não vale para a **D1** (a
> cápsula flutuante) nem para a **D3** (a reação da 3ª lei): as duas existem *porque* o
> corpo é Dynamic. O §2 diz o que acontece com cada uma — e a resposta não é a mesma
> para as duas, que é o achado desta pesquisa.

> ### ⚠️ E as três CURARAM o que este plano usava para justificar o modo novo
>
> **Re-medido em 2026-08-08, antes de a `W-KinMove` ser escrita**
> (`measure_kinematic_case.rs`), pela regra do `CLAUDE.md` §0 aplicada a este
> documento: *quem move o número que justificava algo tem de reconferir a nota.*
>
> | defeito que o §0 deste plano nomeava | o número dele | **hoje, no default** |
> |---|---|---|
> | deriva de rampa (parado 10 s a 30°) | `0,164 m` | **`0,0000 m`** |
> | penetração no impacto | `23 mm` | **`−0,5 mm`** (ele nunca desce abaixo da altura de flutuação) |
>
> ⚠️ **E a tabela só significa alguma coisa por causa da coluna de CONTROLE:** os
> dois zeros são **comprados por um knob no TETO**, não estruturais. Baixando o
> `spring_damping` a um quarto eles voltam inteiros —
>
> | queda | `damping` no teto (o default) | `damping` a ¼ |
> |---|---|---|
> | 0,5 m | −0,5 mm | **155,6 mm** |
> | 10,0 m | −0,5 mm | **295,5 mm** |
>
> e a deriva de rampa faz o mesmo (`0,0000` → `0,0332` a meio curso, `0,0498` a
> um quarto). O mecanismo é o boost: no teto ele **mata a velocidade relativa
> inteira em UM tique**, então o personagem é apanhado no instante em que o raio
> o vê.
>
> ### O que isto faz com a justificação da wave — e o que NÃO faz
>
> ⛔ **Some:** *"o modo cinemático conserta a deriva e a penetração"*. Não
> conserta o que já é zero.
>
> ✅ **Fica, e é o que o Enio de facto pediu** (*"a intenção é que nosso player
> reaja a tudo e **influencie tudo** no mundo físico"*):
>
> 1. **A precisão deixa de depender de um knob.** Sob Snap o resíduo é zero **em
>    qualquer `spring_damping`**, porque não há mola — é estrutural, não afinado.
>    Um artista que baixe o amortecimento para um pouso mais macio hoje **compra
>    de volta** 30 cm de penetração; sob Snap não compra nada.
> 2. **A K6 e a `W-KinPush`** — transmitir peso e empurrar de lado pela NOSSA lei
>    — são capacidade nova, não cura de defeito, e são o que a ordem nomeia.
> 3. **A ESCOLHA**, que a ordem pede explicitamente (*"com a possibilidade de
>    escolha para ligar e desligar esse modo, sem sobrescrever o que já temos"*).
>
> ⚠️ **Consequência para os gates:** o `the_kinematic_player_does_not_creep_up_a_ramp`
> e o `..._does_not_sink_into_the_floor` **têm de correr com o amortecimento
> ABAIXO do teto** — no teto os dois ficariam verdes nos dois modos, e *um gate
> que passa no controle está a medir a coisa errada*. O plano já exigia isso do
> primeiro; agora vale para os dois.

---

## §1 — A pesquisa: o que os outros fazem, e o que foi tentado e abandonado

| produto | o que é | como resolve o contato | empurra o mundo? |
|---|---|---|---|
| **Unity `CharacterController`** | cápsula cinemática, `Move()` | shapecast + slide, `slopeLimit`, `stepOffset`, `skinWidth` | **não** — `OnControllerColliderHit` é o escape MANUAL, escrito pelo jogo |
| **Godot `CharacterBody2D`** | `move_and_slide()` | slide, `floor_max_angle`, `floor_snap_length`, `motion_mode` Grounded/Floating | não; e tem `moving_platform_apply_velocity` porque a plataforma **não** vem de graça |
| **Unreal `CharacterMovementComponent`** | sweep-based | slide + step-up | `PushForce`, **aproximado**, escrito à mão |
| **Box2D v3** | — | nada embutido; o manual mostra shapecast-and-slide à mão | — |
| **rapier 0.35** | `KinematicCharacterController::move_shape` | slide · `autostep` · `snap_to_ground` · `max_slope_climb_angle` · `min_slope_slide_angle` | `solve_character_collision_impulses`, e o **doc dele** diz: *"only approximate as it is not based on a global constraints resolution scheme"* |

⚠️ **O que foi TENTADO e abandonado é a direção OPOSTA à desta wave.** A cápsula
flutuante dinâmica (o `bevy-tnua`, o *Very Very Valet* da GDC — as duas referências da
D1) existe **porque** o controlador cinemático transforma cada interação com o mundo
físico num caso especial escrito à mão: a caixa que ele empurra, a jangada que devia
afundar, o elevador de que ele devia participar. Ninguém migrou de um para o outro:
**Unity e Godot shipam os dois lado a lado, e é isso que este plano copia** — não porque
seja um meio-termo, mas porque são **dois produtos**, com listas de trocas opostas:

| | Dinâmico (o que shipa) | Cinemático (esta wave) |
|---|---|---|
| penetração no impacto | 23 mm por 1 quadro (W2a, medido) | **zero por construção** (o `offset` é preservado no cast) |
| deriva de rampa parado | `0,153 · sen θ · (1 − d)` m/10 s — **zero no teto** que shipa | **zero por construção** (não há mola a integrar) |
| controle exato da posição | não (o solver é o dono) | **sim** (a translação é escrita) |
| empurra caixa / afunda jangada | **sim, exato** (a 3ª lei, D3) | ver **K6** — não é o que o resto da indústria faz |
| degrau / rampa / plataforma | de graça (a D1 apaga o caso especial) | `autostep` + `snap_to_ground` + a velocidade do chão somada à mão |
| custo por tique | um cast + o solver que já roda | um `move_shape` (shapecast por eixo de slide) — **número da K1** |

---

## §2 — As sete decisões

| # | decisão | o motivo curto |
|---|---|---|
| **K1** | É um **MODO do mesmo componente**, não um segundo player | a lei de INTENÇÃO (andar/pular/arrancar/parede/agachar/quina/perdão) é a mesma nos dois; duplicá-la daria duas respostas para *"o que o botão de pulo faz"* |
| **K2** | **Componente VALUADO `PlayerMode`**, ausente = `Dynamic`, nunca campo do `PlatformPlayer` | **zero bump de `PROJECT_SCHEMA`** (componente novo cunha blob-key própria — o idioma do `GravityScale`/`MassOverride`/`Dominance`, com *detach no neutro*), e a razão é a que este módulo escreveu sete vezes: *um bump RECUSA todo projeto já salvo*. ⚠️ **Um MARCADOR seria a representação errada, e a ordem do Enio de 2026-08-07 é quem o prova:** ele nomeou um **terceiro** modo (*"um cinemático puro sangue, como nos games de plataforma mais comuns"*), e a presença de um marcador só sabe dizer duas coisas. Um bit hoje seria um segundo componente amanhã |
| **K3** | O modo de SUPORTE entra na **LEI**, não na ponte | a alternativa é a ponte SUBTRAIR do `accel` as parcelas que dependem do corpo — uma **enumeração**, e enumeração apodrece no dia em que uma força nova entra no fold. E o valor já existe isolado: `let spring = ride_spring(…)` (`lib.rs:489`) é um `Motor` com nome próprio |
| **K4** | **`footing()` continua a porta ÚNICA** de *"onde está o chão"* | o `move_shape` devolve `grounded: bool` — consumi-lo seria a **segunda resposta** para o fato de que a lei inteira depende, a doença que este módulo curou quatro vezes. O `grounded` do rapier é DIAGNÓSTICO, e o gate afirma isso |
| **K5** | A velocidade cinemática mora no **`PlayerState`** | o corpo cinemático não tem velocidade do solver, então alguém tem de a possuir — e o aviso já está escrito no próprio `PlayerState`: *"um estado de player que vivesse num segundo mapa da ponte teria de ser acrescentado àquele ring à mão, e esquecê-lo é um scrub que devolve o mundo de um tique e a memória de outro"* |
| **K6** | A **3ª lei SOBREVIVE** ao modo | ⚠️ o achado da pesquisa: `reaction(cfg, support, impulse, movement)` toma o suporte como ARGUMENTO, e o chão vem do `footing` — **nada nela depende de o corpo ser Dynamic**. Sob Snap o suporte é o **PESO** (`m·g`), e a jangada afunda com a nossa reação exata em vez do impulso aproximado do rapier |
| **K7** | A plataforma móvel entra pelo **`ground_velocity` que já existe** | é o `moving_platform_apply_velocity` do Godot, e o número já viaja no `GroundSample` desde a W3 — a ponte soma `ground_velocity · dt` à translação desejada, e não há segunda pergunta |

### ⚠️ K6 é a decisão que separa este plano do resto da indústria — ORDENADA pelo Enio

Ordem de 2026-08-07: *"o cinemático transmite peso ao chão … a intenção é que nosso
player reaja a tudo e influencie tudo no mundo físico"*.

Unity, Godot e Unreal deixam o empurrão como escape manual porque **não têm a força**:
o controlador cinemático deles não computa quanto peso o personagem transmite. **Nós
temos** — a `react.rs` já a computa e a W6 do plano 06 já a entrega ao chão pelo ponto
de contato. Sob Snap o `support` deixa de ser a mola e passa a ser o peso, e é a **mesma
função** que fecha o resto.

### ⚠️ E *"influencie TUDO"* alarga a K6 para o LADO — pela mesma lei, não por uma segunda

A versão anterior deste plano punha *"empurrar de lado"* fora de escopo, com o argumento
de que a alternativa era o `solve_character_collision_impulses` do rapier (aproximado) e
que **duas fontes de impulso sobre o mesmo par seriam duas respostas**. O argumento
continua certo; **a conclusão estava errada, porque existe uma terceira via que é a
NOSSA lei outra vez:**

> o `move_shape` devolve a translação **EFETIVA**. A diferença entre o que se pediu e o
> que se conseguiu, **projetada na normal do contato**, é exatamente o movimento que
> alguma coisa absorveu — e a força que o absorveu é `m · ((desejado − efetivo) · n) /
> dt`, aplicada naquele ponto.

É a **mesma frase** da 3ª lei que o chão já recebe (*o que eu queria fazer e não fiz,
alguém fez por mim*), medida no outro eixo. Não é o solver do rapier, não é uma segunda
fonte, e não precisa de constante nova: a massa é a mesma que a K2 já autora.

⚠️ **O que continua FORA, e é assimetria honesta, não omissão:** este modo faz o
personagem **influenciar** tudo; ele **não** é influenciado de volta. Um caixote que cai
na cabeça dele não o derruba, porque um corpo cinemático tem massa infinita **por
definição** — e implementar o retorno seria re-escrever um solver que já temos. *A
resposta para "quero que o mundo o empurre" é o modo **Dynamic**, que é precisamente o
que ele faz e por que foi construído primeiro.* Os dois modos são as duas metades da
mesma escolha, e é isso que o chip diz.

---

## §3 — A arquitetura, em uma figura

```
                     PlayerInput (a fita, por tick — inalterada)
                                  │
   footing()  ◄── um cast ────────┤        ⚠️ K4: a ÚNICA resposta a "onde está o chão"
        │                         ▼
        └────────────►  player_motor(…, support: Support)
                                  │            ⚠️ K3: Spring | Snap decide o `spring`
                                  ▼
                    PlayerStep { motor, reaction, nudge, gravity_hold, … }
                                  │
              ┌───────────────────┴───────────────────┐
              ▼ (sem marcador)                        ▼ (com KinematicPlayer)
   apply_player_motor                      integra o motor numa velocidade do
   (impulso + boost no rapier)             PlayerState (K5) → desired_translation
   o solver move o corpo                   + ground_velocity·dt (K7)
              │                                       │
              │                            move_shape(dt, query_pipeline, …)
              │                              → translation (escrita no corpo)
              └───────────────┬───────────────────────┘
                              ▼
              apply_player_reaction(…)   ⚠️ K6: a MESMA porta nos dois modos
```

**As portas únicas, uma linha cada:**

1. *"onde está o chão, como ele se move, e que tipo é?"* → **`footing()`** (K4).
2. *"o que o personagem QUER fazer neste tique?"* → **`player_motor()`**, os dois modos.
3. *"o que segura o personagem?"* → **`Support`**, lido uma vez, dentro da lei (K3).
4. *"quanto o corpo anda?"* → **uma função por modo**, escolhida pelo marcador; nunca
   as duas no mesmo tique.
5. *"o que volta para o chão?"* → **`apply_player_reaction`**, os dois modos (K6).

---

## §4 — Onde encosta em contrato congelado (§6) e em schema

| superfície | encosta? | prova |
|---|---|---|
| `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` | **não** | nada desta wave é uma ferramenta; o gesto é o §14 do Inspector, que já existe |
| `NodeOp=2` / `OpResolver=1` / `NodeManifest=8` | **não** | o módulo de física não alcança o nodegraph |
| `PROJECT_SCHEMA` | **não** (K2) | o componente novo cunha `stable_type_id` próprio — o precedente literal do `GravityScale`/`Dominance`; o `PlatformPlayer` não ganha campo |
| `InputTape` (persistida, W17) | **não** | ela guarda `Vec<PlayerInput>` e nada mais (`tape.rs:94`); o `PlayerState` vive no `PlayerStateRing`, que é **cache da ponte** |
| registro do `ph2d-physics-ecs` | **sim: 28 → 29** | um componente novo; o gate `registers_every_physics_component` cobra |
| `physics_ecs_c9` | **sim, se a cena do hash ganhar um player cinemático** — e ela **não ganha** nesta wave | o c9 mede o caminho determinístico do SOLVER; um corpo cinemático não passa por ele. Fica **byte-idêntico**, com gate |

⚠️ **A prova é por grep no fechamento, nunca por auto-relato** — a política do módulo
desde a W3.

---

## §5 — O que a UI precisa (as quatro condições, independentes)

1. **O componente EXISTE e é autorável** → `every_physics_component_is_authorable`
   (a lista `WRITERS`, que já ficou vermelha uma vez por um split — W-AreaFalloff).
2. **É pintado E registrado** → a row `Body: Dynamic | Kinematic` na §14, um `seg_row`
   como o `Solid | Sensor` e o `Discrete | Continuous`; o `architecture_panel_wiring_parity`
   cobra o `populate`. ⚠️ **TRÊS chips desde a `W-KinPure`** — e a promessa foi paga: o
   terceiro custou **uma linha** na tabela de ids e uma na lista de rótulos, porque o
   `seg_row` recebe uma fatia e nunca um par.
3. **O clique chega ao barramento** → varredura de seam com **`click_real`** (o helper
   que a W-AreaFrame criou porque `seg_row` registra em LAÇO e o `wiring_parity` é cego
   a isso).
4. **A sequência leva a algum lugar** → `inspector_physics_gesture_tests`: trocar o modo
   com o relógio parado tem de deixar o personagem em pé e ANDÁVEL no Play — a condição
   que pegou o *"converta para Capsule"* que teria destruído o tronco.

⚠️ **A metade VISÍVEL:** o contorno do collider já desenha; o que muda é a **cor de
kind** (o corpo passa a ser cinemático, e o overlay já pinta kind por cor desde a W2a).
Nenhum glifo novo — e isso é resposta, não omissão.

---

## §6 — As waves

Cada uma fecha com gate batched verde · mutações · **cena de smoke com números MEDIDOS**
(a sonda headless roda ANTES de a mensagem ser escrita) · entrada no tracker + linha no
`00_plano_waves.md`, na mesma sessão.

⚠️ **Todas as waves deste plano FECHARAM, menos uma.** As nove fechadas (o quê · o mecanismo ·
os números) estão no
[arquivo](../archive/docs-2026-08-18/Physics/07_plano_player_kinematico.md) — leia-as **antes**
de tocar na lei, porque cada uma nomeia o que já foi tentado. Fica aqui só a que não fechou:

### W-KinTune — O QUE O SMOKE PEDIR

Reservada de propósito: `autostep` afinado, `min_slope_slide_angle` exposto, ou nada.

---
## §7 — Os números que serão MEDIDOS (nenhum teto sem tabela — CLAUDE.md §0)

| # | número | como se mede |
|---|---|---|
| 1 | **custo de um `move_shape`** por player por tique, contra o cast+solver de hoje | ✅ **MEDIDO 2026-08-09** (`measure_player_budget`): **+0,34 µs/player, +37%** sobre o dinâmico, linear em N (µs/player plano de 10 a 200) ⇒ **~1200 personagens cinemáticos** cabem nos 1,5 ms do HR-4 (contra ~1648 dinâmicos) nesta máquina. **O orçamento não é o teto deste modo.** Gate de FORMA `the_cost_of_a_player_is_linear_in_their_number` |
| 2 | **deriva de rampa** nos dois modos, varrendo `θ` | ✅ **MEDIDO 2026-08-09** (`measure_ramp_drift_modes`) — **a deriva cinemática é `0,0000` em TODA rampa** (10° a 45°, e também em 60 s), contra a dinâmica que CRESCE (0,0197 → 0,0575 de 10° a 45° a ¼ do teto, e `0,0000` no default, que está no teto). *Sob Snap não há mola a integrar* ⇒ a lei cinemática **não tem termo em θ**. ⚠️ **E a varredura derrubou DUAS coisas:** a lei publicada do `ride.rs` (`0,153·sen θ·(1−d)`) **satura acima de ~20°** — acerta a 10°/20° e erra 13%/23%/29% a 30°/40°/45°, porque foi ajustada noutro rig; e **acima do limite de rampa o cinemático fica PARADO** (§8.3). Gate `the_kinematic_drift_does_not_grow_with_the_ramp` |
| 3 | **penetração** no impacto nos dois modos | ✅ **MEDIDO 2026-08-09** (`measure_kinematic_case::measure_the_impact_penetration_today`, agora com as DUAS colunas) — o dinâmico **no teto** não penetra (`−0,5 mm`, ele nunca mergulha) e **a ¼ do teto afunda 155 → 295 mm CRESCENDO com a queda**; o cinemático fica em **12-47 mm e é PLANO** (46,5 a 5 m e a 10 m) — o sweep do controlador não pode ser ultrapassado. ⚠️ **A primeira tabela mediu com a régua do OUTRO modo:** ela usava `FLOAT_HEIGHT − pior`, que é onde a cápsula FLUTUANTE descansa, e cobrava **387 mm** ao cinemático por estar exatamente onde devia (ele assenta a 0,5566, não flutua). Cada modo mede contra o próprio repouso |
| 4 | `snap_to_ground` mínimo útil | ✅ **MEDIDO 2026-08-09** (`measure_snap_and_step`) — e **os dois números são UM** (`snap_distance` e `step_height` saem ambos da `cling_distance`). **SUBIR segue o número 1:1** (cling 0,15/0,25/0,40/0,60 → degrau 0,20/0,28/0,40/0,60; a cápsula sozinha sobe 0,06). **DESCER é INERTE:** variar o `cling` de 0,05 a 2,00 deixa as 24 células **idênticas**, e a mutação `snap_to_ground: None` dá a **mesma tabela** — o rapier o gateia em `translation.dot(up) < -1e-5` e a lei zera a vertical no chão ⇒ *o mínimo útil não existe, porque nenhum valor faz nada*. **É o MESMO mecanismo da §8.1.** Gate `the_step_it_climbs_is_the_number_the_artist_authored` |
| 5 | quanto o `offset` (a folga do controlador) custa em aparência | ✅ **MEDIDO 2026-08-09** (`measure_foot_gap`) — e **a pergunta estava mal colocada**: a folga não é o `offset` (1 cm relativo), é **onde o último sweep parou**, de **1,0 a 4,5 cm** conforme a aproximação, com amplitude zero. O dinâmico é aritmética exacta (`float − meia-extensão`). ⚠️ **E a caçada achou um defeito MAIOR, ABERTO** — ver a §8.1 |
| 6 | **o que a jangada afunda** nos dois modos (K2) | ✅ **MEDIDO** (`measure_kinematic_case::measure_what_the_raft_feels_in_both_modes`) — com massa **auto** os dois são **idênticos** (−1,1957 contra −1,1960 m/s², 100% de `m·g`): a K6 sobrevive ao modo, como o plano previu. ⚠️ **A massa AUTORADA é que os separa, e só acima de ~1 kg:** a 1,00 kg os dois dão −3,2700 (273%) e a 5,00 kg o dinâmico dá −16,35 (1367%) contra −5,46 (457%) do cinemático |
| 7 | **o que uma perna RÍGIDA custa** (`W-Landing`) — o solavanco ao subir um degrau e o pico da reação na jangada | ✅ **MEDIDO** (`measure_landing::measure_what_a_stiff_leg_costs`, varrendo `k` de 400 a 2000) — o pouso encurta **0,500 → 0,133 s** (o que a rigidez COMPRA), a deriva de rampa fica **`0,0000` em todo `k`** (o §7.8 outra vez: a lei não tem termo em `k`) e o soco na jangada anda **19,6 → 19,9 cm**, ou seja **+1,5% para 5× de rigidez**. ⇒ *a perna rígida é quase de graça*, e o preço que o plano temia não existe nesta faixa |
| 8 | a **deriva de rampa contra a rigidez** — a lei publicada varreu o amortecimento, **não** o `k` | ✅ **MEDIDO 2026-08-09** (`measure_the_drift_against_the_stiffness`): planas a 5 decimais numa faixa de **64×** ⇒ **a lei não tem termo em `k`**, e a resposta a *"deriva"* é UM knob. Gate `the_drift_law_has_no_stiffness_term` |

---

## §7-bis — ⛔ As RECUSAS MEDIDAS (construídas, medidas, revertidas — não refaça)

> Estas quatro saíram dos post-mortems §8.1-§8.4, que foram arquivados. **O que está aqui é o
> caro:** cada uma é uma tentativa que existiu em código, foi medida e piorou o número. Um ⛔
> destes reconstruído é uma jornada perdida. O contexto completo está no
> [arquivo](../archive/docs-2026-08-18/Physics/07_plano_player_kinematico.md), nas §8.1-§8.4.

### §8.1 — o EIXO da absorção (a deriva na rampa)

⛔ **A CURA DO EIXO FOI CONSTRUÍDA, MEDIDA E REVERTIDA — não refaça.** Com a
normal, o personagem **parado** deriva **0,7297 m** (contra ~0,001) e **nunca
assenta**; sangram o `..._does_not_creep_up_a_ramp` e o
`..._landing_does_not_slide_along_the_ramp_normal`. O `up` está lá de propósito
— é o `floor_stop_on_slope` do Godot, que o doc do `kinematic.rs` nomeia.

**As duas metades pedem coisas OPOSTAS do mesmo eixo**, e a `supported_velocity`
não distingue a deriva da GRAVIDADE da descida COMANDADA pela caminhada porque,
quando ela corre, as duas já estão somadas num `v` só. A cura de verdade é
separá-las antes da soma — absorver só o que o motor **não** comandou —, e isso
é desenho com cena própria, não uma linha.

### §8.2 — o caixote que rodopiava: fatiar o empurrão em `N`

**Escolha do Enio: (2).** ⛔ **E ela foi CONSTRUÍDA, MEDIDA e REVERTIDA — não
refaça.** A hipótese era que o braço de alavanca encolhe enquanto o caixote
tomba, então `N` fatias somariam menos torque que uma martelada. Em 16 ms o
caixote quase não gira: medido, **77,51 rad** — *pior* que os 74,29 de origem.

### §8.4 — a ÁGUA no modo cinemático: as duas coisas que NÃO eram a causa

⛔ **E a segunda causa que eu construí não existia.** Escrevi um `authored_weight`
que somava as massas dos colliders quando `rb.mass()` fosse zero, com um doc a
dizer que era ele que fazia a água existir no modo cinemático — **a mutação provou
que era falso** (removê-lo deixou tudo verde), e a medição direta explica:
`rb.mass()` devolve `1,0000` em Dynamic, Kinematic **e** Fixed; o rapier zera a
inversa-massa *efetiva*, não esta. Removido no mesmo commit em que nasceu.

**A FRONTEIRA não foi escolhida — o W-AreaFalloff já a desenhou:** o falloff pesa
os dois **EMPURRÕES** (força, torque) e **deixa o MEIO em paz** (`drag`,
`density`, `form_drag` descrevem uma substância, e uma substância não fica mais
rala perto da própria margem). É exatamente isso que torna o meio respondível por
uma consulta e a força não: ⛔ **a força FICA de fora**, porque precisa do frame
da zona, do espelho e do falloff — re-derivá-los numa query seria uma **segunda
resposta** para *"que empurrão esta zona dá neste ponto?"*, o defeito recorrente
desta linha. Uma corrente não leva um personagem cinemático, e isso está
**nomeado, não esquecido**.

### ⚠️ E o `1,44` ficou nomeado como PENDÊNCIA — medido (2026-08-10), não é uma

O aberto dizia *"o player bobeia ~1,44 m nos dois modos (a cápsula solta faz
0,81)"*, lido como defeito por fechar. A sonda `measure_the_bobbing` atribuiu o
excesso por **ablação da entrada**, e ele tem dono:

| ablação (mesma poça, regime = 2.ª metade de 6 s) | amplitude | vs controle |
|---|---|---|
| cápsula solta (CONTROLE) | `0,8097` | — |
| player default, largado de `+1,5` (no ar) | `1,4408` | `1,78×` |
| **os quatro multiplicadores de gravidade a `1`** | `0,8097` | **`1,00×`** (ao 4.º decimal) |
| sem perna · sem amortecimento · sem raio de chão · perna inteira fora | `1,4408` | `1,78×` (inertes) |
| **largado de `−0,5` (já submerso)** | `0,8326` | **`1,00×`** |
| largado de `−1,5` (submerso fundo) | `3,3214` | `0,99×` |

**A trava FUNCIONA.** O excesso inteiro é a modelagem do arco a agir **no AR**,
antes do primeiro contacto com a água — que é exactamente onde ela é autorada
para agir. O personagem cruza a superfície a **`1,299×`** a velocidade do
controle (`1,687×` de energia) porque `fall_gravity = 2.0`, e a poça devolve isso
como um mergulho mais fundo e um bobeio maior. ⚠️ *A minha previsão de `√2` para
essa razão estava errada* — falta na conta o `peak_gravity = 0.5`, que deixa o
começo da queda mais leve que o mundo; **o número honesto é medido**.

⚠️ **E o que decide que não há defeito é a SEQUÊNCIA, não uma janela** — as duas
podem medir `1,44` no mesmo instante. Amplitude por janela de 3 s, em 30 s:

* controle `1,927 · 0,810 · 0,329 · 0,139 · 0,059 · 0,021 · 0,009 · 0,004 · 0,002 · 0,001`
* player&nbsp;&nbsp;`2,172 · 1,441 · 0,594 · 0,221 · 0,093 · 0,039 · 0,017 · 0,006 · 0,003 · 0,001`

As duas decaem monotonicamente e **convergem no mesmo valor**: é um transiente, e
o meio come-o. Os dois gates novos (`the_water_lock_contains_the_arc_shaping` ·
`the_bobbing_decays_it_does_not_pump`) pinam as duas metades para ninguém
re-derivar o item falso — e ⚠️ **os três gates que já viviam ali ficam VERDES nas
duas mutações** (a trava a não calar ⇒ **857 m**; a fração instantânea em vez da
trava ⇒ **15,3 m** a crescer), porque a trava é comum aos dois modos e uma razão
entre dois doentes não a vê.

**Fica como DECISÃO DE PRODUTO, não como dívida:** um personagem que cai com
`fall_gravity = 2.0` entra na água com o momento que essa queda lhe deu. Querer
que ele entre como uma pedra é mexer no knob que governa o platformer inteiro —
não é um conserto local, e o número acima é o que se estaria a trocar.

⚠️ **O arrasto é load-bearing e tem número:** com `AreaDrag 0` a amplitude sobe
para **2,90 m** e não decai — empuxo sem resistência é uma mola sem
amortecimento, a frase que a fixture da poça já carregava.

⚠️ **Paridade APROXIMADA com o dinâmico, e agora com o número DESTA paridade:** o
solver amortece por SUB-PASSO e esta lei uma vez por TIQUE — `(1+d·h)⁻⁴` contra
`(1+d·4h)⁻¹`; um corpo cinemático não tem sub-passo para dividir. ⚠️ **Até
2026-08-10 isto era precificado por ANALOGIA** (*"a mesma classe que a W-AreaDrag
mediu em 1,25%"*), e uma analogia com outra medição não é a medição desta.
Medido em arrasto **puro** (`measure_the_bobbing`), divergência relativa:

| t | 1 s | 2 s | 3 s | 4 s |
|---|---|---|---|---|
| | **1,149%** | 0,257% | 0,056% | 0,018% |

A analogia estava certa em ordem de grandeza — e a **forma** é o que ela não
dizia: a divergência **decai**, porque a velocidade terminal é `g/d` nos DOIS por
álgebra e ela vive só no transiente. ⚠️ **Por isso um gate na terminal seria
verde por construção** e cego à divergência inteira; o que
`the_drag_parity_between_modes_stays_within_its_measured_price` afirma é o PICO.

**Gates:** 4 na lei (`kinematic_tests.rs`) · 2 na consulta (`buoyed_query.rs`) ·
3 no produto (`player_in_water.rs`). **6 mutações, 5 sangram** — a 6.ª acusou a
minha própria afirmação (o `authored_weight`) e o código saiu.

⚠️ **Um gate MUDOU DE NOME e de afirmação:** o
`the_scale_tops_out_at_one_and_zero_gravity_carries_nothing` pinava o teto que
esta wave removeu, e virou `the_scale_is_the_density_ratio_…`. Ele estava certo
enquanto o único consumidor perguntava `> 0`; *quem acrescenta o consumidor que
precisa da magnitude reconfere a nota que a capava.*

---

## §8 — O que NÃO entra (nomeado, não esquecido)

- ~~**Empurrar de LADO**~~ — **ENTROU** (a `W-KinPush`) por ordem do Enio de
  2026-08-07. ⚠️ O argumento que o excluía continua CERTO e é ele que decide o desenho:
  o `solve_character_collision_impulses` do rapier é aproximado por confissão própria, e
  consumi-lo **ao lado** da nossa reação seriam duas fontes de impulso sobre o mesmo par.
  O que mudou não foi o argumento — foi ter aparecido uma **terceira via** que é a nossa
  lei no outro eixo (§2, K6).
- **Ser EMPURRADO de volta.** O caixote que cai na cabeça não o derruba: massa infinita é
  a definição de cinemático, e o retorno é o que o modo **Dynamic** já faz. Assimetria
  declarada, não esquecida.
- **Rotação do personagem.** Nem Unity nem Godot a oferecem no controlador; e o
  `LockRotation` já é o default deste módulo (D4).
- **O `motion_mode: Floating` do Godot** (jogo top-down / nave). Não tem chão, então
  metade desta lei não se aplica — é outro produto, não um chip a mais. ⚠️ Não confundir
  com o *"puro sangue"* do Enio, que **tem** chão e é a `W-KinPure`.
- **O bake.** Ele já funciona: assar vira o corpo `Kinematic` e entrega a pose. ⚠️ E
  isso levanta uma pergunta de UX que a K1 tem de responder no doc: **um player
  cinemático assado e um player cinemático dirigido são o mesmo `BodyKind` com donos
  diferentes da pose.** A resposta é o `PlatformPlayer` + marcador estar presente ou não
  — mas ela tem de estar ESCRITA, senão o próximo a ler o `BodyKind` conclui o contrário.

---

## §9 — O que a integração vai cobrar

> ⚠️ **Esta seção foi escrita antes da jornada de 2026-08-09 e envelheceu em dois
> pontos, corrigidos abaixo.** O documento vivo é o
> [handoff de integração](handoffs/HANDOFF_INTEGRACAO_line_physics_kin_2026-08-09.md).

- ~~**Registro `ph2d-physics-ecs` 28 → 29**~~ — **isso foi a `W-KinMove`, que já
  integrou em 08/08**; o `main` está em 29 e a jornada de 09/08 **não o move**.
- ~~**`PROJECT_SCHEMA`: NÃO bumpa** (K2)~~ — **bumpou**: `69 → 70`, um degrau
  (`PlatformPlayer.reaction_push`, a `W-KinPush`). A segunda metade da frase é que
  vale, e vale mais: o valor se **CONTA** contra o `main` do dia, nunca se escolhe.
- **Ids novos da §14** (os chips do modo + a massa da `W-KinWeight`), anotados no handoff.
- **Cenas de smoke `=100` · `=101` · `=102` · `=103` · `=104`** (a `=104` é a da ÁGUA) — ⚠️ o roteador é um `match` de strings
  cujo `_` cai na cena 1: um nível inexistente **não avisa**, mostra outra coisa. O `=84`
  não existe de propósito; **105 é o próximo livre** (o último ocupado é o `=104`).
- **Contrato congelado: nenhum. Dep externa nova: nenhuma** (o
  `KinematicCharacterController` é do `rapier2d` que já é dep, e não é feature-gated no
  2D).

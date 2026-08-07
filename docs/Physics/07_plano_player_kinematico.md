# Plano — o PLAYER CINEMÁTICO (o 2º modo)

> Normativo. Companheiro do [`06_plano_player_plataforma.md`](06_plano_player_plataforma.md),
> que decidiu o player DINÂMICO e fechou nomeando este item: *"Player Kinematic — o Enio
> disse que virá um dia. Este plano não o proíbe: a lei pura da `ph2d-platformer` é
> agnóstica de como o motor é aplicado, e é exatamente onde um segundo consumidor
> entraria."* Ordem do Enio, 2026-08-07: **vai**.
>
> ⚠️ **A frase acima é meia-verdade, e a metade que falha decide o tamanho da wave.**
> Ela vale para andar/pular/arrancar/parede/agachar/quina. Não vale para a **D1** (a
> cápsula flutuante) nem para a **D3** (a reação da 3ª lei): as duas existem *porque* o
> corpo é Dynamic. O §2 diz o que acontece com cada uma — e a resposta não é a mesma
> para as duas, que é o achado desta pesquisa.

---

## §1 — A pesquisa: o que os outros fazem, e o que foi tentado e abandonado

| produto | o que é | como resolve o contato | empurra o mundo? |
|---|---|---|---|
| **Unity `CharacterController`** | cápsula cinemática, `Move()` | shapecast + slide, `slopeLimit`, `stepOffset`, `skinWidth` | **não** — `OnControllerColliderHit` é o escape MANUAL, escrito pelo jogo |
| **Godot `CharacterBody2D`** | `move_and_slide()` | slide, `floor_max_angle`, `floor_snap_length`, `motion_mode` Grounded/Floating | não; e tem `moving_platform_apply_velocity` porque a plataforma **não** vem de graça |
| **Unreal `CharacterMovementComponent`** | sweep-based | slide + step-up | `PushForce`, **aproximado**, escrito à mão |
| **Box2D v3** | — | nada embutido; o manual mostra shapecast-and-slide à mão | — |
| **rapier 0.28** | `KinematicCharacterController::move_shape` | slide · `autostep` · `snap_to_ground` · `max_slope_climb_angle` · `min_slope_slide_angle` | `solve_character_collision_impulses`, e o **doc dele** diz: *"only approximate as it is not based on a global constraints resolution scheme"* |

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
| **K2** | **Marcador `KinematicPlayer`** — a presença é o modo —, nunca campo do `PlatformPlayer` | **zero bump de `PROJECT_SCHEMA`**; é o idioma exato do `Ccd` · `LockRotation` · `LockPositionX/Y` · `OneWayPlatform`, e a razão é a mesma que este módulo escreveu sete vezes: *um bump RECUSA todo projeto já salvo* |
| **K3** | O modo de SUPORTE entra na **LEI**, não na ponte | a alternativa é a ponte SUBTRAIR do `accel` as parcelas que dependem do corpo — uma **enumeração**, e enumeração apodrece no dia em que uma força nova entra no fold. E o valor já existe isolado: `let spring = ride_spring(…)` (`lib.rs:489`) é um `Motor` com nome próprio |
| **K4** | **`footing()` continua a porta ÚNICA** de *"onde está o chão"* | o `move_shape` devolve `grounded: bool` — consumi-lo seria a **segunda resposta** para o fato de que a lei inteira depende, a doença que este módulo curou quatro vezes. O `grounded` do rapier é DIAGNÓSTICO, e o gate afirma isso |
| **K5** | A velocidade cinemática mora no **`PlayerState`** | o corpo cinemático não tem velocidade do solver, então alguém tem de a possuir — e o aviso já está escrito no próprio `PlayerState`: *"um estado de player que vivesse num segundo mapa da ponte teria de ser acrescentado àquele ring à mão, e esquecê-lo é um scrub que devolve o mundo de um tique e a memória de outro"* |
| **K6** | A **3ª lei SOBREVIVE** ao modo | ⚠️ o achado da pesquisa: `reaction(cfg, support, impulse, movement)` toma o suporte como ARGUMENTO, e o chão vem do `footing` — **nada nela depende de o corpo ser Dynamic**. Sob Snap o suporte é o **PESO** (`m·g`), e a jangada afunda com a nossa reação exata em vez do impulso aproximado do rapier |
| **K7** | A plataforma móvel entra pelo **`ground_velocity` que já existe** | é o `moving_platform_apply_velocity` do Godot, e o número já viaja no `GroundSample` desde a W3 — a ponte soma `ground_velocity · dt` à translação desejada, e não há segunda pergunta |

### ⚠️ K6 é a decisão que separa este plano do resto da indústria — e o preço

Unity, Godot e Unreal deixam o empurrão como escape manual porque **não têm a força**:
o controlador cinemático deles não computa quanto peso o personagem transmite. **Nós
temos** — a `react.rs` já a computa e a W6 do plano 06 já a entrega ao chão pelo ponto
de contato. Sob Snap o `support` deixa de ser a mola e passa a ser o peso, e é a **mesma
função** que fecha o resto.

⚠️ **O que isso NÃO dá:** a reação vai para o corpo que o **sensor de chão** encontrou.
Uma caixa empurrada de LADO não é chão, então empurrar horizontalmente segue sendo o que
o rapier oferece (`solve_character_collision_impulses`, aproximado) — e a K6 **não o
usa**, porque duas fontes de impulso sobre o mesmo par seriam duas respostas. **Empurrar
de lado fica FORA desta wave, nomeado no §8.**

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
              apply_ground_reaction(…)   ⚠️ K6: a MESMA porta nos dois modos
```

**As portas únicas, uma linha cada:**

1. *"onde está o chão, como ele se move, e que tipo é?"* → **`footing()`** (K4).
2. *"o que o personagem QUER fazer neste tique?"* → **`player_motor()`**, os dois modos.
3. *"o que segura o personagem?"* → **`Support`**, lido uma vez, dentro da lei (K3).
4. *"quanto o corpo anda?"* → **uma função por modo**, escolhida pelo marcador; nunca
   as duas no mesmo tique.
5. *"o que volta para o chão?"* → **`apply_ground_reaction`**, os dois modos (K6).

---

## §4 — Onde encosta em contrato congelado (§6) e em schema

| superfície | encosta? | prova |
|---|---|---|
| `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` | **não** | nada desta wave é uma ferramenta; o gesto é o §14 do Inspector, que já existe |
| `NodeOp=2` / `OpResolver=1` / `NodeManifest=8` | **não** | o módulo de física não alcança o nodegraph |
| `PROJECT_SCHEMA` | **não** (K2) | o marcador cunha `stable_type_id` próprio — o precedente literal do `Ccd`/`LockRotation`; o `PlatformPlayer` não ganha campo |
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
   cobra o `populate`.
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

### K1 — O MODO EXISTE E O PERSONAGEM ANDA (cena `=100`)

O marcador, o `Support::Snap`, o `move_shape`, a velocidade no `PlayerState`, a
plataforma móvel, e o chip da §14.

**Gates, red-first:**

1. **`the_kinematic_player_does_not_creep_up_a_ramp`** — nasce **VERMELHO** medindo o
   dinâmico (o resíduo hoje é `0,153 · sen θ · (1 − d)`, e o gate corre com `d` abaixo
   do teto para o resíduo existir); com o marcador tem de dar **0,000 m**, e é
   **estrutural**, não calibrado.
2. **`the_kinematic_player_does_not_sink_into_the_floor`** — o dinâmico penetra no
   impacto (23 mm/1 quadro, W2a); o cinemático preserva o `offset` — **zero**.
3. **`the_law_is_the_same_in_both_modes`** — o MESMO `PlayerInput` produz o MESMO
   `Motor` de INTENÇÃO nos dois; só o `spring` difere. Mutação: fazer o Snap mexer no
   `walk` sangra.
4. **`the_kinematic_ground_answer_comes_from_footing`** — arch-gate: o `grounded` do
   `EffectiveCharacterMovement` **não** alimenta decisão nenhuma da lei (K4). Mutação:
   ligá-lo ao `footing` sangra.
5. **`the_dynamic_player_is_byte_identical`** — a suíte inteira do player + o `c9`.
6. **`a_kinematic_player_rides_a_moving_platform`** (K7) — o vagão da cena `=90` com o
   marcador: o personagem parado tem de viajar com ele.
7. **`the_kinematic_velocity_survives_a_scrub`** (K5) — arrastar a régua para trás e
   voltar reproduz o mesmo `y`; mutação: guardar a velocidade num mapa fora do
   `PlayerState` sangra **só** este.

⚠️ **A fixture tem de conter o fenômeno:** a rampa precisa de `spring_damping` **abaixo
do teto** (no teto o resíduo dinâmico já é zero e o gate 1 seria verde nos dois modos —
*um gate que passa no controle está a medir a coisa errada*, a lição da W-AreaFalloff).

### K2 — O MUNDO SENTE O PERSONAGEM (cena `=101`)

A K6: sob Snap o `support` é o peso, e a jangada afunda.

**Gates:** a jangada da cena `=72` com o marcador afunda **o mesmo** que com o corpo
dinâmico (a comparação é o oráculo, e ela é honesta porque a força física é a mesma) ·
`Reaction Scale = 0` continua a desligar tudo · a mutação que zera o suporte sob Snap
sangra.

⚠️ **A massa é AUTORADA** (o corpo cinemático não tem massa que o rapier calcule) — e é
por isso que a K2 é wave própria: um número novo na §14 tem as suas quatro condições.

### K3 — O QUE O SMOKE PEDIR

Reservada de propósito. A K1 e a K2 entregam o modo; o que o Enio vir decide se falta
`autostep` afinado, `min_slope_slide_angle` exposto, ou nada.

---

## §7 — Os números que serão MEDIDOS (nenhum teto sem tabela — CLAUDE.md §0)

| # | número | como se mede |
|---|---|---|
| 1 | **custo de um `move_shape`** por player por tique, contra o cast+solver de hoje | a sonda do módulo, N players, contra o HR-4 |
| 2 | **deriva de rampa** nos dois modos, varrendo `θ` | a fixture do gate 1, a mesma varredura que produziu a lei do `ride.rs` |
| 3 | **penetração** no impacto nos dois modos | a fixture da W2a |
| 4 | `snap_to_ground` mínimo útil | contra a altura do degrau que o `autostep` sobe |
| 5 | quanto o `offset` (a folga do controlador) custa em aparência | o personagem paira `offset` acima do chão — é a D1 outra vez, num número menor |
| 6 | **o que a jangada afunda** nos dois modos (K2) | a cena `=72` |

---

## §8 — O que NÃO entra (nomeado, não esquecido)

- **Empurrar de LADO** (a caixa que o personagem arrasta). A K6 entrega o eixo do
  suporte, que é o que o sensor de chão vê; o lado exigiria consumir o
  `solve_character_collision_impulses` do rapier, e o próprio doc dele diz que é
  aproximado — **duas fontes de impulso sobre o mesmo par são duas respostas**. Se o uso
  pedir, é wave com aceitação própria.
- **Rotação do personagem.** Nem Unity nem Godot a oferecem no controlador; e o
  `LockRotation` já é o default deste módulo (D4).
- **Um TERCEIRO modo.** O `motion_mode: Floating` do Godot (jogo top-down / nave) não
  tem chão, então metade desta lei não se aplica — é outro produto, não um chip a mais.
- **O bake.** Ele já funciona: assar vira o corpo `Kinematic` e entrega a pose. ⚠️ E
  isso levanta uma pergunta de UX que a K1 tem de responder no doc: **um player
  cinemático assado e um player cinemático dirigido são o mesmo `BodyKind` com donos
  diferentes da pose.** A resposta é o `PlatformPlayer` + marcador estar presente ou não
  — mas ela tem de estar ESCRITA, senão o próximo a ler o `BodyKind` conclui o contrário.

---

## §9 — O que a integração vai cobrar

- **Registro `ph2d-physics-ecs` 28 → 29** (o gate diz existir *"to hurt"*).
- **`PROJECT_SCHEMA`: NÃO bumpa** (K2) — e se algum campo acabar apendado, o valor se
  **CONTA** contra o `main` do dia, nunca se escolhe.
- **Ids novos da §14** (o chip do modo + a massa da K2), anotados no handoff.
- **Cena de smoke `=100`** — ⚠️ o roteador é um `match` de strings cujo `_` cai na cena
  1: um nível inexistente **não avisa**, mostra outra coisa. O `=84` não existe de
  propósito; **100 é o próximo livre**.
- **Contrato congelado: nenhum. Dep externa nova: nenhuma** (o
  `KinematicCharacterController` é do `rapier2d` que já é dep, e não é feature-gated no
  2D).

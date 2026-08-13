# Auditoria 09 — o Player medido contra Unity, Godot, Unreal e o tnua (2026-08-12)

> **Pedido do Enio:** *"auditoria completa no seguinte sentido: comparando com
> Unity, Godot, Unreal e outras game engines, o que não temos em nosso Player
> Platform que deveríamos ter"*.
>
> ⚠️ **Esta auditoria olha para outro lugar que a do plano 08.** Aquela foi ao
> **catálogo de platformer 2D** (o `bevy_tnua`, o GDevelop, o GDQuest, o asset
> library do Godot) e fechou-o: dos vinte itens, dezanove estão feitos e o vigésimo
> (escalar) está nomeado. Esta vai ao **controlador de personagem das ENGINES** —
> o `UCharacterMovementComponent`, o `CharacterBody2D`, o `CharacterController` +
> a família `Effector2D` —, que é um conjunto **diferente**: ele traz menos
> *verbos* e mais **fronteiras** (o que o personagem PUBLICA, o que a superfície
> lhe diz, o que o mundo lhe pode fazer).
>
> **É plano, não implementação.** Cada achado traz o que existe hoje **lido do
> código com arquivo:linha**, o mecanismo pelo qual falta, e o preço. ⚠️ **Nenhum
> número de produto é escolhido aqui** (§0): quem os escreve é a medição da wave.

---

## §1 — O censo de HOJE, e a surpresa está nas SAÍDAS

**Entradas — cinco:** `drive` · `jump` · `down` · `dash` · `grab`
(`sense.rs:119`).

**Leis — treze módulos:** `ride` · `walk` · `jump` · `react` · `wall` · `dash` ·
`crouch` · `swim` · `ledge` · `glide` · `corner` · `slope` · `kinematic`.

**Saídas — a superfície pública inteira da ponte** (`bridge/player_channel.rs`):

| porta | o que devolve |
|---|---|
| `set_player_input` / `player_input` / `clear_player_input` | a ENTRADA, de volta |
| `player_probe_marks` | os gizmos dos sensores (desenho) |
| `player_is_dropping` / `any_player_is_dropping` | um bit da descida de one-way |

⚠️ **Não existe uma única porta que diga o que o personagem ESTÁ A FAZER.** Nem
*está no chão*, nem *para que lado olha*, nem *acabou de aterrar*. O `facing`
existe e está **enterrado dentro do `DashState`** (`dash.rs:125`) — nasce
`1.0`, é escrito pelo arranque, e nunca sai da lei.

Isso é o que esta auditoria encontrou de maior, e não é um verbo em falta: é uma
**fronteira** em falta.

---

## §2 — A tabela, engine a engine

Legenda: ✅ temos · 🟡 temos por outra via (com o porquê) · ❌ não temos ·
⛔ divergência deliberada (não queremos, com motivo).

### Unreal — `UCharacterMovementComponent`

| deles | nós | onde / porquê |
|---|---|---|
| `MaxWalkSpeed`, `MaxAcceleration` | ✅ | `walk` |
| **`BrakingDecelerationWalking`, `GroundFriction`, `bUseSeparateBrakingFriction`** | ❌ | **§3.C** — acelerar e frear são **o mesmo número** aqui |
| `MaxWalkSpeedCrouched` | ✅ | `crouch_speed` |
| `JumpZVelocity`, `JumpMaxHoldTime` | 🟡 | autoramos **altura**, e a variável é por gravidade (Celeste), não por tempo de aperto |
| `JumpMaxCount` | ✅ | `air_jumps` (W-MultiJump) |
| `AirControl` | ✅ | `air_acceleration` |
| `AirControlBoostMultiplier` + threshold | ❌ | **§3.I** — nicety |
| `FallingLateralFriction` | 🟡 | o `air_acceleration` já freia sem input (documentado em `walk.rs`) |
| `MaxStepHeight`, `bCanStepUp` | 🟡 | **§4.1** — a perna sobe o degrau; sob Snap o `step_height` é **derivado** do `cling_distance` (`bridge/player.rs:491`) |
| `WalkableFloorAngle` | ✅ | `max_slope_deg` → `footing` |
| **`bCanWalkOffLedges`** | ❌ | **§3.G** |
| `PerchRadiusThreshold`, `PerchAdditionalHeight` | 🟡 | **§4.2** — empoleiramos por construção (leque de pés), mas não é AUTORÁVEL |
| `bImpartBaseVelocity{X,Y,Z}` | ✅ | `ground_velocity` |
| **`bImpartBaseAngularVelocity`** | ✅ | **§4.3** — `velocity_at_point` já inclui `ω × r` (`world/player.rs:119`) |
| `bIgnoreBaseRotation` | ⛔ | 2D com `LockRotation`: girar o personagem com a plataforma não é o idioma |
| `MovementMode` + **`MOVE_Custom`** + `OnMovementModeChanged` | ❌ | **§3.F** — é ARQUITETURA, e a recomendação é **não agora** |
| `MOVE_Swimming` | ✅ | W-Swim |
| `MOVE_Flying` / noclip | ❌ | **§3.H** — barato, valor baixo |
| **`LaunchCharacter`** | ❌ | **§3.E** — e o Unreal tem-no *precisamente porque* o controlador comeria a velocidade |
| **`OnLanded`, `bNotifyApex`** | ❌ | **§3.A** |
| `bEnablePhysicsInteraction`, `PushForceFactor`, `StandingDownwardForceScale` | ✅ | **e melhor**: a nossa é a 3.ª lei com três frações autoradas (W6, W-KinPush) |
| `GravityScale` | ✅ | componente por-corpo |
| **`APhysicsVolume::TerminalVelocity`** | ❌ | **§3.D** — e note-se que lá é propriedade do **VOLUME** |
| `MaxSimulationTimeStep`, `MaxSimulationIterations` | ✅ | sub-passos (W11b/W11c) |
| Root motion | ❌ | **§5.K** — fora da fila sem pedido |
| Network prediction / rollback | ⛔ | **§4.6** — e o que temos no lugar é mais forte para um editor |

### Godot — `CharacterBody2D` / `move_and_slide`

| deles | nós | onde / porquê |
|---|---|---|
| `floor_max_angle` | ✅ | `max_slope_deg` |
| `floor_snap_length` | ✅ | `cling_distance` (`0,25` no ponto de partida) |
| **`floor_constant_speed`** | ✅ | **§4.4** — de graça: o eixo é a **tangente** e o alvo é medido nela (`walk.rs:129`) |
| **`floor_stop_on_slope`** | ✅ | **§4.4** — de graça: parado, o alvo é `0` **relativo ao chão** na tangente |
| `up_direction` | ✅ | `up`, e a lei inteira é relativa a ele |
| `motion_mode` (Grounded/Floating) | 🟡 | temos três MODOS (Spring/Snap/Pure), que é outro eixo |
| `platform_floor_layers` / `platform_wall_layers` | ❌ | pequeno: *que camadas contam como plataforma* |
| **`platform_on_leave` (3 modos)** | ⚠️ | **§3.J** — sob Spring a física responde sozinha; **sob Snap é PERGUNTA ABERTA e o 1.º passo é MEDIR** |
| `slide_on_ceiling`, `wall_min_slide_angle`, `max_slides`, `floor_block_on_wall`, `safe_margin` | ⛔ | **§4.5** — são knobs do *solver de slide* do Godot; a nossa saída é força/velocidade para um solver, não uma iteração de deslize |
| **`is_on_floor/wall/ceiling`, `get_floor_normal`, `get_wall_normal`, `get_platform_velocity`, `get_real_velocity`, `get_last_slide_collision`** | ❌ | **§3.A** — o Godot expõe uma superfície de CONSULTA inteira; nós expomos três portas e nenhuma é sobre o estado |

### Unity — `CharacterController` + `Effector2D` + KCC

| deles | nós | onde / porquê |
|---|---|---|
| `slopeLimit` | ✅ | `max_slope_deg` |
| `stepOffset` | 🟡 | **§4.1** |
| `skinWidth`, `minMoveDistance` | 🟡 | o wrapper do controlador cinemático já os carrega |
| `isGrounded`, `velocity`, `collisionFlags` | ❌ | **§3.A** |
| **`OnControllerColliderHit`** / hits com estado (KCC) | ❌ | **§3.A** |
| `PlatformEffector2D` (one-way) | ✅ | W12 + `player_drops` |
| **`SurfaceEffector2D`** (esteira: *"forças tangentes para igualar uma velocidade ao longo da superfície"*) | ❌ | **§3.B** — a nossa esteira exigiria que a plataforma **se movesse de facto** |
| `BuoyancyEffector2D` | ✅ | W-Water / empuxo |
| `AreaEffector2D` / `PointEffector2D` | ✅ | zonas com força, torque, frame, espelho e falloff |
| **`PhysicsMaterial2D.friction`** a alcançar o andar | ❌ | **§3.B** — gelo |
| Personagem-contra-personagem (KCC) | 🟡 | sob Spring os dois são corpos dinâmicos e colidem |

### `bevy_tnua` — o **referencial declarado** deste módulo

A lista de features do README dele, confrontada:

| deles | nós |
|---|---|
| Running · Jumping · Crouching · Variable height · Coyote · Jump buffer | ✅ |
| Running up/down slopes/**stairs** | ✅ / 🟡 (§4.1) |
| Moving platforms · **Rotating platforms** | ✅ / ✅ (§4.3) |
| Jump/fall through platforms | ✅ |
| Air actions | ✅ (W-MultiJump) |
| Obstacle actions: **wall sliding (and jumping)** | ✅ (W13/W23) |
| Obstacle actions: **climbing** | ❌ — o buraco que o plano 08 §4.8 já nomeia |
| **Animation helpers** (*"não a animação, mas as facilidades para decidir que animação tocar"*) | ❌ — **§3.A** |
| Tilt correction | ⛔ 3D; em 2D o `LockRotation` resolve |

⚠️ **Contra o nosso próprio referencial sobram DUAS coisas**, e uma delas é a
mesma que as três engines pedem: **a saída para a animação**.

---

## §3 — Os achados, por alavanca

### 3.A · **O JOGADOR NÃO TEM SAÍDA** ⟨o maior, e não é um verbo⟩

**Medido:** a ponte publica entrada, gizmos e um bit de descida
(`bridge/player_channel.rs`). Não há *grounded*, não há *facing*, não há
*aterrou*.

**Quem tem:** o Godot expõe uma superfície de consulta inteira
(`is_on_floor/wall/ceiling`, `get_floor_normal`, `get_platform_velocity`,
`get_real_velocity`); a Unity expõe `isGrounded`/`velocity` mais
`OnControllerColliderHit`; o Unreal expõe `MovementMode`, `OnLanded`,
`bNotifyApex`, `OnMovementModeChanged`; e o **tnua lista *animation helpers* como
feature de topo**, com um tipo dedicado a decidir que animação tocar.

**O que isto custa hoje, concretamente:** não há como tocar um passo, levantar
poeira ao aterrar, virar o sprite para o lado da caminhada, trocar de animação,
ou ligar um pulo a um sinal da timeline. **Nada disso é lei** — é tudo o mesmo
buraco.

⚠️ **E o canal genérico que já existe é ESTRUTURALMENTE CEGO ao evento mais
importante.** O `SignalOnHit` é dirigido por **contato**
(`bridge/signals.rs` → `contacts::ContactPhase`), e um personagem de perna
flutuante **não toca no chão** — o nosso próprio código di-lo (*"sob Spring esse
número é AUTORADO … e o corpo de facto flutua ali"*, `world/character.rs`). Sob
Spring o `SignalOnHit` **nunca dispara** para o piso; sob Snap dispara. *O mesmo
verbo de gameplay funciona num modo e é inerte no outro, em silêncio* — a mesma
assimetria do §3.E.

**São DOIS canais, não um** — e este repo já pagou para aprender a distinção
([[feedback_a_transient_event_marker_is_its_own_channel]]):

* **A1 · O READOUT** — contínuo, por-frame: regime (chão/ar/parede/agachado/
  nado/beirada/arranque), `facing`, velocidade **relativa ao chão**, normal do
  chão. É um *estado*, lido por quem quiser, sem risco de ordem.
* **A2 · OS EVENTOS** — discretos: aterrou (com a velocidade do impacto), pulou
  (com **qual** pulo: chão, ar, parede), ápice, arrancou, agarrou a beirada,
  entrou/saiu da água. Publicados no **`SignalOutbox` do R0**, que é o consumidor
  que já existe.

⚠️ **A armadilha a nomear antes de construir:** a lei corre **por TIQUE** e um
dispatch pode dever vários. Um evento derivado por diferença entre dois frames da
shell **perde** os tiques do meio — e o replay do scrub reproduz o mundo, então
teria de reproduzir os eventos também. ⇒ *o evento nasce dentro do laço de
tiques, ao lado de quem o causou*, nunca de um diff lá fora.

**Preço:** o A1 é um `struct` e uma porta. O A2 é uma comparação de estado por
tique dentro do laço que já existe. **Nenhum toca a lei.**

### 3.B · **A SUPERFÍCIE NÃO FALA COM A LEI** ⟨gelo e esteira⟩

**Medido:** o chão contribui exactamente **dois** fatos — a `normal` e a
`ground_velocity` (`walk.rs:129`). Uma varredura por `friction` no crate da lei
dá **zero**.

**Quem tem:** a Unity liga `PhysicsMaterial2D.friction` ao movimento e ship a
**`SurfaceEffector2D`**, que é literalmente *"forças tangentes ao longo da
superfície para igualar uma velocidade"* — a esteira; o Unreal diz-lo na própria
doc do `BrakingDecelerationWalking`: *"pode ser usado para simular superfícies
escorregadias como gelo ou óleo"*.

**As duas metades:**

1. **Gelo/lama — atrito por superfície.** O collider já carrega `friction` e já
   tem `MaterialCombine`; a lei simplesmente **nunca o lê**. Um multiplicador
   por-superfície sobre o orçamento de aceleração/travagem é uma linha na
   `GroundSample` e um produto no `walk`.
2. **Esteira — velocidade imposta por superfície ESTÁTICA.** Hoje só é
   exprimível fazendo a plataforma **mover-se de facto**, o que ninguém
   constrói assim. É o mesmo canal: um `surface_velocity` que soma à
   `ground_velocity` na amostra.

⚠️ **É UMA wave, não duas:** as duas metades entram pelo mesmo sítio (o que a
amostra de chão carrega) e saem pelo mesmo (o alvo e o orçamento do `walk`).
⚠️ **E ela depende do §3.C**: *gelo* é um número de **travagem**, e enquanto
acelerar e frear forem o mesmo número, gelo é inexprimível sem também tornar o
personagem lento a arrancar — que é o oposto de gelo.

### 3.C · **ACELERAR E FREAR SÃO O MESMO NÚMERO**

**Medido:** `walk()` usa `cfg.acceleration` para os dois sentidos
(`walk.rs:128-151`). O fator de mudança de direção (`1 + |Δv|/(2·speed)`,
saturado em 2) cobre **inverter**, e não cobre **largar o direcional**.

**Quem tem:** todas — `MaxAcceleration` × `BrakingDecelerationWalking` +
`bUseSeparateBrakingFriction` + `BrakingFrictionFactor` (Unreal); accel × reduce
separados (o padrão do Celeste); `GroundFriction`.

**Porque importa:** é **o** knob do peso do personagem. Deslizar ao parar
(pesado, momentum) contra parar seco (preciso, twitchy) é hoje **inexprimível**
— e é a primeira coisa que um artista mexe.

**Preço:** um campo por regime (chão/ar) e um `if` no sinal de `delta`. É a
wave mais barata desta lista e provavelmente a mais sentida.

### 3.D · **NÃO HÁ TETO DE QUEDA**

**Medido:** não existe velocidade terminal em lado nenhum da lei. O único teto de
descida é o `glide_fall_speed`, e ele **só age enquanto se plana** — ou seja *o
mecanismo já está escrito*, escopado a uma ação.

**Quem tem:** o Unreal põe-no no **volume** (`APhysicsVolume::TerminalVelocity`);
todo platformer 2D o capa.

⚠️ **Não é cosmético, e o mecanismo é este:** a perna é um raio de alcance fixo e
o `corner_lookahead` mede-se em **tiques**. Quanto mais depressa se cai, mais
longe se anda num tique — e mais perto se fica de o sensor de chão não ver o piso
fino que se atravessa. Um teto de queda é também um teto no que os sensores têm
de alcançar.

**Preço:** um campo, e um clamp no mesmo sítio onde o planeio já clampa.
⚠️ **E há uma decisão de produto de graça ao lado:** o Unreal põe-no no volume, e
nós temos zonas — *cair na água devia ter outro teto*.

### 3.E · **NADA DE FORA CONSEGUE EMPURRAR O JOGADOR** ⟨knockback⟩

**Medido:** não há canal de impulso externo na lei. Sob **Spring** o corpo é
dinâmico e um `apply_impulse` chega (é assim que o empuxo e o arrasto de zona
agem — `kinematic.rs:59`), e o `walk` **resiste em vez de apagar** (o boost só
dispara dentro de `a·dt`, ~1,0 m/s por tique com a config de partida — está
medido no topo do `walk.rs`). Sob **Snap/Pure** o corpo é cinemático: um impulso
não faz **nada**, porque quem possui a velocidade é o `KinematicState`.

⚠️ **A assimetria é o achado:** *o mesmo verbo de gameplay — levar um empurrão —
funciona num modo e é silenciosamente inerte no outro.*

**Quem tem:** o Unreal tem `LaunchCharacter(vel, bXYOverride, bZOverride)`
**precisamente porque** o componente de movimento comeria a velocidade.

**Preço:** um campo de entrada (*este tique, some isto à minha velocidade*) que
os dois modos honram, mais uma janela de silêncio do controle aéreo — e essa
janela **já existe** (`wall_jump_lockout`), então não é primitivo novo.

### 3.F · **MODOS DE MOVIMENTO COMO COISA DE PRIMEIRA CLASSE** ⟨arquitetura⟩

O Unreal tem `MovementMode` + `MOVE_Custom`; o tnua é **basis + actions**, onde
um jogo acrescenta a sua própria ação sem tocar no motor. Nós temos **uma**
`player_motor` com treze leis em ordem fixa.

⚠️ **E a ordem fixa não é preguiça — é o que torna as interações demonstráveis.**
Os comentários de ordem no `lib.rs` são *load-bearing* (a trava do nado antes de
tudo o que ela silencia; a beirada **antes** do pulo, porque o mesmo aperto
significa *subir*; o botão gasto na ENTRADA e não descartado na saída). Uma
arquitetura genérica de ações troca isso por uma tabela de prioridade que ninguém
consegue ler.

**Recomendação: NÃO AGORA.** Fica registado como a pergunta do ADR-0075 ao nível
do player (*como é que um jogo acrescenta um gancho sem editar a lei?*), para o
dia em que houver um segundo jogo a pedi-lo.

### 3.G · **`bCanWalkOffLedges`** — não cair da beirada

O Unreal ship. Serve a IA e ao *andar com cuidado*. Pequeno e isolado: é um
veredito a mais no `footing` quando o leque de pés vê o chão acabar.

### 3.H · **Voar / noclip** (`MOVE_Flying`)

Barato, útil para percorrer um nível grande no editor, valor de produto baixo
num editor que já tem câmera livre.

### 3.I · **Air control boost** (`AirControlBoostMultiplier`)

Quando a velocidade lateral no ar é pequena, o controle aéreo é multiplicado —
tira a sensação de *"não consigo sair do lugar"* no topo de um pulo vertical.
Um campo, um `if`.

### 3.J · **`platform_on_leave` sob SNAP** ⟨e o 1.º passo é MEDIR⟩

O Godot tem três modos explícitos ao largar uma plataforma móvel:
`ADD_VELOCITY`, `ADD_UPWARD_VELOCITY` (que existe porque uma plataforma a
**descer** atirar-te-ia para baixo) e `DO_NOTHING`.

Sob **Spring** a pergunta não se põe: o corpo é dinâmico, já tem a velocidade, e
o `lift_momentum` (W10) resolve o problema *vizinho* — o referencial em que o
controle aéreo mede. Sob **Snap** quem possui a velocidade é o `KinematicState`,
e a `ground_velocity` não lhe é somada quando o chão desaparece.

⚠️ **Não afirmo que está partido — afirmo que não está medido.** A wave começa
por uma sonda: *largar uma plataforma a 4 m/s nos dois modos, e comparar a
trajetória*. Se divergirem, a cura é o canal do §3.E (é o mesmo: somar uma
velocidade de fora), não um caso especial.

---

## §4 — O que voltou VERDE pela medição ⟨tão valioso quanto os buracos⟩

*Isto está aqui para ninguém construir uma segunda resposta.*

**4.1 · Degrau (`MaxStepHeight`/`stepOffset`).** Sob **Spring** a perna sobe o
degrau por construção (é a razão de ser de um controlador flutuante). Sob
**Snap** o autostep do rapier está ligado e o `step_height` é **derivado** do
`cling_distance` (`bridge/player.rs:491`) — ⚠️ **de propósito**: *um número, sem
segunda resposta*. Se um dia se quiser autorar o degrau em separado da faixa de
colagem, é decisão de produto, não um buraco.

**4.2 · Empoleirar (`PerchRadiusThreshold`).** O leque de pés (W-FootFan) toma o
chão como o **mais próximo** de N raios, então o personagem fica de pé com um
único pé apoiado — empoleiramos **mais generosamente** que o default do Unreal.
O que lá é autorável (*quanto da cápsula pode ficar de fora*) aqui é
consequência do `foot_spread`.

**4.3 · Plataforma que GIRA.** A `ground_velocity` sai de
`velocity_at_point` (`world/player.rs:119`), que é `v + ω × r` — ou seja **a
rotação já é carregada**, e é por isso que o tnua lista *rotating platforms* e
nós a temos sem nunca lhe ter feito uma wave. O que não fazemos é **girar o
personagem** com ela (`bIgnoreBaseRotation`), e em 2D com `LockRotation` isso é
o certo.

**4.4 · `floor_constant_speed` e `floor_stop_on_slope`.** Os dois caem de graça
da decisão nº 1 do `walk.rs`: o eixo é a **tangente do chão** e o alvo é medido
**nela**, relativo ao chão. Subir uma rampa não abranda, e parado numa rampa o
alvo é zero — não há como escorregar. ⚠️ *São knobs que existem no Godot porque
lá a caminhada é horizontal e a rampa é resolvida pelo deslize; aqui a
representação apaga os dois.*

**4.5 · Os knobs de deslize do Godot** (`max_slides`, `wall_min_slide_angle`,
`slide_on_ceiling`, `floor_block_on_wall`, `safe_margin`) — ⛔ **divergência
deliberada**: descrevem a iteração do `move_and_slide`. A nossa lei entrega
**força e velocidade** a um solver (Spring) ou um deslocamento a um controlador
(Snap), e quem itera é quem sabe.

**4.6 · Determinismo e repetição.** Nenhuma das três engines ship o que temos: o
`physics_ecs_c9` (hash idêntico em três SOs), a **fita** do jogador (v67 do
schema) e o **bake** que a transforma em animação. O equivalente delas é *network
prediction*, que resolve outro problema. ⚠️ **Estamos à frente aqui**, e a
auditoria seria desonesta sem o dizer — como também o é nos **sensores visíveis e
editáveis** (W-Probes), que nenhuma delas oferece para o controlador de
personagem.

---

## §5 — Fora da fila, com motivo

* **K · Root motion.** Movimento dirigido pela animação. Temos timeline, então é
  exprimível — mas é um segundo dono da pose, e o repo já sabe o que isso custa
  ([[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]). Sem pedido.
* **L · Escalar (escada/corda).** O 5.º buraco do catálogo do plano 08 §4.8, e o
  tnua lista *climbing* nas *obstacle actions* — o que **confirma** aquele
  veredito em vez de o mudar. Regime com geometria própria; sem pedido.
* **F · Modos como arquitetura.** §3.F — recomendação explícita de não fazer.
* **Camadas de plataforma** (`platform_floor_layers`) — cai de graça no dia em
  que o filtro de camadas do personagem for autorável.

---

## §6 — A fila, com o racional de ordem

```
A (saída: readout + eventos) → C (frear ≠ acelerar) → B (a superfície fala)
   → D (teto de queda) → E (empurrão de fora) → J (medir o Snap) → G·H·I
```

**Porquê esta e não outra:**

1. **A é o único item que destrava OUTROS MÓDULOS.** Sem ele não há animação, som
   nem FX ligados ao personagem, e o consumidor (o `SignalOutbox` do R0) já
   existe e já está a ser drenado pela shell. **Não toca a lei**, então é também
   o de menor risco.
2. **C antes de B**, e é dependência dura: gelo é um número de **travagem**, e
   enquanto travar e acelerar forem o mesmo campo, *ice* e *lento a arrancar* são
   o mesmo ajuste.
3. **B** fecha as duas metades da superfície de uma vez (gelo e esteira entram
   pela mesma porta da amostra).
4. **D** é um campo e um clamp, com o mecanismo já escrito no planeio.
5. **E** precisa de uma decisão sobre simetria de modo — e **J** é a sonda que
   provavelmente a informa, por isso vem colada.
6. **G·H·I** são um campo e um `if` cada; entram a qualquer momento.

⚠️ **Se a fila for cortada, o corte honesto é depois de C.** A e C, juntos, são
*o personagem passa a poder falar* e *o personagem passa a poder ter peso* — as
duas coisas que as três engines têm e nós não, e as duas que um artista nota no
primeiro minuto.

⚠️ **Cada wave fecha com as QUATRO condições de UI do plano 00** e com uma cena
de smoke de números **medidos**. ⚠️ **O número da cena CONTA-SE no
`physics_smoke.rs`**, nunca numa nota (hoje o máximo é `=108`, e o `=84` não
existe de propósito).

---

## Fontes

* [UCharacterMovementComponent — Unreal Engine 5.8](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UCharacterMovementComponent?lang=en-US) · [unreal.CharacterMovementComponent (Python API)](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/CharacterMovementComponent)
* [CharacterBody2D — Godot (stable)](https://docs.godotengine.org/en/stable/classes/class_characterbody2d.html)
* [Effectors 2D — Unity Manual](https://docs.unity3d.com/Manual/class-SurfaceEffector2D.html) · [PlatformEffector2D](https://docs.unity3d.com/6000.0/Documentation/Manual/2d-physics/effectors/platform-effector-2d-reference.html)
* [bevy-tnua — README](https://github.com/idanarye/bevy-tnua) · [docs.rs](https://docs.rs/bevy-tnua/latest/bevy_tnua/) — o referencial declarado deste módulo

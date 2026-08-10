# Plano — o PLAYER CINEMÁTICO (o 2º modo)

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

### W-Water — O PERSONAGEM FICA DE PÉ EM CIMA DA ÁGUA (cena `=100`)

⚠️ **UMA cena serve as três waves do player dinâmico**, e não é economia: uma poça, uma
jangada a boiar e um personagem a pular nela mostram os três defeitos **ao mesmo tempo** —
ele não pode ficar de pé na água (`W-Water`), a jangada não pode subir ao encontro dele
(`W-ClingPull`), e o pouso não pode escorregar (`W-Landing`). Três cenas separadas fariam
o artista julgar cada um sem ver que eles se tocam.

⚠️ **Esta wave é do player DINÂMICO, não do cinemático** — ela entra aqui porque o Enio
a pediu neste plano (*"nosso player atual não interage corretamente com a água, como a
jangada faz"*) e porque **vem primeiro**: é um defeito VIVO no que shipa hoje, e o modo
novo herdaria metade dele.

⚠️ **E não é sobre água.** Medido pela porta do produto antes de uma linha
(`measure_player_in_water.rs`), com o CONTROLE sendo uma cápsula **idêntica** sem o
`PlatformPlayer` — mesma forma, mesma densidade, mesma poça:

| | cápsula (controle) | player | com a cura |
|---|---|---|---|
| **`y` numa poça FUNDA** | `0,2174` (bóia na linha d'água) | **`0,9023`** | `0,3446` |
| **`y` numa poça RASA** (fundo a `−2`) | `0,2174` | **`0,9023`** | `0,3446` |
| **`x` após 5 s de correnteza** (8 N) | levado | **`0,0000`** | `3,3927` |

**`0,9023` é `float_height`** — ele está **de pé sobre a superfície da poça**, à altura
exata a que a perna o segura sobre qualquer chão. E o `x = 0,0000` é o corolário: quem
está no chão tem a caminhada a frear a velocidade dele para o alvo (zero), então a
correnteza é **apagada**.

**A causa é uma linha, e o repo já escreveu a frase do outro lado.** O `cast_ray` do
wrapper monta `QueryFilter { .., ..default() }`, e `QueryFilterFlags::empty()` no rapier
significa *"no filter"* ⇒ **um SENSOR responde ao raio como matéria sólida**. O
`buoyancy.rs` já diz, sobre o outro eixo: *"um sensor não desloca fluido: um sensor é um
marcador, não matéria"*. **Se não é matéria, não se fica de pé em cima.**

⚠️ **E o alcance do defeito é TODA a família de zonas, não a água:** o mesmo raio
serve o sensor de chão, os DOIS de teto, o de headroom e o de parede — então o
personagem também bate a cabeça num gatilho e escorrega na parede de uma rajada de
vento. **O `cast_ray` tem exatamente cinco consumidores no repo inteiro, e os cinco são
este player, e os cinco querem matéria** (conferido por grep) — é por isso que a cura
mora na PORTA e não num parâmetro: não existe chamador que queira o contrário, e a
pergunta *"onde está o sólido?"* nunca significou outra coisa.

⚠️ **E ela vale para o modo novo:** o `move_shape` monta o próprio `QueryFilter`, e sem
a mesma exclusão o personagem cinemático seria **bloqueado** por um volume de gatilho —
o mesmo defeito, um grau pior.

#### A SEGUNDA metade — `W-Submerged`, FECHADA, e o plano acertou metade

Esta seção previa o defeito e **errou o tamanho dele**. Fica escrita como estava porque
o que ela acertou é o que tornou a wave decidível, e o que ela errou é a lição.

**O que o plano acertou.** Com a cura da `W-Water`, o player bóia a **`0,3446`** contra
os **`0,2174`** da cápsula idêntica — 12,7 cm mais alto —, e a causa é mesmo o
`peak_gravity`: boiar é viver na janela de velocidade quase-zero, ou seja **no ápice de
um pulo que ele nunca deu**. A consulta nova mede isso diretamente: um player boiando lê
`empuxo÷peso = 0,49`, que é **literalmente o valor do `peak_gravity`**.

**O que ele errou, e muda a natureza da wave.** Aquele desnível é cosmético; o defeito
real é que o personagem **sai de quadro**. Medido: largado dentro da poça ele oscila
`−1,05 / +4,71 / +12,08 / −20,31`. E a ablação nomeia um culpado que esta seção nem
cita — **`fall_gravity`**:

| ablação (largado de `y = 3`) | amplitude final |
|---|---|
| o que shipa | **14,55 m** (diverge) |
| `fall = 1`, o resto shipa | **0,11 m** |
| `peak = 1`, o resto shipa | 15,27 m (não ajuda) |
| `takeoff = 1`, o resto shipa | 14,55 m (inerte) |

> A modelagem do arco é **não-conservativa por construção**: subir com `g` e descer com
> `2·g` devolve o corpo ao mesmo nível com **`√2` da velocidade** — o dobro da energia,
> por ciclo. Num platformer isso é inofensivo porque todo arco termina absorvido pelo
> CHÃO. Sobre uma superfície que **restaura**, a ficção acumula.

⚠️ **E a cura óbvia — desvanecer a modelagem pela fração submersa — foi CONSTRUÍDA,
MEDIDA e é insuficiente:** ela cura o repouso (amplitude `0,0046` contra `0,0050` do
controle) e deixa o corpo largado de 1,5 m **divergindo a 11 m**, porque a energia é
ganha **no AR**, entre dois mergulhos, onde não há fluido nenhum a medir.

**Por isso não é decisão de produto, é uma LEI**, e ela não pergunta nada ao artista:

> *A modelagem do arco vale entre dois contatos com o CHÃO. Um corpo que o fluido tomou
> saiu desse arco, e só volta a ele quando pousa em algo sólido.*

Implementada como a trava `JumpState::waterborne` (o fluido arma, o chão desarma) —
`crates/ph2d-platformer/src/jump.rs`. **Resultado**, com a cápsula idêntica como
controle em toda linha:

| largado de | player y / amplitude | controle y / amplitude |
|---|---|---|
| 0,5 | 0,2279 / **0,0051** | 0,2280 / 0,0050 |
| 1,5 | 0,2321 / **0,0393** | 0,2287 / 0,0214 |
| 3,0 | 0,2514 / **0,1892** | 0,2362 / 0,0689 |
| 6,0 | 0,2744 / **1,1288** | 0,2592 / 0,2773 |
| −2,0 (dentro) | 0,2078 / **0,2772** | 0,2077 / 0,2770 |

⚠️ **A linha que prova que a cura é a certa é a última:** com a MESMA condição inicial
(os dois já submersos) player e cápsula são indistinguíveis à quarta decimal. O resíduo
das quedas altas é o `fall_gravity` a fazer o mergulho ser mais rápido — que é o que o
artista autorou, e decai.

⚠️ **Consequência NOMEADA:** raspar a água ao atravessar uma poça de um salto deixa o
resto daquela queda com a gravidade do MUNDO. É o preço de não ter um limiar mágico
entre *"encostei"* e *"estou dentro"*, e ele erra para o lado da física honesta.

**Gates, red-first:**

1. **`a_player_does_not_stand_on_water`** — nasce **VERMELHO** em `0,9023`; verde
   quando o `y` cair para dentro da poça. O oráculo é a **linha d'água do CONTROLE**,
   nunca um literal.
2. **`a_player_is_carried_by_a_current_like_a_raft`** — `x = 0,0000` hoje.
3. **`a_trigger_volume_is_not_a_ceiling`** — a outra face do mesmo raio, para a cura ser
   afirmada como propriedade da PORTA e não como conserto do chão.
4. **`a_floating_player_is_not_forever_at_the_apex`** — a metade B, com o número da
   ablação (`+0,1272` contra `+0,0000`).
5. **O que NÃO pode mudar:** o `=91` (a escada de pranchas), o `=88` (as rampas) e o
   `c9` — uma plataforma jump-through é um collider **sólido** com um hook, e nenhuma
   cena do hash tem sensor no caminho de um raio. Gate + `c9` byte-idêntico.

### W-ClingPull — A PERNA NÃO PUXA O CHÃO PARA CIMA (cena `=100`, a mesma)

Report: *"ao pular na jangada, ao primeiro toque, em vez de empurrar a jangada para
baixo, ela é ATRAÍDA para o player, e só depois de se aproximar é que recebe o impulso"*.

**Medido** (`measure_landing::measure_first_touch_on_a_raft`), jangada a boiar em
repouso, personagem caindo de 3 m: no instante em que o sensor a alcança (`dist =
1,0758`, alcance `1,15`) ela **SOBE de `0,1634` a `0,2738` — 96,9 mm** — e só começa a
descer quando `dist` cruza a altura de repouso (`0,9`).

**A causa está escrita no próprio código, uma linha acima do sítio:**

```rust
// Positivo = está BAIXO demais e a mola empurra para cima; negativo = está
// alto demais dentro do `cling_distance` e ela PUXA para baixo.
let offset = cfg.float_height - s.distance;
```

Na faixa de *cling* o `offset` é **negativo** ⇒ a perna puxa o personagem para baixo ⇒ a
3ª lei transmite fielmente o oposto e **puxa a jangada para cima**.

⚠️ **A cura é uma frase sobre o que uma perna É:** o *cling* é uma **conveniência de
modelagem** (é o que mantém o personagem colado ao descer uma lomba), não um músculo. Uma
perna real **não puxa o chão para si**. Então a reação transmite **só a metade que
EMPURRA** — `offset > 0` — e a puxada fica sendo o que sempre foi: um truque sobre o
personagem, invisível para o mundo.

⚠️ **E ela não é só sobre jangadas:** o mesmo se vê numa gangorra (a ponta sobe antes de
descer) e num caixote solto. A água só a torna óbvia porque o corpo é leve e livre.

**Gates, red-first:** a jangada **nunca sobe** acima do repouso durante uma aterragem
(nasce vermelho em `+96,9 mm`) · a metade que EMPURRA continua a chegar, com o número do
`W-KinWeight` como controle · e a mutação que devolve o `offset` cru à reação sangra o
primeiro sem tocar o segundo.

### W-Landing — O POUSO PARA DE ESCORREGAR (cena `=100`, a mesma)

Report: *"a desaceleração ao encostar no chão é muito lenta e fica artificial"*.

**Medido** (`measure_landing::measure_landing_profile`, queda de 3 m nos defaults):
toque no tique 39, repouso no **69** — **30 tiques, 0,500 s** —, e o perfil **não é uma
parada, é um decaimento**: `−0,258 · −0,229 · −0,204 · −0,181 …`, razão **0,888 por
tique**. Ele nunca chega; ele se aproxima para sempre. E **não afunda** (mais baixo
`0,9023` contra repouso `0,900`), então não é quique — é **arrasto**.

**A causa é aritmética, e o número previsto bate com o medido:** com `damping = 1,0` — o
que shipa — o boost apaga a velocidade relativa INTEIRA a cada tique, então o único
movimento que sobra é o que a mola produz em UM tique, e o resto decai por

```text
1 − k·dt²  =  1 − 400/3600  =  0,889          (medido: 0,888)
```

⇒ **quem manda no pouso é a RIGIDEZ, e o amortecimento não.**

| `spring_strength` (com `damping = 1,0`) | tiques | segundos | afunda | `1 − k·dt²` |
|---|---|---|---|---|
| **400 — o que shipa** | 30 | **0,500** | −0,2 cm | 0,889 |
| 800 | 18 | 0,300 | −0,1 | 0,778 |
| 1200 | 13 | 0,217 | −0,1 | 0,667 |
| 1600 | 10 | 0,167 | −0,1 | 0,556 |
| **2000** | **8** | **0,133** | −0,0 | 0,444 |
| 3000 | 5 | 0,083 | −0,0 | 0,167 |

⚠️ **O caminho ÓBVIO é o pior, e a varredura o mostra.** Baixar o amortecimento também
acelera o pouso (`0,50` dá os mesmos `0,133 s`) — mas ele é o knob que a **W11c** pôs no
teto para zerar a deriva de rampa, e a lei publicada diz o preço: `deriva(10 s) = 0,153 ·
sen θ · (1 − d)`, ou seja `0,0382 m` a 30° se voltar para `0,50`. **A rigidez compra o
mesmo pouso por ZERO de deriva** — não há troca a fazer.

| | pouso | deriva 30°/10 s |
|---|---|---|
| hoje (`k = 400`, `d = 1,0`) | 0,500 s | 0,0000 m |
| baixar o amortecimento (`d = 0,50`) | 0,133 s | **0,0382 m** |
| **subir a rigidez (`k = 2000`, `d = 1,0`)** | **0,133 s** | **0,0000 m** |

⚠️ **O que a rigidez CUSTA não está medido, e é isso que a wave tem de fechar:** `2000`
é **5× o valor do `bevy-tnua`** que o ponto de partida cita, e uma perna rígida
**transmite o terreno** — um degrau pequeno vira um solavanco, e a reação da 3ª lei chega
mais forte à jangada. **Por isso esta wave vem DEPOIS da `W-ClingPull`:** ela amplifica
tanto o empurrão quanto a puxada, e a puxada é a que não devia existir.

**Gates, red-first:** o pouso assenta em menos de `0,20 s` (nasce vermelho em `0,500`) ·
**não afunda** abaixo do repouso (o que protege contra "curar" isto por baixo do
amortecimento) · a **deriva de rampa continua `0,0000`** (é o que separa esta cura da
tentadora) · e a razão de decaimento medida bate com `1 − k·dt²`, que é o gate que impede
alguém de re-derivar o número por tentativa.

### W-KinMove — O MODO EXISTE E O PERSONAGEM ANDA (cena `=101`) — **FECHADA (2026-08-08)**

O componente, o `Support::Snap`, o `move_shape`, a velocidade no `PlayerState`, a
plataforma móvel, e o chip da §14. **A MEDIÇÃO derrubou três coisas deste plano,
e as três ficam escritas porque a próxima pessoa faria as mesmas.**

#### 1. A RÉGUA da perna — *"absorva a gravidade quando a `footing` disser chão"*

A lei absorve a componente que aponta para o chão, e sem ela um personagem
**parado numa rampa de 30° desliza `0,0279 m` em 10 s** (é o `floor_stop_on_slope`
do Godot, que shipa **ligado**; e a deriva é insensível ao limite de rampa —
não é o *auto-slide* do rapier, é o slide genérico do deslocamento pedido).

⚠️ **Mas a régua da `footing` é a da PERNA** (`float_height + cling_distance`),
calibrada para uma cápsula que **paira**. Sob Snap não há perna, e absorver a
esse alcance **congelava o personagem a 0,4 m NO AR**, exatamente onde nasceu,
com todos os outros gates verdes (a caminhada andava, a rampa não derivava).

A cura foi **corrigir a régua, não tirar a absorção**:
`PhysicsWorld::body_foot_distance` — *onde os pés deste corpo de facto ficam* —
substitui o `float_height` autorado sob Snap.

#### 2. DUAS perguntas, e a `footing` só responde UMA

A régua corrigida ainda deixava o personagem pendurado a **1,237 m** com o chão
em **1,000**: a faixa do `cling` existe para o gesto não morrer num degrau, e ela
não é *"encostei"*.

⇒ `KinematicState.grounded` passa a vir do **CONTROLADOR**. É a pergunta do
**INTEGRADOR** (*"há algo a segurar-me AGORA?"*), distinta da pergunta da **LEI**
(*"posso pular?"*), que continua a `footing` nos dois modos — e a K4 sobrevive
intacta, agora com arch-gate que afirma a PROPRIEDADE (a lei recebe a amostra do
cast; o `grounded` do controlador tem exatamente UM leitor, o `kinematic_settle`).

#### 3. A BARRA do gate de impacto — *"zero por construção"*

O plano prometia zero. Ele afunda **4,7 cm**, e os 4,7 cm são a **PELE do
controlador** (`predict_ground = offset + 0.05`, o `skinWidth` da Unity).

⚠️ **Ela NÃO cresce com a queda** — `0,044 / 0,012 / 0,047 / 0,047` para quedas
de `0,5 / 2 / 5 / 10 m`, contra `0,052 / 0,149 / 0,261 / 0,296` do dinâmico.
*Estrutural* não quer dizer *zero*: quer dizer **limitado por uma constante da
geometria em vez de pela energia do impacto**, e o oráculo passou a medir o
**CRESCIMENTO**. Uma barra absoluta media a pele.

#### O que a UI custou, e o defeito que ela quase shipou

O chip escreve **os dois campos** por uma porta (`PlayerMode` + `RigidBody.kind`).
⚠️ E a quarta condição de UI falhava **dos dois lados**, por duas cópias da mesma
pergunta: `build_player_info` e `apply_player_edit` escreviam `kind == Dynamic`
cada um por sua conta, então clicar `Kinematic` fazia a §14 **desaparecer** e o
clique de volta era **recusado** pela outra cópia. Uma porta só
(`player_section_applies`) fecha as duas metades.

#### Consequências NOMEADAS, não escondidas

- **Os dois modos repousam a alturas diferentes** (`1,400` contra `1,057`): o
  dinâmico PAIRA por desenho (a D1) e o cinemático POUSA. Trocar de modo move o
  personagem ~34 cm para baixo, e isso é o que o modo É.
- **A caminhada é IDÊNTICA** (`11,830 m` em 2 s nos dois, a três decimais).
- A 3ª lei (K6) já corre nos dois modos, pela MESMA porta: sob Snap o `push` é
  zero e zero **já É o peso**. O que a `W-KinWeight` ainda deve é a **massa
  AUTORADA**.

**Números:** registro `ph2d-physics-ecs` **28 → 29** · `PROJECT_SCHEMA`
**intocado (60)** · `physics_ecs_c9` **byte-idêntico** (`dd5230d7…`, 108 corpos) ·
nenhum ADR · zero `Cargo.toml` · **10 mutações, 9 sangram** (a sobrevivente é o
`continue` do `drive_kinematic`, inerte HOJE pela ORDEM — documentada no sítio).

#### ⚠️ O SMOKE de 2026-08-08 reprovou a cena, e achou um defeito de LEI

Dois reports, e eles têm causas DIFERENTES — um é da cena, o outro do produto.

**(a) *"O ciano está com uma mola extremamente exagerada, um pula-pula"*** — e
estava certo: **5913 mm** de quique, medido. ⚠️ **A causa é a cena, e ela precisa
de TRÊS condições ao mesmo tempo** (`probe_scene_101::probe_what_makes_the_bounce_29_metres`):

| rampa | berço | `damping` | quique |
|---|---|---|---|
| plano | fora do repouso | 0,25 | **0,0 mm** |
| rampa | no repouso | 0,25 | **0,0 mm** |
| rampa | fora do repouso | 0,50 | **0,0 mm** |
| **rampa** | **fora do repouso** | **0,25** | **5913 mm** |

O gatilho era o **BERÇO**: a cena punha o ciano a `0,334 m` da rampa quando a
perna dele repousa a `0,900` — meio metro de mola **comprimida**, que vezes a
rigidez de 2000 é uma catapulta de `1132 m/s²`. O amortecimento a ¼ não a engole;
o do teto engole.

⚠️ **A lição não é sobre o valor `0,25`** — sozinho ele dá zero. É sobre a
FRONTEIRA: *o número escolhido para uma FIXTURE não atravessa para a mão do
artista.* Lá ele deixa de ser *"o valor que faz o fenômeno aparecer"* e passa a
ser *"o que este produto é"*. Gate:
`shells/desktop/tests/a_smoke_scene_ships_the_default_tuning.rs`.

**(b) *"O laranja ao pousar se aproxima da rampa … como se fosse atraído por uma
força cuja direção é a normal da rampa"*** — ⚠️ **e a seta que ele desenhou É o
mecanismo, literalmente.**

Com `drive = 0` a `walk` cancela a velocidade **ao longo da tangente do chão**.
Uma queda vertical tem componente tangencial em qualquer inclinação, então o
freio a lê como escorregão e a apaga — e **o que sobra de uma queda vertical sem
a tangente é a NORMAL**. O personagem deixa de cair para baixo e passa a entrar
na rampa perpendicularmente. Ablação pelo knob `acceleration`, que desliga o
freio pela porta do artista: **com freio `−0,0711 m`, sem freio `+0,0001 m`** — o
freio é a causa inteira.

A cura é de **ORDEM, não de lei**: o `settle` deixa no estado a queda que o mundo
bloqueou, o `kinematic_advance` a apaga — e **entre os dois corria a LEI**. Hoje
a ponte chama a MESMA porta (`supported_velocity`, agora `pub` com dois
consumidores) antes de a lei ler.

| queda | antes | depois |
|---|---|---|
| 0,5 m | −0,102 m | **−0,068 m** |
| 1,5 m | −0,071 m | **−0,044 m** |
| 3,0 m | −0,057 m | **−0,023 m** |

⚠️ **O modo DINÂMICO fica byte-intocado** (`+0,1469 / +0,3079 / +0,3905` antes e
depois): a correção vive no ramo `writes_own_pose`.

⚠️ **E o RESÍDUO está nomeado:** o tique de **CONTATO** ainda dá um chute, porque
ali o personagem está mesmo no ar e nenhuma absorção é devida. O que a correção
remove são os tiques SEGUINTES — medido no deslocamento **depois** do contato,
`4,4 mm` contra `39,0 mm` (**8,8×**), e é esse o oráculo do gate. Fechá-lo por
inteiro exigiria a `walk` perguntar ao **controlador** se está apoiada, o que
quebra a K4 — decisão de produto, não dívida mecânica.

##### ⛔ O parágrafo acima está ERRADO nas DUAS metades (smoke de 2026-08-09)

O Enio voltou com foto: *"o laranja quando pousa na rampa ainda se desloca um
pouquinho para cima"*. Ele estava certo, e as duas afirmações do parágrafo caem:

1. **Não era resíduo pequeno.** O gate media o deslocamento **depois** do
   contato porque a sua janela começava *no* tique em que o `dx` aparece —
   `else if dx.abs() > 1e-4 { touched = true }` marca o contato e **não soma o
   `dx` desse tique**. Ele media a CAUDA. Pela sonda da cena 101, o pouso
   inteiro desliza **22,8 mm**, e **17 deles são o tique do contato**: *uma
   janela que começa depois do evento não mede o evento*.
2. **A cura não toca a K4.** Ela não põe a `walk` a perguntar ao controlador —
   ela dá à lei a velocidade certa, e quem decide *"há chão?"* passa a ser o
   **`footing`**, que a K4 já nomeia como a resposta da lei nos dois modos. O
   defeito era a absorção consultar `was.kin.grounded`: a pergunta do
   **INTEGRADOR** (*"eu TOQUEI no mundo?"*), respondida pelo tique **ANTERIOR**,
   que no contato ainda diz *no ar*. O `footing` já respondeu quando a lei
   corre — o cast lê a pose de **depois** do `settle`.

**A cura é de ORDEM outra vez:** o bloco que escolhe `vel` desceu para **depois**
do `let stand = footing(...)`, e pergunta `stand.is_some()`. Medido pela mesma
sonda, largando na vertical sobre a rampa estática:

| | deslize do pouso | dinâmico afunda | c9 |
|---|---|---|---|
| antes | **22,8 mm** (uphill) | 0,0000 | `dd5230d7…` |
| depois | **0,0 mm** — cai reto | 0,0000 | `dd5230d7…` |

⚠️ **Os dois gates que deviam ter pego isto estavam VERDES, e um deles pinava o
defeito:** o arch-gate `the_law_is_handed_the_velocity_the_ground_already_holds`
afirmava `arm.contains("was.kin.grounded")` — com a prosa a chamar-lhe *"a porta
COMPARTILHADA com o integrador"* — **num arquivo chamado
`the_law_asks_footing_not_the_controller.rs`**. Hoje ele exige `stand.is_some()`,
proíbe o `was.kin.grounded` e afirma o degrau novo (`footing` resolvido **antes**
da absorção); o comportamental virou
`the_kinematic_landing_does_not_slide_along_the_ramp_normal`, cujo oráculo não
precisa achar tique nenhum: *largado na vertical sobre rampa estática, todo
deslocamento lateral é o defeito*. As duas mutações sangram (22,8 mm · o gate de
ordem).

#### ⚠️ E a §0 deste plano ganhou a coluna que lhe faltava

A tabela do topo mede o **dinâmico** no default e conclui, corretamente, que o
modo novo *"não conserta o que já é zero"*. O que ela nunca disse é o número do
**cinemático** no mesmo default (`probe_what_differs_at_the_shipping_default`):

| queda | afunda SPRING (default) | afunda SNAP (default) |
|---|---|---|
| 0,5 m | **0,0000 m** | 0,0436 m |
| 2,0 m | **0,0000 m** | 0,0124 m |
| 10,0 m | **0,0000 m** | 0,0465 m |

⇒ **No default que shipa, o cinemático afunda ~4,5 cm onde o dinâmico afunda
zero.** Isto não revoga a justificação do modo — ela nunca foi *"afunda menos"*,
foi *"o número não depende de knob nenhum e não cresce com a queda"*, e as duas
metades continuam medidas. Mas a frase honesta é: **hoje o modo cinemático troca
4,5 cm de pele por independência de afinação**, e quem decide se esse é um bom
negócio é o Enio, com a cena `=101` passo 4 na mão.

#### O plano ORIGINAL desta wave, para referência

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

### W-KinWeight — O CHÃO SENTE O PERSONAGEM (cena `=102`)

A K6 no eixo vertical: sob Snap o `support` é o peso, e a jangada afunda.

**Gates:** a jangada da cena `=72` no modo cinemático afunda **o mesmo** que com o corpo
dinâmico (a comparação é o oráculo, e é honesta porque a força física é a mesma) ·
`Reaction Scale = 0` continua a desligar tudo · a mutação que zera o suporte sob Snap
sangra.

⚠️ **A massa é AUTORADA** (o corpo cinemático não tem massa que o rapier calcule) — e é
por isso que esta é wave própria: um número novo na §14 tem as suas quatro condições.

### W-KinPush — E O QUE ESTÁ AO LADO TAMBÉM (cena `=102`) — **FECHADA**

A outra metade da ordem *"influencie tudo"*: o que o `move_shape` **não**
conseguiu mover, projetado na normal, volta como impulso no ponto de contato.

⚠️ **A cena é a `=102` e não a `=103`:** a `W-KinWeight` fechou sem cena (ela
virou uma wave de UI — ver a §6 dela), e deixar um buraco no roteador para uma
cena que nunca existiu é como um número de smoke passa a apontar para outra
coisa em silêncio.

**Medido, ANTES de uma linha ser escrita** (`measure_kinematic_push.rs`), num
personagem a andar contra um caixote solto por 3 s:

| modo | caixote (m) | personagem (m) |
|---|---|---|
| DINÂMICO (controle) | 16,55 | 17,55 |
| CINEMÁTICO | **0,0000** | 0,99 |

E depois: **16,54** contra os 16,55 do controle, com a ordem da MASSA preservada
(densidade 0,25 / 1 / 4 / 16 → 16,96 / 16,54 / 15,08 / 2,90).

#### A lei, e o que a torna imune ao sinal da biblioteca

`transfer = n · (blocked · n)`, com `blocked = pedido − efetivo`. A projeção é na
**LINHA** da normal, então **o sentido de `n` não importa** — e isso não é
elegância, é blindagem: qual das duas testemunhas de um shapecast produz a
normal é convenção do rapier, e uma lei que dependesse dela seria um sinal
invertido à espera de uma atualização de dependência. (O próprio rapier usa a
mesma forma no `solve_character_collision_impulses`.)

A conversão para impulso sai pela **porta da reação vertical**
(`apply_player_reaction`, renomeada de `apply_ground_reaction` porque agora ela
tem dois chamadores e nem todo corpo empurrado é chão): `m_player · transfer/dt`,
no ponto de contato — que é o que dá o **torque** e faz um caixote atingido em
cima tombar.

#### ⚠️ TRÊS achados que mudaram o desenho ou a leitura

1. **O peso NÃO é contado duas vezes, e isso foi MEDIDO em vez de assumido.** A
   preocupação era o canal lateral entregar ao chão o que a K6 já entrega. Numa
   jangada sem peso próprio, `push 0` e `push 1` dão a MESMA aceleração
   (`−0,1196` nas quatro células da tabela, parado e a andar) — porque a
   `supported_velocity` já removeu a componente que aponta para o chão antes de
   o deslocamento ser pedido. **Nenhum filtro foi preciso.**
2. **NÃO HÁ RESSONÂNCIA, e a razão é estrutural.** O risco nomeado era *empurra,
   o caixote foge, o slide segue, empurra outra vez*. Medida a folga em regime
   com o caixote SOLTO e com ele **encostado numa parede** (o caso duro, massa
   efetiva infinita): amplitude **0,0000 m** nas quatro combinações. O que volta
   é o que foi **BLOQUEADO**, então um caixote que foge deixa de bloquear e o
   empurrão cai na mesma medida — a lei é auto-limitada por construção.
3. ⚠️ **E é essa mesma propriedade que torna a CONTAGEM DUPLA invisível.** Duas
   mutações (somar em vez de dedupar; não limpar a lista de contatos)
   **sobreviveram a todos os gates de comportamento**. Instrumentada, a segunda
   mostra o segundo personagem a empurrar o caixote do primeiro com **0,0235
   m/tique contra os 0,0015 dele** — dezasseis vezes — e o caixote viaja
   **0,9039 m nos DOIS casos, aos quatro decimais**: o excesso é absorvido pelo
   bloqueio a menos. ⇒ o preço real de não dedupar é **CUSTO** (`O(N²)` nos
   players), e os gates disto são os de UNIDADE, nunca um de mundo.

#### O knob

`ReactionConfig.push` (o terceiro escalar da 3ª lei) + `PlatformPlayer.reaction_push`
→ **`PROJECT_SCHEMA` 69→70** (campo de componente serializado; PROVISÓRIO até a
integração). Nasce em **1,0**, com o `support` e ao contrário do `movement`:
empurrar o que se esbarra é o que um corpo faz.

⚠️ **A row é INCONDICIONAL e o rótulo diz o escopo** (*"KINEMATIC only: a dynamic
body already pushes"*), em vez de aparecer e sumir com o modo. É o precedente que
a própria §14 já tem em quatro números (`Lift Momentum` é inerte em chão
estático, `Crouch Speed` sem `crouch_height`, `Wall Reach` sem `Wall Slide`): a
seção escreve o escopo no rótulo, porque um controle que some é mais difícil de
aprender que um que explica onde age.

#### Os gates

Comportamento (`player_push.rs`, 8): empurra com o dinâmico ao lado como controle
· a massa do CAIXOTE manda · a massa do PERSONAGEM manda · o escalar é um **dial,
não um interruptor** · a parede estática não cede e ele PARA nela · o chão não é
empurrado duas vezes · não ressoa · um caixote deixado para trás não é puxado de
volta. Unidade: 4 na lei pura, 5 na dedup, 1 na porta do controlador. Arch-gate
`the_push_is_our_law.rs`: o solver aproximado da biblioteca **não é chamado**, o
empurrão sai pela porta da reação, e o que viaja é o bloqueio do **tique
inteiro**.

**7 mutações, 7 sangram** — e **duas delas só depois de um gate NOVO**: a que
apaga a escala (o early-out em `push == 0` sobrevive, então o canal ainda liga e
desliga, e nenhum gate autorava um valor no MEIO) e a que não limpa a lista de
contatos.

**c9:** lane nova (`C9 Push Player` + caixote pesado + chão), **111 corpos**,
hash `adb72352…` (debug ≡ release). ⚠️ A sensibilidade dela foi **provada por
ablação**: com `reaction_push = 0` o hash muda — sem isso seria uma lane que diz
cobrir um ramo que ela não atravessa.

### W-KinPure — O TERCEIRO MODO — **FECHADA** (cena `=103`)

O *"puro sangue"* que o Enio nomeou: nada de reação, nada de empurrão — o platformer
clássico, em que o mundo físico é cenário. ⚠️ **Ele é BARATO por construção depois das
duas waves acima** (é o mesmo caminho de movimento com a reação e o empurrão calados) e
**é por isso que ele não vem primeiro**: construí-lo antes faria a K6 parecer um caso
especial dele, quando é o contrário — *o pobre é o rico com dois canais desligados*.

#### O que a MEDIÇÃO disse antes de uma linha ser escrita

`measure_kinematic_pure.rs`, com a terceira coluna obtida do jeito que ela era
alcançável ANTES da wave: o modo Snap com os três escalares da reação a zero.

| modo | jangada | caixote | andou | pulo | levado / plataforma | caixote para em |
|---|---|---|---|---|---|---|
| dinâmico | −0,4764 | 16,5537 | 11,8302 | 2,1017 | 3,9527 / 4,0000 | +0,4425 |
| **CINEMÁTICO** | −0,4205 | 16,5383 | 11,8300 | 2,0421 | 7,9194 / 4,0000 | +0,4990 |
| cine + zeros | **0,0000** | **0,0000** | 11,8300 | 2,0421 | 7,9194 / 4,0000 | +0,4990 |

⚠️ **E ela decidiu o TAMANHO da wave, que é para o que uma sonda serve:** a terceira
coluna já dava o comportamento inteiro do platformer clássico ⇒ **o modo NÃO é
capacidade nova**. Ele é uma declaração de intenção com **uma porta**, e a wave diz isso
em vez de vender o contrário. O que ele acrescenta de facto é o que os zeros não podem
dar:

1. **sobrevive a alguém mexer num slider** — um modo é uma decisão, três zeros são uma
   coincidência;
2. **o painel pode NÃO OFERECER** o que não é lido (o card REACTION some, e a row de
   massa da W-KinWeight junto);
3. **um lugar só** para a pergunta, em vez de três números que teriam de concordar.

⚠️ **Duas coisas que a tabela derrubou, e que teriam virado gates errados:**
*"cenário quer dizer que o mundo não o vê"* — não: a coluna do caixote lançado é
**positiva nos três modos**, ele é SÓLIDO, e um platformer clássico não atravessa
caixas. E *"ser levado é influenciar"* — não: a plataforma o carrega igual, e calar esse
canal teria sido calar o errado.

#### O desenho

**`PlayerMode::Pure`** (tag 2), e a lei mora em **duas portas, uma pergunta**:

- `PlayerMode::transmits()` — *o que ele faz ao mundo volta para o mundo?*
- `PoseOwner::Player(PlayerMode)` — o modo **viaja dentro** da resposta de posse.

⚠️ **A segunda é o que impede o defeito silencioso:** perguntar `world.get::<PlayerMode>`
outra vez seria uma leitura que **não passa pela reconciliação com o `BodyKind`**, e um
`Pure` num corpo `Dynamic` calaria a 3ª lei de um player que a ponte está a simular como
dinâmico. É a discordância que o `pose_owner` existe para tornar impossível.

⚠️ **E o silenciamento é UMA linha, no `cfg`** (`cfg.react = ReactionConfig::OFF`), de
propósito: as duas metades da 3ª lei saem dali — o PESO (a `reaction`) e o EMPURRÃO (o
`KinMove.react`, copiado do mesmo `cfg`). Calá-las num ponto é o que impede uma wave
futura de acrescentar uma terceira metade e esquecer-se de a calar.

⚠️ **O modo CALA, não APAGA:** os escalares autorados ficam no componente, e voltar ao
Kinematic devolve o que o artista escreveu. Zerar os knobs no chip perderia trabalho em
silêncio.

#### O que a wave moveu fora dela

⚠️ **A nota da W-KinWeight foi RECONFERIDA** (CLAUDE.md §0 — *quem move o número
reconfere a nota*): ela abriu a row de massa para *"um player cinemático"*, e naquele dia
isso **era** *"alguém lê a massa"*. Sob o puro sangue nada a lê, então o `mass_is_read`
ganhou a terceira condição — sem ela, o toggle Auto/Manual voltaria a ser o controle
morto que aquela wave existiu para curar.

#### Gates

6 no crate (`tests/player_pure.rs`, cada um com CONTROLE) · 1 na porta de posse ·
2 de painel (presença **e** ausência do card) · 1 de gesto na shell (a quarta condição
de UI) · 1 do `mass_is_read` através dos três modos · 4 headless na cena.
**3 mutações, 3 sangram** — silenciar nunca (3 gates), `Pure` transmitir (3), `Pure`
não dirigir a própria pose (2, e são justamente os que dizem *é o mesmo controlador*).

⚠️ **`physics_ecs_c9` INTOCADO** (`adb72352…`, 111 corpos, debug ≡ release) — e a
ausência de lane é deliberada, com o motivo escrito no próprio harness: o `Pure` não
acrescenta aritmética, ele é a **ausência** de dois termos que as lanes de lá já
atravessam. Uma lane dele moveria o hash sem cobrir um `f32` novo em três OSes, e o
hash idêntico é o que torna verificável a promessa de byte-neutralidade.

**Zero schema, zero componente novo, zero dep.** Smoke: **`PH2D_PHYSICS_SMOKE=103`**.

### W-KinCarry — A PLATAFORMA ERA CONTADA DUAS VEZES (K7) — **FECHADA**

Não é feature: é um defeito que a sonda da `W-KinPure` mediu de passagem e que eu
registrei como aberto em vez de fechar. Um vagão andava **4,00 m** e o personagem
cinemático era levado **7,92**, enquanto o dinâmico media 3,95.

**A atribuição saiu de uma ablação de DUAS variáveis** (o eixo × a tração, esta
última pela porta do ARTISTA — `PlatformPlayer::acceleration`), cada célula com o
seu controle (`tests/measure_kinematic_carry.rs`):

| eixo | modo | tração | levado | razão |
|---|---|---|---|---|
| horizontal | dinâmico | cheia | 3,9527 | 0,99× |
| horizontal | dinâmico | **zero** | 0,0000 | **0,00×** |
| horizontal | CINEMÁTICO | cheia | 7,9194 | **1,98×** |
| horizontal | CINEMÁTICO | **zero** | 3,9666 | 0,99× |
| vertical | qualquer | qualquer | ~4,0 | ~1,00× |

⚠️ **A linha `dinâmico / zero` é a que fecha:** desligada a tração, o modo dinâmico
**não é levado de todo** — logo quem carrega é a **caminhada**, e só ela. A `walk`
mede tudo relativo ao chão e empurra `body_velocity` até o referencial dele; o
`kinematic_advance` somava `ground_velocity` **outra vez**. O eixo vertical nunca
teve o problema porque o eixo da caminhada é a **tangente**.

**A cura** é a porta `ph2d_platformer::ground_carry` — *o que o chão ainda deve* =
`g` menos a projeção no **MESMO** `perp_cw(normal)` que a caminhada usa. Ela tem
sentido físico completo: um elevador leva pelo **contato** (a normal, sempre paga) e
uma esteira leva por **atrito** (a tangente, que a tração modela) — sem tração o
chão liso deixa de arrastar de lado e continua a levantar, que é o que o modo
dinâmico sempre fez. ⚠️ E o parâmetro do `kinematic_advance` passou a ser a
**AMOSTRA**, não um `Vec2`: passar o número cru de novo é inexprimível por tipo.

**Depois:** toda célula horizontal bate com o controle dinâmico **até a quarta
decimal** (3,9527/3,9527 · 0,0000/0,0000 · coast 0,0472/0,0472).

⚠️ **O gate anterior era uma barra de UM LADO SÓ** — `travelled > 3.0` sobre uma
plataforma que anda 4,0, tão contente com 8 quanto com 4, e é por isso que o
defeito viveu. O oráculo novo é **o outro MODO** (a lei é a mesma, só muda quem
escreve a pose), o que o torna imune a re-afinações da tração que moveriam qualquer
literal pinado.

⚠️ **E o c9 GANHOU lane, ao contrário da `W-KinPure`** — a razão é uma ausência que
ninguém tinha nomeado: **nenhuma lane do harness tinha plataforma móvel**, logo
`point_velocity` nunca saía de zero em três OSes, e ela é a entrada de um número do
rapier que o nosso código projeta e transforma numa pose. A laje é **dinâmica e
flutua** (`GravityScale(0)` + `InitialVelocity`) porque o laço do harness não
escreve na cena. Hash **`cf900d0a…`, 113 corpos**, debug ≡ release; e a lane não é
decorativa — sob a mutação ela dá `3daddcd7…`.

**Zero schema, zero componente novo, zero dep.** 3 gates, 3 mutações, 3 sangram.

### W-FloatFloor — O PISO DA PERNA É GEOMÉTRICO, E O PRODUTO JÁ O HONRA

⚠️ **Esta não é uma wave: é uma caçada que terminou em *não há defeito*, e o que
ela deixa é um gate e duas notas corrigidas.** Ela começou porque a sonda do
repouso mede, numa rampa de 30° com `float_height = 0,50` — o
`RideConfig::STARTING_POINT` —, o personagem a **descer 0,59 m oscilando 0,27 m**.

**Varrido finamente contra a previsão** (`measure_float_floor.rs`):

| rampa | `min_float_height` | onset medido | no default 0,50 |
|---|---|---|---|
| 10° | 0,5031 | 0,5100 | +0,0200 · amp 0,0035 |
| 20° | 0,5128 | 0,5200 | +0,0393 · amp 0,0134 |
| 30° | 0,5309 | 0,5400 | **−0,68 m · amp 0,34** |
| 40° | 0,5611 | 0,5700 | **−5,14 m · amp 3,30** |
| 45° | 0,5828 | 0,5900 | **−9,53 m · amp 6,73** |

O onset segue a fórmula com **um passo de varredura** de folga em toda linha ⇒ o
piso é geométrico. E o **gesto do artista já o honra**: `apply_player_edit(Add)`
chama a mesma porta e multiplica por **1,2** — a 45° isso dá `0,6994` contra um
onset de `0,59`, **19% de folga**, com gate próprio na shell.

⚠️ **O que não estava escrito em lugar nenhum é a FORMA da falha abaixo do piso:**
ela não cresce devagar — a 45° o personagem **cai 9,5 m oscilando 6,7 m**. É isso
que torna a margem load-bearing em vez de decorativa.

⚠️ **E o `0,50` fica onde está, de propósito:** ele é o ponto de partida do
MODELO, e quem o veste com a geometria de um corpo concreto é o gesto. A tabela
do `measure_idle` mostrava a patologia **sem** o piso ao lado, e foi ela que
mandou nesta caçada — hoje ela imprime o piso no cabeçalho de cada rampa.

**O gate que ficou** (`the_predicted_floor_is_the_floor_the_simulator_shows`)
pergunta ao **SIMULADOR** se o onset segue a fórmula, o que o `ride_tests` não
pode fazer — ele confere `min_float_height` contra a tabela do próprio doc, um
espelho que continuaria verde se as duas se movessem juntas. ⚠️ **A 1ª versão
dele não tinha dentes com este mesmo nome:** ela afirmava que `piso × 1,2` fica
quieto, e a margem de 20% **absorve a fórmula errada** (piso colapsado numa
constante ⇒ `0,60` contra onset `0,59`, e passava). 3 mutações na fórmula do
produto, **3 sangram**.

⚠️ **E DUAS notas foram RECONFERIDAS porque outra wave moveu o número delas**
(CLAUDE.md §0): a §5 do `CLAUDE.md` afirmava **0,164 m de resíduo de rampa no
default** — verdade em 04/08, e a **W11b** o levou a 0,0331 e a **W11c** a
**0,0000 EXATO** de 20° a 45°; e o *"pulinho involuntário"* que a W11 declarou
**não-reproduzível em cinco configurações** reproduz sim — **abaixo do piso**,
que é uma configuração que aquelas cinco não continham.

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

## §8.1 — FECHADO: o personagem DESCE a rampa aos pulos (2026-08-09)

⚠️ **Achado por uma mutação que SOBREVIVEU.** Neutralizar o `snap_to_ground`
passava em toda a suíte, e eu atribuí isso à falta de uma fixture com **descida
caminhada**. ⚠️ **A atribuição estava ERRADA** (medido em 2026-08-09 pelo
`measure_snap_and_step`, ver §7.4): com a fixture escrita a mutação **continua
invisível**, porque o `snap_to_ground` **nunca dispara neste produto** — o
rapier o gateia em `result.translation.dot(up) < -1e-5` e a lei cinemática zera
a vertical enquanto no chão. *A fixture era necessária e não suficiente: não
havia o que detectar.* E o mecanismo do snap inerte **é o mesmo** que o do salto
abaixo — curar um devolve o outro de graça.

Escrita a fixture, o modo cinemático salta da superfície e o **dinâmico é
`0,0000` EXACTO nas nove células**:

| salto (m) | 1 m/s | 3 m/s | 6 m/s |
|---|---|---|---|
| 10° | 0,0170 | 0,0329 | 0,0806 |
| 25° | 0,0412 | 0,1459 | **0,5652** |
| 40° | 0,0654 | 0,4434 | **1,3258** |

A 40°/6 m/s ele se afasta **mais que a própria altura**, e não é transiente — o
traço mostra a folga a bater entre 0,51 e 1,07 m a descida inteira.

**Mecanismo, atribuído por ablação:** a `supported_velocity` absorve ao longo do
`up`, e numa rampa a descer isso cancela a componente vertical da velocidade —
que é exactamente a parte da caminhada que o faz seguir a superfície. Trocado o
eixo pela NORMAL, o salto vai a **0,0000** a 25° e 40° em toda velocidade. É a
mesma premissa que a W11 corrigiu no `damping_axis` (*"verdadeira só no plano"*)
e que ficou por corrigir aqui.

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

### A cura, no mesmo dia: um PISO para a absorção

A saída não era escolher um dos dois eixos — era dar à absorção um **piso**, e
a régua dele **não é o alvo da caminhada**, é geometria: *dada a velocidade
tangencial que ele já tinha, que descida seguir esta superfície exige?*
(`ph2d_platformer::surface_descent`, `along × (tangente · up)`, nunca positiva).

| salto (m) | 1 m/s | 3 m/s | 6 m/s |
|---|---|---|---|
| 10° | 0,0170 → **0,0029** | 0,0329 → **0,0043** | 0,0806 → **0,0043** |
| 25° | 0,0412 → **0,0064** | 0,1459 → **0,0096** | 0,5652 → **0,0096** |
| 40° | 0,0654 → **0,0082** | 0,4434 → **0,0123** | 1,3258 → **0,0188** |

⚠️ **`normal == up` reduz LITERALMENTE à lei anterior** (a tangente é
perpendicular a `up`, o piso é zero) ⇒ **chão plano e SUBIDA são
byte-idênticos**, e é essa a rede de segurança da mudança.

⚠️ **Duas descobertas, cada uma nascida de um gate VERMELHO** — e as duas são a
mesma lição, *a régua não pode ser derivada do número que ela vai corrigir*:

1. **A referência é a velocidade que ele JÁ TINHA**, não o `v` do tique. Com o
   `v`, a componente tangencial da **própria gravidade** alimentava o piso e o
   `..._does_not_creep_up_a_ramp` voltava (**0,0299 m**).
2. **O piso só vale para movimento CONTINUADO no chão.** Num POUSO a referência
   é uma queda, e uma queda vertical sobre uma rampa **tem componente
   tangencial**: o `..._landing_does_not_slide_along_the_ramp_normal` media
   **0,01432 m**. O piso é gateado em `was.kin.grounded` — a pergunta do
   INTEGRADOR, que é justamente a que o arch-gate vizinho PROÍBE para a
   absorção; por isso as duas vivem em bindings separados, e há gate a exigir a
   separação.

⚠️ **E o `min(0)` do piso é CONSERVADORISMO, não correção** — a mutação que o
remove sobreviveu, e o doc afirmava que sem ele o personagem *"seria lançado
ladeira acima"*. Medido: ele **sobe 1,2-1,7% mais rápido**. O `min` existe para
a subida ficar byte-idêntica; ligá-lo é decisão própria, com cena própria.

⚠️ **Resíduo honesto de 1-2 cm**, da mesma ordem da folga do pé da §7.5 (1,0 a
4,5 cm) — é o mesmo *"onde o último sweep o deixou"*, e um gate em `0,0000`
pediria uma exatidão que o controlador do rapier não dá.

⚠️ **E isto RE-ABRIU a §7.4:** o `snap_to_ground` deixou de ser inerte (o
deslocamento do tique passou a ter componente para baixo, que é o que o rapier
exige) ⇒ **o mínimo útil existe e está entre 0,08 e 0,10 m**, com o default de
0,25 a dar 2,5× de margem. Gate `the_ground_snap_is_load_bearing_again`.

⚠️ **O `physics_ecs_c9` NÃO se moveu com a cura** — nenhuma lane tinha um player
cinemático numa rampa, ou seja o harness de determinismo era **cego à lei
nova**. Lane acrescentada (`C9 Ramp`): 113 → **115 corpos**, hash
`7bb90663…` (debug ≡ release), e a mutação do piso a move.

---

## §8.2 — FECHADO: o caixote rodopiava quando o cinemático o empurra (2026-08-09)

Report do Enio, no smoke da cena `=102`: *"se o caixote não tiver com rotação
travada, o contato com o player kinematic causa rotação exagerada"*.

⚠️ **A wave sabia e escondeu na FIXTURE.** A cena `=102` trava a rotação dos
caixotes com um comentário que o admite (*"sem isto os caixotes TOMBAM… e a cena
passa a medir tombo, não empurrão"*). Era a decisão certa para aquela cena e a
errada para o módulo: *um defeito contornado por uma fixture não é um defeito
nomeado*.

Medido (`measure_push_spin`), giro total DESENROLADO em 3 s de empurrão:

| caixote (meia-extensão) | dinâmico | **cinemático** | alavanca |
|---|---|---|---|
| 0,30 | 0,3175 rad | **74,2906** (≈12 voltas) | 0,600 |
| 0,50 | 0,0043 | **12,8937** | 0,400 |
| 0,80 | 0,0007 | 0,0418 | 0,100 |
| 1,00 | 0,0006 | 0,0021 | −0,100 |

**O giro segue a ALAVANCA** — a altura do contato acima do centro de massa do
caixote — e desaparece quando ela desaparece. ⇒ o mecanismo é o `at` do
`apply_impulse_at_point`: o bloqueio inteiro do tique entra como **UM impulso
num ponto alto**, e `r × F` faz o resto. O dinâmico empurra com força
**sustentada por sub-passo**; o cinemático com uma **martelada por tique**.

⚠️ **A metade LINEAR está calibrada** (a W-KinPush mediu `16,54` contra os
`16,55` do dinâmico) — o que ninguém escolheu foi o torque que veio junto.

**As três saídas, com o preço de cada uma — decisão do Enio:**

1. **Impulso CENTRAL no empurrão lateral** (`apply_impulse`, sem ponto). Uma
   linha, e o giro vai a zero. ⚠️ Preço: **a porta é COMPARTILHADA** com a
   reação vertical da W6, onde o ponto é a razão de a jangada INCLINAR sob o
   personagem (a cena `=103` demonstra) — separar exige um 2º canal na porta, e
   *"um caixote empurrado no peito não tomba nunca"* é uma escolha de produto.
2. **Espalhar o impulso pelos sub-passos** em vez de o entregar de uma vez.
   Aproxima o dinâmico pelo mecanismo certo (é literalmente a diferença medida),
   mas mexe no laço de empurrão e no `physics_ecs_c9`.
3. **Um teto de torque** derivado da inércia do corpo atingido. Mantém o tombo
   honesto e corta o rodopio, ao preço de um número novo a calibrar — e a §0
   exige que ele saia de uma medição, não de um palpite.

### A cura, e o caminho até ela

**Escolha do Enio: (2).** ⛔ **E ela foi CONSTRUÍDA, MEDIDA e REVERTIDA — não
refaça.** A hipótese era que o braço de alavanca encolhe enquanto o caixote
tomba, então `N` fatias somariam menos torque que uma martelada. Em 16 ms o
caixote quase não gira: medido, **77,51 rad** — *pior* que os 74,29 de origem.

⚠️ **O que a tentativa deixou de bom foi a PORTA.** Separar a metade lateral da
vertical era exatamente a objeção que encarecia a opção (1), e construir (2)
produziu essa separação de graça. Com ela, (1) custa uma linha:

| caixote | dinâmico (andou/girou) | cinemático ANTES | cinemático DEPOIS |
|---|---|---|---|
| 0,30 | 7,13 / 0,3175 | 21,36 / **74,29** | **7,27 / 0,00005** |
| 0,50 | 7,02 / 0,0043 | 7,57 / **13,93** | 6,99 / 0,00 |
| 0,80 | 6,25 / 0,0007 | 6,03 / 0,0311 | 6,05 / 0,00 |

⚠️ **E o deslocamento passou a bater com o dinâmico** (7,27 contra 7,13, onde
antes eram 21,36): a viagem descontrolada era **realimentação do próprio
rodopio** — um caixote a girar apresenta faces diferentes ao sweep, é bloqueado
mais e recebe mais empurrão. Curar o torque calibrou a linha em **todo tamanho**,
e não só no caso de rotação travada em que a `W-KinPush` a mediu.

⚠️ **O trade fica NOMEADO:** um caixote empurrado no peito **nunca tomba**,
enquanto o dinâmico tomba um pouco. É escolha de produto.

⚠️ **O resíduo tem dono:** sobram `4,5e-5 rad`, do PONTO da reação vertical (que
fica) e do atrito do solver — por isso o gate afirma uma RAZÃO (`< 1%` do
controle) e não um zero.

⚠️ **Uma cerca foi DERRUBADA com medição, não apagada:** o arch-gate exigia a
porta compartilhada, argumentando que *"uma segunda porta seria uma segunda
resposta"*. O argumento estava certo sobre a CONVERSÃO e errado sobre a
pergunta — a vertical pergunta **ONDE** (o ponto é a inclinação da jangada), a
lateral pergunta **QUANTO**. Ele passou a afirmar a separação, mais a metade que
sobrevive: as duas convertem com a massa do PLAYER.

⚠️ **E o hash era CEGO outra vez, pela MESMA fixture que escondeu o defeito:** o
caixote do `C9 Push` tinha `LockRotation`, como a cena `=102`. Removido — 117
corpos, hash **`fb27f676…`** (debug ≡ release), e um ponto de volta na porta
lateral o move na hora (`463b40f9…`).

Gate: `the_pushed_crate_no_longer_spins_and_the_shove_matches_the_dynamic`, que
substitui o verde-sobre-o-defeito e cuja própria mensagem mandava reescrevê-lo
no dia em que o número diminuísse.

---

## §8.3 — FECHADO: o cinemático ficava PARADO numa rampa que o próprio `max_slope` recusa (2026-08-09)

Achado a fechar o item 2 do §7, não por reporte. O dinâmico escorrega de uma
rampa de 50°/60° e sai de quadro (24,2 m / 16,5 m em 5 s); **o cinemático fica
imóvel, para sempre** — `x = 0,0000` EXACTO em 300 tiques.

⚠️ **É o `max_slope` a não fazer o que o nome dele promete, e a metade que
falta é exactamente a que a W9 não cobriu.** O doc do `slope.rs` conta que em
2026-08-04 o número autorado deixava de ser honrado a SUBIR (o modo-ar escalava
o que a perna recusara) e a cura foi o veredito `Steep`. A metade de DESCER
nunca foi conferida: hoje o artista escreve 45 e o personagem fica em pé numa
parede de 80°.

### Atribuído em quatro passos, cada um descartando um suspeito

1. **Ele repousa na rampa, não paira** — distância perpendicular ao plano menos
   o suporte da cápsula dá **5,5 cm** a 60° (5,2 a 50°), a folga do controlador.
   A mola de flutuação não está a segurá-lo.
2. **O limite é INERTE ali** — trocar `max_slope_deg` de 45 para 90 para 5 dá a
   MESMA pose a 60° e a 80°, enquanto a 45° ela muda (1,3175 contra 1,3741).
   *O veredito de rampa não é quem congela.*
3. **O rapier manda DESLIZAR** — `is_nonslip_slope = ângulo <= min_slope_slide_angle`
   é `60 <= 45`, falso, então o `handle_slide` dele cai no ramo *"let it slide"*.
   Logo o deslocamento **pedido** é que era zero.
4. **E era** (instrumentado, depois removido): `stand=None vel=[0,0]
   wanted=[0,0] grounded_was=true`.

### A causa: DUAS respostas para a mesma pergunta, no mesmo tique

- A **ponte** absorve com `stand.is_some()` — *"isto é CHÃO?"*, a **K4**, que
  conhece o limite de rampa.
- O **integrador** (`kinematic_advance`) absorve com `state.grounded` — *"eu
  TOQUEI?"*, a resposta do CONTROLADOR, que não conhece limite nenhum.

E é a segunda que decide o `wanted`. Numa rampa de 60° a lei **recusa a
superfície** e **absorve a gravidade na mesma**, então não sobra deslocamento
para o deslizamento do rapier redirecionar.

⚠️ **O `state.grounded` está ali de propósito** (o comentário no `kinematic.rs`
o diz: *"a pergunta do INTEGRADOR, não a da lei — e a MESMA porta que a ponte
usa para não entregar à lei uma queda que este passo vai apagar"*), então isto
**não é um `if` errado, é um contrato a decidir**. Não é a cura da §8.1: com
`footing = None` o piso dela é zero e a expressão reduz à de antes.

### As três saídas, com o preço de cada uma — decisão do Enio

1. **A absorção pede as DUAS** (`state.grounded && footing.is_some()`). Chão
   plano e rampa caminhável ficam **byte-idênticos** (as duas verdadeiras); só o
   caso *toco-mas-não-é-chão* muda, e ali o rapier já sabe deslizar. ⚠️ Preço: o
   gate `the_ground_absorbs_only_what_points_into_it` do `ph2d-platformer` usa
   uma fixture com `grounded: true` e **amostra `None`** — ela passaria a
   significar *"no ar"*, e o contrato da lei muda com ela.
2. **Um piso derivado da superfície ÍNGREME**, reusando o maquinário da §8.1: em
   vez de cortar a absorção, deixá-la absorver só o que uma rampa de 60° de
   facto segura. É o mais fiel e o mais caro (a `surface_descent` hoje toma a
   velocidade que ele JÁ tinha, e parado ela é zero — precisaria de um segundo
   termo).
3. **Aceitar**, e então o `max_slope` passa a ser documentado como *"o que ele
   consegue SUBIR"* e nada mais. Custa uma linha de doc e uma promessa a menos.

**Escolha do Enio: (1).**

### A cura

`kinematic_advance` passa a absorver com `state.grounded && footing.is_some()`.

| rampa | limite 45, ANTES | limite 45, DEPOIS | limite 90 | limite 5 |
|---|---|---|---|---|
| 45° | `1,3175` | **`1,3175`** (igual) | 1,3174 | escorrega |
| 60° | `1,8101` (imóvel) | **escorrega −0,71** | fica (é caminhável) | escorrega |
| 80° | `3,7754` (imóvel) | **escorrega −7,58** | fica | escorrega |

⚠️ **As colunas caminháveis são byte-idênticas** (10° a 45°, a varredura do §7.2
inteira) — ali as duas respostas são verdadeiras, então a expressão não muda. E
o knob voltou a MOVER o comportamento: antes 45/90/5 davam a mesma pose.

⚠️ **O ritmo do escorregão cresce com a inclinação** (0,98 m em 10 s a 50° ·
1,45 a 60° · quase queda livre a 80°): o `kinematic_settle` devolve a velocidade
que o mundo não deixou acontecer, então perto do limite ele escorrega devagar.
Não é o mesmo número do corpo dinâmico (24 m) e **não devia ser** — um corpo
dinâmico em queda por uma rampa não tem controlador a segurá-lo.

⚠️ **Uma fixture da lei mudou de sentido, e era o que devia acontecer:** o
`the_ground_absorbs_only_what_points_into_it` declarava `grounded: true` e
passava amostra `None` — *"estou no chão"* sem fornecer chão nenhum. Isso agora
descreve **tocar numa parede**, que é o caso oposto; ela recebeu um `flat()`.

⚠️ **E o hash de determinismo era CEGO à mudança** — a lane `C9 Ramp` da §8.1 é
de −20°, caminhável, onde as duas respostas coincidem. Lane nova **`C9 Steep`**
(60°, o dobro do limite): 115 → **117 corpos**, hash
`49f223f8…` (debug ≡ release). ⚠️ **A primeira versão dela não discriminava:** a
fita é UMA para todos os players (`drive = +1` nos primeiros 90 tiques) e a
rampa descia para a direita, então o personagem escorregava empurrado pela
própria fita nas DUAS leis — *cobertura aparente sobre um fixture que não
continha o fenômeno*. Subindo para a direita, o empurrão trabalha CONTRA o
escorregão e só a lei nova o deixa descer (299,88 contra 300,15).

Gate novo `nothing_is_absorbed_on_a_surface_the_law_refused`, com o CONTROLE ao
lado (com chão, absorve).

---

## §8.4 — FECHADO: a ÁGUA não existia para o modo cinemático (W-KinFluid, 2026-08-09)

Pergunta do Enio: *"testou o kinemático na água? ou ele não funciona lá?"* —
**não tinha sido testado**, e as três sondas do `measure_player_in_water` são
dinâmico-only. Medido agora (poça funda, 4 s, sem input, `measure_the_kinematic_player_in_water`):

| sujeito | y final | afundou |
|---|---|---|
| cápsula solta (CONTROLE) | 0,3928 | **1,1072 m** — boia |
| player dinâmico | 0,4107 | **1,0893 m** — boia |
| player **CINEMÁTICO** | **−138,17** | **139,6739 m** |

Ele atravessa a poça em **queda livre** — 1,78× os 78,5 m analíticos de 4 s, que
é o `fall_gravity` do platformer a trabalhar. A água não o toca.

### O mecanismo, e ele já estava escrito noutro lugar

O empuxo e o arrasto de uma zona chegam por `apply_impulse` no CORPO, e um corpo
cinemático tem **massa infinita** para o solver. É literalmente a frase que a
`W-KinPush` escreveu para explicar por que o `move_shape` não empurrava um
caixote: *"um corpo cinemático tem massa INFINITA para o solver, então o
`move_shape` desliza contra um caixote solto sem lhe transmitir nada"*. A mesma
assimetria, do outro lado — ali ele não DAVA, aqui ele não RECEBE.

⚠️ **E EU ESCREVI AQUI QUE *"metade da água já atravessa"*, e era FALSO.** A
frase dizia que a `Buoyed` é lida **fora** do ramo de modo (`bridge/player.rs`),
logo a trava do fluido (`JumpState::waterborne`, W-Submerged) valeria nos dois —
*"o personagem cinemático SABE que está molhado e afunda como uma pedra"*. A
**fiação** estava certa; o **VALOR** que ela entregava era morto: medido, `buoyed`
lia `0,0000` para o corpo cinemático e `3,99` para o mesmo corpo dinâmico, na
mesma poça, no mesmo tique — a trava nunca armava. *Uma afirmação sobre a
estrutura não é uma medição do número que passa por ela.*

### As três saídas, com o preço de cada uma — decisão do Enio

1. **A lei integra a água, como já integra a gravidade.** O `kinematic_advance`
   já é o lugar onde *"a gravidade é aplicada AQUI, e é a assimetria central dos
   dois modos"*; o empuxo e o arrasto entram pela mesma porta, a partir da
   `Buoyed` que a ponte **já lê**. ⚠️ Preço: a lei passa a precisar da
   densidade/arrasto da zona, não só de *"quanto peso ela carrega"* — a
   `Buoyed` de hoje é um escalar.
2. **A ponte converte a zona num MOTOR** e deixa a lei intacta. Mais barato, e
   é a 2ª resposta para *"o que a água faz a este corpo?"* — divergiria do
   caminho dinâmico no dia em que um dos dois ganhasse um caso.
3. **Aceitar**, e então o modo cinemático é documentado como *"para cenas
   secas"*. ⚠️ Custa a metade que JÁ funciona: o personagem continuaria a saber
   que está molhado enquanto afunda, que é o pior dos três estados.

**Recomendação: (1).** É a que põe a água no lugar onde a gravidade já está, e a
única em que as duas metades da água passam a concordar.

### ⇒ O Enio escolheu (1). E a causa raiz não era a que este plano nomeou

**Era mais funda que a massa.** `ActiveCollisionTypes::default()` do rapier é
`DYNAMIC_DYNAMIC | DYNAMIC_FIXED | DYNAMIC_KINEMATIC` — **nenhum par que comece
em KINEMATIC está lá** —, então uma poça ESTÁTICA e um player Snap nunca chegavam
a existir um para o outro no grafo de interseção. Não era o impulso a não alcançar
massa infinita: era **o par que não existia**.

⛔ **E a segunda causa que eu construí não existia.** Escrevi um `authored_weight`
que somava as massas dos colliders quando `rb.mass()` fosse zero, com um doc a
dizer que era ele que fazia a água existir no modo cinemático — **a mutação provou
que era falso** (removê-lo deixou tudo verde), e a medição direta explica:
`rb.mass()` devolve `1,0000` em Dynamic, Kinematic **e** Fixed; o rapier zera a
inversa-massa *efetiva*, não esta. Removido no mesmo commit em que nasceu.

| peça | onde | o que muda |
|---|---|---|
| **o sensor vê o cinemático** | `world/collider_build.rs` | `ActiveCollisionTypes::all()` **só em sensor** — o teste do rapier é `co1 ou co2`, basta um lado abrir, e um sensor cuja razão de existir é NOTAR coisas não pode ser cego a metade das espécies de corpo |
| **o teto de `1` saiu** | `world/queries.rs` | a razão era capada, e ela **satura já a `y = 0,2`** com metade do corpo fora (medido) ⇒ `g·(1−1) = 0` deixaria o personagem pendurado no meio da poça para sempre |
| **`fluid_at`** | `world/queries.rs` | UMA varredura devolve empuxo **e** arrasto; `buoyed()` delega. O arrasto é o **MÁXIMO** sobre as zonas, e é isso que apaga a dedup que um corpo composto exigiria |
| **a lei integra** | `platformer/kinematic.rs` | `Fluid { buoyed, drag }` → `g·(1 − buoyed)` e `v /= 1 + d·dt`, **onde a gravidade já entrava** |

**A FRONTEIRA não foi escolhida — o W-AreaFalloff já a desenhou:** o falloff pesa
os dois **EMPURRÕES** (força, torque) e **deixa o MEIO em paz** (`drag`,
`density`, `form_drag` descrevem uma substância, e uma substância não fica mais
rala perto da própria margem). É exatamente isso que torna o meio respondível por
uma consulta e a força não: ⛔ **a força FICA de fora**, porque precisa do frame
da zona, do espelho e do falloff — re-derivá-los numa query seria uma **segunda
resposta** para *"que empurrão esta zona dá neste ponto?"*, o defeito recorrente
desta linha. Uma corrente não leva um personagem cinemático, e isso está
**nomeado, não esquecido**.

### Os números

| | antes | depois |
|---|---|---|
| cinemático, poça funda, 4 s | afunda **139,67 m** | boia (**1,086 m** acima da largada) |
| `buoyed` de um corpo cinemático | `0,0000` | `3,9912` — o mesmo que o dinâmico |
| `physics_ecs_c9` | `fb27f676…`, 117 corpos | **idêntico** |

⚠️ **O PRIMEIRO número que eu reportei desta cura estava errado, e a lição é de
oráculo:** escrevi *"assenta a `0,4140`, três milímetros do dinâmico"* — e era
**um instante de uma oscilação**. Nesta poça o player bobeia **1,44 m** de
amplitude entre o 3.º e o 6.º segundo. *Uma amostra única de um sistema que
oscila não é um repouso.*

⚠️ **E a oscilação NÃO é desta wave:** o corpo **dinâmico** faz `1,4408` na mesma
cena contra os `1,4394` do cinemático — concordam na quarta decimal —, e a
cápsula solta (sem lei de player nenhuma) faz `0,8097`. O que bobeia é o *player*
na água, nos **dois** modos, e é anterior a isto. Por isso o oráculo do gate é a
**PARIDADE ENTRE MODOS** (`tests/player_in_water.rs`), nunca um literal de linha
d'água: a espécie do corpo deixa de ser uma pergunta que a água faça.

⚠️ **O arrasto é load-bearing e tem número:** com `AreaDrag 0` a amplitude sobe
para **2,90 m** e não decai — empuxo sem resistência é uma mola sem
amortecimento, a frase que a fixture da poça já carregava.

⚠️ **Paridade APROXIMADA com o dinâmico, nomeada:** o solver amortece por
SUB-PASSO e esta lei uma vez por TIQUE — `(1+d·h)⁻⁴` contra `(1+d·4h)⁻¹`, a mesma
classe de diferença que a W-AreaDrag mediu em 1,25%. Um corpo cinemático não tem
sub-passo para dividir.

**Gates:** 4 na lei (`kinematic_tests.rs`) · 2 na consulta (`buoyed_query.rs`) ·
3 no produto (`player_in_water.rs`). **6 mutações, 5 sangram** — a 6.ª acusou a
minha própria afirmação (o `authored_weight`) e o código saiu.

⚠️ **Um gate MUDOU DE NOME e de afirmação:** o
`the_scale_tops_out_at_one_and_zero_gravity_carries_nothing` pinava o teto que
esta wave removeu, e virou `the_scale_is_the_density_ratio_…`. Ele estava certo
enquanto o único consumidor perguntava `> 0`; *quem acrescenta o consumidor que
precisa da magnitude reconfere a nota que a capava.*

### A cena: `PH2D_PHYSICS_SMOKE=104`

Uma poça **funda** e **três cápsulas idênticas** largadas de `y = 1,5` no mesmo
instante — **verde** a solta (sem lei de player), **âmbar** o dinâmico, **azul**
o cinemático. ⚠️ **Os três sujeitos são a cena inteira**, porque o oráculo desta
wave é a paridade e uma paridade precisa de dois lados na tela; um número não se
lê num screenshot. Gate próprio afirma que eles **diferem só no que a wave muda**
(mesma forma, mesma densidade, mesma altura de largada) — sem ele a cena mostraria
uma diferença que não é a que a wave produziu, e o artista concluiria o oposto.

| sujeito | afunda em 4 s | oscila (3.º→6.º s) |
|---|---|---|
| cápsula solta (controle) | 1,1070 m | 0,8097 |
| player **dinâmico** | 1,0855 m | **1,4357** |
| player **cinemático** | 1,0860 m | **1,4394** |

⚠️ **A poça é funda de propósito:** com fundo ao alcance da perna o sensor de chão
responde, o personagem fica de pé e a água deixa de ser a única coisa a agi-lo —
a cena mediria outra pergunta (há gate na premissa, porque premissa só escrita em
prosa é a que a próxima edição apaga).

⚠️ **E o gate da mensagem pegou o meu próprio erro:** a primeira versão dos
números veio da fixture **irmã** (a poça do `measure_player_in_water`, que tem
outra geometria) e ele reprovou — *uma cena cuja mensagem cita números tem de
medir os DELA*. Sonda: `probe_smoke_104`.

**Aberto:** `form_drag` não alcança a lei (é um kernel por-aresta sobre o
polígono, não um escalar do meio) · a **força** da zona (acima) · e **nadar** —
controlar a subida dentro d'água — é produto, não correção.

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

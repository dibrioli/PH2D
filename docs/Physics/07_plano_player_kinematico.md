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
   cobra o `populate`. ⚠️ **Dois chips agora, TRÊS quando a `W-KinPure` chegar** — e é
   por isso que o `seg_row` recebe uma fatia, nunca um par: acrescentar o terceiro tem de
   ser uma linha na tabela, não uma reescrita da row.
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

#### A SEGUNDA metade, e ela não é a mesma linha

Com a cura, o player bóia a **`0,3446`** contra os **`0,2174`** da cápsula idêntica —
**12,7 cm mais alto**. A ablação nomeia a causa **exatamente**: com os multiplicadores
de gravidade do pulo neutros (`takeoff`/`peak`/`fall`/`cut` = 1) ele bóia em
**`0,2174`, `+0,0000` contra o controle**.

> O `peak_gravity` (default **0,5**) dispara numa janela de **velocidade vertical perto
> de zero** — e é isso que boiar **é**. Um personagem a flutuar vive permanentemente no
> **ápice de um pulo que ele nunca deu**, e leva um empurrão para cima de meia gravidade
> para sempre.

⚠️ **No AR isso nunca acontece** (a gravidade tira-o da janela em poucos tiques, e o
*hang time* no ápice é a feature); na água a janela é o **regime permanente**. Por isso
a cura não é apagar o perdão — é decidir *o que qualifica um ápice*, e essa decisão é
de produto. **Fica NOMEADA aqui e medida; o que o Enio escolher é a metade B da wave.**

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

### W-KinMove — O MODO EXISTE E O PERSONAGEM ANDA (cena `=101`)

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

### W-KinWeight — O CHÃO SENTE O PERSONAGEM (cena `=102`)

A K6 no eixo vertical: sob Snap o `support` é o peso, e a jangada afunda.

**Gates:** a jangada da cena `=72` no modo cinemático afunda **o mesmo** que com o corpo
dinâmico (a comparação é o oráculo, e é honesta porque a força física é a mesma) ·
`Reaction Scale = 0` continua a desligar tudo · a mutação que zera o suporte sob Snap
sangra.

⚠️ **A massa é AUTORADA** (o corpo cinemático não tem massa que o rapier calcule) — e é
por isso que esta é wave própria: um número novo na §14 tem as suas quatro condições.

### W-KinPush — E O QUE ESTÁ AO LADO TAMBÉM (cena `=103`)

A outra metade da ordem *"influencie tudo"*: o que o `move_shape` **não** conseguiu
mover, projetado na normal, volta como impulso no ponto de contato.

**Gates, red-first:** um caixote solto no caminho **não se move** antes da wave e é
empurrado depois · a **massa manda** (o caixote leve anda mais que o pesado sob o mesmo
personagem) · **uma parede estática não absorve nada** (o impulso vai para um corpo de
massa infinita e o ledger fecha em zero) · e a **mutação que devolve o impulso pelo
`solve_character_collision_impulses` do rapier** tem de sangrar, porque a wave é sobre
*qual* lei empurra, não sobre *se* empurra.

⚠️ **O risco desta wave é ESTABILIDADE, não correção, e o gate tem de o medir:** um
personagem que empurra por impulso a 60 Hz pode entrar em ressonância com o slide (ele
empurra, o caixote foge, o slide o segue, ele empurra outra vez). O número que decide é
a **amplitude em regime** encostado num caixote parado — o mesmo kill-criterion que a W6
do plano 06 usou para a plataforma dinâmica.

### W-KinPure — O TERCEIRO MODO (não construída agora)

O *"puro sangue"* que o Enio nomeou: nada de reação, nada de empurrão — o platformer
clássico, em que o mundo físico é cenário. ⚠️ **Ele é BARATO por construção depois das
duas waves acima** (é o mesmo caminho de movimento com a reação e o empurrão calados) e
**é por isso que ele não vem primeiro**: construí-lo antes faria a K6 parecer um caso
especial dele, quando é o contrário — *o pobre é o rico com dois canais desligados*.

### W-KinTune — O QUE O SMOKE PEDIR

Reservada de propósito: `autostep` afinado, `min_slope_slide_angle` exposto, ou nada.

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
| 7 | **o que uma perna RÍGIDA custa** (`W-Landing`) — o solavanco ao subir um degrau e o pico da reação na jangada | a cena `=91` (a escada de pranchas) e a `=100`, varrendo `k` |
| 8 | a **deriva de rampa contra a rigidez** — a lei publicada varreu o amortecimento, **não** o `k` | a varredura do `platform_idle`, com `k` no eixo |

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

- **Registro `ph2d-physics-ecs` 28 → 29** (o gate diz existir *"to hurt"*).
- **`PROJECT_SCHEMA`: NÃO bumpa** (K2) — e se algum campo acabar apendado, o valor se
  **CONTA** contra o `main` do dia, nunca se escolhe.
- **Ids novos da §14** (os chips do modo + a massa da `W-KinWeight`), anotados no handoff.
- **Cenas de smoke `=100` · `=101` · `=102` · `=103`** — ⚠️ o roteador é um `match` de strings
  cujo `_` cai na cena 1: um nível inexistente **não avisa**, mostra outra coisa. O `=84`
  não existe de propósito; **100 é o próximo livre** (o último ocupado é o `=99`).
- **Contrato congelado: nenhum. Dep externa nova: nenhuma** (o
  `KinematicCharacterController` é do `rapier2d` que já é dep, e não é feature-gated no
  2D).

# Plano 10 — a fila da auditoria 09, desenhada (2026-08-12)

> **Ordem do Enio:** *"ok. Planeje, salve o plano e comece"*, sobre a fila da
> [auditoria 09](09_auditoria_engines.md).
>
> Este doc é **plano**. Cada wave traz o desenho, as decisões com o preço ao
> lado, e o que a fecha (gates · mutação · as quatro condições de UI do plano 00
> · cena de smoke com números **medidos**). ⚠️ **Nenhum número de produto é
> escolhido aqui** (§0): quem os escreve é a medição da wave.

```
A (a saída) ✅ → C (o peso) ✅ → B (a superfície) ✅ → D (o teto) → E (o empurrão)
   → J (a sonda do Snap) → G · H · I
```

> **Estado (2026-08-13):** **A**, **C** e **B** fechadas — a próxima é a **D**.

---

## §1 — Wave **A**: a SAÍDA ⟨**FECHADA** 2026-08-13⟩

> **FECHOU** em cinco commits (`3e0ac07e6` A1 · `4bcf12a9b`+`80d72be32` A2 ·
> `4758f15ad` A3 · `933fa02fd` A5; o A4 é o que cada um deles carrega).
> **Pendente de smoke** — cena **113**.
>
> ⚠️ **O que a medição mudou no que está escrito abaixo:**
>
> * **A2 ganhou uma porta que o plano não previu.** A mutação *"a subtração do
>   chão vira zero"* sangrava UM gate, e a conta estava escrita **duas vezes**: a
>   lei projeta a velocidade relativa no `up`, a mola do `ride` na **NORMAL**, e
>   por serem eixos diferentes ninguém tinha visto que era a mesma subtração.
>   `rise_over` → **`relative_along(body, ground, axis)`**, três chamadores, uma
>   fórmula, byte-idêntica — e a mesma mutação passou a sangrar nos DOIS módulos.
> * **A3 fundiu-se na porta que já existia** em vez de abrir uma segunda: o
>   evento do player é a **terceira fonte** do `signal_events`, ao lado dos
>   contatos e dos sensores, então a shell que já drena aquilo **não mudou uma
>   linha**. E os três pulos são três NOMES (`player.jumped.ground|air|wall`),
>   pela lei do ADR-0143 que o `SignalOnLeave` já enuncia.
> * **A5 teve o roteiro REESCRITO pela sonda, três vezes.** Parede no meio do
>   caminho ⇒ o percurso morria nela (`x = 10.50` para sempre); atrás do início
>   ⇒ ele chegava pelo CHÃO, onde a lei recusa agarrar-se; colada à ponta do
>   degrau ⇒ bloqueio. O que funciona é **um metro de vão** depois do degrau.
> * ⚠️ **O readout é APAGADO nas duas descontinuidades** (scrub e `hold`), e isso
>   não estava no plano: sem passo não há lei, e publicar *"no chão, a 4 m/s"*
>   com a física desarmada seria um número errado apresentado como certo. A §14
>   diz **`not simulating`** em vez de deixar um vão.
> * ⚠️ **A cena é a 113 e não a 105:** esta linha está 74 commits à frente do
>   `main`, e a jornada da ÁGUA já tomou 105..112. *Um número de cena escolhido
>   numa linha paralela é PROVISÓRIO* — quem integra conta contra o `main` do dia.
> * **O `physics_ecs_c9` NÃO se move com esta wave** (medido: `1699123f…` antes
>   e depois dos cinco commits) — a diferença contra o `main` é da jornada da
>   água, que já estava na linha.


**O buraco, medido:** a superfície pública inteira da ponte para o player são
seis portas (`bridge/player_channel.rs`) e **nenhuma diz o que o personagem está
a fazer**. Sem isso não há passo, poeira, sprite a virar, animação, nem um pulo
ligado a um sinal.

⚠️ **São DOIS canais, e a distinção já foi paga por este repo**
([[feedback_a_transient_event_marker_is_its_own_channel]]): um **estado** lê-se
quando se quiser; um **evento** acontece uma vez e quem não o leu perdeu-o.

### A1 · O READOUT — `PlayerView`

`bridge.player_view(entity) -> Option<PlayerView>`, escrito no fim do laço de
tiques, lido por quem quiser.

**O que carrega, e porquê cada um:**

| campo | porquê |
|---|---|
| `footing: FootingKind` (`Ground` · `Steep` · `Air`) | ⚠️ **TRÊS, não um bool** — a W9 des-colapsou *no ar* de *encostado numa rampa íngreme demais* porque pedem coisas opostas da caminhada; um `grounded: bool` no readout **re-colapsaria** o que a lei separou |
| `wall: Option<f32>` (o lado) | o `get_wall_normal` do Godot, na forma que um side-scroller usa |
| `crouching` · `swimming` · `ledging` · `dashing` · `gliding` | **fatos, não um veredito** — ver o aviso abaixo |
| `facing: f32` | é o que vira o sprite, e hoje não sai da lei |
| `velocity: Vec2` · `ground_velocity: Vec2` | ⚠️ **os dois FATOS**; a diferença (a velocidade relativa, que é a que casa com um ciclo de caminhada) é **interpretação do consumidor** e não é publicada — publicar os três seria a mesma coisa dita duas vezes |
| `ground_normal: Option<Vec2>` | o `get_floor_normal` |
| `air_jumps_left: u32` · `dash_charged: bool` · `grab_left: f32` | as RESERVAS — é o que um HUD desenha |

⚠️ **Fatos, nunca um `enum` de regime único.** Agachado *e* no chão são
simultâneos; arrancar *no ar* também. Um enum teria de escolher uma prioridade —
e essa escolha é do JOGO (é literalmente o que o `TnuaAnimatingState` do tnua
existe para ajudar a fazer), não do motor. Um enum aqui seria uma segunda
resposta ao lado dos bits, e a primeira a divergir.

⚠️ **O estado publicado é o do ÚLTIMO tique do dispatch**, e isso é o certo para
um *estado*: ele descreve o agora. Quem precisa dos tiques do meio precisa de um
**evento**, que é o A2 — e é essa a fronteira entre os dois canais.

**⚠️ A0 — a armadilha que NÃO existe, verificada antes de planear:** o `facing`
mora dentro do `DashState` (`dash.rs:179-190`), e a pergunta era *"ele continua a
ser mantido com o arranque desligado?"*. **Sim** — o `dash_step` é chamado
**incondicionalmente** pela lei (`lib.rs:370`; o `cfg.armed()` guarda só o
COMEÇAR, lá dentro) e escreve o `facing` antes de qualquer retorno.

⚠️ **Mas o campo muda de casa na mesma wave, e o argumento é do próprio repo:** o
doc do `PlayerState` recusa guardar o arranque dentro do `JumpState` porque seria
*"um nome que mente por conveniência de armazenamento"*. `DashState.facing` é
exactamente isso — o arranque **lê-o** (para escolher a direção), quem o
**escreve** é a caminhada, e a partir desta wave quem o **publica** é o readout.
⇒ sobe para `PlayerState.facing`, e o `dash_step` passa a recebê-lo. **Zero
schema** (o `PlayerState` vive no ring de checkpoints, em memória).

### A2 · OS EVENTOS — `PlayerEvent`

| evento | de onde sai |
|---|---|
| `Landed { speed }` | transição `Air/Steep → Ground`; a `speed` é a **relativa ao chão**, medida ANTES do tique (é ela que dimensiona poeira e som) |
| `Jumped { kind: Ground \| Air \| Wall }` | ⚠️ **o KIND vem da LEI** — ver abaixo |
| `Apex` | a subida relativa cruza de `+` para `−` (o `bNotifyApex` do Unreal) |
| `Dashed` | o arranque começa |
| `LedgeGrabbed` | a trava do pendurar arma |
| `EnteredWater` / `LeftWater` | a trava do nado vira |

⚠️ **O evento nasce DENTRO do laço de tiques, comparando o par
`(estado_antes, estado_depois)`** — nunca de um diff entre dois frames da shell.
Um dispatch pode dever vários tiques, e um diff de frame **perde os do meio**: um
pulo que sai e aterra dentro do mesmo dispatch não teria acontecido. É a mesma
razão pela qual o `W-TickContacts` moveu o diff de contatos para o tique.

⚠️ **Quase tudo é derivável do par de estados, e o `Jumped.kind` NÃO é.** A lei
sabe se o pulo veio do chão, do ar ou de uma parede; a ponte teria de o adivinhar
(*"estava no chão no tique anterior?"*) — uma segunda resposta que erra
exactamente no caso interessante. ⇒ **o `JumpStep` passa a dizê-lo**, e é o único
toque desta wave na lei.

⚠️ **A lista é limpa no início de cada dispatch**, como a de contatos: um replay
de scrub re-produz os tiques, logo re-produz os eventos deles — e isso é o
correcto, porque o que se está a assistir é aquela passagem do tempo.

### A3 · O que fecha a volta ⟨a metade visível⟩

⚠️ **Um canal sem consumidor é o defeito que a `W-Signal` nomeou** (*"quatro
canais de leitura existem desde o W7 e nenhum deles faz nada acontecer"*).

* **O readout** ganha um bloco **read-only no topo da §14** — postura, `facing`,
  velocidade. É o que a Unity mostra no inspector dela, e é o que torna a
  afinação do player observável sem um `println`.
* **Os eventos** são encaminhados pela shell ao **`SignalOutbox` do R0**, com
  nomes estáveis (`player.landed`, `player.jumped`, …), reusando o consumidor que
  já existe. ⚠️ **Opt-in por-player**, com uma row `Emit Signals` que nasce
  **DESLIGADA**: sem isso toda cena de smoke com um personagem passaria a cuspir
  toasts, e o custo cairia sobre waves que nada têm com esta. É o mesmo idioma do
  `SignalOnHit`, que também é um opt-in autorado.

⚠️ **A ponte NÃO ganha dependência da `ph2d-runtime`** — ela expõe eventos
**tipados** e quem os funde numa saída é a **shell**, que já é a dona do
consumidor e já drena a outra fonte. É o precedente literal do `bridge/signals.rs`.

### A4 · Gates

* o readout diz `Ground` só quando a lei diz `Ground`, e **`Steep` quando é
  íngreme** (o gate que impede o re-colapso);
* **`facing` segue o eixo com o arranque DESLIGADO** — o gate que prova que o
  campo não é do dash;
* uma aterragem produz **exactamente um** `Landed`, e a `speed` dele é a de
  aproximação (não a de depois do tique);
* ⚠️ **o gate que carrega a wave:** um dispatch que deve **três** tiques, com um
  pulo e uma aterragem no meio, entrega os dois eventos. **A mutação é derivar
  por diff de frame** — ela perde os do meio e só este gate sangra;
* o `Jumped.kind` distingue os três, e a mutação *"adivinha pelo `grounded`
  anterior"* erra o pulo de parede;
* seam: a row `Emit Signals` pinta, regista, o clique chega, e ligada **faz
  aparecer um toast** (a SEQUÊNCIA leva a algum lugar).

### A5 · Smoke

Cena nova: o personagem corre, salta, aterra, arranca e agarra uma beirada, com
`Emit Signals` **ligado** — os toasts aparecem na ordem certa. A cena **imprime
o readout** a cada meio segundo. ⚠️ *Se a linha do readout não aparecer, pare.*

---

## §2 — Wave **C**: o PESO ⟨frear ≠ acelerar⟩ ⟨**FECHADA** 2026-08-13⟩

> **FECHOU.** Cena **114**, `PROJECT_SCHEMA` **78→79** (`PlatformPlayer.brake_scale`,
> ⚠️ **PROVISÓRIO** — o valor se CONTA contra o `main` do dia), `physics_ecs_c9`
> **byte-idêntico** (`1699123f…`, 117 corpos), **10 mutações, 10 sangram**.
>
> ⚠️ **O que a medição mudou no que está escrito abaixo:**
>
> * ⛔ **O gate *"`2.0` para em METADE da distância"* está REFUTADO — ele para em
>   `0,343×`.** A previsão vinha do modelo contínuo (`v²/2a`, onde dobrar `a`
>   corta a distância ao meio) e a lei não é contínua: o fator de viragem faz `a`
>   crescer com a sobra, e a paragem inteira cabe em **3 a 5 tiques**, onde quem
>   manda é o ramo do `boost`. O gate que shipou afirma o que a sonda deu —
>   monotónica, e o dobro corta **mais** que metade. Tabela (perfil de partida):
>   `0,25 → 0,8486 m` · `0,50 → 0,3957` · `1,00 → 0,1700` · `2,00 → 0,0583` ·
>   `4,00 → 0,0000` (um tique).
> * **O AR ficou de FORA, e o plano não decidia isto.** *"O orçamento"* era
>   ambíguo entre *o do regime activo* e *o do chão*; quem decide é a
>   `air_acceleration`, que **já é** a resposta do ar à mesma pergunta (o doc dela
>   promete que `0` conserva o arco) — um segundo número sobre aquele
>   comportamento seria a falha de duas portas. Gate próprio
>   (`the_brake_leaves_the_air_alone`) para ninguém *"completar"* a wave depois.
> * ⚠️ **A ausência de teto ficou MEDIDA em vez de argumentada:** a lei é
>   auto-limitada (a sobra que cabe num tique é escrita EXATA, então subir o
>   número nunca ultrapassa o alvo), e o ponto de saturação é
>   `speed / (turn·accel·dt)` — **função da config** (4,0 no perfil de partida,
>   40 na cena de smoke), que é precisamente por que não cabe num `MAX_*`. O teto
>   de **100** do slider é CONVENIÊNCIA, e o `populate` diz isso.
> * ⚠️ **A cena autora uma aceleração BAIXA de propósito, e é a wave a morder a
>   si própria:** com o perfil de partida a paragem inteira mede **17 cm** e a
>   diferença entre freio 1 e 2 são **onze centímetros** — a cena seria um
>   contraste que ninguém vê. Com `accel = 8` as três derrapadas medem
>   **9,26 / 2,95 / 1,43 m**, e o gelo CAI no poço.
> * ⚠️ **E o clippy pegou um gate que comparava duas CONSTANTES** (*"assertion has
>   a constant value"*) — reescrito para medir a geometria MONTADA, ele nasceu
>   **vermelho**: o poço de cada raia passava por baixo do deck da seguinte
>   (`LANE_SPAN` 26 contra um poço que vai a 28). *Um oráculo que não olha para o
>   produto não vê o produto.*
> * **O fn `paint_player_section` cruzou o teto de 200 LOC** com a row nova ⇒
>   split por responsabilidade (`paint_verbs`: os dois `Fit`, a corrida gravada e
>   o `Remove` — *o pai decide o que a seção MOSTRA, o filho pinta o que ela FAZ*).


**Medido:** `walk()` usa `cfg.acceleration` nos dois sentidos
(`walk.rs:128-151`). O fator de mudança de direção cobre **inverter** e não
cobre **largar o direcional**.

**O desenho:** um **`brake_scale`** (fração do orçamento de aceleração usada
quando se está a abrandar), aplicado quando o eixo está solto.

⚠️ **Multiplicador e não um segundo número absoluto**, por duas razões e a
segunda é que decide: **(1)** `1.0` reduz **literalmente** ao mundo de hoje ⇒ o
`physics_ecs_c9` não se move e a arte já autorada não muda; **(2)** é o **mesmo
slot** em que o gelo da wave B multiplica ⇒ existe **UM** lugar onde se decide
*quão depressa este personagem pode mudar de velocidade*, em vez de dois que se
compõem por acidente.

⚠️ **`0` é legítimo e significa *não freia*** — não é *"desligado"*. É por isso
que um campo absoluto com neutro `0` estaria errado: ali `0` teria de significar
*"usa a aceleração"*, e o valor que o artista quer para gelo seria inexprimível.

⚠️ **"Frear" = o eixo está solto**, que é a definição do
`bUseSeparateBrakingFriction` do Unreal (*braking = quando não se acelera*). Com
o eixo apertado quem manda é o fator de viragem, que já existe e já está medido.

**Gates:** `1.0` é byte-idêntico (fingerprint do `c9`) · `0` conserva a
velocidade com o eixo solto · `2.0` para em metade da distância · a mutação
*"aplica o brake também com o eixo apertado"* estraga a viragem.

---

## §3 — Wave **B**: a SUPERFÍCIE fala ⟨gelo e esteira⟩ ⟨**FECHADA** 2026-08-13⟩

**Medido:** o chão contribui **dois** fatos (`normal`, `ground_velocity`), e
`friction` não aparece uma única vez no crate da lei.

**O desenho:** a amostra de chão passa a carregar mais dois:

* **`grip: f32`** (neutro `1.0`) — multiplica o orçamento de aceleração/travagem.
  Gelo é `grip` baixo, e **é por isso que esta wave depende da C**: sem o
  `brake_scale`, baixar o `grip` faria o personagem *também* arrancar devagar,
  que é o oposto de gelo.
* **`surface_velocity: Vec2`** (neutro `[0,0]`) — soma-se à `ground_velocity`. É
  a **`SurfaceEffector2D`** da Unity: *"forças tangentes ao longo da superfície
  para igualar uma velocidade"*. Hoje uma esteira exige que a plataforma **se
  mova de facto**, que ninguém constrói assim.

⚠️ **Componente novo no collider, e NÃO o `Collider.friction`.** A tentação é
*"um número só"*, e ela está errada por três razões: **(a)** a perna **flutua** —
o atrito de contato do solver literalmente nunca se aplica ao personagem;
**(b)** uma esteira não tem análogo nenhum em `friction`; **(c)** acopladas,
*"esta rampa é escorregadia para o personagem"* passaria a significar *"e todo
caixote desliza nela"*, e uma esteira **não-escorregadia** ficaria inexprimível.

⚠️ **Quem responde é o MESMO raio que ganhou.** O leque de pés toma o chão como o
**mais próximo** de N raios; a superfície tem de vir desse mesmo vencedor, senão
o personagem anda no gelo e derrapa na madeira no mesmo tique.

---

### ⟨FECHADA⟩ o que de facto shipou, e onde o plano errou

**O componente é `WalkSurface { grip, belt }`** — e o segundo campo mudou de
forma contra o plano: ele previa `surface_velocity: Vec2`, e o que shipa é um
**ESCALAR ao longo da tangente**. A razão é que o erro fica **inexprimível**: um
vetor autorado em eixos de mundo tem componente ao longo da NORMAL numa rampa, e
uma superfície não empurra ninguém para dentro nem para fora de si mesma. Como
escalar, uma esteira em rampa sobe sozinha e uma plataforma que gira leva a
correia com ela.

⚠️ **E o preço disso foi MEDIDO, não argumentado.** Com o eixo de mundo o arco
percorrido explode — `6,000 m` a 0°, **22,832** a 20° e **63,526** a 40°, com 55
m de subida —, porque a componente normal fantasma faz a PERNA brigar com um chão
que parece afundar. O oráculo do gate é a **invariância do arco**: uma correia é
um escalar, então o comprimento percorrido *não sabe da inclinação*.

**A correia entra na `ground_velocity`, não num campo próprio da amostra** — a
lei já mede tudo relativo ao chão, e uma esteira é literalmente *um chão que anda
sem o corpo andar*. Somada ali, ela chega de graça a todo consumidor daquele
campo.

**A superfície NÃO viaja no `BodyDesc`**, e a decisão tem duas metades: ela é
VIVA (o artista arrasta o slider e o personagem responde no tique seguinte) e o
SOLVER não precisa dela — só o laço do player a lê. Enfiá-la no descriptor
custaria uma linha em cada um dos ~147 sítios que o constroem.

**A faixa do slider é MEDIDA:** o `grip` satura em **4,0** com os defaults de
produto (0,6000 m em 0,1 s a 4, 6, 8, 12 e 20, idênticos) — ⚠️ e a aritmética
que eu previa dizia **6,0** (`speed / (accel · dt)`). O teto **não é um cap**:
onde a saturação cai é propriedade do PLAYER, não da superfície.

**A metade VISÍVEL:** a correia ganha uma **seta verde-água** (uma esteira parada
é indistinguível de um chão — o argumento que deu a seta ao campo de força), e o
**`grip` não ganha marca nenhuma**, de propósito: ele não tem lado, e vê-se no
personagem derrapando — o argumento com que o arrasto de área recusou a sua.

**Limitação NOMEADA:** só a lei do PLAYER lê a superfície. Um caixote sobre a
esteira **não** é levado por ela; o `SurfaceEffector2D` da Unity leva qualquer
rigidbody, e aqui o alcance é o que o nome diz.

**Cena `PH2D_PHYSICS_SMOKE=115`** — quatro raias, e a do meio é o **CONTROLE**
(sem componente nenhum). ⚠️ A primeira mensagem dela citava os números da suíte
da ponte, onde os personagens são **LANÇADOS** à mesma velocidade; nesta cena o
artista **arranca**, e o gelo — que nunca arranca — derrapava *menos* (1,16 m
contra 2,95 do controle). Com o gesto do roteiro: gelo **8,27 m e CAI**,
controle 2,95, borracha 0,87.

**`PROJECT_SCHEMA` INTOCADO** (componente opcional é chaveado pelo hash do nome
do tipo) e **`physics_ecs_c9` intocado** — a superfície não alcança o solver.

---

## §4 — Wave **D**: o TETO de queda ⟨**FECHADA** 2026-08-14⟩

**Medido:** não existe velocidade terminal. O mecanismo **já está escrito** — o
`glide_fall_speed` é exactamente um teto de descida, escopado a uma ação.

**O desenho:** `max_fall_speed` (`0` = sem teto = o mundo de hoje ao bit), pela
**mesma porta** do planeio. ⚠️ **Quando os dois estão vivos vence o MENOR** —
duas portas a clampar a mesma velocidade dariam ao planeio o poder de *acelerar*
uma queda que o teto já limitou.

⚠️ **Vale nos dois modos:** sob Spring é um boost contra a gravidade do solver;
sob Snap é um clamp no `KinematicState`. Um teto que só valesse num modo é a
assimetria que esta jornada já encontrou duas vezes.

⚠️ **E há uma decisão de produto de graça ao lado:** o Unreal põe a velocidade
terminal no **VOLUME** (`APhysicsVolume::TerminalVelocity`), e nós temos zonas —
*cair na água devia ter outro teto*. **Fora do primeiro corte**, nomeado aqui.

---

### ⟨FECHADA⟩ o que de facto shipou, e onde o plano errou

**O "medido" acima era herdado, e foi RE-MEDIDO antes de uma linha ser escrita**
(`ph2d-physics-ecs/tests/measure_terminal.rs`, a sonda que o §0 exige). Largando
de mil metros, a descida por segundo é:

| s | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| m | 5,94 | 24,86 | 44,48 | 64,10 | 83,72 | 103,33 | 122,95 | **142,57** |

Monotónica, **nos dois modos**. A premissa estava certa e agora tem número.

⛔ **E a prescrição de DUAS implementações está REFUTADA.** O plano escreveu *"sob
Spring é um boost contra a gravidade do solver; sob Snap é um clamp no
`KinematicState`"* — mas o freio do planeio já sai da lei como um **`Motor`**, e
o `kinematic_advance` **consome `motor.boost`**. Medido com o planeio armado a
`4,0`, os dois modos assentam numa descida constante (**4,25** Spring · **4,33**
Snap) em vez de continuarem a acelerar. **Uma porta serve os dois modos**, e a
segunda implementação seria a segunda resposta à mesma pergunta.

⚠️ **O controlo positivo da sonda pagou-se na primeira corrida:** as duas colunas
saíam **idênticas**, que é a assinatura de uma fixture que não contém o fenómeno.
A altura de repouso — o discriminante conhecido, porque os dois modos pousam a
alturas diferentes de propósito — media **1,4005 nos dois**. A causa é que
`pose_owner` pergunta ao **KIND do corpo** antes de olhar para o `PlayerMode`:
*é o corpo que existe que importa, não o que foi pedido*, e inserir só o
componente sobre um corpo `Dynamic` é **inerte**. Com o par (`PlayerMode` +
`RigidBody { Kinematic }`) o discriminante volta: **1,4005 × 1,0101**.

⚠️ **E a fixture da própria sonda mentiu duas vezes antes de servir:** com
`DROP = 200` o corpo **POUSAVA** aos ~5 s e as últimas colunas descreviam o CHÃO,
lendo-se como *"assentou"*; e depois de a subir para 1000, o probe do CONTROLO
passou a medir **meio-voo** com o nome de repouso (65,44 / 64,66) — ele ganhou
uma queda curta própria.

**O que shipou:** módulo novo `ph2d-platformer::descent`, e o `glide_motor`
**morreu nele**. As duas leis **propõem um número** (`descent_ceiling` fica com o
menor) e existe **um** `descent_motor` — a composição deixa de ser algo que
alguém tem de acertar e passa a ser o que a representação permite escrever.
⚠️ **O guard (`rel_up >= −ceiling`) é o teto E a prova de que `delta` é
positivo:** uma linha, duas perguntas, e é por isso que este módulo não consegue
acelerar uma queda em instante nenhum.

⚠️ **Os sete gates do planeio foram RE-APONTADOS, não reescritos** — eles sempre
mediram a lei do TETO, que agora é partilhada, e é por isso que o teto de queda
entra por baixo de gates que já provavam o comportamento do planeio, em vez de
estrear numa suíte própria que nada obriga a concordar com a antiga.

⚠️ **O teto assenta ~6% ACIMA do número autorado** (`4,0` → **4,25**/**4,33**): o
freio é aplicado no topo do tique e a gravidade soma **dentro** dele. Está
nomeado porque um gate de igualdade exacta nasceria **vermelho sobre produto
correto** — o oráculo honesto é *a descida PARA de crescer*.

**A metade VISÍVEL é um card PRÓPRIO** (o 12º, `FALL`), e não uma row dentro do
GLIDE: os dois são tetos da mesma velocidade e compõem por uma porta só, mas um
dura **o que o dedo durar** e o outro vale **sempre** — o card é onde o artista
lê *o que está a autorar*, e juntá-los pedia que ele descobrisse a diferença
lendo a dica. **Sem teto no valor digitável**, pelo §0: o número que a faixa tem
de conseguir descrever é o da medição (142,57), não um redondo confortável.

**`PROJECT_SCHEMA` 79 → 80** (`max_fall_speed` apendado ao `PlatformPlayer`;
postcard é posicional ⇒ quebra dura). O default `0,0` **desliga** a lei, então
todo projeto salvo antes reabre a cair exactamente como caía, e o
`physics_ecs_c9` fica **byte-idêntico** — a prova de que o degrau não move física
nenhuma.

**Gates:** 10 no kernel (3 novos) + 3 no ECS **pelos dois modos** + 2 na shell + a
varredura de seam. **4 mutações, 4 sangram:** a ponte largar o teto (os três do
ECS) · `min` → `max` (o gate da composição, no kernel **e** no produto) · o braço
da shell escrever no **VIZINHO** (`glide_fall_speed` — o modo de falha que esta
wave de facto produz: dois campos de nome quase igual, fiados no mesmo commit) ·
e o clamp negativo.

⚠️ **E a varredura do seam deixou de comparar a contagem com um literal ao lado
dela:** um número escrito à mão só sabe dizer *"a tabela mudou"*, e quem o bumpa
para ficar verde faz exactamente o que ele existia para impedir. Comparada com o
`len()` da própria lista, a asserção passa a afirmar a propriedade que interessa:
*toda row pintada é varrida aqui*.

**Cena `PH2D_PHYSICS_SMOKE=116`** — três raias iguais largadas de 16 m, e só os
números diferem: sem teto **1,52 s**, com teto **4,05 s**, com teto+planeio e o
dedo preso **9,22 s**. ⚠️ **O passo 1 manda NÃO tocar em nada**, e é essa metade
que separa esta cena da irmã 112: o planeio precisa do dedo, o teto não pergunta
nada ao jogador. ⚠️ **O número da cena é CONTADO do `match`** — a nota da §5 do
`CLAUDE.md` dava a `=105` como a próxima livre e tinha envelhecido em onze cenas.

**Segue FORA de escopo, e agora com o mecanismo ao lado:** o teto por **ZONA**
(`APhysicsVolume::TerminalVelocity`). Ele não é *"mais um número"* — a zona
precisaria de o entregar por consulta, e re-derivar ali o frame, o espelho e o
falloff seria a **segunda resposta** a *"que empurrão esta zona dá neste
ponto?"*, exactamente a recusa que a `W-KinMove` já nomeou para a FORÇA.

---

## §5 — Wave **E**: o EMPURRÃO de fora

**Medido:** sob Spring um impulso chega ao corpo dinâmico e o `walk` **resiste**
em vez de o apagar (o boost só dispara dentro de `a·dt`); sob **Snap/Pure** um
impulso não faz **nada**, porque quem possui a velocidade é o `KinematicState`.

**O desenho:** uma porta `launch_player(entity, velocity)` que enfileira **um
tique** de acréscimo, honrada pelos **três** modos pela mesma porta — o
`LaunchCharacter` do Unreal, que existe precisamente porque o controlador comeria
a velocidade.

⚠️ **Soma, não substitui**, no primeiro corte (o `bXYOverride` do Unreal é uma
segunda pergunta e entra quando houver quem a peça). ⚠️ E ele **silencia o
controle aéreo** por uma janela — primitivo que **já existe**
(`wall_jump_lockout`), e reusá-lo é o que impede o empurrão de ser comido pelo
próprio jogador a segurar a direção contrária.

---

## §6 — As pequenas, e a sonda

* **J · a sonda do Snap** — *largar uma plataforma a 4 m/s nos dois modos e
  comparar a trajetória*. ⚠️ **Medir antes de afirmar**: se divergirem, a cura é
  o canal da wave E (somar uma velocidade de fora), não um caso especial.
* **G · `bCanWalkOffLedges`** — um veredito a mais quando o leque vê o chão
  acabar.
* **H · voar/noclip** — um modo de depuração.
* **I · air control boost** — um campo e um `if`.

---

## §7 — O que fecha CADA wave

1. as **quatro condições de UI** do plano 00 (o componente existe · é pintado e
   registado · o clique chega ao barramento · **e a sequência leva a algum
   lugar**);
2. **gates com mutação provada**, e a mutação é escrita ANTES de se dizer que o
   gate serve;
3. uma **cena de smoke com números MEDIDOS**, que imprime o que montou;
4. o `physics_ecs_c9` **conferido** — e onde ele se mover, a wave diz porquê.

⚠️ **O número da cena CONTA-SE no `physics_smoke.rs`** — e este parágrafo já
envelheceu uma vez: ele dizia *"hoje o máximo é `=112`"* e o `match` trazia
**`=115`** quando a wave D o foi ler. **Conte o `match`, nunca esta linha**; o
`=84` não existe de propósito, e a wave D levou o **`=116`**.

⚠️ **O `PROJECT_SCHEMA` conta-se contra o `main` do dia**, e o degrau da escada é
escrito no MESMO commit.

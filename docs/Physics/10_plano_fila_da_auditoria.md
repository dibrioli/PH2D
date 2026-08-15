# Plano 10 — a fila da auditoria 09, desenhada (2026-08-12)

> **Ordem do Enio:** *"ok. Planeje, salve o plano e comece"*, sobre a fila da
> [auditoria 09](09_auditoria_engines.md).
>
> Este doc é **plano**. Cada wave traz o desenho, as decisões com o preço ao
> lado, e o que a fecha (gates · mutação · as quatro condições de UI do plano 00
> · cena de smoke com números **medidos**). ⚠️ **Nenhum número de produto é
> escolhido aqui** (§0): quem os escreve é a medição da wave.

```
A (a saída) ✅ → C (o peso) ✅ → B (a superfície) ✅ → D (o teto) ✅ → E (o empurrão) ✅
   → J (a sonda do Snap) ✅ → G ✅ · H ⛔ · I ⛔
```

> **Estado (2026-08-15): a fila FECHOU.** **A**, **C**, **B**, **D**, **E**, **J**
> e **G** construídas; **H** e **I** **recusadas por MEDIÇÃO** (§6) — a capacidade
> do noclip já existe pelos gestos do editor, e o sintoma que o boost de controlo
> aéreo cura não existe neste produto. ⚠️ *Duas recusas medidas valem tanto quanto
> sete waves construídas: elas são o que impede a fila de voltar.*

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

## §5 — Wave **E**: o EMPURRÃO de fora ⟨**FECHADA** 2026-08-14⟩

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
### ⟨FECHADA⟩ o que de facto shipou, e onde o plano errou

**O "medido" acima foi RE-MEDIDO** (`ph2d-physics-ecs/tests/measure_launch.rs`),
e as duas metades confirmaram-se com número:

* uma explosão ao lado do personagem alcança **1** corpo sob Spring e **ZERO**
  sob Snap e Pure — o botão existia, o toast dizia *"0"*, e o personagem ficava
  parado ao lado do estouro;
* e sob Spring, onde ela SEMPRE chegou, a caminhada apaga-a em **9 tiques
  (0,15 s)**: `13,92 m/s` no primeiro, `0,000` no décimo, **com o jogador a não
  tocar em nada**. ⚠️ Quem come é o **FREIO**, não o direcional — e isso muda o
  desenho: a janela não é sobre o *controlo aéreo*, é sobre a caminhada inteira.

⚠️ **A JANELA é do CHAMADOR, e não o `WallConfig::jump_lockout`.** O plano dizia
*"reusá-lo"*, e o que se reusa é o **mecanismo** (um relógio que cala a
caminhada, lido pelo `if` que já existe no `lib.rs`); o **número** é de quem
empurra — uma explosão e uma almofada de salto não são donas do personagem pelo
mesmo tempo, e ler o número da parede faria um knob significar duas coisas.

⚠️ **E ela NÃO é o `JumpState::wall_lock`, por um argumento que o próprio doc
daquele campo faz:** ele morre ao pousar, porque *"quem aterrou espera dirigir"*
— e um empurrão acontece na maior parte das vezes **com o pé no chão**. Campo
próprio, `PlayerState::push_lock`, que escorre no ar e no chão.

⚠️ **Ele mora no `PlayerState` e não num mapa da ponte**, e a razão está escrita
no doc daquele tipo: é o `PlayerState` que o **ring de tiques âncora** guarda, e
uma janela noutro mapa teria de ser acrescentada ao ring à mão — esquecê-la é um
scrub que devolve o mundo de um tique e a memória do controlador de outro.

**A porta:** `PhysicsBridge::launch_player(entity, velocity, lock)`, num mapa
**próprio** e **DRENADO** — a entrada do dedo é *set-and-hold* (um dispatch que
deve quatro tiques aplica a MESMA a todos eles) e um empurrão é um **evento**;
guardado ali, ele seria entregue quatro vezes. E ela **descarta o ring**, pela
razão exacta do `explode`: é uma descontinuidade que a fita não grava.

⚠️ **A velocidade sai pela porta do `boost`, e é a MESMA descoberta da wave D:**
os dois modos já a consomem (`kinematic_advance` soma-o; a ponte dinâmica
manda-o ao solver como impulso), então uma segunda entrega por modo seria a
segunda resposta à mesma pergunta.

**A metade VISÍVEL é uma CORREÇÃO, sem UI nova:** a **explosão** que já está no
Inspector passa a alcançar os três modos. Ela converte impulso em velocidade
**pela massa real** e com a **mesma falloff** (`ph2d_physics::blast_falloff`,
porta única), e a um player dinâmico dá **só a janela** — o solver já lhe
entregou o impulso. Medido, o mesmo estouro leva-o **5,81 m** em meio segundo
contra **1,03** antes.

⚠️ **E ela conta UM corpo por personagem:** a primeira versão somava a varredura
de players por cima do que o `PhysicsWorld::explode` já contara, e o toast dizia
*"2 corpos"* para um personagem sozinho na cena.

**`PROJECT_SCHEMA` INTOCADO** (nada disto é serializado) e **`physics_ecs_c9`
byte-idêntico** — `1699123f…`, 117 corpos, medido contra o commit ANTERIOR à wave
D num worktree temporário. ⚠️ **E a diferença contra o `main` NÃO é destas waves:**
medido no `main` de hoje, ele dá **`fb27f676…`** — exactamente o que a §5 do
`CLAUDE.md` regista —, e o que o move é a **jornada da ÁGUA**, que já estava na
linha (a wave A registou-o). *Nem a D nem a E lhe tocam.*

**Cena `PH2D_PHYSICS_SMOKE=117`** — três personagens IGUAIS, um por modo, e o
artista estoura debaixo de cada um com a ferramenta que já existe. ⚠️ **Duas
versões dela mediram a caixa em vez do modo:** uma *régua viva* ao lado de cada
personagem entrava no caminho (à frente, o do meio andava `0,740 m` contra
`4,979` do vizinho; atrás, a caixa de um é a da frente do outro) — as raias estão
a 4 m e o empurrão leva-os 5 a 8, então **qualquer** objecto entre eles é um
obstáculo. ⚠️ **E os números do gate eram inventados** (`raio 6`), com o da
direita exactamente na borda do alcance: `0,000 m`, um gate vermelho sobre
produto correto. Os defaults da ferramenta são `3`/`10`, e é deles que a cena
vive.

⚠️ **ASSIMETRIA nomeada, não escondida:** o mesmo estouro leva o dinâmico
**5,57 m** e os cinemáticos **8,09** — eles saem quase juntos (`15,3` contra
`17,9 m/s`) e **param diferente**, porque um é travado pelo SOLVER quando a
janela acaba e os outros pela caminhada, que rampeia. Está no roteiro, para o
smoke julgar.

**Gates:** 8 no ECS (a porta, a janela, o dedo contrário, o dreno, a contagem, a
massa e a explosão nos três modos) + 3 na cena. **6 mutações, 6 sangram.**
⚠️ **Uma delas SOBREVIVEU primeiro, e a culpa era da fixture:** trocar a massa
real por `1.0` deixava tudo verde porque o personagem pesa **1,0 kg**. E a
fixture que a corrigiu nasceu ERRADA por sua vez — ela autorava o
`MassOverride` **depois** de o personagem assentar, e o `reconcile` só
re-descreve um corpo **em repouso**, então media o mesmo número com e sem ele.
Autorada antes do primeiro tique: `0,218 m` contra `7,106`.

**Aberto, com o preço ao lado:** o campo de ATRAÇÃO ainda não alcança um player
de pose própria (é sustentado, não um evento — pede um canal por-tique, não esta
porta) · o `bXYOverride` do Unreal (substituir em vez de somar) segue fora, e
entra quando houver quem o peça.

---

## §6 — As pequenas, e a sonda

* **J · a sonda do Snap** ⟨**FECHADA** 2026-08-14, e a medição mudou a wave⟩ —
  *largar uma plataforma a 4 m/s nos dois modos e comparar a trajetória*.
  **Medido** (`measure_platform_leave`, meio segundo de voo, o CONTROLE é a mesma
  cena com a plataforma parada):

  | plataforma (4 m/s) | Spring | Snap | Pure |
  |---|---|---|---|
  | vagão horizontal | **100%** | **100%** | **100%** |
  | elevador SOBE | 100% | 107% | 107% |
  | elevador DESCE | 78% | 95% | 95% |

  ⚠️ **O Snap NÃO diverge, e o item §3.J da auditoria fecha VERDE por medição:** a
  premissa dela (*"sob Snap a `ground_velocity` não lhe é somada quando o chão
  desaparece"*) está **refutada** — a memória do `lift_momentum` (W10) funciona
  nos três modos, e a diferença que resta no elevador que desce (78 × 95%) é a
  **folga da mola** do corpo dinâmico, física real e não um defeito. ⇒ *a cura do
  canal da wave E, que o plano prescrevia, não era necessária.*

  ⚠️ **E a sonda achou o que a auditoria também lista** (linha 95 da tabela dela,
  `platform_on_leave (3 modos)`): **a política faltava**, e os três modos
  partilhavam o buraco. Pular de um elevador a descer dava pico **0,378 m**
  (Spring) e **0,016** (Snap) contra ~1,87 num chão parado — o artista autora dois
  metros e recebe um centímetro e meio. Não é bug do solver: a altura autorada era
  medida **contra a plataforma** (o `ADD_VELOCITY` do Godot, e o único mundo que
  existia). Construída a política (`PlatformLift` · `Full` / `Up Only` / `None`),
  com o default **`Full` = o mundo que já shipava, byte a byte**. Cena **118**.

* **G · `bCanWalkOffLedges`** ⟨**FECHADA** 2026-08-15, e a medição reescreveu a
  wave DUAS vezes⟩ — a linha do plano dizia *"um veredito a mais quando o leque
  vê o chão acabar"*, e a §0 mandou medir antes de escrever a lei. As duas
  refutações, por ordem:

  **(1) O leque NÃO consegue responder à pergunta.** O primeiro corte lia a
  quina dos **pés que perderam o chão** — e a sonda mediu que ele acende sobre
  uma **fenda de 5 cm**, que o corpo de 40 cm atravessa sem esforço:

  | fenda | tiques acesos / sobre a fenda | atravessou? |
  |---|---|---|
  | 0,05 | 1 / 7 | sim |
  | 0,10 | 2 / 8 | sim |
  | 0,30 | 5 / 11 | sim |

  O leque só amostra **DENTRO** da pegada, então *"o chão acaba"* e *"há um
  buraco à minha frente"* chegam-lhe idênticos, e nenhum arranjo dos pés que já
  existem os separa. ⇒ o sensor passou a ser a **MESMA perna castada à frente**
  (`cast_leg` verbatim, com as duas leis de redução intactas), o que faz a trava
  e o leque concordarem **por construção**: ele recusa andar exactamente onde
  deixaria de ser segurado.

  **(2) O alcance NÃO podia ser um knob, e a medição matou o que eu tinha
  escrito.** Com um `ledge_look` autorado, a 8 m/s um `0,30` deixa o personagem
  **CAIR** e um `0,60` o segura — e a fronteira é exactamente `v²/(2a) = 0,533`:
  *o valor certo do knob era função de OUTROS DOIS knobs*, a forma que este repo
  já removeu uma vez (o Conserve do sculpt). O alcance passou a ser **derivado**,
  e ele tem **duas metades, cada uma de quem a sabe**: a lei dá a distância de
  paragem (`v²/2a`) e a ponte soma a **meia-largura do corpo**, porque a pergunta
  certa é *"quando eu parar, ainda haverá chão onde a minha BORDA estiver?"*.

  ⚠️ **Sem a segunda parcela o alcance é o CASO DE FRONTEIRA** — ele trava no
  instante em que a perna deixa de o segurar. Medido a 2 m/s, ele acabava
  equilibrado num pé só sobre o lábio **e caía na mesma**, enquanto as outras
  velocidades escapavam por um fio (pelo bónus de mudança de direção). Com as
  duas metades, ninguém cai:

  | vel (m/s) | alcance derivado | borda do corpo vs quina | caiu? |
  |---|---|---|---|
  | 1 | 0,0083 | +0,175 | não |
  | 2 | 0,0333 | +0,181 | não |
  | 4 | 0,1333 | +0,156 | não |
  | 6 | 0,3000 | +0,135 | não |
  | 8 | 0,5333 | +0,048 | não |
  | 12 | 1,2000 | −0,131 | não |

  ⚠️ **A fenda LARGA é a semântica, não um defeito:** um vão que a perna não
  vence é um patamar, e a trava recusa andar para ele mesmo que a inércia o
  cruzasse — que é literalmente o que *"não ande para fora"* significa. Medido: a
  0,50 m ele pára, e o CONTROLE sem trava cruza-o.

  Mais o `bCanWalkOffLedgesWhenCrouching` (o **sneak-to-the-brink** do Unreal),
  que só **APERTA** — a porta é a `walk_for`, que o doc dela já reservava para
  *"o dia em que um terceiro termo tiver de encolher"*. Cena **119**.
* **I · air control boost** ⟨**RECUSADA por medição** 2026-08-15 — o sintoma que
  ela existe para curar não existe neste produto⟩ — a auditoria descreve o item
  por um SINTOMA (*"tira a sensação de 'não consigo sair do lugar' no topo de um
  pulo vertical"*), e a §0 manda medir o fenómeno antes de escrever a cura.
  Sonda `measure_air_control`, pulo vertical parado com o direcional apertado:

  | segura a partir do tique | deriva (m) | vel no ápice | tiques no ar | pico (m) |
  |---|---|---|---|---|
  | 1 | 6,88 | **5,9999** | 73 | 1,75 |
  | 5 | 6,53 | **5,9999** | 73 | 1,75 |
  | 10 | 6,09 | **5,9999** | 73 | 1,75 |
  | 20 | 4,98 | **6,0000** | 73 | 1,75 |

  ⚠️ **No ápice ele já corre à velocidade de CRUZEIRO** (`speed = 6,0`) —
  *não há o que um multiplicador acelere*. E multiplicar o `air_acceleration`
  confirma-o pelo outro lado: **8× a aceleração compra 8,5% de deriva e move a
  velocidade do ápice em ZERO.**

  | × base | air_accel | deriva (m) | vel no ápice |
  |---|---|---|---|
  | 0,5 | 10,0 | 6,20 | 5,9999 |
  | 1,0 | 20,0 | 6,88 | 5,9999 |
  | 2,0 | 40,0 | 7,22 | 5,9999 |
  | 4,0 | 80,0 | 7,39 | 5,9999 |
  | 8,0 | 160,0 | **7,47** | **5,9999** |

  **O regime em que o sintoma EXISTE também está medido — e o knob que o cura já
  shipa.** Varrendo o `air_acceleration` para baixo, a fração do cruzeiro que ele
  alcança no ápice é:

  | air_accel | deriva (m) | vel no ápice | fração do cruzeiro |
  |---|---|---|---|
  | 0,5 | 0,72 | 0,44 | **7%** |
  | 1,0 | 1,27 | 0,88 | 15% |
  | 2,0 | 2,32 | 1,72 | 29% |
  | 5,0 | 4,84 | 3,99 | 67% |
  | 10,0 | 6,20 | 6,00 | **100%** |
  | 20,0 ⟨o default⟩ | 6,88 | 6,00 | **100%** |

  ⚠️ **É por isso que o Unreal precisa do multiplicador e nós não:** lá o
  `AirControl` é uma **FRAÇÃO da velocidade de caminhada** (5% por default),
  então subi-lo globalmente faz o personagem *voar à velocidade cheia o tempo
  todo* e o boost existe para resgatar só o começo; aqui ele é uma **aceleração
  própria** (20 m/s², que alcança o cruzeiro em 18 dos 73 tiques de voo) e o
  CAP continua a segurar o teto. Um `air_control_boost` seria a **segunda porta**
  para *"quero mais controlo no ar"*, cuja primeira já está no painel — a forma
  que este repo removeu no Conserve do sculpt. **Não construir.**

* **H · voar/noclip** ⟨**RECUSADA por medição** 2026-08-15 — a capacidade já
  existe pelos gestos que o editor tem⟩ — a auditoria dá-lhe valor baixo com uma
  razão escrita (*"útil para percorrer um nível grande no editor, valor de
  produto baixo num editor que já tem câmera livre"*), e a §0 manda medir a
  premissa antes de a aceitar **ou** de a recusar. Sonda `measure_noclip`, com
  uma parede sólida em `x ∈ [4, 8]` e o toggle **Physics** desmarcado:

  | pedido | ficou |
  |---|---|
  | no meio da PAREDE ⟨6,0 · 4,0⟩ | **6,0000 · 4,0000** |
  | do outro LADO dela ⟨12,0 · 0,9⟩ | **12,0000 · 0,9000** |
  | bem lá em CIMA ⟨6,0 · 20,0⟩ | **6,0000 · 20,0000** |

  E a sim **retoma dali**: largado em `(12,0 · 6,0)`, oitenta tiques de Play
  deixam-no em `(12,0000 · 0,9005)` — **0,0000 m de deriva lateral**, caindo no
  lugar, do outro lado da parede. ⚠️ **Com CONTROLE**, senão as duas linhas
  descreveriam um mundo sem parede: empurrado contra ela com o relógio a andar,
  ele **pára em x = 3,8011** contra uma parede que começa em 4,0.

  ⇒ *desmarcar Physics → arrastar → marcar* já é o noclip inteiro, com a metade
  que importa (**a física retoma de onde ele foi largado**) gateada desde o W4b.

  ⚠️ **O que a recusa NÃO cobre, dito por inteiro:** o `MOVE_Flying` do Unreal é
  um modo em que se **voa com as teclas durante o play**, e isso não existe aqui
  (a MÃO da cena `=52` agarra por mola *através do solver*, logo colide). O que
  o gesto existente dá é **teleporte com o relógio parado** — que é exactamente
  o caso de uso que a auditoria nomeia (*percorrer um nível grande no editor*) e
  não o que ela não nomeia. Construir o modo custaria um membro de enum de modo,
  um chip, a fiação e um gate, para uma segunda resposta a uma pergunta já
  respondida. **Não construir** — e se um dia o pedido for *voar jogando*, é
  feature de produto com smoke próprio, não a limpeza de uma pendência.

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

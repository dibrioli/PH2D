# HANDOFF MESTRE — `line/physics` → `main` (2026-08-05)

> ⛔ **SUPERSEDED pelo [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-08.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-08.md).**
> Este documento para na **W23**, e a jornada seguiu (W24..W27 + o plano 07
> inteiro + o segundo modo). **Os NÚMEROS daqui estão velhos** — o
> `PROJECT_SCHEMA` já não é 59, o registro já não é 28 e o `c9` já não é
> `74d4ea5d…`. O que continua válido, e é por isso que ele não foi apagado, é o
> **mecanismo** das waves W11b..W23 (§4..§5j), que o handoff de 08/08 aponta em
> vez de copiar.

**A linha está FECHADA e PARADA.** 12 commits, 4 waves de produto.
Nada integrado, nada pushado.

> Este handoff **supersede** o [`HANDOFF_INTEGRACAO_line_physics_W11b_2026-08-05.md`](HANDOFF_INTEGRACAO_line_physics_W11b_2026-08-05.md),
> que descreve só o começo da jornada (a W11b/W11c) e cujo título deixou de ser
> verdadeiro quando a W12 e a W13 entraram. O detalhe de mecanismo daquelas duas
> continua lá e não foi copiado para cá.

---

## §1 — O que entra

| commit | o quê |
|---|---|
| `2b4f0df0d` | o degrau **v54** que faltava na escada do `PROJECT_SCHEMA` |
| `865aa410f` | **W11b** — o que cancela a gravidade passa a ser integrado como ela |
| `43852a422` | o handoff da W11b |
| `f3cfb9b96` | o veredito do 1º smoke + duas tentativas medidas e mortas |
| `aec55c5b6` | **W11c** — o default do amortecimento sobe ao TETO |
| `ce78651cf` | **W12** — descer da plataforma jump-through |
| `eebac6d1e` | a W12 no plano e no mapa |
| `14c95c974` | **W13** — AS PAREDES |
| `33dfc5cbc` | a W13 no plano e no mapa |
| `0ae18e65f` | este handoff |
| `05edd9c73` | **W14** — O ARRANQUE |
| `d10f1a556` | a W14 no plano e no mapa |
| `73471284a` | **W15** — O AGACHAR |
| `695ca8c25` | a W15 no plano e no mapa |
| `de73fc6d6` | **W16** — ASSAR UM PLAYER: o bake replaya a CORRIDA, não o dedo |
| `971d2fdc9` | a W16 no plano e no mapa |
| `33e76d43c` | **W17** — A CORRIDA SOBREVIVE AO ARQUIVO |
| `555c27393` | a W17 no plano, no mapa e neste handoff |
| **(esta wave)** | **W18** — o piso do agachar é dito em voz alta |

**Smoke:** W11b/W11c **APROVADAS** (*"Smoke OK"*, 2026-08-05), **W12 e W13
APROVADAS** na mesma data (*"Smoke OK. SIGA"*) e **a W14 APROVADA** logo a seguir
(*"Smoke OK. Siga"*), **a W15 APROVADA** a seguir (*"Smoke OK. SIga"*) e **a W16
APROVADA** logo depois (*"Smoke OK. SIga"*) e **a W17 APROVADA** a seguir
(*"smoke OK. Siga"*). **A W18 é a única pendente** — integrar não é aprovar.

---

## §2 — Os números que se contam

| número | veredito |
|---|---|
| **`PROJECT_SCHEMA`** | ⚠️ **55 → 59** (W13, W14, W15 e W17, um degrau cada; **a W16 e a W18 não bumpam**) — ver §3 |
| `FLIP_SCHEMA` · `VEC_SCENE` | intocados (13 · 14) |
| registro `ph2d-physics-ecs` | **INTOCADO** (28) — nenhum componente novo |
| registro `ph2d-ecs` (as 3 casas) | **INTOCADO** |
| gizmo ids | **nenhum novo** — o próximo livre segue **974** |
| ADR | **nenhum** ⇒ a linha fica fora de toda disputa de número |
| contrato congelado | **4/4 verde**, rodado |
| `Cargo.toml` | **zero** — nenhuma dep, nenhuma crate |
| **`physics_ecs_c9`** | **`74d4ea5d…`, 108 corpos, debug ≡ release** |
| suítes | **debug e release**, mais a shell — verdes, zero falhas |

⚠️ **O `c9` moveu-se UMA vez na jornada inteira, e foi na W11b/W11c** (a altura de
repouso do player mudou). **A W12, a W13, a W14, a W15, a W16 e a W17 são
byte-neutras** — a descida, o arranque e o agachar exigem botões que a fita do
harness não segura; as paredes, o arranque e o agachar nascem **desligados**; a
W16 não toca o caminho do `dispatch`, só o do bake; e a W17 muda **quando** a fita
grava, o que o `c9` (que não a lê) não pode ver. Isso não é sorte: é a prova
executável de que as capacidades novas são opt-in.

⚠️ **E é justamente por isso que a W17 tem gates PRÓPRIOS de gravação:** um `c9`
byte-idêntico prova que a simulação não mudou e diz **zero** sobre a fita.

---

## §3 — ⚠️ O bump, e por que ele é PROVISÓRIO

**W17:** ⚠️ o degrau que **não** é um campo de componente — é um campo de
**ARQUIVO**, `player_tape`, com o que o dedo do jogador fez tique a tique. Fora do
`ProjectState` pelo motivo de `motion`/`timeline`/`physics`: aquele é a unidade do
undo GLOBAL, e um Ctrl+Z do canvas não deve rebobinar uma gravação. A linha
escreve **59**.

**W15:** o `PlatformPlayer` ganhou **dois** campos (`crouch_height`,
`crouch_speed`) — mesmo raciocínio posicional. A linha escreve **58**. ⚠️ E vale
notar o que este degrau **não** traz: nenhuma dimensão de collider muda, porque
agachar aqui é uma perna mais CURTA e não um corpo menor.

**W14:** o `PlatformPlayer` ganhou **três** campos (`dash_speed`, `dash_time`,
`dash_cooldown`) — mesmo raciocínio posicional.

**W13:** o `PlatformPlayer` ganhou **cinco** campos (`wall_slide_speed`,
`wall_jump_height`, `wall_jump_push`, `wall_jump_lockout`, `wall_reach`).
Apendados ao componente, e o postcard é **posicional** ⇒ um save v55 lido por v56
chega ao fim dos bytes no primeiro campo novo. O número é o que transforma isso
num erro de **VERSÃO** em vez de num postcard a falhar longe da causa.

⚠️ **O valor se CONTA contra o `main` do dia da integração, nunca se escolhe.**
Esta linha escreve **58**; se outra linha da janela bumpar, o certo pode não estar
em nenhum dos dois lados do conflito — foi o que aconteceu três vezes com a
`line/FLIP` (30 · 32/33/34 · 47) e uma com o próprio handoff desta linha, que
contou UM degrau onde havia DOIS.

⚠️ **E o `project.rs` pode não conflitar mesmo assim:** se as duas linhas
escreverem o mesmo literal, o git funde limpo e o bump da segunda **evapora com a
suíte verde**. Quem denuncia é o conflito do `project_schema_tests.rs` ao lado, e
a tripla que ele pina — **`(59, 13, 14)`** aqui.

---

## §4 — W12: descer da plataforma (cena `=91`)

O plano 06 §4 agendava isto como *"o mecanismo existe (`world/oneway.rs`); a
feature é o gesto, e é uma wave curta depois da W8"*. A previsão sobreviveu à
construção.

**O gesto é `down + jump`** — não `down` sozinho: quem segura baixo enquanto anda
não pode cair da plataforma sem ter pedido, e o dia em que existir um agachar o
botão já estará com o significado certo.

⚠️ **A lei diz COMEÇAR; a ponte diz quando ACABA** — a divisão do sensor de quina
da W10. E o **fim da descida não é um relógio**: *"eu já passei?"* tem resposta
exata (a caixa do personagem inteiramente abaixo da caixa da plataforma), e um
temporizador erraria justamente onde dói — plataforma grossa, queda lenta —
re-solidificando com o personagem dentro dela.

⚠️ **O sensor tem de excluir a plataforma, e não só o solver:** quem segura o
personagem no ar é a **MOLA**, e ela age porque o raio achou chão. Daí o
`cast_ray_skipping`, com o `cast_ray` a **delegar** — uma porta, duas faces.

⚠️ **O bit viaja no corpo que CAI** (`DROP_THROUGH_BIT`, o 2º consumidor que o
doc do `ONE_WAY_BIT` previa), escrito em **TODOS** os colliders do corpo (a lição
da W-Compound), e **por tique**, nunca no `BodyDesc`.

**7 mutações, 7 sangram.** Caso degenerado nomeado: um vão menor que o personagem
deixa a descida armada para sempre — cena já quebrada sem descida nenhuma.

---

## §5 — W13: as paredes (cena `=92`)

⚠️ **O §4 previa duas waves e são UMA** — as duas metades partilham a pergunta
*estou agarrado?*, e separá-las daria duas respostas para *o que conta como
parede*. A previsão foi corrigida **na linha em que foi escrita**.

⚠️ **Uma parede é o que a PERNA já recusou.** Sem segundo limiar: um
`wall_min_angle` discordaria do `Max Slope` autorado.

### ⚠️ A medição derrubou DUAS frases minhas

**(1) A lei que eu escrevi.** O escorregamento era um TETO, raciocinado. O knob é
**INERTE**: medido, quem empurra contra uma parede **não cai** — 9 cm em um
segundo, por **atrito** (`DEFAULT_FRICTION = 0,5`) mais a gravidade do **ÁPICE**
(metade do peso, auto-reforçante). A lei que ficou **DEFINE** a velocidade.

| `wall_slide_speed` | desceu em 1 s |
|---|---|
| 0,0 (desligado) | **0,09 m** ← a COLA |
| 1,0 | 0,71 m |
| 3,0 | 2,76 m |
| 6,0 | 5,51 m |
| 12,0 | 11,01 m |

**(2) *"O afastamento satura"*.** Ele **não satura** — cresce linear, porque com o
controle aéreo calado nada freia a horizontal. Quem satura é a **ALTURA**, e é
dali que sai o `jump_lockout = 0,2 s`.

| `jump_lockout` | subiu (de 2,0 autorados) | afastou |
|---|---|---|
| 0,00 s | 1,621 m (81%) | 0,462 m |
| 0,10 s | 1,921 m (96%) | 1,137 m |
| **0,20 s** | **1,932 m (97%)** | **1,737 m** |
| 0,50 s | 1,932 m (97%) | 3,437 m |

⚠️ E o pulo entregar 76% era a **mesma doença que o `lift_momentum` da W10
nomeou** — *"quem apagava era a ASSISTÊNCIA"*.

**5 mutações, 4 sangram**, e a 5ª nomeia uma **defesa em camadas** (o
`drive * side` do `cling` é inalcançável pela ponte; quem o mata é o gate de
unidade). **Nasce DESLIGADA** — card **WALLS** próprio na §14, cinco rows.

---

---

## §5b — W14: o arranque (cena `=93`)

Um botão (**`Q`**) e o personagem dispara em linha reta na direção para onde
olha — a última das duas *actions* que o §4 do plano 06 deixava fora.

⚠️ **O que ele custa não é um termo a somar, é um REGIME:** enquanto dura, a
**perna cala**, a **caminhada cala** e a **gravidade é cancelada**. As três são
uma frase só — *durante o arranque o personagem é uma velocidade* —, e é isso
que faz o desenho ser uma reta em vez de um arco que depende de onde ele
começou.

⚠️ **A velocidade é DEFINIDA, nunca somada** (a lição da W13). Medido, o
percurso é o autorado **ao milímetro**:

| `dash_speed` | percorrido | autorado | a andar, nos mesmos 9 tiques |
|---|---|---|---|
| 8 | 1,200 m | 1,200 | 0,900 m |
| 12 | 1,800 m | 1,800 | 0,900 m |
| **18** | **2,700 m** | **2,700** | 0,900 m |
| 26 | 3,900 m | 3,900 | 0,900 m |

⚠️ **O que impede voar é a CARGA, não a recuperação** — um arranque por
tempo-de-voo, reposto pelo pé no chão. Um relógio sozinho deixaria esperar e
arrancar de novo, para sempre; e é por isso que a carga **é lei e não knob**:
expô-la seria oferecer ao artista um bug com um slider.

⚠️ **`JumpState` + `DashState` viram `PlayerState`**, um tipo só — e a razão não
é estética. É ele que a **fita** guarda no ring de tiques âncora, então um estado
de player num segundo mapa da ponte teria de ser acrescentado ali **à mão**, e
esquecê-lo é um scrub que devolve o mundo de um tique e a memória do controlador
de outro. `JumpState` **mantém o nome**: ele é o que `jump_step` toma e devolve.

**10 mutações, 10 sangram.** ⚠️ **E DUAS fixtures minhas nasceram VERDES sobre
nada, as duas pela mesma doença** — o oráculo media uma coisa que não era o
botão:

1. A recusa do 2º arranque media o **deslocamento cru**, e depois de um arranque
   o corpo continua a 18 m/s: **1,981 m de pura inércia** liam-se como *"o
   segundo saiu"*. O oráculo certo é a DIFERENÇA contra um controle.
2. A mutação do ring era invisível porque o estado que ela largava era, **por
   acidente da cena, igual ao que mantinha** (a âncora caía antes do arranque; e
   depois, a corrida não ia longe o bastante para o personagem POUSAR, então
   *"agora"* também dizia *carga gasta*). Corrigida, ela diverge **0,80 m**.

⚠️ **Tecla `Q`, por CONFLITO medido:** `X`/`C`/`V` são o clipboard e o
pathfinder, e um **modificador** seria pior do que uma tecla ocupada — `Shift`
qualifica meia dúzia de handlers deste app, e um botão de jogo que também
qualifica outros é um botão com dois donos.


## §5c — W15: o agachar (cena `=94`)

**O que ele é:** segurar BAIXO baixa o personagem e o deixa passar por onde não
cabe de pé. Soltar debaixo de um teto **não** o levanta.

⚠️ **A previsão do plano 06 §4 estava ERRADA, e a construção corrigiu-a.** Ela
mantinha o agachar de fora dizendo que ele *"exige **encolher o collider**"* e que
por isso tropeçava na premissa que a W-Compound derrubou (*"um corpo tem
exactamente um collider"*). **Não exige.** O personagem é uma cápsula
**FLUTUANTE** (D1): a silhueta dele acima do chão vale `float_height +
meia-altura do collider`, e baixar a perna baixa a silhueta INTEIRA pelo mesmo
delta, com a forma intocada — medido, **topo `1,602 → 1,102`** para uma perna que
encurtou **`0,500`**. É a D1 a apagar mais um caso especial, o mesmo que ela já
fizera a degrau, rampa e plataforma móvel, e é também o que o `bevy-tnua` faz.

⚠️ **A faixa de agarre CRESCE pelo que a perna encurtou, e isso não é ajuste.** A
soma `float_height + cling_distance` é **exactamente** o que o `within_reach`
compara; se ela encolhesse, a **PONTE** — que casta e julga com a config
AUTORADA — veria chão onde a lei já não vê. Crescendo-a pelo `drop`, a soma fica
**INVARIANTE**, e o problema de ordenar *"quem decide primeiro"* nunca chega a
existir: é a representação a apagar o caso especial. Gateado nos dois lados.

⚠️ **O estado NÃO é função pura do botão** — levantar-se é **RECUSADO** sob um
teto, e é por isso que existe um `CrouchState`, a viajar no `PlayerState` (o
mesmo ring da fita que o pulo e o arranque já usam).

⚠️ **Zero significa coisas DIFERENTES nos dois números:** na altura desliga a
capacidade; na velocidade é um agachar em que não se anda, que é uma escolha. Os
dois hovers dizem qual é o seu zero.

⚠️ **A wave não acrescenta ENTRADA nenhuma** — o botão de BAIXO existe desde a
W12, com o significado certo, e é isso que torna o gesto barato.

**Medido (2026-08-05):**

| pergunta | número |
|---|---|
| a silhueta baixa | `1,602 → 1,102` (autorado `0,500`) |
| ele passa sob um teto que o para de pé | de pé **x = 4,80** · agachado **x = 9,97** |
| o CONTROLE (teto alto) | de pé **x = 29,77** |
| a caminhada agachado | **3,973 m** contra **11,765** em 120 tiques (razão **0,338**) |

⚠️ **UM LIMITE MEDIDO E NOMEADO — o PISO GEOMÉTRICO.** A altura agachada não pode
descer abaixo de `half_height + radius` (**0,50** na cápsula das fixtures): abaixo
disso a cápsula enterra no chão e quem resolve é o solver (medido: pedir `0,30`
devolve `0,500`, um erro de `+0,200`). **A lei pura não pode clampá-lo** — ela não
conhece formas, de propósito —, então o piso vive onde a geometria vive: no rig,
no componente e no aviso da cena.

**8 mutações, 8 sangram.** ⚠️ **Uma sobreviveu, e a causa foi de FIXTURE — a
mesma classe que já custara uma rodada à W14:** o gate de scrub media a pose **no
instante do alvo**, onde ela vem do `restore` do rapier e está certa em qualquer
caso; o que a memória do controlador estraga é o que vem **A SEGUIR**. *Um gate de
scrub que não CONTINUA não testa o ring, testa o restore.* Corrigido, a mutação
diverge **`0,602 → 0,850`** na vertical (o topo encostado na laje) e **2,1 m** na
horizontal.

⚠️ **O que NÃO está gateado, e está NOMEADO:** a grade de três raios do sensor de
teto tem gate na lei, e o `blocked` tem gate no produto — mas *"o laço percorre os
TRÊS deslocamentos"* só é observável sob um teto **PARCIAL**, e uma fixture dessas
teria de calibrar a aresta da laje contra a posição MEDIDA de um personagem a
andar, dentro de uma janela de 0,2 m. Seria um gate que falha por deriva de
fixture em vez de por defeito. Trocar o laço por um raio central **sobrevive à
suíte**.

---

## §5d — W16: assar um player (cena `=95`)

**O que ele é:** o artista joga, aperta Bake, e a corrida que ele acabou de dar
vira curva na timeline.

⚠️ **O item estava marcado como *desbloqueado desde a W7*, e estava desbloqueado
e QUEBRADO.** O `bake_trajectories_with_scene` dirige os players pelo
`player_input` **RETIDO** — o dedo do instante do clique —, e nunca leu a fita.
É a frase do topo do próprio `bake.rs` dita ao outro eixo: *"um bake que não
avança a cena simula uma cena DIFERENTE"*.

**Medido antes de uma linha ser escrita** (corrida gravada de 90 tiques):

| o bake | o que ele grava |
|---|---|
| a corrida, ao vivo | `x = 8,765` |
| sem fita, dedo PARADO | X **CONSTANTE** (amplitude `0,000`) |
| sem fita, dedo na ESQUERDA | **`x = −8,765`** — o espelho exacto |
| com a fita (o que shipa) | `x = 8,765` |

⚠️ **A primeira linha da tabela é o defeito que não PARECE um defeito:** com o
canal X constante, nenhuma track horizontal é escrita, e o artista recebe uma
curva só de Y para um personagem que andou nove metros — lê-se como *"o botão
não gravou nada"*. E o modo de falha real não é *"nada acontece"*, é **"grava o
que quer que você esteja a segurar"**, que às vezes parece certo; por isso o gate
segura o **oposto** da corrida.

⚠️ **O mesmo `None` significa coisas DIFERENTES nos dois caminhos.** A fita
devolve `None` fora do alcance dela para dizer *"use a segurada"* — correcto **ao
vivo** (o artista ainda está a jogar) e errado num **bake** (a corrida acabou).
Daí o adaptador **`RecordedRun`**, que faz a cauda ficar parada, e que cobre as
duas pontas (antes do início da gravação também).

⚠️ **E a minha previsão sobre essa cauda estava ERRADA — a mutação não sangrou.**
Eu escrevi um gate afirmando que sem o adaptador ela *"segue o dedo"*; medindo, o
`take_taped_input` sobrescreve a entrada retida no primeiro tique gravado e **não
a restaura** ao calar, então ela **repete o ÚLTIMO tique da gravação, para
sempre** (`2,765 → 8,765`, contra `2,935` com o adaptador). O dedo só manda com a
fita **vazia** — duas causas, dois números, e a minha previsão cobria a que não
estava a acontecer. O gate foi reescrito contra a propriedade MEDIDA.

⚠️ **A CONTRADIÇÃO está escrita, e no ROTEIRO** (era o que o item do §4 pedia):
assar vira o corpo `Kinematic`, e a lei do player não dirige massa infinita —
então **depois do bake o personagem para de responder ao teclado**. É a mesma
contradição que a W-BakeJoint mediu do outro lado, e é o que *assar* significa. O
**passo 6** da cena manda o artista tentar dirigi-lo, para ele a encontrar de
propósito em vez de a reportar como bug.

⚠️ **A cena nasce PAUSADA** (a lista `PAUSED_SCENES`), pela razão da `=7` e mais
uma: ela pede uma corrida JOGADA, e com o relógio já a andar o começo da fita
descreveria segundos em que ninguém tinha o teclado.

**4 mutações, 4 sangram** — a do caminho sem fita derruba os quatro gates de
comportamento; a do adaptador só o da cauda; a que trava o corpo derruba os
quatro; e a da FIAÇÃO (`&mut self.player_tape` → uma fita vazia) derruba só o
arch-gate, porque **nenhum** gate de comportamento alcança o laço de render.

**Nenhum bump** (`PROJECT_SCHEMA` fica **58**), `c9` byte-idêntico, zero
`Cargo.toml`, nenhum ADR. Superfície pública nova na `ph2d-physics-ecs`:
`bake_trajectories_with_scene_and_tape` e `RecordedRun` — ⚠️ e **há UMA
implementação, as outras delegam**, o molde exacto da família do `dispatch`.

---

## §5e — W17: a corrida sobrevive ao arquivo (cena `=96`)

**O último item aberto do §4 do plano 06** (*"Persistir a fita (W7)"*) — e ele não
era um campo de arquivo.

### A correção, medida antes de qualquer linha

`measure_player_tape` (sonda, `#[ignore]`), 120 frames pela porta do produto:

| célula | antes | depois |
|---|---|---|
| sem player, Physics ARMADO | **120** | **0** |
| sem player, Physics OFF | **120** | **0** |
| **com player, Physics ARMADO** | **120** | **120** |
| com player, Physics OFF | **120** | **0** |

A fita gravava **o relógio andando**, não uma corrida. ⚠️ **As duas consequências
só existem depois de ela viver num arquivo:** todo projeto do app carregaria uma
corrida de ninguém, e — porque o toggle Physics nasce **DESMARCADO** — abrir um
projeto e *assistir* à timeline apagaria a corrida gravada, do tique em diante,
**em silêncio**. Um artefato destruído pelo ato de olhar para ele.

A condição nova (`simulate && players > 0`) mora no chamador, e a contagem sai da
consulta que o `hand_input_to_players` **já faz** — perguntá-la de novo seria uma
segunda varredura do mundo para o mesmo fato. ⚠️ **Entregar não é gravar:** a
entrada continua a ser ENTREGUE com a simulação desarmada (é o que faz armar o
Physics retomar do que o dedo está a fazer), e é só a GRAVAÇÃO que passou a
exigir que o tique tenha acontecido.

### A persistência

`ph2d_physics_ecs::TapeWire` + `InputTape::to_wire`/`from_wire`, e o
`ProjectFile.player_tape`. **`PROJECT_SCHEMA` 58 → 59.**

⚠️ **A tradução mora na crate-PONTE para o `PlayerInput` não aprender serde** — a
`ph2d-platformer` é a crate da lei pura (sem rapier, sem ECS, sem formato de
arquivo), e ensinar-lhe a serializar seria a primeira aresta na direção errada.

⚠️ **Os botões viajam num BITMASK, e não como quatro `bool`s**, por uma razão que
esta linha já pagou duas vezes: o `PlayerInput` ganhou o `down` na W12 e o `dash`
na W14. Num `u8` o quinto botão é um **bit novo no mesmo byte** — o layout do
arquivo não se move, um leitor velho ignora o bit e um leitor novo lê `false` num
arquivo velho. Com quatro `bool`s, o quinto seria **um byte novo por tique**, ou
seja um bump de schema por botão.

**Peso medido:** 1 s = 0,5 kB · 10 s = 4,7 kB · **60 s = 28,1 kB** ⇒ nenhum teto:
o que decide o tamanho é quanto o artista jogou.

⚠️ **O load INSTALA, nunca funde** — uma fita costurada com a da sessão anterior
descreveria uma corrida que ninguém deu; é o irmão exato do que o `project_forget`
faz com o relógio, a fila de undo e a timeline.

### A metade visível: um botão, e a ausência dele

**`Clear Recorded Run (N.N s)`** na §14, oferecido **só quando existe corrida**.
⚠️ **A ausência dele é o outro readout** — sem corrida não há o que descartar, e o
número vai no **RÓTULO do próprio controle que o resolve**: o precedente do
`Fit to Collider (needs > 0.50 m)`, pintado dez linhas acima dele. Um readout
separado seria uma segunda superfície dizendo o mesmo fato.

⚠️ **É o único verbo da §14 que não é escrita de componente** (a fita mora na
shell), então é **interceptado no laço de ações**, onde `self` é mutável — o lugar
e a razão exatos do `Join` da §11. E interceptar **não é higiene**: descartar é
idempotente, então espalhá-lo pela seleção não corromperia nada HOJE. É
precisamente essa forma que apodrece — o Ctrl+V do editor de nós colava duas vezes
porque um dispatch duplicado *"nunca tinha importado enquanto todos os verbos eram
idempotentes"*.

⚠️ **A fita é GLOBAL numa seção por-entidade**, e isso é honesto hoje pelo mesmo
desenho do `hand_input_to_players`: há um teclado, logo um dedo. Quando houver um
segundo, os dois se movem juntos.

### Gates e mutações

**9 mutações, 9 sangram.** As três que valem ler:

- **M7** — o braço do `ClearRun` vazio: **todos** os gates de seam e de
  comportamento ficam VERDES, o botão fica pintado e clicável, e nada acontece.
  É por isso que o arch-gate existe (o laço de ações exige janela).
- **M4** — o `from_wire` perde o `first`: a fita volta do mesmo TAMANHO
  descrevendo a corrida noutro instante. É por isso que o gate compara **tique a
  tique**, e num intervalo que começa ANTES e acaba DEPOIS do gravado.
- **M6** — o save grava `TapeWire::default()`: compila, passa os dois gates de
  load (que constroem o arquivo à mão) e **perde toda corrida que alguém jogar**.

⚠️ E os dois gates de load **não são um o inverso do outro**: o de esquecimento
passaria com um load que FUNDISSE as fitas (a nova cobriria a velha tique a
tique, por ser mais longa). Ele é o que separa *instalar* de *fundir*.

**LOC:** o `project_tests.rs` bateu 650 ⇒ os gates da fita saíram para o **FILHO**
`project_tape_tests.rs` (as fixtures do pai — `headless_app`, `write_project`,
`tmp_path` — são as portas dele; copiá-las seria um segundo escritor de arquivo de
projeto).

**Superfície pública nova:** `TapeWire` + `InputTape::{to_wire, from_wire}` ·
`PlayerFieldEdit::ClearRun` · `InspectorPlayerInfo.recorded_run_seconds` ·
`ids::INSP_PLAYER_CLEAR_RUN`. `c9` **byte-idêntico**, zero `Cargo.toml`, nenhum
ADR.


---

## §5f — W18: o piso do agachar é dito em voz alta (sem cena própria)

**O item aberto do handoff da W15** — e ⚠️ **a medição refutou a premissa da
nota**, que é o que vale ler aqui.

### O que a nota dizia, e o que o produto faz

A nota previa *"quem escrever `0,20` numa cápsula de `0,50` vê o corpo
enterrado"*. Medido (`measure_crouch_floor`, sonda `#[ignore]`): o corpo **não
enterra — ele SATURA**. O solver segura a cápsula tangente com **1 mm** de folga
na altura mais extrema (o `normalized_allowed_linear_error`, o mesmo 1,3 mm que a
W2a já mediu e declarou *"não é o que ninguém viu"*), e a pose é **perfeitamente
estável**: `0,0000 m` de variação ao longo de um segundo. **Nem tremor, nem
afundamento, nem nada na tela.**

**O defeito verdadeiro é pior de encontrar: abaixo do piso o slider é MORTO.**
Medido na rampa, onde o piso é `half_height + radius / cos θ`:

| rampa | piso | autorado `0,50` | autorado `0,30` |
|---|---|---|---|
| 0° | 0,500 | folga 0,002 | 0,000 |
| 30° | 0,531 | **0,027** | **0,027** |
| 45° | 0,583 | **0,059** | **0,058** |

Duzentos milímetros de curso de slider, **um milímetro** de resposta — e a 45° já
é assim a partir de `0,50`, que num plano parece perfeitamente bom. É o modo de
falha exato de um botão que MENTE.

### A cura: o espelho do que a perna de pé já tem

**`Fit Crouch to Collider (needs > 0.58 m)`** no fim da §14, ao lado do
`Fit to Collider`. O piso vai no **rótulo do próprio controle que o resolve** — a
lei que esta seção já escreve.

⚠️ **A MESMA função (`fitted_float`), e é o desenho inteiro:** o piso é da FORMA e
da rampa, **não** de qual perna está em uso. Uma segunda fórmula para a de baixo
divergiria no dia em que a caixa ganhasse a dela.

⚠️ **E ele nunca passa da perna de pé** (`fit.min(p.float_height)`): um agachar
mais alto que estar em pé não é um agachar — sem o `min`, o `crouch_step`
passaria a **levantar** o personagem quando ele segurasse BAIXO.

⚠️ **Oferecido só com o agachar ARMADO** (`crouch_height > 0`): em zero a
capacidade está desligada e não há defeito nenhum, e um botão que a ligasse pelas
costas conflataria *dar um agachar* com *consertar o que ele mede*. E uma **CAIXA
não o ganha** (outra fórmula) — o irmão exato do gate da perna de pé.

### A mutação que sobreviveu, e o defeito era da fixture

**4 mutações, 4 sangram** — mas a **M3** (`p.crouch_height = p.float_height`, o
Fit copiando a perna em vez de derivar o piso) **passou na primeira rodada**. A
fixture fazia `FitFloatHeight` **antes**, então a perna já estava no valor
ajustado e *"semear do piso"* e *"copiar a perna"* davam **o mesmo número**. Com a
perna posta em `1,50` as duas respostas ficam a um metro de distância, e a mesma
mutação sangra com **`1,500` contra `0,600`**.

**Sem cena de smoke própria, de propósito:** o defeito é **invisível na tela** (foi
essa a medição), então uma cena que o *mostrasse* teria de mostrar o painel, e o
que se julga é o rótulo do botão — a `=94` (o corredor baixo) já monta o agachar,
e o passo é abrir a §14 e ler o número. **Nenhum bump** (`PROJECT_SCHEMA` fica
**59**), `c9` byte-idêntico, zero `Cargo.toml`, nenhum ADR.


---

## §5i — W22: o sensor lateral vê o FLANCO, não a cintura (cena `=98`)

**O item aberto da W13**, e o próprio aviso de `bridge::player::probe_wall` já o
nomeava desde a wave: *"a altura é o MEIO do corpo … uma beirada que só alcance
os pés (ou só os ombros) não é vista"*.

### ⚠️ A nota SUBESTIMAVA o defeito, e a medição veio antes de uma linha

A frase descrevia uma beirada rasa. O que a sonda
(`ph2d-physics-ecs/tests/measure_wall_flank.rs`) achou é maior: com uma parede
que tem uma **fresta** na altura da cintura, e **pé e ombro ainda encostados na
pedra**, o **pulo de parede é recusado por inteiro**.

| fresta | sobreposição pé/ombro | pulo de parede |
|---|---|---|
| parede sólida | 0,500 m | **2,162 m** |
| 0,60 m | 0,200 m | 1,997 m |
| 0,70 m | 0,150 m | 2,292 m |
| **0,75 m** | 0,125 m | **0,000 m** |
| **0,80 m** | 0,100 m | **0,000 m** |
| 0,90 m | 0,050 m | **0,000 m** |

⚠️ **Abaixo de ~0,70 m o defeito era INVISÍVEL, e não por acaso:** o **buffer do
pulo** guarda o aperto até o bloco de baixo reaparecer. Um gate escrito naquela
faixa passaria com o sensor cego — é por isso que a cena e o gate usam **0,8**.

⚠️ **E o oráculo NÃO é o escorregamento.** Ele quase não denuncia (pior descida
0,0500 → 0,0632 m/tique), porque a **cola** que o `platform_wall` documenta
(atrito + gravidade do ápice) segura o personagem de qualquer jeito. O pulo não
tem cola: ou a lei vê parede naquele tique, ou o botão não faz nada.

### ⚠️ Três cenas foram construídas e duas não continham o fenômeno

Uma parede que *começa* num topo não serve — medido, o personagem **anda por cima
dela e vai embora** (atravessa `x = 1,2 … 9,7` em queda livre sem nunca
encostar). Uma beirada solta também não: ele a atravessa em três tiques. A
**fresta** segura, e o defeito aparece no instante em que o buraco passa pela
cintura.

### O desenho: a ponte AMOSTRA, a lei DECIDE

`wall_offsets(half_height)` dá **três** alturas (cintura · pés · ombros), a ponte
casta uma por uma e entrega o **array inteiro** (`WallProbe`) — que é o padrão
exato dos outros dois sensores multi-amostra desta ponte (`Headroom`,
`CeilingProbe`). O `cling` reduz: fica a **mais próxima que É parede**, e cada
motivo de descarte já era uma regra daquela função (nada visto · normal
degenerada · inclinação que a perna aceita).

⚠️ **Reduzir na PONTE foi recusado com motivo:** ela ficaria dona de *"qual destas
superfícies é a parede?"*, que é a pergunta que o `wall.rs` existe para
responder — e a redução divergiria da classificação no dia em que o `max_slope`
autorado se movesse.

⚠️ **A cintura é a PRIMEIRA da lista, e a ordem é load-bearing:** numa parede
plana as três distâncias empatam, o desempate vai para a primeira, e a resposta é
a de sempre. É isso que torna **byte-idêntica** toda parede que já funcionava —
e o `physics_ecs_c9` confirma no nível que importa: `74d4ea5d…`, 108 corpos,
**debug ≡ release**, o mesmo hash de antes da wave.

⚠️ **E isto resolve de graça um caso que um raio só não tinha como resolver:** uma
rampa aos pés (que a perna ACEITA, logo não é parede) deixou de cegar o tronco
encostado na parede.

### O que ficou de limitação, nomeado

Uma fresta **mais estreita que meia altura**, entre duas amostras, segue
invisível — e uma **maior que o corpo** não é parede nenhuma, ali a recusa está
certa. A cura dos três sensores é a mesma (um *shape cast*, que este wrapper
ainda não tem), e é a mesma frase que o `headroom_offsets` já carrega.

### Números

**`PROJECT_SCHEMA` 59 INTOCADO** · registro do `ph2d-physics-ecs` **fica em 28** ·
gizmo ids **nenhum novo** (próximo livre segue **974**) · **nenhum ADR** ·
contrato congelado **4/4 + 3/3** (rodado) · **zero `Cargo.toml`** · `c9`
byte-idêntico. **9 gates, 5 mutações, 5 sangram** — ⚠️ a mutação *"volta a ler só
a cintura"* sangra o gate de comportamento com o número exato do defeito
(`subiu 0.000 m contra 2.162 m`).

⚠️ **Superfície pública nova na `ph2d-platformer`:** `WALL_SAMPLES` ·
`wall_offsets` · `WallHit` · `WallProbe`; e **`cling` mudou de assinatura**
(recebe `Option<&WallProbe>` e devolve `WallSample` por valor). `wall.rs` passou
de 613 para 409 linhas com o `mod tests` a sair para o irmão `wall_tests.rs` (o
precedente do `crouch_tests`).

### Smoke

**`PH2D_PHYSICS_SMOKE=98`** — o poço da 92 com uma parede **lisa** (o controle) e
outra **com janelas** de 0,8 m. ⚠️ A cena imprime quantas janelas montou; **se
essa linha não aparecer, pare**. O que se julga é apertar PULO com a cintura numa
janela: antes desta wave o botão não fazia nada ali.

## §5j — W23: o agarrar-se, e a reserva que o torna um custo (cena `=99`)

**A outra metade do item aberto da W13**, ao pé da letra: *"ficar parado numa
parede é outra mecânica, com botão próprio, e não se alcança escrevendo `0` no
`Wall Slide`"*.

### ⚠️ A pesquisa não ensina o BOTÃO, ensina o CUSTO

Duas famílias, e as duas convergem: **resource-gated** (Celeste — botão + reserva
que se gasta) e **ability-gated** (Ori/Hollow Knight — a parede é uma habilidade
que se ganha, sem recurso). O que decidiu foi o que a referência **tentou e
abandonou**: o Celeste começou **sem reserva** e o jogo ficava resolvível
pendurando-se; um **temporizador** simples também foi tentado lá e abandonado por
não distinguir *escalar* de *pendurar*.

Aqui a reserva é **um número em segundos** (`Wall Grab (s)`), e ⚠️ **o zero não é
um caso especial — ele é exato**: segurar por zero segundos **é** não ter
agarrar-se. Isso põe o knob no mesmo idioma dos irmãos (`coyote_time`,
`corner_reach`, `slide_speed`) sem um `bool` ao lado a discordar do número.

⚠️ **UM knob, e a assimetria do Celeste NÃO foi construída** (lá subir custa mais
que pendurar): o segundo número teria o valor certo **em função do primeiro**,
que é a ergonomia que este repositório trata como bug de desenho
[[feedback_ergonomics_verdict_is_a_design_bug]].

### O desenho: uma expressão, dois regimes

O `wall_slide` já **DEFINE** a velocidade vertical enquanto se está agarrado (a
lei que a medição da W13 obrigou). O agarrar-se não soma um termo — ele troca o
**alvo**: `0` agarrado, `−slide_speed` solto. Um segundo termo daria dois donos
do mesmo número, e o sintoma seria um personagem que *quase* para.

⚠️ **E o grip CAVALGA o `cling`, não o duplica.** A pergunta *estou numa parede?*
é feita uma vez e já tem dono; agarrar-se acrescenta duas condições ao que ela
respondeu (o botão está apertado, e ainda há reserva). Por isso ele herda de
graça as três metades do `cling` — é parede pela régua da perna, o jogador
empurra contra ela, e está a descer.

⚠️ **A reserva volta ao cheio no CHÃO, de uma vez.** Qualquer outra regra
(recarga por segundo, ao soltar) ensina o jogador a **esperar parado**, que é
exactamente o que a reserva existe para não ser.

⚠️ **O gasto é guardado, não o que sobra** (`GrabState.spent`): o artista pode
mover o `Wall Grab` com o personagem pendurado, e um *"quanto sobra"* guardado
ficaria acima da reserva nova — um número que descreve um mundo que deixou de
existir. E ele mora no **`PlayerState`** porque é esse tipo que a **fita** guarda
no ring de tiques âncora; um estado noutro lugar teria de ser acrescentado àquele
ring à mão.

### ⚠️ A tecla saiu de uma varredura, não de gosto

`E` tem **três** donos, `F` quatro, `G` e `C` seis, `X` sete, `V` cinco, `T` é o
painel de Tokens (sem guarda nenhuma) e `B` é o contorno de collider que **toda**
cena de física usa. Sobra **`R`**, com **um** — e esse um é o
`ReverseSelectedKeys` da timeline, que vive atrás de `timeline_panel_open()` **E**
`has_selection`: **exactamente a guarda sob a qual o `D` do próprio player já
convive** desde a wave das joias. Não é uma coexistência nova, é a que já shipa.

### ⚠️ O botão novo NÃO move o formato da fita

A fita guarda os botões num **bitmask** (`(f32, u8)`), então o `BIT_GRAB` é um bit
livre e **o postcard vê os mesmos bytes**. Uma corrida gravada antes desta wave
volta com o agarrar em zero — que é o que ela de facto tinha. Um campo novo na
tupla teria custado o bump por si só e recusado todo arquivo já salvo.

⚠️ **E isso expôs um gate cuja fixture envelheceu no dia certo:** o
`an_unknown_button_bit_is_ignored_rather_than_misread` codificava *"bit
desconhecido"* como o **bit 3** — que esta wave reclamou. Ele ficou **VERMELHO**,
que é o comportamento correto, e a fixture passou para o **bit 7**: ali ela só
pode expirar quando o byte encher, e nesse dia a conversa é outra (um segundo
byte é mudança de FORMATO, não um bit livre).

### Números

**`PROJECT_SCHEMA` 59→60** — ⚠️ **PROVISÓRIO**, o valor se CONTA contra o `main`
do dia (v60: o `PlatformPlayer` ganhou `wall_grab_stamina`, um campo apendado,
postcard posicional). Tripla do pin **`(60, 13, 14)`**. Registro do
`ph2d-physics-ecs` **fica em 28** · gizmo ids **nenhum novo** (próximo livre segue
**974**) · **nenhum ADR** · contrato congelado **4/4 + 3/3** (rodado) · **zero
`Cargo.toml`** · `c9` **byte-idêntico** (`74d4ea5d…`, 108 corpos, debug ≡
release — a capacidade nasce desligada). **12 gates, 7 mutações, 7 sangram.**

⚠️ **Superfície pública nova na `ph2d-platformer`:** `GrabState` · `grab_step` ·
`WallConfig::grab_stamina` · `PlayerInput::grab` · `PlayerState::grab`; e
**`wall_slide` mudou de assinatura** (ganhou `gripping: bool`).
`PLAYER_ROW_COUNT` **31→32**.

### Smoke

**`PH2D_PHYSICS_SMOKE=99`** — três estações idênticas, beiral a **4,4 m** (alto
demais para um pulo de parede sozinho). ⚠️ A cena **abre com a capacidade
desligada, de propósito**: o passo 2 é o controle, e os passos 3-4 pedem ao
artista que escreva `0,5` e depois `2,0` na §14 — o gesto que a wave entrega é
justamente esse número na mão dele.

## §6 — Ordem

1. `git rebase main` (ou merge). Os arquivos compartilhados são o `project.rs`
   (um doc-comment + o literal), o `project_schema_tests.rs` (a tripla) e o
   roteador de smoke — **CONTE o `PROJECT_SCHEMA`, não o copie** (§3).
2. Rodar o gate da árvore combinada **em DEBUG E RELEASE** (esta linha tem
   precedente registado de vermelho só-em-debug).
3. Recomputar o `physics_ecs_c9` **depois** do rebase: deve dar **`74d4ea5d…`**,
   e conferir debug ≡ release. ⚠️ Se der `2278035e…`, a W11c perdeu-se; se
   MUDAR, alguma coisa desta jornada deixou de ser opt-in e isso é o achado.
4. Rodar os gates que **não** correm numa varredura por-crate: o
   `architecture_workspace_file_loc_cap`, o `file_loc_caps` da shell, o
   `no_tofu_glyphs` e o `arch_safe_clamp_only` — esta linha já shipou vermelhos
   latentes por eles não serem alcançados por `cargo test -p`.

---

## §7 — Smoke

⚠️ **`PH2D_PHYSICS_SMOKE=97` — AS DUAS BORDAS DA DESCIDA (W20).** Tres escadas
de pranchas identicas; so o VAO muda (1,80 · 2,00 · 1,60). O meio e o controle;
a esquerda e o vao que ANTES nao descia; a direita e o limite honesto. E aperte
`B`: enquanto ele atravessa, toda prancha da cena fica apagada.

```
env PH2D_PHYSICS_SMOKE=81 cargo run -p ph2d-host-desktop --release   # rampa 30° (W11c)
env PH2D_PHYSICS_SMOKE=88 cargo run -p ph2d-host-desktop --release   # o par 40°/50°
env PH2D_PHYSICS_SMOKE=85 cargo run -p ph2d-host-desktop --release   # a jangada (o PESO)
env PH2D_PHYSICS_SMOKE=91 cargo run -p ph2d-host-desktop --release   # A ESCADA DE PRANCHAS (W12)
env PH2D_PHYSICS_SMOKE=92 cargo run -p ph2d-host-desktop --release   # O POÇO (W13)
env PH2D_PHYSICS_SMOKE=93 cargo run -p ph2d-host-desktop --release   # O ABISMO (W14)
env PH2D_PHYSICS_SMOKE=94 cargo run -p ph2d-host-desktop --release   # O CORREDOR BAIXO (W15)
env PH2D_PHYSICS_SMOKE=95 cargo run -p ph2d-host-desktop --release   # A CORRIDA VIRA ANIMACAO (W16)
env PH2D_PHYSICS_SMOKE=96 cargo run -p ph2d-host-desktop --release   # A CORRIDA SOBREVIVE AO ARQUIVO (W17)
```

⚠️ **Cada cena imprime o que montou.** Se a linha `[physics-smoke NN]` não
aparecer, pare: a cena não montou e o resto do smoke não diz nada.

- **`=91`** — o que se julga é **um andar por aperto**. Se ele for ao chão de uma
  vez, a retirada da descida quebrou.
- **`=92`** — o vão é **2,4 m de propósito**, mais largo do que um pulo de parede
  atravessa sozinho (1,74 m medidos): subir exige soltar a direção a meio do voo.
- **`=93`** — o abismo tem **11 m de propósito**, e o **passo 3 é o controle**:
  um pulo sozinho **não** o atravessa. Se atravessar, o vão está curto e o resto
  do roteiro não diz nada.
- **`=94`** — as alturas são **aritmética da cápsula, medida**: topo de pé
  **1,402**, agachado **1,052**, o túnel baixo em **1,20** e o alto — o
  **CONTROLE** — em **1,60**. O passo 6 é ele: se o segundo túnel também o
  parasse, a cena estaria a medir a laje e não o agachar. E o **passo 4** é a
  estrela: soltar o botão LÁ DENTRO **não** o levanta.
- **`=95`** — ela nasce **PAUSADA** e pede uma corrida JOGADA. O **passo 5** é o
  veredito (o personagem refaz o que você fez, com a física DESARMADA) e o
  **passo 6 é a contradição**, deliberada: depois do bake ele não responde ao
  teclado, porque virou animação.
- **`=96`** — ⚠️ **o passo 5 manda FECHAR o app**, e não é cerimônia: um Ctrl+S
  seguido de Ctrl+O na mesma sessão devolve a fita que a sessão já tinha, então
  não prova nada sobre *sobreviver a fechar o app*. E o **passo 1 é o controle**:
  antes de correr, o botão `Clear Recorded Run` **não pode existir**.

---

## §5g — W19: a escada de pranchas tem uma janela útil, e as duas bordas são defeitos (sem cena própria)

⚠️ **Esta wave não muda o produto.** `retire_drops` está byte-idêntico ao
`main`; o que ela entrega é a MEDIÇÃO, quatro gates que pinam o mundo de hoje,
e duas notas corrigidas onde elas estavam.

**A premissa que caiu.** O `retire_drops` isentava o caso degenerado com
*"um vão menor que o personagem deixa a descida armada para sempre — essa cena
já está quebrada sem descida nenhuma (o personagem não cabe ali)"*. A metade
entre parênteses é **falsa**: entre **1,15 m e 1,55 m** de vão ele fica em pé no
degrau de baixo, **perfeitamente estável** (0,0000 m em 60 tiques), com a cabeça
a atravessar o de cima — o idioma de uma jump-through. E o preço não é a prancha
de onde ele desceu: o bit viaja no **CORPO** e o gancho limpa os contatos com
**qualquer** plataforma one-way, então uma descida eterna apaga **todas as
pranchas da cena, para sempre**.

**A segunda borda, que é da lei que JÁ SHIPA.** A aposentadoria de hoje dispara
**a meio da queda** — a prancha volta a ser sólida com o personagem ainda a
atravessá-la, e o contato atira-o para cima:

| meia-espessura | vão | o que acontece |
|---|---|---|
| 0,15 | 1,60 – 1,70 | desce, e as pranchas ficam **fantasma** para sempre |
| 0,15 | **1,75 – 1,85** | **arremessado de volta** — o botão parece não fazer nada |
| 0,15 | 1,90 + | funciona |
| 0,10 | 1,50 – 1,60 | fantasma |
| 0,10 | 1,65 + | funciona |

⚠️ **A cena `=91` usa `RISE = 2,0` com pranchas de 0,15 — dez centímetros acima
da borda de arremesso.** O header dela justificava esse número por *"0,3 m de
margem"* da lei, sem saber que apertar 15 cm o quebra. Corrigido lá, com a
tabela e um **não aperte o `RISE`**.

⛔ **Três leis construídas e REPROVADAS**, cada uma trocando um regime por outro:

1. **Centro do corpo abaixo da base da prancha.** A virada do cuspe foi medida e
   é limpa — segue a base em quatro espessuras (0,05/0,10/0,20/0,30 ⇒
   −0,05/−0,10/−0,20/−0,30). ⚠️ **Mas é medida EM REPOUSO**, e aplicá-la a uma
   QUEDA custou o retrato: numa prancha de 0,15 ele desce a 5,79 e volta a
   repousar em **7,05, dois degraus ACIMA**.
2. **Exigir que ele tenha POUSADO.** Dispara no instante em que o raio ALCANÇA o
   degrau, ainda uma altura-de-perna acima dele e a cair depressa.
3. **"Deixou de descer".** Não tem régua: em repouso a mola deixa **1e-7 m/tique**
   descendente (33 de 40 tiques), então o sinal é moeda ao ar — e o
   `sleep_linear_threshold` do mundo, que seria a régua autorada, é largo demais
   e devolve o arremesso.

**O momento seguro de re-solidificar uma prancha que o SOBREPÕE não é função da
pose sozinha**, e uma lei que o afirme é verde num regime e vermelha no outro.

**O que shipa:** `crates/ph2d-physics-ecs/tests/measure_drop_retire.rs` (5 sondas
`#[ignore]`, incluindo o mapa da janela) e `platform_drop_ladder.rs` (4 gates).
⚠️ **Dois dos gates afirmam o DEFEITO, não a cura** — o precedente do
`the_documented_hardening_is_still_there_and_this_is_its_number` do Painter — e
ficam **vermelhos no dia em que a lei mudar**, de propósito: a cura muda QUANDO
uma prancha volta a ser sólida, que se sente e não se prova, e quem a fizer tem
de passar por ali deliberadamente, julgando **as duas bordas ao mesmo tempo**.

**Zero mudança de produto** · `PROJECT_SCHEMA` fica **59** · `c9` byte-idêntico
(`74d4ea5d…`, 108 corpos, debug ≡ release) · zero `Cargo.toml` · nenhum ADR.

---

## §5h — W20: a descida morre quando para de fazer trabalho (cena `=97`)

A W19 mediu duas bordas e reprovou três leis. Esta mede o **mecanismo** e fecha a
borda de cima — a que na tela se lê como *o botão não faz nada*.

### O mecanismo, por ablação

⚠️ **Não era o raio, era o SOLVER.** A retirada faz duas coisas (o mapa cai, e o
bit do solver é limpo) e separá-las decide:

| ablação | o que acontece |
|---|---|
| nunca aposentar | desce um degrau limpo (`−0,797`) |
| mapa cai, **bit fica** | desce um degrau limpo (`−0,7973`) |
| a lei de hoje | **arremessado de volta ao degrau de cima** |

O parceiro do contato é a prancha que ele acabou de deixar, e o pico é de
**0,3267 N·s entre sub-passos** — com o `impulse` de fim de tique a ler
`0,0000`, que é a lição da W-ImpactForce outra vez. ⚠️ **E a caixa mente ali:**
o corpo estava **0,016 m abaixo** da prancha, sem sobreposição nenhuma. É por
isso que toda lei geométrica tentada aqui morreu.

### A lei, e por que as duas metades

`retire_drops` aposenta quando **já passei** *e* **a prancha já parou de me
pegar**. A segunda metade vem de um livro-razão novo (`world::oneway::DropLedger`)
que o gancho preenche durante o `step` e a ponte lê no topo do tique seguinte.

⚠️ **A evidência sozinha REGRIDE.** Quando a prancha fica inteiramente DENTRO da
caixa do personagem não existe *lado*, e a normal do manifold **oscila** — o
ponto de contato salta de `−0,486` para `+0,490` em dois tiques. Uma lei só de
evidência aposenta no primeiro "não" dessa oscilação e a prancha o empurra: com
pranchas de `0,10` nos vãos `1,10`–`1,25` ele **deixava de descer**. A geometria
não oscila, e é ela que segura a evidência.

⚠️ **A leitura é SÓ-LEITURA do cone.** `update_as_oneway_platform` **trava**
(`user_data`) no primeiro contato do par; chamá-lo durante a travessia gravaria
*permitido* e a prancha o pegaria no tique seguinte à aposentadoria. O que as
duas partilham é o **cone** (`ALLOWED_COS`, literal exato, gateado contra o
ângulo); o que não partilham é o **latch**, e essa é a única diferença.

### O resultado, célula a célula

| meia-espessura | vão | antes (W19) | agora |
|---|---|---|---|
| 0,15 | 1,60 – 1,70 | fantasma | fantasma |
| 0,15 | **1,75 – 1,85** | **ARREMESSADO** | **ok** |
| 0,15 | 1,90 + | ok | ok |
| 0,10 | 1,50 – 1,60 | fantasma | fantasma |
| 0,10 | 1,65 + | ok | ok |

**Três células curadas, nenhuma outra movida.** E a cena `=91` deixou de viver
dez centímetros acima de um penhasco: a margem passou de `0,10` para `0,25`.

⚠️ **O que resta ganhou uma LEI, não uma faixa:** a descida sobrevive
**exactamente** onde a caixa de repouso ainda SOBREPÕE a prancha — bicondicional,
sem exceção nas duas espessuras varridas, e gateado. Ali a prancha *de facto* o
pegaria (o cone devolve `+1,000`, medido), então as saídas eram *fantasma* ou
*cuspido*.

### A metade visível

⚠️ **Uma prancha fantasma era indistinguível de uma sólida** — e é essa a forma
como toda esta classe de defeito ficou silenciosa. O contorno de toda plataforma
one-way passa a apagar-se enquanto alguém a atravessa (`PASSABLE_RGBA`), pela
mesma família do par idle/active do sensor.

### Gates e mutações

5 gates em `platform_drop_ladder.rs` (incluindo o bicondicional sobre 10
células), 2 no overlay, 1 no cone. **5 mutações, 4 sangram:**

| mutação | efeito |
|---|---|
| sem a metade da evidência | o gate do arremesso e o da lei, VERMELHOS |
| sem a metade da geometria | quatro gates VERMELHOS |
| o livro-razão relata SEM o cone | ⚠️ **sobrevive — ver abaixo** |
| o livro-razão nunca relata | dois gates VERMELHOS |
| o livro-razão não é limpo por tique | o controle e a lei, VERMELHOS |

⚠️ **A sobrevivente está documentada, não escondida.** Relatar *"existe manifold"*
em vez de *"a prancha pegaria"* dá a mesma resposta em toda célula varrida,
porque a vida do manifold e a do cone coincidem nessas geometrias. O cone fica
porque sem ele a duração da descida passaria a ser um fato sobre a **margem do
broad phase**, não sobre a física — e um número que ninguém escolheu governando
um gesto é como esta feature adoeceu da primeira vez.

### Números

`PROJECT_SCHEMA` fica em **59** · registro do `ph2d-physics-ecs` **fica em 28** ·
gizmo ids **nenhum novo** (próximo livre segue **974**) · `physics_ecs_c9`
**`74d4ea5d…`, 108 corpos, byte-idêntico** (a saída do solver não se move — só o
instante em que o bit cai) · **zero `Cargo.toml`** · **nenhum ADR** · contrato
congelado intacto.

⚠️ **Superfície pública nova:** `ph2d_physics::{ALLOWED_ANGLE, ALLOWED_COS}` ·
`PhysicsWorld::drop_is_catching` · `PhysicsBridge::{player_is_dropping,
any_player_is_dropping}` · `physics_overlay::outlines` e `::draw` ganharam um
parâmetro `ghost`.

⚠️ **LOC:** `world.rs` cruzou 700 com o campo novo ⇒ `spawn_world_anchor` foi
para o irmão `world/convenience.rs`, que já é a casa dos construtores que se
pedem por um ponto em vez de por um `BodyDesc`. E `physics_overlay_scene_tests.rs`
cruzou 600 ⇒ os dois gates da prancha atravessada saíram por ASSUNTO para
`physics_overlay_passable_tests.rs`.

⚠️ **E um arch-gate alheio foi RE-ANCORADO, não contornado:**
`the_overlay_is_handed_the_tool_marks` fatiava `render_loop/mod.rs` por
**distância em bytes** (`&src[i..i + 3000]`), e um comentário novo com um `⚠️`
— três bytes — pôs o corte no meio de um caractere e o fez **PANICAR** em vez de
julgar. A janela passou a acabar onde a **chamada** acaba, que é a propriedade
que ele percorre; mutação re-provada.

### ⛔ E a cura da borda de baixo foi construída, MEDIDA e REVERTIDA (W21)

Este handoff dizia, uma versão atrás, que o bit da descida viaja no **CORPO** e
que por isso *"enquanto a descida da faixa que sobra vive, **nenhuma** prancha é
sólida para ele"* — e prescrevia a descida **por-PLATAFORMA** como cura.

Ela foi construída inteira: conjunto de pares `(corpo, plataforma)` no lugar do
bit como autoridade do gancho, evidência por par, o gesto de armar a levar também
as plataformas que o corpo **já sobrepõe** (do grafo de contatos, porque a
prancha dos pés não aparece lá — a perna o segura acima dela), e o raio do sensor
a ignorar a lista inteira em vez de um handle.

**E foi revertida, porque a medição não achou diferença nenhuma.** Numa cena com
a escada apertada e uma prancha **SOLTA** ao lado, mais abaixo:

| mundo | onde ele para |
|---|---|
| controle (sem descida) | `−1,998` — a prancha solta o segura |
| **bit GLOBAL (o que shipa)** | **`−1,998` — ela o segura na mesma** |
| **por-plataforma** | **`−1,998` — idêntico** |

⚠️ **A afirmação era grande demais, e o mecanismo diz por quê:** o bit global
limpa **contatos do SOLVER**, e quem segura este personagem é a **MOLA** — o raio
dela só ignora a plataforma da travessia, então toda outra prancha continua a
pegá-lo. O custo real de uma descida viva é a prancha que ela nomeia, não a cena.

⚠️ **E a sonda falhou o próprio controle DUAS vezes** antes de decidir: primeiro
as pranchas da fixture mediam 40 de meia-largura e ele não tinha de onde sair;
depois ela o deixava andar 400 tiques e ele **atravessava** a prancha solta a
caminho do outro lado do mundo, então o fim media o passeio e não a queda. *Um
A/B em que os dois lados dão o mesmo número só vale depois de o controle dar um
número diferente.*

Fica a sonda (`measure_whether_a_live_drop_really_dissolves_the_whole_scene`,
com o controle embutido) e este parágrafo, para ninguém reconstruir a cura sem
saber que ela já foi medida.

## §8 — Aberto, com o preço ao lado

- ~~**W11c:** o pouso perdeu os 24 mm de quique que o `Spring Damping` em meio
  curso dava~~ — **FECHADO pela W26** (2026-08-07), e a medição mostrou que a
  troca tem um **TERCEIRO eixo** que esta nota escondia:

  | | `spring_damping` | `substeps` |
  |---|---|---|
  | deriva de rampa | `∝ (1 − d)` | **`∝ 1/n`** |
  | quique do pouso | `∝ (1 − d)` | **INDEPENDENTE** |

  ⇒ **os dois não estão soldados.** Medido (30°, 10 s parado; queda de 1,5 m no
  plano): a `d = 0,25` o quique fica em **32,7 mm a `n = 4`** e **32,4 mm a
  `n = 12`** enquanto a deriva cai de **0,0575 m para 0,0194 m**. No teto medido
  do outro knob (`MAX_SUBSTEPS = 12`, W2b) sobra **99% do quique com um terço da
  subida**. O tooltip do **Leg Damping** passou a dizê-lo — sem ele o artista
  baixa o knob, vê o personagem andar sozinho e não tem como saber que o outro
  paga.
  ⚠️ **E a tabela de sub-passos do `BUGS_physics.md` §7 estava STALE:** lá a
  deriva **CRESCE** com `n` e ajusta `A·(1 − 1/(4n))`, cujo limite não é zero —
  ela foi medida **antes da wave `gravity_hold`**, e depois dela a série
  **inverteu** (cai pela metade a cada dobra, com o `n = 1` idêntico). Quem a
  lesse hoje concluiria que subir sub-passos não ajuda. Corrigida no lugar, com a
  velha mantida como o que **decidiu** aquela wave.
  ⚠️ **Ela também reconcilia o item `BUGS §7 (3)`:** fatiar o motor *"corta a
  deriva 4×"* é exactamente **uma potência de `n`** no default — fatiar nunca
  removeria o defeito, só deslocaria a série de um degrau, e o degrau já é
  comprável pelo knob que o artista tem.
  ⚠️ **O default NÃO se move:** ele fica no teto (deriva zero exacta), que é o
  que o smoke de 05/08 pediu. O que a wave entrega é o caminho de volta para
  quem quer o quique.
- ~~**W12:** a escada de pranchas tem uma **janela útil** e as DUAS bordas dela
  são defeitos~~ — **FECHADO pela W27** (2026-08-07). A de cima já era da W20; a
  de baixo caiu quando alguém finalmente **mediu o preço dela**.
  ⚠️ **Ela estava registada pelo SINTOMA.** *"As pranchas ficam fantasma"*
  descreve o contorno; medido (`measure_what_an_armed_drop_costs`), o preço era o
  personagem descer um degrau e **ficar lá para sempre** — `−0,598 → −0,598` a
  1,60, `−0,198 → −0,198` a 1,20, em **toda** célula da janela. Uma **ARMADILHA**,
  não um enfeite.
  ⚠️ **E a cura não foi nenhuma das quatro que este handoff prescreveu.** As três
  da W19 eram sobre *quando* aposentar e cada uma trocava um regime por outro; a
  da W21 (descida por-PLATAFORMA) foi construída, medida — **nenhuma diferença** —
  e revertida. A que fechou é uma cláusula de **INTENÇÃO**:

  ```text
    aposenta  ⇔  estou a SUBIR  ∨  (já passei  ∧  a prancha parou de me pegar)
  ```

  Uma descida travada existe para deixar passar **para BAIXO**; no instante em
  que o corpo sobe, quem decide já é o **cone** do one-way, que deixa passar por
  baixo por conta própria. Manter o bit ali não protegia nada e prendia.
  ⚠️ **Ela não reabre a borda de cima, e a razão é o SINAL:** aquele defeito é a
  prancha voltar a ser sólida com ele a **CAIR** através dela, e a cláusula só
  dispara com a velocidade para cima. Os gates daquela borda ficam verdes ao lado
  do desta.
  ⚠️ **E o gate que pinava o defeito fez o que foi escrito para fazer:** ele
  afirmava o fantasma com a instrução de ficar VERMELHO no dia em que a lei
  mudasse, e ficou — com o número que ele próprio previu. Reescrito de propósito.
  ⚠️ **`c9` byte-idêntico** (`74d4ea5d…`, 108 corpos, debug ≡ release): nenhuma
  cena do hash exercita uma descida.
- ~~**W13:** o sensor lateral olha só a altura do **MEIO** do corpo~~ —
  **FECHADO pela W22** (§5i), e ⚠️ **a medição mostrou que a nota subestimava**:
  não era *"uma beirada não é vista"*, era **o pulo de parede RECUSADO por
  inteiro** com pé e ombro na pedra.
- ~~**W13:** não há *wall grab*~~ — **FECHADO pela W23** (§5j). O item está
  cumprido ao pé da letra: botão próprio (`R`), e o `0` no `Wall Slide` continua
  a não o alcançar. ⚠️ **O que NÃO foi construído, e por quê:** não se **ESCALA**
  a parede (subir e descer agarrado pede um eixo VERTICAL que a entrada deste app
  não tem — *cima* já é o pulo, e um botão novo por direção seria um input model
  novo), e a **assimetria do Celeste** (subir custar mais que pendurar) foi
  recusada porque o segundo knob teria o valor certo em função do primeiro.
- ~~**W15:** o piso geométrico da altura agachada~~ — **FECHADO pela W18**, e
  ⚠️ **a medição refutou a premissa desta nota**: o corpo **não enterra, ele
  SATURA**. Ver a §5f.
- ~~**W16:** a fita não é persistida~~ — **FECHADO pela W17.**
- ~~**W17:** a fita é **uma só**, e o botão de descartá-la mora numa seção
  por-entidade~~ — **FECHADO pela W25** (2026-08-07), e a medição mostrou que o
  defeito era **de alcance**, não de modelo: o `build_player_info` devolve `None`
  para tudo o que não é um corpo **Dynamic selecionado**, então apagar o
  personagem prendia a corrida — ela continua no arquivo (W17), continua a ser o
  que o Bake replaya (W16), e não havia gesto nenhum que a alcançasse.
  ⚠️ **A cura NÃO é uma fita por-player, e a recomendação anterior está
  corrigida:** com um teclado há um dedo, e o `hand_input_to_players` já o
  entrega a todos — fitas por-entidade gravariam **N cópias idênticas** da mesma
  corrida (uma segunda resposta a *"o que o dedo fez?"* sem um segundo dedo) e
  custariam um bump de `PROJECT_SCHEMA` por uma capacidade que ninguém consegue
  dirigir. O que faltava era o fato ter **casa onde os fatos do documento
  moram**: o painel de MUNDO (`W`) ganhou o readout e os dois verbos.
  ⚠️ **Duas VISTAS, uma PORTA** (`run_stash::apply`) — o precedente exato do
  `Show Colliders` contra a tecla `B`. Duas cópias do `mem::take` fariam a mesma
  coisa hoje, e é essa forma que apodrece.
  ⚠️ **O segundo dedo continua a não existir**, e é ele — não este item — que
  torna a fita por-player necessária. Nada aqui o impede.
- ~~**W17:** descartar a corrida **não passa pelo undo**~~ — **FECHADO pela W24**
  (2026-08-07). A fita continua **fora** do `ProjectState`, de propósito (um
  Ctrl+Z do canvas não deve rebobinar uma gravação), então a cura não é o undo
  global: descartar **move** a corrida para um guardado de SESSÃO (`mem::take`)
  e o mesmo lugar da tela oferece **Restore Discarded Run**.
  ⚠️ **O ciclo de vida é DERIVADO, não mantido:** o botão de devolver só é
  oferecido com a fita viva vazia, então gravar de novo o esconde e o próximo
  descarte sobrescreve o guardado — não existe caminho em que uma corrida velha
  ressuscite, e é por isso que os dois botões nunca coexistem.
  ⚠️ **O guardado não viaja no arquivo**: uma corrida descartada foi descartada,
  e um arquivo que a carregasse ressuscitaria o que o artista apagou.
  O botão segue **sem confirmação**, e agora isso é honesto — o gesto é
  reversível num clique.
- **Do plano 06 §4, o que sobra:** **player Kinematic** — e ele é o item que o
  Enio disse que virá um dia. Este plano não o proíbe: a lei pura da
  `ph2d-platformer` é agnóstica de como o motor é aplicado, e é exatamente onde
  um segundo consumidor entraria.

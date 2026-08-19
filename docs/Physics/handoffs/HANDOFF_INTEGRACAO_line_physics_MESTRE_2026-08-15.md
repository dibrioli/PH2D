# Handoff de integração MESTRE — `line/physics` (2026-08-15)

**Status:** FECHADO 2026-08-15 · no `main` em `053551f70` (o commit que trouxe este arquivo).

> **A linha NÃO integra nem faz ship** (CLAUDE.md §0.7). Este documento é o que o
> integrador precisa para não colidir nem regredir. DIRETRIZ §1.5.9.
>
> ⚠️ **Ele SUPERSEDE o
> [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md)
> apenas como *o que integrar agora*.** O **detalhe de mecanismo** das waves até
> a ÂNCORA dos gizmos continua LÁ (e o das sete waves de sensores continua no de
> 08-11), e **nada disso foi copiado**. O que é NOVO aqui é a **fila da auditoria
> 09**: sete waves construídas, **duas recusadas por medição**, e a **CAUDA de
> cinco waves** (§2e) que fechou o §3.A e o item das camadas.

**Os seis itens que a DIRETRIZ §1.5.9 exige, e onde estão:**

| # | item | onde |
|---|---|---|
| 1 | identidade (branch · HEAD · merge-base · nº commits) | **§1** — e a caixa **medida** do rebase |
| 2 | foundational/compartilhado tocado + porquê | **§3c** |
| 3 | símbolos que podem COLIDIR, com valores literais | **§3d** — a resposta é **NENHUM**, por construção |
| 4 | contratos congelados encostados | **§3** — **nenhum**, rodado |
| 5 | o que só o `ship.sh` pega | **§5b** |
| 6 | ordem/dependências + o que NÃO foi smokado | **§2d · §6** |

---

## 1. Identidade

| | |
|---|---|
| branch | `line/physics` |
| HEAD | **o tip de `line/physics`** ⚠️ ver abaixo |
| merge-base com `main` | `76788440adbabb0e5b12f8fdafecc6f1e1183e1a` |
| commits | **117** |
| diff | 224 arquivos, **+42.342 / −2.336** |

⚠️ **O HEAD não é escrito aqui de propósito, e a razão é aritmética:** o commit
que o escreve MUDA o HEAD, então um sha nesta tabela é falso no instante em que é
commitado. O que identifica a entrega é o **merge-base** acima mais *"o tip da
branch"*.

⚠️ **A INTEGRAÇÃO É FAST-FORWARD TRIVIAL, e isto é MEDIDO, não suposto** (medido
em 2026-08-15, depois de um `git fetch origin main`):

| pergunta | medido |
|---|---|
| o `main` local andou desde o fork? | **não** — `merge-base(main, HEAD)` **é** o tip do `main` (`76788440a`) |
| o `origin/main` andou? | **não** — `main..origin/main` = **0** |
| o `main` local está à frente do remoto? | **sim, 5 commits** (`origin/main..main` = 5) |

⇒ **Não há conflito possível com o `main`**, porque o `main` não se mexeu: não há
rebase a fazer, e `--ff-only` passa. ⚠️ **Mas esta caixa envelhece** — se outra
linha integrar antes desta, re-meça as três linhas acima **antes** de tocar em
qualquer coisa; é a única parte deste doc que muda sozinha.

⚠️ **O push da jornada leva 5 + 116 commits** (os 5 que o `main` local já tinha
por pushar mais os desta linha) — quem fizer o ship conta com isso no babysit do
CI.

---

## 2. O que é NOVO nesta entrega

A entrega tem **três partes**, e elas se JULGAM de formas diferentes: a **fila da
auditoria** (sete waves construídas, duas recusadas por medição — todas
**smokadas pelo Enio**) · a **auditoria FINAL** dos três blocos, que **não tem
smoke** e cujo argumento está na §2c · e a **CAUDA de cinco waves** (§2e), que
fechou o §3.A e o item das camadas e **também não tem smoke**, pelo mesmo
argumento (são consultas de READOUT e um filtro de sensor, cada uma com gate
mutação-provado; nenhuma muda o que o artista vê sem ele pedir).

**A fila da [auditoria 09](../09_auditoria_engines.md), desenhada no [plano
10](../10_plano_fila_da_auditoria.md) e executada na ordem que ele fixou.** O
detalhe de cada wave — o desenho, o que a medição REFUTOU do plano, os gates e as
mutações — vive no próprio plano 10, em seções `⟨FECHADA⟩` ao lado do desenho
original. Aqui só o mapa:

| | wave | o que ela é | cena |
|---|---|---|---|
| **A** | `W-PlayerOut` | o jogador **PUBLICA** estado e transições; a §14 mostra e o evento vira **SINAL** | `=113` |
| **C** | `W-Brake` | **frear ≠ acelerar** — a fração do orçamento gasta com o eixo solto | `=114` |
| **B** | `W-Surface` | a superfície **FALA** com a lei: tração (gelo) e **esteira** | `=115` |
| **D** | `W-Fall` | a queda tem **TETO** (não havia velocidade terminal: **142,57 m/s aos 8 s** e a crescer) | `=116` |
| **E** | `W-Launch` | o **mundo empurra** o personagem — a explosão passa a alcançar os três modos | `=117` |
| **J** | `W-Leave` | a sonda do Snap fechou **VERDE** e o buraco era outro: a altura autorada era medida **contra a plataforma** | `=118` |
| **G** | `W-Brink` | `bCanWalkOffLedges` — ele **PARA na quina**, com o alcance **DERIVADO** | `=119` |
| **H** | ⛔ | **voar/noclip — RECUSADA por medição** | — |
| **I** | ⛔ | **air control boost — RECUSADA por medição** | — |

### 2b. As duas recusas, porque elas são metade da entrega

⚠️ **Uma recusa medida vale tanto quanto uma wave construída: ela é o que impede
o item de voltar.** As duas sondas ficam no repo (`--ignored`, imprimem e não
afirmam, cada uma com o seu **CONTROLE**).

**I — `AirControlBoostMultiplier`.** A auditoria descreve o item por um SINTOMA
(*"não consigo sair do lugar no topo de um pulo vertical"*) e a §0 manda medir o
fenómeno antes da cura. Medido (`measure_air_control`): **no ápice ele já corre à
velocidade de CRUZEIRO** — `5,9999` contra `speed = 6,0`, em toda a varredura —,
e **8× o `air_acceleration` compra 8,5% de deriva e move a velocidade do ápice em
ZERO**. O regime em que o sintoma EXISTE também está medido (abaixo de
`air_accel ≈ 10`: 67% / 29% / 15% / 7% do cruzeiro), e **o knob que o cura já está
no painel**. É por isso que o Unreal precisa do multiplicador e nós não: lá o
`AirControl` é uma **FRAÇÃO da velocidade de caminhada** (5% por default); aqui é
uma **aceleração própria** que alcança o cruzeiro em 18 dos 73 tiques de voo. Um
`air_control_boost` seria a **segunda porta** para a mesma pergunta.

**H — `MOVE_Flying` / noclip.** A capacidade **já existe pelos gestos que o editor
tem**, e a sonda mede isso em vez de o supor (`measure_noclip`): com o toggle
**Physics** desmarcado o artista põe o personagem **dentro de uma parede sólida**
(`6,0000 · 4,0000`), **do outro lado dela** (`12,0000 · 0,9000`) e **20 m acima**
(`6,0000 · 20,0000`) — exato a quatro casas —, e o Play **retoma dali** (deriva
lateral **0,0000 m**). Com **CONTROLE**: empurrado contra a parede com o relógio a
andar ele **pára em x = 3,8011** contra uma parede que começa em 4,0. ⚠️ **O que a
recusa NÃO cobre está dito por inteiro:** voar **com as teclas durante o play** não
existe aqui (a MÃO agarra por mola *através do solver*, logo colide) — o gesto
existente dá **teleporte com o relógio parado**, que é exactamente o caso de uso
que a auditoria nomeia.

⚠️ **E a fixture do noclip nasceu ERRADA, com a primeira tabela a MENTIR:** o
gesto do toggle desmarcado é **`PhysicsBridge::hold`**, e **não**
`dispatch(playing = false, …)` — aquela porta, com o alvo a CRESCER, entra no braço
`Greater` e **DÁ PASSO** (o doc dela chama-lhe *"um scrub para a FRENTE enquanto
pausado"*, porque o estado da sim é função do TIQUE e não do botão de play), e o
`readback` do passo seguinte devolvia a pose escrita à mão.

---

### 2c. ⚠️ A AUDITORIA FINAL — três blocos, e o mecanismo é UM

Levantados vários agentes sob lentes independentes, **três delas convergiram na
mesma causa estrutural**: o veredito do `bridge::pose_owner` **não alcançava a
shell**, que respondia às mesmas quatro perguntas por conta própria a partir do
`PlayerMode`. Isso é a segunda cópia de uma resposta que aquele módulo existe
para dar **uma** vez.

**O caso que decide:** um player **ASSADO** (`Kinematic` + `PlatformPlayer`, sem
`PlayerMode` ⇒ `default()` é `Dynamic`) resolve para `PoseOwner::Scene` — a pose
vem de uma curva, o `drive_players` nem entra no laço, e **nenhum** dos doze
cards da §14 é lido por ninguém. O painel pintava os doze como se corresse.
⚠️ **O `pose_owner_tests` já PINAVA esse fato desde a W-KinMove** — ninguém
tinha ligado os dois lados.

| bloco | o que é |
|---|---|
| **1** | `PlayerLiveness` nasce ao lado da lei, cada campo sendo a condição literal de um `if` do `drive_players`; `PhysicsBridge::player_liveness` é a porta, e a shell **lê** em vez de re-derivar. As três rows da MOLA (`Float Height`, `Leg Stiffness`, `Leg Damping`) e o botão *Fit to Collider* saem sob Snap |
| **2** | `Remove Platform Player` **prendia** o corpo · as frações da 3ª lei não eram frações na caixa de texto · o slider da rigidez oferecia **27,8×** o que o kernel honra |
| **3** | o gate de dicas era **auto-referente no conjunto** · seis linhas do `09_auditoria_engines.md` diziam ❌ sobre o que já shipa · as notas de cena de smoke |

⚠️ **A `Cling Distance` FICA sob Snap, e é por isso que o card LEG não se esconde
inteiro:** ali ela é o `snap_distance` **e** o `step_height` do controlador — o
número mais vivo da seção —, e a cura preguiçosa (esconder o card, o precedente
do `Pure`) o levaria junto.

⚠️ **O `Remove` era um BECO SEM SAÍDA:** o gesto do modo escreve DUAS metades (o
`PlayerMode` e o `RigidBody.kind`) e o `Remove` desfazia UMA. O que sobrava era um
corpo `Kinematic` sem player — o estado que a §14 **não oferece** —, então o
artista removia o comportamento e ficava preso, com o corpo a **deixar de cair**
em silêncio e um `PlayerMode` órfão a viajar no arquivo.

⚠️ **E o teto da rigidez é o §0 a morder em casa:** a linha do registro dizia
*"sem teto medido"*, **verdade no dia em que foi escrita**, e a `W-Landing`
(07/08) mediu o teto em `1/dt²` = 3600 e pôs o clamp na LEI sem ninguém
reconferir a nota. A correção achou um **segundo** fato que ninguém procurava: a
tabela de faixas do painel é a segunda cópia dos defaults da lei, e já divergira
em dois campos (rigidez `400` contra `2000`, amortecimento `0,5` contra `1,0`) —
invisível porque o `sync_physics` sobrescreve o store ao selecionar.

⚠️ **E seis linhas da tabela do `09_auditoria_engines.md` diziam ❌ sobre o que a
PRÓPRIA FILA construiu:** `LaunchCharacter` (W-Launch, 14/08) · `OnLanded` /
`bNotifyApex` (W-PlayerOut, 13/08) · `TerminalVelocity` (W-Fall, 14/08) ·
`is_on_floor/wall` + as três consultas do Godot (W-PlayerOut) · `isGrounded` /
`velocity` · `BrakingDecelerationWalking` (W-Brake, 13/08). Elas estavam
**certas em 12/08**. A §3.B prescrevia um bloqueador (*"depende do §3.C"*) que a
W-Brake já pagou.

⚠️ **Um número que a auditoria reportou NÃO reproduziu, e fica registado:** ela
dizia *"seis controles da §14 sem tooltip"*; medido, a §14 pinta **70** ids
próprios, a varredura cobria **57**, e os **três** sem dica são todos **chrome do
painel** (o scrollbar, o cabeçalho da seção, o círculo de cor). *Nenhum controle
da §14 está descoberto* — o defeito era o gate ser **cego a 15 dos 70**, e doze
deles terem dica por sorte.

### 2d. ⚠️ Por que os blocos da §2c NÃO têm smoke, e por que isso é defensável

Dos itens, os que mudam o **produto** são: três controles que **desaparecem**
onde já eram inertes, um botão que **desaparece** onde a lei apagava o efeito
dele, um gesto que passa a **devolver o corpo** em vez de o prender, dois clamps
e uma faixa de slider. **Todos são bugs a sumir**, e o oráculo de cada um já é um
gate com mutação provada — pergunta mais afiada que a que o olho faz aqui.

⚠️ **O que o smoke acrescentaria é a metade que o gate não vê:** *o card LEG com
duas rows lê bem?* Isso é julgamento de LAYOUT, e o Enio o julga em qualquer
cena de player cinemático (`=101`) sem roteiro novo.

---

### 2e. ⚠️ A CAUDA — cinco waves, e a auditoria 09 sai de nove ❌ para quatro

Depois dos três blocos da §2c a linha fechou a **cauda do §3.A** (as consultas
que faltavam ao `PlayerView`) e o item das **camadas**. Detalhe de mecanismo, gate
a gate, no tracker (`HANDOFF_line_physics.md`, cinco seções `⬛ W-*` de 15/08).

| wave | o que ela é |
|---|---|
| **W-WallNormal** | `get_wall_normal` — a normal da parede entra no readout, e o **TETO virou uma recusa MEDIDA** em vez de um item aberto |
| **W-Ceiling** | `is_on_ceiling` — o teto vira um **FATO** do `PlayerView`, com margem que se auto-silencia (a velocidade reportada decai a ~9% de um toque cheio no corte geométrico) |
| **W-Bonked** | o espelho do `Landed` na outra ponta: `PlayerEvent::Bonked { speed }`, ⚠️ **ABSOLUTA e não relativa** — o bit `ceiling` só existe com o personagem NO AR, onde a velocidade de chão publicada é `[0,0]`, logo a forma relativa do `Landed` **já reduz** à absoluta aqui |
| **W-HitNormal** | `OnControllerColliderHit` — o contato ganha **ORIENTAÇÃO**; a normal viaja com o ponto sob o mesmo teste de profundidade, em `ContactReport` · `PeakSample` · `BodyContact` · `ContactEvent` |
| **W-WallMaterial** | `platform_wall_layers` — **esta superfície não é parede**, por CORPO/PEÇA |

⚠️ **Três achados que o integrador precisa, e nenhum é sobre o diff:**

**(1) Uma defesa que nenhum gate consegue ver não é uma defesa.** No `W-HitNormal`
o rapier **não ordena o par**, então a normal publicada leva um flip de sinal
quando os handles vêm trocados — o ramo dispara **976 vezes** na suíte e
**nenhuma cena o distingue** (ele nunca vence o teste de profundidade). A cura
não foi caçar fixture: foi **extrair `published_normal` como função pura** com
gate próprio. *Um ramo alcançável que nenhum oráculo separa é código sem
cobertura, por mais verde que a suíte esteja.*

**(2) O oráculo é a PROPRIEDADE, nunca o eixo.** O primeiro gate da normal
reprovou **produto correto**: ele exigia alinhamento com o eixo dominante da
separação de centros, e uma cápsula alta contra a face de um caixote baixo tem
centros separados em `y` (0,586) com normal **horizontal** (`[−0,997, 0,079]`).
*Direção de centro não é normal de superfície* — o que a geometria garante é o
**semiespaço** (`n · (b−a) > 0`), e é isso que o gate afirma hoje.

**(3) Duas metades do mesmo item tiveram vereditos OPOSTOS, e medir decidiu.**
O `platform_floor_layers` (*o que me CARREGA*) **já era exprimível** — a sonda
`measure_kinematic_carry` mostra uma plataforma horizontal a levar `0,99×` com
tração cheia e **`0,00×` com `WalkSurface { grip: 0 }`**, nos dois modos; a
sonda **já estava no repo**. Só o `platform_wall_layers` (*o que me SEGURA*) era
gap real — a lei aceitava parede **só por inclinação**. ⚠️ **E o filtro cai no
SENSOR, não na lei:** uma superfície marcada some do array de acertos como se o
raio não tivesse batido, então os três verbos (deslizar, agarrar-se, pular)
somem juntos e o `cling` **não aprende o conceito** — a mesma divisão que a
matriz de colisão já faz.

⚠️ **`NoWallCling` é marcador (presença = booleano) ⇒ ZERO `PROJECT_SCHEMA`** — é
a razão de o número não se mexer nesta cauda; um campo apendado a um struct
serializado é postcard **posicional** e teria bumpado.

⚠️ **E o CONTROLE reprovou a minha fixture DUAS vezes**, o que é o padrão desta
casa a funcionar: mandei o personagem **escalar** e o controle chegou a 1,389 m
(ele não escala — um pulo de parede empurra para LONGE, e o gate da peça passava
**por vácuo**); re-aimado para a **descida**, controle 1,499 m contra marcado
0,151 — **ao contrário**, porque encostado à parede o atrito de Coulomb segura um
corpo dinâmico quase parado nos dois casos. O oráculo honesto é **a amostra
existir**: medido, **119 de 120 tiques agarrado contra 0** — e ⚠️ **o gate afirma
`> 30` contra `== 0`, não o 119**, porque a contagem exacta é afinação do solver e
a PROPRIEDADE é *agarra* contra *não agarra*; um bar em 119 falharia no dia em que
um sub-passo mudasse, pelo motivo errado.

---

## 3. Superfície de colisão

| | |
|---|---|
| `PROJECT_SCHEMA` | **70 → 82** ⚠️ **PROVISÓRIO — CONTE contra o `main` do dia** |
| a tripla do pin | **`(70, 13, 14)` → `(82, 13, 14)`** em `shells/desktop/src/project_schema_tests.rs` ⚠️ *é `src/`, não `tests/`* |
| registro `ph2d-physics-ecs` | **29 → 32** (`PlayerSignals` · `WalkSurface` · `NoWallCling`) — ⚠️ os blocos da §2c **não acrescentam nenhum**; o 32º é a cauda (§2e) |
| registro `ph2d-ecs` + os **dois** espelhos | **INTOCADOS** (`git diff` vazio em `crates/ph2d-ecs/`) |
| gizmo ids | **nenhum novo** — o último segue **973**, próximo livre **974** |
| ids novos | **todos `hash_node_id`** ⇒ fora de todo gate de contagem |
| scrollbar ids | nenhum novo |
| ADR | **NENHUM** ⇒ a linha fica **fora de toda disputa de número** |
| `Cargo.toml` / `Cargo.lock` | **ZERO** — nenhuma crate nova, nenhuma dep nova |
| `ph2d-i18n` | **INTOCADO** |
| contrato congelado | **3/3 + 4/4 + 11/11**, rodados (nós · tools · vector) |
| cenas de smoke | **105 → 119** (quinze novas); **próxima livre: 120** ⚠️ e o `CLAUDE.md` dizia `105` (o número do `main`, onde ele estava certo) — corrigido |

### 3a. ⚠️ O `PROJECT_SCHEMA` são DOZE degraus, e é aqui que a colisão mora

⚠️ **ESTA LINHA PARTIU O `project.rs`, e é o ponto de merge mais sensível da
entrega.** A escada inteira **e a constante** saíram para o irmão **NOVO**
`shells/desktop/src/project_schema.rs` (498 linhas), e o `project.rs` perdeu 374
linhas de doc-header no mesmo corte. *Uma linha que acrescente um degrau dentro do
`project.rs` funde **limpa** contra um arquivo de onde a escada saiu* — o modo de
falha exacto que o corte do `project.rs` de 04/08 produziu na `line/Vector`, e a
razão de esta caixa existir.

⚠️ **O valor se CONTA, nunca se escolhe** — e esta colisão **passa MUDA quando
duas linhas escrevem o MESMO literal**: o git não sabe o que o número significa,
e o bump da segunda evapora com a suíte inteira verde. São **TRÊS** sítios a
conferir, não um: o literal (`project_schema.rs`), **a escada ao lado dele**, e a
**tripla** do `project_schema_tests.rs`. Escreva a entrada da escada no MESMO
commit que renumera (a lição do degrau v69, que chegou ao `main` com a linha
AUSENTE).

Os doze degraus, em uma linha cada: **v71** o nado · **v72–v73** os sensores
editáveis · **v74** o pulo do ar · **v75** a beirada · **v76** o planeio ·
**v77–v78** o sensor da beirada (posição e extensão; ⚠️ o v78 existe porque
*`reach_y` é TAMANHO e não POSIÇÃO*, e o v77 tinha mapeado os dois no mesmo
número) · **v79** o freio · **v80** o teto de queda · **v81** o que a plataforma
dá ao pulo · **v82** a trava de beirada.

⚠️ **Todos nascem NEUTROS** — cada rung documenta que o projeto salvo na versão
anterior reabre exactamente como estava. O bump é pelo caminho **INVERSO** (o
postcard é posicional: um leitor velho tem de RECUSAR em vez de ler lixo
bem-formado).

### 3b. O `physics_ecs_c9`

**`2d7f9d5145d09de646f1a3a6da544b67e3497ada475eac85f761089e3d78658d`**,
**121 corpos**, 120 passos, **debug ≡ release** (medido nesta árvore, nos dois
perfis). ⚠️ **Os 121 são 117 + a lane PAREADA da `W-WallMaterial`** (uma parede
marcada e uma não, de propósito — uma lane só cobriria a rota do filtro e não a do
sensor que segue a passar), e ela **DISCRIMINA**: tirar o filtro leva o hash a
`9dbb8bc2…`.

⚠️ **Ele MOVE contra o `main`** (`fb27f676…`, 117 corpos) **por DUAS causas
distintas, e as duas são medidas:** a **CONTAGEM** sobe porque a `W-WallMaterial`
acrescentou a lane pareada (o `git diff` sobre
`crates/ph2d-physics-ecs/src/bin/physics_ecs_c9/` deixou de ser vazio, e é essa a
única lane nova da linha); e o **HASH dos 117 antigos** já movia antes dela, pela
**LEI do player** — a cena carrega quatro lanes com `PlatformPlayer::default()`, e
esta linha mudou defaults (o mais visível é o **leque de pés**, que nasce com
**três** raios contra o raio único do `main`). Cada degrau v79..v82 declara
byte-identidade individualmente; o movimento acumulado vem das waves anteriores à
fila, cujo detalhe está nos handoffs de 08-11 / 08-12.

### 3c. O FOUNDATIONAL / COMPARTILHADO tocado (item **2** da DIRETRIZ §1.5.9)

Tudo fora de `crates/ph2d-{physics,physics-ecs,platformer}/` e de `docs/Physics/`,
**medido por `git diff main...HEAD --name-only`**:

| onde | arquivos | o que é |
|---|---|---|
| `shells/desktop/src/**` | **65** (42 + 23 no `render_loop`) | a ponte, o overlay, o Inspector do lado da shell, as cenas de smoke |
| `crates/ph2d-panel-inspector/**` | **11** | a §11/§12/§14 — as rows dos knobs |
| `shells/desktop/tests/**` | **4** | os arch-gates de shell |
| `crates/ph2d-editor-core/src/{ids,screens/hero}/**` | **4** | os ids (abaixo) + os tipos de info do Inspector |
| `.typos.toml` | **1** | ⚠️ **lista COMPARTILHADA** — ver abaixo |
| `CLAUDE.md` | **1** | ⚠️ **1 insert / 1 delete**, tudo DENTRO do bullet §5 desta linha |

⚠️ **Os dois de risco são os dois últimos, e os dois foram editados na forma que
NÃO colide:**

* **`.typos.toml`** — a linha **ACRESCENTOU** uma entrada (`^PILAR$`) em vez de
  reescrever a vizinha (`^pilar$`, que não a cobria porque as regexes são
  sensíveis a maiúsculas). O comentário no próprio arquivo diz porquê: *num merge,
  um `(?i)` na linha existente arriscaria perder o que outra linha lhe tenha
  acrescentado*. **Só ADICIONE** [[feedback_a_shared_list_is_merged_against_todays_main]].
* **`CLAUDE.md`** — a mudança é **inteira dentro do bullet da física**: corrige uma
  frase FALSA (a nota dizia *"uma lista de `if level == N` e o primeiro vence"*,
  que é o mecanismo do **Vector**, copiado; o roteador da física é um `match`, logo
  um número repetido é `unreachable pattern` **no compilador**) e move a próxima
  cena livre de `105` para `120`. ⚠️ **Nenhuma linha do §0–§4 (o roteador) é
  tocada** — é a forma de menor colisão possível neste arquivo.

### 3d. Símbolos que podem COLIDIR (item **3** da §1.5.9) — **NENHUM**

⚠️ **A resposta é vazia, e é por construção, não por sorte:** os **41** ids novos
(`crates/ph2d-editor-core/src/ids/inspector.rs` + o arquivo novo
`inspector_player.rs`) são **todos `hash_node_id`** — medido, **zero** literais
numéricos no diff daquele diretório. Logo:

* não entram em **nenhum contador** (`node_id_collisions` cobre-os por hash);
* **não há valor literal para o integrador grepar** contra outra linha (§1.5.5);
* **gizmo ids: nenhum novo** — o último segue **973**, próximo livre **974**;
* **scrollbar ids: nenhum novo**;
* **variants de enum novos:** `PlayerEvent::Bonked` (apendado) e o componente
  `NoWallCling` — ⚠️ o primeiro é **exaustivo num `match`**, então quem casar sobre
  `PlayerEvent` fora da crate **falha a COMPILAR**, que é o modo de falha certo, e
  não um símbolo que colide em silêncio.

**O ÚNICO número desta linha que colide com outra é o `PROJECT_SCHEMA`** (§3a), e
ele **se CONTA contra o `main` do dia** — nunca se lê daqui.

---

## 4. Mudanças de comportamento, nomeadas

1. **Um player publica estado e transições** (`W-PlayerOut`) — o readout vivo na
   §14 e, **com o opt-in autorado ligado**, sinais no barramento. ⚠️ **Nasce
   DESLIGADO**, e é decisão de custo: sem isso toda cena de smoke com um
   personagem passaria a cuspir toasts.
2. **Frear deixou de ser acelerar** (`W-Brake`) — o campo nasce em `1`, que é o
   mundo de antes da wave, ao bit.
3. **A superfície fala com a lei** (`W-Surface`) — `WalkSurface` é componente
   OPCIONAL; ausente, é o neutro. ⚠️ **Oferecido em TODO collider**, porque a
   superfície que importa é quase sempre um chão **estático**.
4. **A queda tem teto** (`W-Fall`) — nasce em `0`, que **desliga** a lei.
5. **O mundo empurra o personagem** (`W-Launch`) — a explosão passa a alcançar os
   três modos; antes ela não tinha canal para um player de pose própria.
6. **O pulo numa plataforma** (`W-Leave`) — a política nasce em `Full`, onde a
   porta devolve o valor VERBATIM.
7. **Ele pode parar na quina** (`W-Brink`) — os dois campos nascem em `true` (a
   CAPACIDADE, nunca a trava), e o sensor **nem sequer casta** com a trava
   desarmada. ⚠️ **O alcance NÃO é um degrau de schema**: ele é **DERIVADO**
   (`v²/2a` da lei + meia-largura da ponte), porque o knob que ele substituiu
   tinha o valor certo **em função de outros dois** — medido, a 8 m/s um `0,30`
   deixava CAIR e um `0,60` segurava, com a fronteira exactamente em `0,533`.

**E as da CAUDA (§2e) — três são ADITIVAS e uma é opt-in:**

8. **O contato ganha ORIENTAÇÃO** (`W-HitNormal`) — ⚠️ **é a única mudança de
   SUPERFÍCIE PÚBLICA da cauda**: `ContactReport` · `PeakSample` · `BodyContact` ·
   `ContactEvent` ganharam `normal: [f32; 2]`, então quem os constrói por literal
   (fixtures, o overlay) precisa do campo. **O solver não o lê** ⇒ é READOUT, e é
   por isso que o c9 é indiferente a ele.
9. **O `PlayerView` ganhou teto e normal de parede** (`W-Ceiling` ·
   `W-WallNormal`) — campos novos num struct de leitura; nada no `step` muda.
10. **`PlayerEvent::Bonked`** (`W-Bonked`) — variante nova, e o
    `player_signal_name` é `match` **exaustivo** ⇒ quem casar sobre `PlayerEvent`
    fora desta crate **falha a compilar**, que é o modo de falha certo. Publica
    `player.bonked` no barramento **sob o mesmo opt-in do item 1** (nasce
    desligado).
11. **Uma superfície pode deixar de ser parede** (`W-WallMaterial`) —
    `NoWallCling` é MARCADOR, ausente ⇒ o mundo de antes da wave, ao bit; e ele é
    por **CORPO/PEÇA**, não por camada (um bitmask obrigaria o artista a arrumar o
    nível em camadas para exprimir uma propriedade de **material**).

---

## 5. O gate de fechamento — o que foi rodado, e o resultado

Tudo abaixo nesta árvore (⚠️ *nenhum kill de relógio deste repo significa coisa
nenhuma com o load alto* — a passagem final correu com o load a subir de **3,4**
para **36** enquanto a suíte da shell compilava, e **nenhum gate de relógio
falhou**; se algum falhar no dia, **re-rode isolado antes de suspeitar do merge**).

| gate | resultado |
|---|---|
| `cargo test -p ph2d-physics-ecs -p ph2d-physics -p ph2d-platformer -p ph2d-panel-inspector -p ph2d-editor-core --release` | **verde por EXIT CODE**, **304 suítes `ok`**, 0 falhas |
| `cargo test -p ph2d-host-desktop --release` | **verde por EXIT CODE**, **2902 passados** / 141 suítes / 0 falhas (inclui o `file_loc_caps` da shell) |
| `arch_safe_clamp_only` | 2/2 |
| `architecture_workspace_file_loc_cap` | 2/2 |
| `architecture_contract_surface` (nós) | 3/3 |
| `architecture_tool_contract_surface` | 4/4 |
| `architecture_vector_contract_surface` | 11/11 |
| `cargo clippy --workspace --all-targets --release` | **limpo** — exit 0, **zero** warnings |
| `cargo fmt --all -- --check` | **limpo** — exit 0 ⚠️ *depois de dois arquivos da wave D* |
| `typos` | **limpo** ⚠️ *depois de UM vermelho da cauda* (abaixo) |
| `physics_ecs_c9` | `2d7f9d51…`, **121 corpos**, 120 passos, **debug ≡ release** (as duas corridas dão o mesmo hash) |

⚠️ **RE-RODADO depois dos três blocos da §2c, e o resultado é o que se esperava:**
`1699123f9ed2844fa5159bc842a4e583f0675cdd88bb8895e2654ac706053787`, **117
corpos**, 120 passos — **o mesmo hash**. Nenhum dos três blocos toca o solver:
dois são a fronteira de AUTORIA (o painel e a escrita do Inspector) e o terceiro
é gate e documento.

⚠️ **E RE-RODADO OUTRA VEZ depois da CAUDA (§2e)** — os números da tabela acima são
os desta última passagem, e o hash **MOVEU** para
`2d7f9d51…` / **121 corpos** pela lane nova da `W-WallMaterial` (§3b), que é o
esperado: as outras quatro waves da cauda são READOUT (`ContactReport` ganhou um
campo; ninguém no `step` o lê) e o c9 é indiferente a elas.

⚠️ **As suítes são verdes por EXIT CODE, não por `grep`** — e a armadilha mordeu
nesta mesma passagem: a primeira corrida terminou com `[exited with code 0]` que
era do **`grep` do pipe**, não do cargo. Medido pelo status do cargo:
`CARGO EXIT: 0`, com o `grep` de `FAILED|panicked|error` a devolver **1** (nada
encontrado), que é o sinal certo ao contrário. *O pipe mascara o código de saída*
[[feedback_pipe_masks_script_exit_code]].

⚠️ **E o `typos` pegou UM vermelho meu, da própria cauda:** o gate
`the_published_order_wins_over_the_librarys_…` (`librarys` → **`libraries`**),
renomeado e re-rodado verde. É a **quinta** vez que a varredura de fecho desta
linha acha um latente que um `cargo test -p` por crate **não alcança** — o
`typos` e o `fmt` não correm na suíte de nenhuma crate, e um nome de teste só é
lido por eles.

### 5b. O que só o `ship.sh` pega (item **5** da DIRETRIZ §1.5.9)

⚠️ **A tabela acima roda `--release`, e o `ship.sh` roda o perfil `ci-test`, que
HERDA do `dev`** — ou seja, com `overflow-checks` e `debug-assertions` **ligados**,
que o `--release` desliga. Este repo tem precedente do que isso esconde (o
`wrapping_sub` do `ph2d-flip-colorize`, que só panicava em debug). **Fechado onde
a linha de facto escreveu código:**
`cargo test -p ph2d-physics-ecs -p ph2d-physics -p ph2d-platformer --profile ci-test`
⇒ **`CARGO EXIT: 0`, 253 suítes `ok`**, zero `attempt to add/subtract/multiply
with overflow`.

**O que fica genuinamente por rodar, e o que cada um pode dizer:**

| só no ship | o que esta linha lhe dá | risco |
|---|---|---|
| `cargo machete` (deps não usadas) | **zero `Cargo.toml`/`Cargo.lock`** ⇒ nada de novo | **nenhum desta linha** |
| `cargo deny` (licenças/bans/fontes) | idem — **nenhuma dep nova** | **nenhum desta linha** |
| `cargo audit` (RUSTSEC) | idem | ⚠️ **dependente do TEMPO** — pode ficar vermelho **sem uma linha de código mudar**, e aí não é regressão desta linha |
| `clippy --all-features` | eu corri `--all-targets`, **não** `--all-features` | features desligadas podem esconder lint |
| `nextest` na workspace INTEIRA sob `ci-test` | eu corri 5 crates em `--release` + 3 em `ci-test` | as crates **não tocadas** não foram re-rodadas neste perfil |

⚠️ **E a deriva pré-fork não existe aqui, pela medição da §1:** como o `main`
**não andou** desde o fork, tudo o que o `ship.sh` achar de latente já estava no
`main` **antes** desta linha — ela não o introduziu nem o pode ter drenado
[[project_integration_prefork_lines_ship_drift]].

⚠️ **Nota de ambiente que NÃO vale aqui:** a memória do repo regista que
`target/` é symlink para `/dev/shm` e que uma passada de debug encheu o tmpfs.
**Isto é a árvore PRIMÁRIA** — uma worktree tem `target/` próprio **em disco**
(DIRETRIZ §1.5.1), e esta mediu **151 GB de artefactos com 153 GB livres (84% do
disco)**. O `ship.sh` do integrador corre na primária, onde a nota original vale.

⚠️ **E pegou um TERCEIRO na varredura final:** o commit que trocou
`slope.abs().tan()` por `libm::tanf(slope.abs())` deixou a chamada quebrada em
três linhas, e com o nome mais curto ela cabe numa. **Quarta vez nesta linha.**

⚠️ **O `fmt` pegou dois arquivos VERMELHOS herdados da wave D**
(`measure_terminal.rs` · `player_terminal.rs`) — corrigidos no último commit. É a
**terceira vez nesta linha** que uma varredura de fecho acha fmt latente: um
`cargo test -p` por crate **não roda o `fmt`**, e o arquivo fica vermelho até
alguém varrer a árvore inteira.

---

## 6. Smoke

**Nada nesta entrega está pendente de smoke** — as sete waves foram aprovadas pelo
Enio à medida que fecharam, as duas recusas não têm o que smokar (elas são
medição), e a **AUDITORIA FINAL** (§2c) mais a **CAUDA** (§2e) têm o argumento
escrito no §2d.

⚠️ **A cauda NÃO acrescentou cena, e é decisão, não esquecimento:** quatro das
cinco waves são **READOUT** (um campo novo no `PlayerView` / na `ContactReport`),
que um smoke não distingue de um campo ausente sem alguém pôr um readout na tela
para o gesto; a quinta (`W-WallMaterial`) é um **filtro de sensor** cujo oráculo é
a contagem de tiques agarrado, e ela **está no `physics_ecs_c9`** como lane
pareada. ⚠️ **O que NÃO está coberto por cena, e fica nomeado:** nenhuma cena
shipada demonstra o `player.bonked` — o percurso do `physics_smoke_out` não tem
teto —, e acrescentar um teto a um percurso **já smokado** é decisão do Enio, não
minha.

Rodar, se o integrador quiser reconferir:
`env PH2D_PHYSICS_SMOKE=<n> cargo run -p ph2d-host-desktop --release`

* **`=113`** a saída do player · **`=114`** o freio · **`=115`** a superfície ·
  **`=116`** o teto de queda · **`=117`** o empurrão · **`=118`** o pulo na
  plataforma · **`=119`** a trava de beirada.
* ⚠️ **`=84` não existe, de propósito** — o roteador é um `match` de literais e o
  compilador é o gate (um segundo braço com o mesmo literal é `unreachable`).
* ⚠️ **O número da próxima cena se CONTA lendo o `match`** em
  `shells/desktop/src/physics_smoke.rs`, **nunca uma nota** — a nota da §5 do
  `CLAUDE.md` já envelheceu em onze cenas uma vez.

As sondas que decidiram H e I rodam assim:

```
ph2d-run cargo test -p ph2d-physics-ecs --release --test measure_noclip      -- --ignored --nocapture --test-threads=1
ph2d-run cargo test -p ph2d-physics-ecs --release --test measure_air_control -- --ignored --nocapture
```

---

## 7. Aberto, com o preço ao lado

* **O campo de ATRAÇÃO ainda não alcança um player de pose própria** — ele é
  sustentado, não um evento, e pede um canal **por-tique**, não a porta do
  `W-Launch`.
* **O `bXYOverride` do Unreal** (substituir em vez de somar no empurrão) segue
  fora; entra quando houver quem o peça.
* **A trava de beirada não tem gesto de canvas** — ela é autorada por chip na
  §14, como as irmãs. Um marcador visual da quina seria overlay, e o overlay do
  player já tem dono.
* **O resto da lista aberta do módulo** (o horizonte do plano 02 §8, a paridade
  de arrasto do modo cinemático, o bobbing na poça) está onde estava: nada nesta
  entrega o toca.

**E o que a auditoria final (§2c) deixou aberto, com o número ao lado:**

* ~~**Quatro consultas do Godot seguem em falta**~~ — ⚠️ **FECHADAS pela cauda
  (§2e), e esta linha do handoff sobreviveu ao fato por uma janela:**
  `is_on_ceiling` (W-Ceiling) · `get_wall_normal` (W-WallNormal) ·
  `collisionFlags` (W-Ceiling) · `get_last_slide_collision` (W-HitNormal).
  ⚠️ **E a última delas estava 80% respondida antes de eu abrir a wave** — o canal
  de contatos já carregava par, ponto e carga; faltava a **orientação**. *Meça o
  item antes de construir o que a tabela descreve.*
* ~~**A §3.B (gelo/esteira) está DESBLOQUEADA**~~ — ⚠️ **ela já estava CONSTRUÍDA
  pela `W-Surface`**, e a seção prescrevia trabalho que existia (`395fbecae`
  corrigiu o doc). A dependência que ela declarava (o §3.C) tinha sido paga pela
  `W-Brake`, e o `WalkSurface` pagou o resto.
* **A guarda de LOC do `seam_player.rs`** (1399 linhas) é legítima: o
  `architecture_workspace_file_loc_cap` isenta `**/tests/**` **de propósito**.
  Não é dívida escondida — está medido.
* **O `player_liveness` cai no recuo do `RigidBody` autorado** quando a ponte
  ainda não construiu o corpo (o quadro do clique em *Add*). É deliberado e está
  no doc-comment: sem ele a §14 piscaria inerte por um quadro, no gesto que a
  acabou de criar.

### 7b. ⚠️ Restam QUATRO ❌ na auditoria 09, e **nenhum é trabalho pendente**

A tabela saiu de **nove** ❌ para quatro, e os quatro estão fechados por
**decisão** ou por **medição** — não por falta de tempo. *Um ❌ que significa
«recusado com motivo» e um ❌ que significa «ninguém fez» leem igual na tabela, e
é por isso que cada um tem a seção ao lado:*

| item | por que fica |
|---|---|
| `AirControlBoostMultiplier` | **§3.I** — *nicety*, e o knob que cura o sintoma já está no painel (§2c) |
| `MovementMode` + `MOVE_Custom` | **§3.F** — é ARQUITETURA, e a recomendação escrita é **não agora** |
| `MOVE_Flying` / noclip | **§3.H** — **medido e decidido** pela sonda `measure_noclip` (§2c) |
| Root motion | **§5.K** — fora da fila sem pedido |

⚠️ **E o único buraco REAL que sobra contra o referencial não é da auditoria 09:**
*obstacle actions: **climbing***, que o **plano 08 §4.8** já nomeia.

---

## 8. Ordem de leitura para quem integrar

1. **§3 deste doc** — a superfície de colisão, e em particular o `PROJECT_SCHEMA`
   e o corte do `project.rs`.
2. **[plano 10](../10_plano_fila_da_auditoria.md)** — o mecanismo de cada wave
   **da fila**, com as seções `⟨FECHADA⟩` que dizem **onde o plano errou** e o que
   a medição refutou.
3. **O tracker** ([`HANDOFF_line_physics.md`](HANDOFF_line_physics.md)), cinco
   seções `⬛ W-*` de 15/08 — o mecanismo da **CAUDA** (§2e), que o plano 10 não
   cobre porque ela nasceu do §3.A da auditoria e não da fila.
4. **[08-12](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md)** e
   **[08-11](HANDOFF_INTEGRACAO_line_physics_sensores_2026-08-11.md)** — para o
   porquê de cada número das waves anteriores à fila.

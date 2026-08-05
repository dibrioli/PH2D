# HANDOFF MESTRE — `line/physics` → `main` (2026-08-05)

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
| **(esta wave)** | **W15** — O AGACHAR, e a W15 no plano, no mapa e neste handoff |

**Smoke:** W11b/W11c **APROVADAS** (*"Smoke OK"*, 2026-08-05), **W12 e W13
APROVADAS** na mesma data (*"Smoke OK. SIGA"*) e **a W14 APROVADA** logo a seguir
(*"Smoke OK. Siga"*). **A W15 é a única pendente** — integrar não é aprovar.

---

## §2 — Os números que se contam

| número | veredito |
|---|---|
| **`PROJECT_SCHEMA`** | ⚠️ **55 → 58** (W13, W14 e W15, um degrau cada) — ver §3 |
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
repouso do player mudou). **A W12, a W13, a W14 e a W15 são byte-neutras** — a
descida, o arranque e o agachar exigem botões que a fita do harness não segura, e
as paredes, o arranque e o agachar nascem **desligados**. Isso não é sorte: é a
prova executável de que as quatro capacidades novas são opt-in.

---

## §3 — ⚠️ O bump, e por que ele é PROVISÓRIO

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
a tripla que ele pina — **`(58, 13, 14)`** aqui.

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

```
env PH2D_PHYSICS_SMOKE=81 cargo run -p ph2d-host-desktop --release   # rampa 30° (W11c)
env PH2D_PHYSICS_SMOKE=88 cargo run -p ph2d-host-desktop --release   # o par 40°/50°
env PH2D_PHYSICS_SMOKE=85 cargo run -p ph2d-host-desktop --release   # a jangada (o PESO)
env PH2D_PHYSICS_SMOKE=91 cargo run -p ph2d-host-desktop --release   # A ESCADA DE PRANCHAS (W12)
env PH2D_PHYSICS_SMOKE=92 cargo run -p ph2d-host-desktop --release   # O POÇO (W13)
env PH2D_PHYSICS_SMOKE=93 cargo run -p ph2d-host-desktop --release   # O ABISMO (W14)
env PH2D_PHYSICS_SMOKE=94 cargo run -p ph2d-host-desktop --release   # O CORREDOR BAIXO (W15)
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

---

## §8 — Aberto, com o preço ao lado

- **W11c:** o pouso perdeu os 24 mm de quique que o `Spring Damping` em meio curso
  dava. O slider devolve-o; a troca está nomeada no handoff da W11b §5.
- **W12:** um vão entre plataformas **menor que o personagem** deixa a descida
  armada para sempre. A cena já está quebrada sem descida nenhuma.
- **W13:** o sensor lateral olha só a altura do **MEIO** do corpo — uma beirada
  que alcance só os pés não é vista (a mesma limitação honesta da folga lateral da
  W10). E não há *wall grab*: ficar **parado** numa parede é outra mecânica, com
  botão próprio, e não se alcança escrevendo `0` no `Wall Slide`.
- **W15:** o **piso geométrico** da altura agachada não é clampado pela lei (ela
  não conhece formas) nem avisado na UI — quem escrever `0,20` numa cápsula de
  `0,50` vê o corpo enterrado, sem nada a dizer por quê. A cura natural é a mesma
  do `min_float_height` que a §14 já mostra para a perna de pé: **um readout, não
  um clamp**. Nomeado, não construído.
- **Do plano 06 §4, o que sobra:** **bake de um player** (desbloqueado desde a
  W7 — *"com a fita, assar passa a fazer sentido"*) · persistir a fita · player
  Kinematic.

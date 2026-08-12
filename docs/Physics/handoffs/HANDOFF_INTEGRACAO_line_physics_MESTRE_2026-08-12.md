# Handoff de integração MESTRE — `line/physics` (2026-08-12)

> **A linha NÃO integra nem faz ship** (CLAUDE.md §0.7). Este documento é o que o
> integrador precisa para não colidir nem regredir. DIRETRIZ §1.5.9.
>
> ⚠️ **Ele SUPERSEDE o
> [`HANDOFF_INTEGRACAO_line_physics_sensores_2026-08-11.md`](HANDOFF_INTEGRACAO_line_physics_sensores_2026-08-11.md)
> apenas como *o que integrar agora*** — o **detalhe de mecanismo** das sete
> waves de sensores (`W-Swim` · `W-SwimLine` · `W-ZoneForce` · `W-ShapeCast` ·
> `W-Probes` · `W-Probes2` · `W-FootFan`) continua LÁ e **não foi copiado**. Leia
> os dois: este para a superfície de colisão e para as DUAS waves novas, aquele
> para o porquê de cada número das anteriores.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/physics` |
| HEAD | **o tip de `line/physics`** ⚠️ ver abaixo |
| merge-base com `main` | `76788440adbabb0e5b12f8fdafecc6f1e1183e1a` |
| commits | **51** |
| diff | 103 arquivos, +18.174 / −999 |

⚠️ **Todos são pós-integração de 2026-08-10** (a jornada `W-KinMove` / modo
cinemático, que já está no `main`). Nada aqui foi entregue antes.

⚠️ **O HEAD não é escrito aqui de propósito, e a razão é aritmética:** o commit
que o escreve MUDA o HEAD, então um sha nesta tabela é falso no instante em que é
commitado. O que identifica esta entrega é o **merge-base** acima mais *"o tip da
branch"* — que é o que um integrador usa de qualquer forma.

**O assunto é o PLAYER, em quatro metades.** As duas primeiras estão no handoff
de 08-11 (o catálogo do plano 08, e os SENSORES). As duas novas aqui são o **PULO
DO AR** (§2) e a **BEIRADA** (§2b).

---

## 2. A primeira wave nova — `W-MultiJump`, o pulo do ar

**O `air actions counter` do tnua**, e o item mais pedido do catálogo do plano
08. `JumpConfig` ganhou **`air_jumps`** (a contagem; `0` desliga) +
**`air_jump_height`** (metros). A carga recarrega no CHÃO, no MESMO braço
`if grounded` do coyote — o **terceiro** consumidor da porta única
`JumpState::on_ground` (os outros: o coyote e o ARRANQUE), sem uma 2ª cópia do
predicado, que é exatamente o que o plano exigia.

### O que o plano NÃO previu, e é a parte que precisa de leitura

⚠️ **O proxy do ARRANQUE apodreceu com o terceiro pulo.** O `lib.rs` perguntava
*"a TRANSIÇÃO para o ar"* (`!antes.airborne && depois.airborne`) — exato enquanto
todo pulo começava com o pé em ALGO (chão, parede), e **falso para um pulo do
AR**, que acontece com `airborne` **já verdadeiro**. Ele dizia *não* justamente
no gesto que mais se encadeia com um arranque, contra o que o próprio comentário
de lá promete (*"um pulo de QUALQUER tipo cancela o arranque"*).

Nasceu **`JumpStep::jumped`**, e com ele o **terceiro** gate de cancelamento —
que o comentário do gate irmão da parede **já previa sem o saber** (*"é o gate
seguinte que a apanha, e por isso os dois existem"*).

### As decisões, cada uma com o motivo

⚠️ **A altura do ar é em METROS, não uma fração do primeiro pulo.** Este módulo
tem **três** pulos, e o da parede já é altura absoluta
(`WallConfig::jump_height`) — uma escala aqui faria dois falarem metros e um
falar multiplicador, na mesma seção do painel.

⚠️ **A precedência é a força do APOIO: chão > parede > ar.** Um pulo de parede
**não gasta carga** (a parede é apoio próprio), e o bloco do ar **não tem guard
de *"não estou no chão"***: os dois ramos acima já RETORNARAM em todo caso com
apoio, então chegar ali **é** estar no ar — um `!grounded` seria a 2ª cópia de
uma condição já decidida, e a cópia que envelhece quando um quarto apoio
aparecer.

⚠️ **`next.buffer = 0.0` no ramo do ar é load-bearing:** sem ele o mesmo aperto
re-dispara em tiques consecutivos e queima **as três cargas em ~6 tiques** —
três boosts empilhados, um foguete.

⚠️ **`takeoff: false`, pela mesma física do pulo de parede:** a 3ª lei devolve ao
chão o que o pé nele empurrou, e este pé não empurrou nada. Marcá-lo afundaria
uma jangada com um pulo dado no ar acima dela.

### Medido (`measure_multi_jump`, pela porta do produto)

| gesto | pico acima do repouso |
|---|---|
| um toque | **0,6176 m** |
| dois toques (o 2º no ar) | **1,2326 m** |
| um toque com 0 / 1 / 3 cargas | **0,6176 nos três** |
| duas rodadas com um pouso no meio | **1,2326 nas duas** |
| aperto SEGURADO, um pulo | **1,903 m** |
| aperto SEGURADO, dois pulos | **4,028 m** |

As duas últimas são o que escolhe as prateleiras da cena `=110`.

---

## 2b. A segunda wave nova — `W-Ledge`, a beirada

**O exemplo que o Enio deu quando o plano 08 foi escrito** (§4.5), e o item que
faltava para o player alcançar o que um plataforma 2D moderno oferece: o
personagem que erra o pulo **agarra o parapeito** em vez de escorregar pela face,
e sobe dali.

### ⚠️ A bifurcação que o plano tratava como decisão DISSOLVEU

O §4.5 nomeava uma escolha de arquitetura — *o pendurar escreve POSE (e a
simulação para de mandar) ou escreve VELOCIDADE?* — e depois de a lei estar
escrita **a pergunta não existe**: nem o pendurar nem a subida escrevem pose. Os
dois são um **`boost`** (o que estava lá é substituído) mais o cancelamento de
gravidade pelo canal `PlayerStep::gravity_hold` que o **arranque** já usava.

Isso é o que a torna barata e o que a torna correta ao mesmo tempo:

- **a lei é a MESMA nos dois modos** — o `kinematic_advance` integra o motor, a
  ponte dinâmica aplica o impulso, e nenhum dos dois aprende o que é uma beirada;
- **o solver continua a resolver contatos** — um personagem pendurado que
  encontre geometria no caminho da subida é parado por ela, e não a atravessa.

### O SENSOR é UM raio, e o que ele recusa é de graça

`ledge_ray` é **um raio para BAIXO, à frente da cabeça**: origem em
`[cx + lado·(meia_largura + grab), topo + grab]`, alcance `2·grab`.

⚠️ **`distance == 0` é a recusa de *"a parede continua acima da minha cabeça"*, e
ela não custa uma linha de lógica** — é o contrato de penetração que o
`cast_ray` já publica: origem DENTRO de geometria devolve zero. Sem isso o
personagem agarraria o meio de uma parede lisa.

⚠️ **E o `x` do raio É o alvo da subida.** `across = 2·meia_largura + grab` põe a
borda de DENTRO do corpo exatamente sobre o ponto que o raio provou ser
prateleira — então **a beirada não depende do sensor de parede/flanco**. Um
sistema a menos para manter de acordo.

### DOIS limiares de UM número (o idioma do `swim_enter`)

- **agarrar** exige `lip_rise > 0` (o lábio acima da cabeça);
- **segurar** aceita a banda inteira `[−grab, grab]`.

⚠️ **Sem essa assimetria o servo desliga-se a si próprio:** ele existe para levar
o topo do corpo ATÉ o lábio, e no instante em que consegue, `lip_rise` cruza zero
— com um limiar só, o sucesso dele seria a condição de largar.

### A subida é DISPARADA POR BORDA, e o smoke provou porquê

`LedgeState::was_jump` (gravado **também no braço ocioso**) faz do mantle um
gesto de **aperto novo**. Com disparo por NÍVEL o pendurar era **invisível**:
chega-se a uma beirada *a pular contra ela*, com o dedo já em baixo, então o
personagem subia no mesmo tique em que agarrava e o artista via um pulo alto.

### A subida é um L, não uma diagonal

Sobe primeiro, atravessa depois — uma diagonal corta a quina do bloco e o solver
a bloqueia.

### Medido, pela porta do produto (`measure_the_ledge_armed`)

| | |
|---|---|
| pendurado | o topo do corpo assenta a **2,5 mm** do lábio |
| depois da subida | **de pé** em `lábio + float_height` (4,4005 contra 4,4) |
| varredura | os **seis** pares `(grab, speed)` dão o mesmo número |

⚠️ **E dois números de PRODUTO que a cena obrigou a medir, porque a aritmética de
ar livre mente:** um pulo **colado à parede** alcança **0,745 m** contra os
**1,903 m** do ar livre — o atrito contra a face come **61%** da subida —, e o
topo do corpo pica em **2,145 m**. A primeira versão da cena 111 pôs o patamar
alto em 2,60 usando o número do ar livre, e **o corpo nunca chegava à janela**.

⚠️ **E empurrar contra uma parede com `wall_slide_speed = 0` NÃO faz cair:** o
atrito segura (medido: 0,14 m em 1,5 s ≈ 7,5 cm/s). O controlo de um gate de
produto teve de deixar de ser *"ele cai"* para ser *"ele não chega ao lábio"* — o
que a capacidade muda não é *cair ou não*, é **onde ele pára**.

---

## 3. Superfície de colisão

| item | valor | nota |
|---|---|---|
| `PROJECT_SCHEMA` | **70 → 75** | ⚠️ **cinco degraus**, ver §4 |
| tripla do pin | `(75, 13, 14)` | `project_schema_tests.rs` |
| `physics_ecs_c9` | **`1699123f9ed2844f…`, 117 corpos** | debug ≡ release, medido no tip. ⚠️ **NÃO se move com NENHUMA das duas waves novas** |
| registro `ph2d-physics-ecs` | **29, INTOCADO** | nenhum componente novo |
| registro `ph2d-ecs` + os 2 espelhos | **INTOCADOS** | |
| gizmo ids | **nenhum novo** (o último segue **973**, próximo livre **974**) | |
| ids novos | **15, todos `hash_node_id`** | ⇒ fora de todo gate de contagem |
| ADR | **nenhum** | ⇒ a linha fica **fora de toda disputa de número** |
| `Cargo.toml` / `Cargo.lock` | **ZERO** | nenhuma crate nova, nenhuma dep nova |
| contrato congelado | **4/4** | rodado, não auto-relatado |
| `PLAYER_ROW_COUNT` | **42 → 46** | duas rows do pulo do ar + duas da beirada |
| cenas de smoke | maior **111** (próxima livre **112**) | ⚠️ o `=84` não existe, de propósito |

⚠️ **O `c9` intocado é a PROVA de que os degraus v74 e v75 não movem física.** As
duas capacidades nascem DESLIGADAS (`air_jumps = 0` · `ledge_grab = 0`), então
nenhuma lane do hash as exercita — e o hash é o único oráculo que não depende de
eu afirmar coisa alguma.

⚠️ **E a linha cria ZERO ADR** (`git diff --name-only main...HEAD -- docs/architecture/decisions/`
é **vazio**) ⇒ ela fica fora de toda disputa de número desta janela.

---

## 4. Os degraus de schema

Os três primeiros estão no handoff de 08-11 (§4 de lá): **v71** os três campos
do nado · **v72** os quatro números dos sensores · **v73** os dois da perna em
leque.

**v74 (`W-MultiJump`):** o `PlatformPlayer` ganhou `air_jumps` +
`air_jump_height`, **no MEIO do struct** (logo depois do `jump_height`, que é
onde eles se leem), e o postcard é posicional ⇒ **quebra dura**.

⚠️ **Este degrau NÃO move física:** a contagem nasce em `0`, que é a capacidade
DESLIGADA — o precedente do wall slide e do wall jump —, então um projeto salvo
em v73 reabre com o pulo exatamente como estava. É o oposto do v73, que move (e
o handoff de 08-11 diz por quê).

**v75 (`W-Ledge`):** o `PlatformPlayer` ganhou `ledge_grab` + `ledge_speed`,
também no MEIO do struct ⇒ **quebra dura**.

⚠️ **Este degrau também NÃO move física:** o alcance nasce em `0`, que é a
capacidade desligada, então um projeto salvo em v74 reabre exatamente como
estava — e o `c9` intocado é quem o prova.

⚠️ **PROVISÓRIO — o valor se CONTA contra o `main` do dia.** Três linhas já
colidiram neste número por o terem *escolhido*, e da última vez o certo não
estava em nenhum dos dois lados. ⚠️ **E a colisão passa MUDA quando as duas
linhas escrevem o MESMO literal:** o `project.rs` não conflita e o git não sabe o
que o número significa — confira nos **DOIS** arquivos (`project.rs` **e**
`project_schema_tests.rs`).

---

## 5. LOC — os dois cortes

### O corte de `jump.rs` (`W-MultiJump`)

`jump.rs` estava em **676** e a wave o levou a **796 > 700**. Corte por
**RESPONSABILIDADE**, não por tamanho: **`jump_config.rs`** leva *o que o artista
AUTORA* (o `JumpConfig` e o `STARTING_POINT`, quase inteiramente doc-comments com
as tabelas que escolheram cada número) e o pai fica com *o que acontece num
TIQUE* (`JumpState` / `JumpStep` / `jump_step`).

Re-exportado pelo pai (`pub use jump_config::JumpConfig`) ⇒ **nenhum caminho de
chamador muda**. Os dois arquivos ficam em **584** e **231**.

⚠️ **Um filho de `src/jump.rs` precisa de `#[path]`** — sem ele o compilador
procura `src/jump/jump_config.rs`; é a convenção que `player_leg.rs` e
`height_modes.rs` já seguem.

### E o corte de `player_probes.rs` (`W-Ledge`)

A beirada levou-o a **709 > 700**. Corte pela mesma linha: o que **PERGUNTA** ao
mundo fica no pai (os raios, as varreduras, o `probe_ledge`), e o que **RELATA o
que foi visto** sai para **`player_marks.rs`** (`record_marks` +
`preview_player_probes`, 272 linhas).

⚠️ **O `record_marks` recebe a beirada em DOIS níveis** (`Option<Option<&LedgeProbe>>`)
— o padrão que a perna já usava: `None` é *"a lei não perguntou"* e `Some(None)`
é *"perguntou e não há lábio"*. Colapsá-los faz o marcador do sensor desaparecer
da tela exatamente quando ele é mais útil (o artista a mirar uma beirada e a
falhar).

---

## 6. Gates e mutações

**10 gates novos de lei/produto** + **4 de cena** + as duas rows na varredura de
seam.

- `ph2d-platformer/src/jump_air_tests.rs` (7) — a carga · a altura própria · a
  recarga junto com o coyote · o pulo que não empurra · a parede que não gasta ·
  um aperto uma carga · o controle com `air_jumps = 0`.
- `ph2d-platformer/src/lib_dash_tests.rs` (1) — o **terceiro** cancelamento.
- `ph2d-physics-ecs/tests/player_multi_jump.rs` (3) — pela porta do produto.
- `shells/desktop/src/physics_smoke_multi_jump_tests.rs` (4) — a aritmética em
  tempo de COMPILAÇÃO · o contraste · o controle · **o pouso**.

**7 mutações, 7 sangram:** a feature inteira · o buffer não consumido · a
recarga no ar · a altura errada · empurrar o chão · o ar preceder a parede · o
proxy antigo de volta.

### Da `W-Ledge`

- `ph2d-platformer/src/ledge_tests.rs` (13) — o controle com a beirada desligada
  (o mundo de antes desta wave) · apanhar PARA a queda · o servo leva o TOPO ao
  lábio · os dois limiares · largar é a ausência do gesto · a subida sobe antes
  de atravessar · não deixa nada para trás · nada a interrompe · SUBSTITUI a
  velocidade em vez de somar · o sensor só é pedido onde a lei pode agir · o
  pulo que o trouxe não o faz subir.
- `ph2d-physics-ecs/tests/player_ledge.rs` (7) — pela porta do produto.
- `shells/desktop/src/physics_smoke_ledge_tests.rs` — a aritmética das alturas em
  tempo de COMPILAÇÃO contra os números MEDIDOS do pulo colado à parede.

**11 mutações, 11 sangram:** o `distance == 0` que recusa a parede · os dois
limiares colapsados · disparo por nível · atravessar antes de subir · a subida
deixar de ser comprometida · o motor SOMAR em vez de substituir · a travessia
esquecer a meia-largura de dentro · a subida esquecer a `float_height` · a LEI
recusar no chão · a PORTA do sensor recusar no chão · o cancelamento de
gravidade.

### ⚠️ Três coisas que as mutações acharam nesta wave, e ficam escritas

**(a) As duas camadas de *"no chão não há beirada"* têm cada uma o seu gate,** e
foi conferido em separado: a da LEI sangra o `letting_go_is_the_absence_of_the_gesture`,
a da PORTA do sensor (a camada de CUSTO) sangra o
`the_probe_is_only_wanted_where_the_law_can_act`. Gates diferentes ⇒ a política
de [[feedback_layered_defenses_need_per_layer_gates]] está honrada.

**(b) A mutação do cancelamento de gravidade SOBREVIVEU, e a cura foi um gate
NOVO — não uma desculpa.** Tirar a beirada do `gravity_hold` deixava treze gates
da lei e cinco dos seis do produto **VERDES**, porque o PENDURAR não consegue
medir esse termo: o servo re-mira em todo tique a partir da velocidade VIVA,
então a gravidade de um tique é absorvida pelo seguinte e o assentamento move
**0,1 mm**. A SUBIDA é o outro regime — alvo CONSTANTE —, e lá o termo vale
**1,011× o autorado com ele contra 1,048× sem** (`ledge_speed = 2,0`, dez
tiques). Gate novo `the_climb_walks_at_the_speed_the_artist_wrote` + sonda
`measure_whether_the_climb_walks_at_the_authored_speed`.

**(c) Colapsar os dois limiares NÃO sangra o produto, e isso é observação
honesta, não buraco.** Os gates da LEI sangram; os de produto ficam verdes porque
a trepidação que o colapso introduz cabe dentro da tolerância de 2 cm com que o
oráculo de POSE mede o assentamento. O oráculo está certo (o que o jogador vê é
onde ele pára), e a lei tem os seus próprios gates.

### ⚠️ Duas coisas que as mutações acharam em MIM, e ficam escritas

**(a) O gate novo do arranque nasceu SEM DENTES.** A fixture punha o personagem
a CAIR, e quem nunca pulou entra com `airborne` **falso** — ali o proxy antigo
acerta por acidente. Medido: sem `state.jump.airborne = true` a mutação do proxy
**não sangra**; com ela, sangra sozinha.

**(b) Eu escrevi uma afirmação FALSA num doc de gate.** O gate de PRODUTO dizia
ser o guardião do consumo do buffer, e ele fica **VERDE** sob aquela mutação —
naquele caminho a decolagem do CHÃO já zerou o buffer antes de o personagem
chegar ao ar, então o aperto guardado nunca alcança o ramo do ar. Quem apanha é o
irmão de unidade, cuja fixture entra **já no ar** com uma borda fresca. Doc
corrigido em vez de contrabandeado.

---

## 7. Como rodar o gate

```
cargo test -p ph2d-platformer -p ph2d-physics-ecs -p ph2d-panel-inspector --no-fail-fast
cargo test -p ph2d-host-desktop --no-fail-fast
cargo run -q -p ph2d-physics-ecs --bin physics_ecs_c9 --release   # e sem --release
```

⚠️ **`--no-fail-fast` não é preferência:** sem ele o primeiro binário vermelho
esconde o resto, e na jornada de 08-11 a diferença foi entre *"um gate caiu"* e
*"dez caíram"*.

---

## 8. Smokes

| cena | o quê |
|---|---|
| `PH2D_PHYSICS_SMOKE=105` | **nadar** |
| `=106` | a **correnteza** nos três modos |
| `=107` | as quatro **pedras** que não cabem num raio |
| `=108` | **o que ele vê** — os cinco sensores |
| `=109` | **A FENDA** — o leque, com o controle DENTRO do quadro ✅ **aprovado 2026-08-12** |
| `=110` | **A PRATELEIRA ALTA** — o pulo do ar |
| **`=111`** | **O PARAPEITO** — a beirada |

⚠️ **A cena 110 em uma frase:** duas raias iguais, dois personagens iguais, e o
teclado dirige **os dois ao mesmo tempo** (`hand_input_to_players` entrega a
todo `PlatformPlayer`) — então um gesto só move os dois lado a lado, e o controle
está **dentro do quadro**. A prateleira **baixa (1,5 m)** cabe num pulo e prova
que os DOIS sabem pular; a **alta (3,0 m)** cai no vão entre 1,903 e 4,028, então
só o da direita a alcança.

⚠️ **A cena 111 em uma frase:** a mesma forma da 110 — duas raias iguais, dois
personagens iguais, o teclado a dirigir os dois — e a única diferença entre eles
é **o alcance do braço** (0 à esquerda, 0,60 m à direita). O patamar **baixo
(1,0 m)** cabe num pulo e prova que os DOIS sabem pular; o **alto (2,4 m)** fica
acima dos 2,145 m que o topo do corpo alcança **colado à parede**, então só o da
direita o alcança — e ele fica **pendurado**, não em cima.

⚠️ **O passo 8 é o que fecha a lei:** com a beirada armada, andar contra o
patamar BAIXO **no chão**, sem pular, **não** deve pendurar — um degrau que se
sobe a pé não é beirada.

**Próxima cena livre: `112`** (o `=84` não existe, de propósito).

---

## 9. Aberto, com o preço ao lado

Os itens do handoff de 08-11 continuam abertos e **não foram tocados**
(`min_float_height` conservadora · a metade *"um degrau íngreme não é chão"* sem
fixture de unidade).

Da wave nova:

- **A carga não é VISÍVEL na tela**, e é decisão, não esquecimento: um contador
  de pulos restantes é um **HUD**, e este app não tem um. A metade visível desta
  wave é o próprio comportamento — o personagem pula uma segunda vez, e a cena
  `=110` é onde isso se julga. Um readout na §14 seria um número que o artista
  não pode usar enquanto joga.
- **Um pulo do ar não zera o `wall_lock`.** Depois de um pulo de parede o
  controle aéreo fica calado por `jump_lockout` (0,2 s), e um pulo do ar dentro
  dessa janela não o encurta. É pequeno (o relógio corre de qualquer forma) e
  **não foi medido**: se o smoke mostrar que atrapalha, o lugar é o mesmo braço
  que já zera o coyote.
Da `W-Ledge`:

- **Não há alça de canvas para o alcance**, e é a mesma decisão do contador de
  pulos: o que se julga é o comportamento, e a cena `=111` é onde ele se julga.
  As duas rows na §14 são a autoria.
- **A beirada não interage com o agarrar-se à parede** (`W23`, o
  `wall_grab_stamina`): quem tem os dois armados agarra a beirada assim que ela
  entra na janela, porque a lei da beirada corre antes. **Não foi medido** se
  isso incomoda — se o smoke mostrar que sim, o lugar é a precedência, não um
  knob novo.
- **Um lábio em movimento** (uma plataforma que sobe) não foi exercitado. O
  sensor é re-lançado em todo tique, então a lei deve seguir — mas *deve* não é
  *foi medido*.

- **A fila do plano 08 continua:** ~~`W-Ledge`~~ ✅ → `(W-Glide?)` → *o ajuste*.
